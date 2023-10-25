mod connection;

use std::sync::Arc;

use tokio::net::UdpSocket;

use crate::{
    client::{packets::SOURCE_SIMPLE_PACKET_MAX_SIZE, SteamQueryClient},
    config::ServerConfig,
    server::connection::{Connection, CONNECTION_POOL},
};

pub struct SteamQueryCacheServer {
    config: ServerConfig,
    client: Arc<SteamQueryClient>,
    socket: Arc<UdpSocket>,
}

impl SteamQueryCacheServer {
    pub async fn new(config: ServerConfig) -> std::io::Result<Self> {
        let socket: UdpSocket = UdpSocket::bind(config.bind.clone()).await?;
        let client: SteamQueryClient = SteamQueryClient::new(config.host.clone()).await?;
        Ok(Self {
            config,
            socket: Arc::new(socket),
            client: Arc::new(client),
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
                        let connection =
                            Connection::new(self.socket.clone(), self.client.clone(), addr).await;
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
