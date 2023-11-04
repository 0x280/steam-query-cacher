use std::{collections::HashMap, net::SocketAddr, sync::Arc};

use once_cell::sync::Lazy;
use tokio::{
    net::UdpSocket,
    sync::{mpsc, RwLock},
};

use crate::client::{
    packets::{
        a2s_info::A2SInfo, s2c_challenge::S2CChallenge, QueryHeader, SourceChallenge,
        SOURCE_PACKET_HEADER, a2s_player::A2SPlayer, a2s_rules::A2SRules,
    },
    SteamQueryClient,
};

use super::{challenge_cache::ChallengeCache, query_cache::QueryCacheManager};

pub static CONNECTION_POOL: Lazy<RwLock<HashMap<SocketAddr, Arc<mpsc::Sender<Vec<u8>>>>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));

#[derive(Debug)]
pub struct Connection {
    socket: Arc<UdpSocket>,
    addr: SocketAddr,
    client: Arc<SteamQueryClient>,
    pub tx: Arc<mpsc::Sender<Vec<u8>>>,
    rx: mpsc::Receiver<Vec<u8>>,
    challenge_cache: Arc<ChallengeCache>,
    query_cache: Arc<QueryCacheManager>,
}

impl Connection {
    pub async fn new(
        socket: Arc<UdpSocket>,
        client: Arc<SteamQueryClient>,
        addr: SocketAddr,
        challenge_cache: Arc<ChallengeCache>,
        query_cache: Arc<QueryCacheManager>,
    ) -> Self {
        let (tx, rx) = tokio::sync::mpsc::channel(1_000);
        let tx: Arc<mpsc::Sender<Vec<u8>>> = Arc::new(tx);

        let instance: Self = Self {
            socket,
            client,
            addr,
            rx,
            tx,
            challenge_cache,
            query_cache,
        };

        instance
    }

    async fn read(&mut self) -> Result<Vec<u8>, std::io::Error> {
        tokio::select! {
            _ = tokio::time::sleep(std::time::Duration::from_secs(5)) => {
                return Err(std::io::Error::new(std::io::ErrorKind::TimedOut, format!("Timed out")))
            }
            res = self.rx.recv() => {
                match res {
                    Some(data) => {
                        log::trace!("Received {} bytes from {}", data.len(), self.addr);
                        return Ok(data)
                    }
                    None => {
                        return Err(std::io::Error::new(std::io::ErrorKind::Other, format!("Failed to receive from channel")))
                    }
                }
            }
        };
    }

    async fn send(&mut self, buf: Vec<u8>) -> Result<(), std::io::Error> {
        self.socket.send_to(&buf, self.addr).await?;
        log::trace!("Sent {} bytes to {}", buf.len(), self.addr);
        Ok(())
    }

    async fn handle_connection(mut self) -> Result<(), std::io::Error> {
        log::info!("Handling connection from {}", self.addr);

        loop {
            let buf = self.read().await?;

            let header = i32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]]);
            if header != SOURCE_PACKET_HEADER {
                log::warn!("Received packet with invalid packet header from {}", self.addr);
                return Ok(());
            }

            let header: QueryHeader = match QueryHeader::try_from(buf[4]) {
                Ok(header) => header,
                Err(e) => {
                    // TODO: blacklist ip
                    log::warn!(
                        "Received invalid packet from {}: {}",
                        self.addr,
                        e
                    );
                    return Ok(());
                }
            };

            if header == QueryHeader::A2SInfo {
                let packet: A2SInfo = match A2SInfo::try_from(&buf.as_slice()[4..]) {
                    Ok(packet) => packet,
                    Err(e) => {
                        log::warn!(
                            "Received packet with invalid query header from {}: {}",
                            self.addr,
                            e
                        );
                        return Ok(());
                    }
                };

                let challenge: SourceChallenge =
                    self.challenge_cache.get_challenge(&self.addr).await;

                if match packet.challenge {
                    Some(challenge) => challenge != challenge,
                    None => true,
                } {
                    log::trace!("Sending challenge to {}", self.addr);
                    let s2c_challenge = S2CChallenge::new(challenge);
                    let mut bytes: Vec<u8> = s2c_challenge.into();
                    i32::to_le_bytes(SOURCE_PACKET_HEADER)
                        .iter()
                        .for_each(|b| bytes.insert(0, *b));
                    self.send(bytes).await?;
                    continue;
                }

                let a2s_info = self.query_cache.a2s_info().await?;
                let mut bytes: Vec<u8> = a2s_info.into();
                i32::to_le_bytes(SOURCE_PACKET_HEADER)
                    .iter()
                    .for_each(|b| bytes.insert(0, *b));

                log::trace!(
                    "Sending {} bytes to {}: {:?}",
                    bytes.len(),
                    self.addr,
                    bytes
                );

                self.send(bytes).await?;
            } else if header == QueryHeader::A2SPlayer {
                let packet: A2SPlayer = match A2SPlayer::try_from(&buf.as_slice()[4..]) {
                    Ok(packet) => packet,
                    Err(e) => {
                        log::warn!(
                            "Received invalid packet from {}: {}",
                            self.addr,
                            e
                        );
                        return Ok(());
                    }
                };

                let challenge: SourceChallenge =
                    self.challenge_cache.get_challenge(&self.addr).await;

                if match packet.challenge {
                    Some(challenge) => challenge != challenge,
                    None => true,
                } {
                    log::trace!("Sending challenge to {}", self.addr);
                    let s2c_challenge = S2CChallenge::new(challenge);
                    let mut bytes: Vec<u8> = s2c_challenge.into();
                    i32::to_le_bytes(SOURCE_PACKET_HEADER)
                        .iter()
                        .for_each(|b| bytes.insert(0, *b));
                    self.send(bytes).await?;
                    continue;
                }

                let a2s_player = self.query_cache.a2s_player().await?;
                let mut bytes: Vec<u8> = a2s_player.into();
                i32::to_le_bytes(SOURCE_PACKET_HEADER)
                    .iter()
                    .for_each(|b| bytes.insert(0, *b));

                log::trace!(
                    "Sending {} bytes to {}: {:?}",
                    bytes.len(),
                    self.addr,
                    bytes
                );

                self.send(bytes).await?;
            } else if header == QueryHeader::A2SRules {
                let packet: A2SRules = match A2SRules::try_from(&buf.as_slice()[4..]) {
                    Ok(packet) => packet,
                    Err(e) => {
                        log::warn!(
                            "Received invalid packet from {}: {}",
                            self.addr,
                            e
                        );
                        return Ok(());
                    }
                };

                let challenge: SourceChallenge =
                    self.challenge_cache.get_challenge(&self.addr).await;

                if match packet.challenge {
                    Some(challenge) => challenge != challenge,
                    None => true,
                } {
                    log::trace!("Sending challenge to {}", self.addr);
                    let s2c_challenge = S2CChallenge::new(challenge);
                    let mut bytes: Vec<u8> = s2c_challenge.into();
                    i32::to_le_bytes(SOURCE_PACKET_HEADER)
                        .iter()
                        .for_each(|b| bytes.insert(0, *b));
                    self.send(bytes).await?;
                    continue;
                }

                let a2s_rules = self.query_cache.a2s_rules().await?;
                let mut bytes: Vec<u8> = a2s_rules.into();
                i32::to_le_bytes(SOURCE_PACKET_HEADER)
                    .iter()
                    .for_each(|b| bytes.insert(0, *b));

                log::trace!(
                    "Sending {} bytes to {}: {:?}",
                    bytes.len(),
                    self.addr,
                    bytes
                );

                self.send(bytes).await?;
            } else {
                let resp = self.client.proxy_request(buf).await?;
                self.send(resp).await?;
            }
        }
    }

    pub async fn start(self) {
        let addr = self.addr;

        let mut pool = CONNECTION_POOL.write().await;
        pool.insert(addr, self.tx.clone());
        drop(pool);

        tokio::spawn(async move {
            if let Err(e) = self.handle_connection().await {
                if e.kind() == std::io::ErrorKind::TimedOut {
                    log::info!("Connection ({}) timed out: {}", addr, e);
                } else {
                    log::error!("Failed to handle connection ({}): {}", addr, e);
                }
            }

            {
                let mut pool = CONNECTION_POOL.write().await;
                if pool.contains_key(&addr) {
                    pool.remove(&addr);
                }
            }
        });
    }
}
