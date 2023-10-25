use super::{headers::{QueryHeader, PacketHeader}, packet::{QueryPacket, SimplePacket}};

///////////////////////////////////////////////////////////////////////////
/// A2S_PLAYER
/// https://developer.valvesoftware.com/wiki/Server_queries#A2S_PLAYER
///////////////////////////////////////////////////////////////////////////

pub type A2SPlayer = SimplePacket<_A2SPlayer>;

impl A2SPlayer {
    pub fn new(challenge: Option<i32>) -> Self {
        Self {
            header: PacketHeader::SimplePacket,
            payload: _A2SPlayer::new(challenge),
        }
    }
}

#[derive(Debug, Clone)]
#[repr(C)]
pub struct _A2SPlayer {
    pub header: QueryHeader,
    pub challenge: i32,
}

impl _A2SPlayer {
    pub fn new(challenge: Option<i32>) -> Self {
        Self {
            header: QueryHeader::A2SPlayer,
            challenge: challenge.unwrap_or(-1), // -1 is the value to use to request a challenge
        }
    }
}

impl QueryPacket for _A2SPlayer {}

impl TryFrom<Vec<u8>> for _A2SPlayer {
    type Error = std::io::Error;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        let header = match QueryHeader::try_from(value[0]) {
            Ok(header) => header,
            Err(_) => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Invalid header",
                ))
            }
        };

        if header != QueryHeader::A2SPlayerReply {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Invalid header for A2SPlayerReply: {:?}", header),
            ));
        }

        let challenge = i32::from_le_bytes(match value[1..5].try_into() {
            Ok(challenge) => challenge,
            Err(_) => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Invalid challenge",
                ))
            }
        });

        Ok(Self { header, challenge })
    }
}

impl Into<Vec<u8>> for _A2SPlayer {
    fn into(self) -> Vec<u8> {
        let mut data: Vec<u8> = Vec::new();
        data.push(self.header.into());
        data.extend_from_slice(&self.challenge.to_le_bytes());
        data
    }
}

///////////////////////////////////////////////////////////////////////////
/// A2S_PLAYER_RESPONSE
/// https://developer.valvesoftware.com/wiki/Server_queries#A2S_PLAYER
///////////////////////////////////////////////////////////////////////////

pub type A2SPlayerReply = SimplePacket<_A2SPlayerReply>;

#[derive(Debug, Clone)]
#[repr(C)]
pub struct A2SPlayerInfo {
    pub index: u8,
    pub name: String,
    pub score: i32,
    pub duration: f32,
}

#[derive(Debug, Clone)]
#[repr(C)]
pub struct _A2SPlayerReply {
    pub header: QueryHeader,
    pub num_players: u8,
    pub players: Vec<A2SPlayerInfo>,
}

impl QueryPacket for _A2SPlayerReply {}

impl TryFrom<Vec<u8>> for _A2SPlayerReply {
    type Error = std::io::Error;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        let mut data = value.as_slice();

        let header = match QueryHeader::try_from(data[0]) {
            Ok(header) => header,
            Err(_) => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Invalid header",
                ))
            }
        };
        data = &data[1..];

        if header != QueryHeader::A2SPlayerReply {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Invalid header for A2SPlayerReply: {:?}", header),
            ));
        }

        let num_players = match u8::try_from(data[0]) {
            Ok(players) => players,
            Err(_) => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Invalid player count",
                ))
            }
        };
        data = &data[1..];

        let mut players = Vec::with_capacity(num_players as usize);

        for _ in 0..num_players {
            let index = match u8::try_from(data[0]) {
                Ok(index) => index,
                Err(_) => {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "Invalid player index",
                    ))
                }
            };
            data = &data[1..];

            let name = match String::from_utf8(data.iter().take_while(|&&c| c != 0).cloned().collect()) {
                Ok(name) => name,
                Err(_) => {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "Invalid name",
                    ))
                }
            };
            data = &data[name.len() + 1..];

            let score = i32::from_le_bytes(match data[0..4].try_into() {
                Ok(score) => score,
                Err(e) => {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        format!("Invalid player score: {}", e),
                    ))
                }
            });
            data = &data[4..];

            let duration = f32::from_le_bytes(match data[0..4].try_into() {
                Ok(duration) => duration,
                Err(e) => {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        format!("Invalid player duration: {}", e),
                    ))
                }
            });
            data = &data[4..];

            players.push(A2SPlayerInfo {
                index,
                name,
                score,
                duration,
            });
        }

        Ok(Self {
            header,
            num_players,
            players,
        })
    }
}

impl Into<Vec<u8>> for _A2SPlayerReply {
    fn into(self) -> Vec<u8> {
        let mut data: Vec<u8> = Vec::new();
        data.push(self.header.into());
        data.push(self.num_players);
        for player in self.players {
            data.push(player.index);
            data.extend_from_slice(player.name.as_bytes());
            data.push(0);
            data.extend_from_slice(&player.score.to_le_bytes());
            data.extend_from_slice(&player.duration.to_le_bytes());
        }
        data
    }
}
