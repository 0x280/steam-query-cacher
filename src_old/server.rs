use std::{sync::Arc, net::SocketAddr, ops::Deref};

use tokio::net::UdpSocket;
use caches::{Cache, LRUCache};
use rand::Rng;

use crate::{config::ConfigServer, server_cache::ServerCache, Config, client::{SteamQueryClient, packets::{self, headers::QueryHeader}}};

struct Server {
    name: String,
    cache: Arc<ServerCache>,
    socket: Arc<UdpSocket>,
}

impl Server {
    async fn new(server: &ConfigServer) -> std::io::Result<Self> {
        Ok(Self {
            name: server.name.clone(),
            socket: Arc::new(UdpSocket::bind(("0.0.0.0", server.bind_port)).await?),
            cache: Arc::new(ServerCache::new(server)),
        })
    }

    fn generate_challenge() -> i32 {
        let mut rng = rand::thread_rng();
        rng.gen()
    }

    async fn listen(&self) -> std::io::Result<()> {
        let (tx, mut rx) = tokio::sync::mpsc::channel(1_024);

        let socket = self.socket.clone();
        let cache = self.cache.clone();
        tokio::spawn(async move {
            let mut lru = match LRUCache::<SocketAddr, i32, _>::new(1_024) {
                Ok(lru) => lru,
                Err(e) => {
                    log::error!("Failed to create LRU cache: {}", e);
                    return;
                }
            };
            while let Some((buffer, addr)) = rx.recv().await {
                let buffer: Vec<u8> = buffer;
                let pkt = match packets::generic_packet::GenericPacket::try_from(buffer.clone()) {
                    Ok(packet) => packet,
                    Err(e) => {
                        log::error!("Failed to parse packet: {}", e);
                        continue;
                    }
                };

                match pkt.payload.header {
                    QueryHeader::A2SInfo => {
                        let pkt = match packets::a2s_info::A2SInfo::try_from(buffer) {
                            Ok(pkt) => pkt,
                            Err(e) => {
                                log::error!("Failed to parse packet: {}", e);
                                continue;
                            }
                        };

                        let challenge: Option<i32> = lru.get(&addr).cloned();
                        let challenge = match challenge {
                            Some(challenge) => challenge,
                            None => {
                                let challenge = Self::generate_challenge();
                                lru.put(addr, challenge);
                                challenge
                            }
                        };

                        if pkt.payload.challenge.is_none() || pkt.payload.challenge.unwrap() != challenge {
                            let packet = packets::s2c_challenge::S2CChallenge::new(challenge);
                            let packet: Vec<u8> = packet.into();
                            if let Err(e) = socket.send_to(&packet, addr).await {
                                log::error!("Failed to send packet: {}", e);
                            }
                            continue;
                        }

                        let info = match cache.a2s_info().await {
                            Ok(info) => info,
                            Err(e) => {
                                log::error!("Failed to get info: {}", e);
                                continue;
                            }
                        };

                        let d = info.deref().clone();
                        let packet: Vec<u8> = d.into();
                    },
                    _ => {
                        log::error!("Invalid packet header");
                        continue;
                    }
                };
            }
        });

        log::info!("{} listening on {}", self.name, self.socket.local_addr()?);

        let mut buf = [0; 1400];
        loop {
            let (len, addr) = self.socket.recv_from(&mut buf).await?;
            if let Err(e) = tx.send((buf[..len].to_vec(), addr)).await {
                log::error!("Failed to send packet to channel: {}", e);
            }
        }
    }
}

pub struct SteamQueryServer {
    servers: Vec<Arc<Server>>,
}

impl SteamQueryServer {
    pub async fn new(config: &Config) -> std::io::Result<Self> {
        let mut servers: Vec<Arc<Server>> = Vec::new();

        for s in &config.servers {
            servers.push(Arc::new(Server::new(&s).await?));
        }

        Ok(Self { servers })
    }

    pub async fn listen(&self) -> std::io::Result<()> {
        // concurrently listen on all servers
        let mut set = tokio::task::JoinSet::new();

        for s in &self.servers {
            let s = s.clone();
            set.spawn(async move { s.listen().await });
        }

        set.spawn(async move {
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            log::info!("Connecting to server");
            let client = SteamQueryClient::new("127.0.0.1:28165").await?;
            log::info!("Sending info request");
            let info = client.a2s_info().await?;
            log::info!("Received info: {:?}", info);
            Ok(())
        });

        while let Some(result) = set.join_next().await {
            if let Err(e) = result {
                log::error!("Failed to listen on server: {}", e);
            }
        }

        Ok(())
    }
}
