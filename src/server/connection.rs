use std::{collections::HashMap, net::SocketAddr, sync::Arc};

use once_cell::sync::Lazy;
use tokio::{
    net::UdpSocket,
    sync::{mpsc, RwLock},
};

use crate::client::{
    packets::{QueryHeader, SOURCE_PACKET_HEADER},
    SteamQueryClient,
};

pub static CONNECTION_POOL: Lazy<RwLock<HashMap<SocketAddr, Arc<mpsc::Sender<Vec<u8>>>>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));

#[derive(Debug)]
pub struct Connection {
    socket: Arc<UdpSocket>,
    client: Arc<SteamQueryClient>,
    addr: SocketAddr,
    pub tx: Arc<mpsc::Sender<Vec<u8>>>,
    rx: mpsc::Receiver<Vec<u8>>,
}

impl Connection {
    pub async fn new(
        socket: Arc<UdpSocket>,
        client: Arc<SteamQueryClient>,
        addr: SocketAddr,
    ) -> Self {
        let (tx, rx) = tokio::sync::mpsc::channel(1_000);
        let tx: Arc<mpsc::Sender<Vec<u8>>> = Arc::new(tx);

        let instance: Self = Self {
            socket,
            client,
            addr,
            rx,
            tx,
        };

        instance
    }

    async fn read(&mut self) -> Result<Vec<u8>, std::io::Error> {
        tokio::select! {
            _ = tokio::time::sleep(std::time::Duration::from_secs(1)) => {
                return Err(std::io::Error::new(std::io::ErrorKind::TimedOut, format!("Timed out")))
            }
            res = self.rx.recv() => {
                match res {
                    Some(data) => {
                        log::info!("Received {} bytes from {}", data.len(), self.addr);
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
        log::debug!("Sent {} bytes to {}", buf.len(), self.addr);
        Ok(())
    }

    async fn handle_connection(mut self) -> Result<(), std::io::Error> {
        log::info!("Handling connection from {}", self.addr);
        let buf = self.read().await?;

        let header = i32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]]);
        if header != SOURCE_PACKET_HEADER {
            log::warn!("Received packet with invalid header from {}", self.addr);
            return Ok(());
        }

        let header: QueryHeader = match QueryHeader::try_from(buf[4]) {
            Ok(header) => header,
            Err(e) => {
                log::warn!(
                    "Received packet with invalid header from {}: {}",
                    self.addr,
                    e
                );
                return Ok(());
            }
        };

        if header == QueryHeader::A2SInfo {
            let a2s_info = self.client.a2s_info().await?;
            let mut bytes: Vec<u8> = a2s_info.into();
            i32::to_le_bytes(SOURCE_PACKET_HEADER)
                .iter()
                .for_each(|b| bytes.insert(0, *b));

            log::debug!(
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

        Ok(())
    }

    pub async fn start(self) {
        let addr = self.addr;

        let mut pool = CONNECTION_POOL.write().await;
        pool.insert(addr, self.tx.clone());
        drop(pool);

        tokio::spawn(async move {
            if let Err(e) = self.handle_connection().await {
                log::error!("Failed to handle connection: {}", e);
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
