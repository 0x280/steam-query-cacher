mod challenge_cache;
mod connection;
mod query_cache;

use std::sync::Arc;

use tokio::net::UdpSocket;

use crate::{
    client::{packets::SOURCE_SIMPLE_PACKET_MAX_SIZE, SteamQueryClient},
    config::ServerConfig,
    server::connection::{Connection, CONNECTION_POOL},
};

use self::{challenge_cache::ChallengeCache, query_cache::QueryCacheManager};

pub struct SteamQueryCacheServer {
    config: ServerConfig,
    client: Arc<SteamQueryClient>,
    socket: Arc<UdpSocket>,
    challenge_cache: Arc<ChallengeCache>,
    query_cache: Arc<QueryCacheManager>,
}

impl SteamQueryCacheServer {
    pub async fn new(config: ServerConfig) -> std::io::Result<Self> {
        let socket: Arc<UdpSocket> = Arc::new(UdpSocket::bind(config.bind.clone()).await?);
        let client: Arc<SteamQueryClient> = Arc::new(SteamQueryClient::new(config.host.clone()).await?);
        let challenge_cache: Arc<ChallengeCache> = Arc::new(ChallengeCache::new().await);
        let query_cache: Arc<QueryCacheManager> = Arc::new(QueryCacheManager::new(client.clone()));
        Ok(Self {
            config,
            socket,
            client,
            challenge_cache,
            query_cache,
        })
    }

    pub async fn listen(&self) {
        log::info!("Listening on {}", self.config.bind);

        loop {
            let mut buf = Vec::with_capacity(SOURCE_SIMPLE_PACKET_MAX_SIZE);
            let (_len, addr) = match self.socket.recv_buf_from(&mut buf).await {
                Ok((len, addr)) => (len, addr),
                Err(e) => {
                    log::error!("Failed to receive from socket: {}", e);
                    continue;
                }
            };

            let tx;

            {
                let pool_item;

                {
                    let pool = CONNECTION_POOL.read().await;
                    pool_item = match pool.get(&addr) {
                        Some(tx) => Some(tx.clone()),
                        None => None,
                    };
                }

                match pool_item {
                    Some(_tx) => {
                        tx = _tx.clone();
                    }
                    None => {
                        log::info!("New connection from {}", addr);
                        let connection = Connection::new(
                            self.socket.clone(),
                            self.client.clone(),
                            addr,
                            self.challenge_cache.clone(),
                            self.query_cache.clone(),
                        )
                        .await;
                        tx = connection.tx.clone();
                        connection.start().await;
                    }
                }
            }

            if let Err(e) = tx.send(buf).await {
                log::error!("Failed to send to channel: {}", e);
            }
        }
    }
}
