pub mod packets;

use tokio::net::{ToSocketAddrs, UdpSocket};

use packets::{
    a2s_info::{A2SInfo, A2SInfoReply},
    generic_packet::GenericPacket,
};

use packets::{headers::QueryHeader, s2c_challenge::S2CChallenge};

use crate::client::packets::{a2s_player::A2SPlayer, a2s_rules::A2SRules};

use self::packets::{a2s_player::A2SPlayerReply, a2s_rules::A2SRulesReply};

pub struct SteamQueryClient {
    socket: UdpSocket,
}

impl SteamQueryClient {
    pub async fn new<A>(addr: A) -> Result<Self, std::io::Error>
    where
        A: ToSocketAddrs,
    {
        let socket: UdpSocket = UdpSocket::bind("0.0.0.0:0").await?;
        socket.connect(addr).await?;
        Ok(Self { socket })
    }

    pub async fn a2s_info(&self) -> Result<A2SInfoReply, std::io::Error> {
        let mut a2s_info_packet: Option<A2SInfoReply> = None;
        let mut challenge: Option<i32> = None;

        for _ in 0..3 {
            let packet: A2SInfo = A2SInfo::new(challenge);
            log::debug!("Sending: {:?}", packet);
            let packet: Vec<u8> = packet.into();
            self.socket.send(&packet).await?;

            let mut buffer = Vec::with_capacity(1400);
            self.socket.recv_buf(&mut buffer).await?;
            let buffer = buffer.to_vec();

            let packet: GenericPacket = GenericPacket::try_from(buffer.clone())?;
            log::debug!("Received header: {:?}", packet.payload.header);
            match packet.payload.header {
                QueryHeader::S2CChallenge => {
                    let packet = S2CChallenge::try_from(buffer)?;
                    log::debug!("Received: {:?}", packet);
                    challenge = Some(packet.payload.challenge);
                }
                QueryHeader::A2SInfoReply => {
                    let packet = A2SInfoReply::try_from(buffer)?;
                    a2s_info_packet = Some(packet);
                    break;
                }
                _ => {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "Invalid packet header",
                    ));
                }
            }
        }

        match a2s_info_packet {
            Some(packet) => Ok(packet),
            None => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Failed to receive A2S_INFO packet",
            )),
        }
    }

    pub async fn a2s_player(&self) -> Result<A2SPlayerReply, std::io::Error> {
        let mut a2s_player_packet: Option<A2SPlayerReply> = None;
        let mut challenge: Option<i32> = None;

        for _ in 0..3 {
            let packet: A2SPlayer = A2SPlayer::new(challenge);
            log::debug!("Sending: {:?}", packet);
            let packet: Vec<u8> = packet.into();
            self.socket.send(&packet).await?;

            let mut buffer = Vec::with_capacity(1400);
            self.socket.recv_buf(&mut buffer).await?;
            let buffer = buffer.to_vec();

            let packet: GenericPacket = GenericPacket::try_from(buffer.clone())?;
            log::debug!("Received header: {:?}", packet.payload.header);
            match packet.payload.header {
                QueryHeader::S2CChallenge => {
                    let packet = S2CChallenge::try_from(buffer)?;
                    log::debug!("Received: {:?}", packet);
                    challenge = Some(packet.payload.challenge);
                }
                QueryHeader::A2SPlayerReply => {
                    let packet = A2SPlayerReply::try_from(buffer)?;
                    a2s_player_packet = Some(packet);
                    break;
                }
                _ => {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "Invalid packet header",
                    ));
                }
            }
        }

        match a2s_player_packet {
            Some(packet) => Ok(packet),
            None => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Failed to receive A2S_PLAYER packet",
            )),
        }
    }

    pub async fn a2s_rules(&self) -> Result<A2SRulesReply, std::io::Error> {
        let mut a2s_rules_packet: Option<A2SRulesReply> = None;
        let mut challenge: Option<i32> = None;

        for _ in 0..3 {
            let packet: A2SRules = A2SRules::new(challenge);
            log::debug!("Sending: {:?}", packet);
            let packet: Vec<u8> = packet.into();
            self.socket.send(&packet).await?;

            let mut buffer = Vec::with_capacity(1400);
            self.socket.recv_buf(&mut buffer).await?;
            let buffer = buffer.to_vec();

            let packet: GenericPacket = GenericPacket::try_from(buffer.clone())?;
            log::debug!("Received header: {:?}", packet.payload.header);
            match packet.payload.header {
                QueryHeader::S2CChallenge => {
                    let packet = S2CChallenge::try_from(buffer)?;
                    log::debug!("Received: {:?}", packet);
                    challenge = Some(packet.payload.challenge);
                }
                QueryHeader::A2SRulesReply => {
                    let packet = A2SRulesReply::try_from(buffer)?;
                    a2s_rules_packet = Some(packet);
                    break;
                }
                _ => {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "Invalid packet header",
                    ));
                }
            }
        }

        match a2s_rules_packet {
            Some(packet) => Ok(packet),
            None => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Failed to receive A2S_RULES packet",
            )),
        }
    }
}
