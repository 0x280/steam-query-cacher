pub mod packets;

use tokio::net::{ToSocketAddrs, UdpSocket};

use packets::a2s_info::A2SInfo;

use self::packets::{
    a2s_info_reply::A2SInfoReply, a2s_player::A2SPlayer, a2s_player_reply::A2SPlayerReply,
    a2s_rules::A2SRules, a2s_rules_reply::A2SRulesReply, QueryHeader, SourceQueryRequest,
    SourceQueryResponse, SOURCE_PACKET_HEADER, SOURCE_SIMPLE_PACKET_MAX_SIZE,
};

#[derive(Debug)]
pub struct SteamQueryClient {
    socket: UdpSocket,
}

impl SteamQueryClient {
    pub async fn new<T>(addr: T) -> std::io::Result<Self>
    where
        T: ToSocketAddrs,
    {
        let socket = UdpSocket::bind("0.0.0.0:0").await?;
        socket.connect(addr).await?;

        Ok(Self { socket })
    }

    async fn send_packet<T: SourceQueryRequest>(&self, packet: T) -> std::io::Result<()> {
        log::trace!("sending packet: {:?}", packet);
        let mut bytes: Vec<u8> = packet.into();
        i32::to_le_bytes(SOURCE_PACKET_HEADER)
            .iter()
            .for_each(|b| bytes.insert(0, *b));

        log::trace!("sending packet bytes: {:?}", bytes);
        self.socket.send(&bytes).await?;

        Ok(())
    }

    async fn recv_packet_bytes(&self) -> std::io::Result<Vec<u8>> {
        let mut buf: Vec<u8> = Vec::with_capacity(SOURCE_SIMPLE_PACKET_MAX_SIZE);

        tokio::select! {
            _ = tokio::time::sleep(std::time::Duration::from_secs(5)) => {
                return Err(std::io::Error::new(std::io::ErrorKind::TimedOut, "Timed out"));
            },
            result = self.socket.recv_buf(&mut buf) => {
                result?;
            }
        }
        log::trace!("received packet bytes: {:?}", buf);

        Ok(buf)
    }

    pub async fn query<T: SourceQueryRequest, U: SourceQueryResponse>(
        &self,
        mut packet: T,
    ) -> std::io::Result<U>
    where
        for<'a> <U as TryFrom<&'a [u8]>>::Error: std::fmt::Display,
    {
        loop {
            self.send_packet(packet.clone()).await?;

            let mut packet_bytes: Vec<u8> = self.recv_packet_bytes().await?;

            let header: i32 = i32::from_le_bytes([
                packet_bytes[0],
                packet_bytes[1],
                packet_bytes[2],
                packet_bytes[3],
            ]);
            if header != SOURCE_PACKET_HEADER {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("Invalid packet header: {}", header),
                ));
            }

            // discard the first 4 bytes of the packet
            packet_bytes = packet_bytes[4..].to_vec();

            let header: u8 = packet_bytes[0];
            let header = match QueryHeader::try_from(header) {
                Ok(header) => header,
                Err(_) => {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "Invalid query header",
                    ))
                }
            };

            if header == QueryHeader::S2CChallenge {
                log::trace!("Received challenge packet");
                let challenge: i32 = i32::from_le_bytes([
                    packet_bytes[1],
                    packet_bytes[2],
                    packet_bytes[3],
                    packet_bytes[4],
                ]);
                log::trace!("Challenge: {}", challenge);

                packet.set_challenge(challenge);
                continue;
            }

            if header != U::packet_header() {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("Invalid packet header: {:?}", header),
                ));
            }

            let packet: U = match U::try_from(&packet_bytes) {
                Ok(packet) => packet,
                Err(e) => {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        format!("Failed to parse packet: {}", e),
                    ))
                }
            };
            log::trace!("Received packet: {:?}", packet);

            return Ok(packet);
        }
    }

    #[allow(dead_code)]
    pub async fn a2s_info(&self) -> std::io::Result<A2SInfoReply> {
        let packet: A2SInfo = A2SInfo::new();

        self.query::<A2SInfo, A2SInfoReply>(packet).await
    }

    #[allow(dead_code)]
    pub async fn a2s_player(&self) -> std::io::Result<A2SPlayerReply> {
        let packet: A2SPlayer = A2SPlayer::new();

        self.query::<A2SPlayer, A2SPlayerReply>(packet).await
    }

    #[allow(dead_code)]
    pub async fn a2s_rules(&self) -> std::io::Result<A2SRulesReply> {
        let packet: A2SRules = A2SRules::new();

        self.query::<A2SRules, A2SRulesReply>(packet).await
    }

    pub async fn proxy_request(&self, request: Vec<u8>) -> std::io::Result<Vec<u8>> {
        self.socket.send(&request).await?;

        let mut buf: Vec<u8> = Vec::with_capacity(SOURCE_SIMPLE_PACKET_MAX_SIZE);

        tokio::select! {
            _ = tokio::time::sleep(std::time::Duration::from_secs(5)) => {
                return Err(std::io::Error::new(std::io::ErrorKind::TimedOut, "Timed out"));
            },
            result = self.socket.recv_buf(&mut buf) => {
                result?;
            }
        }
        log::trace!("received packet bytes: {:?}", buf);

        Ok(buf)
    }
}
