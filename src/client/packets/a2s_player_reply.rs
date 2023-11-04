use super::{QueryHeader, SourceQueryResponse};

#[derive(Debug, Clone, PartialEq)]
#[repr(C)]
pub struct A2SPlayerReply {
    pub header: QueryHeader,
    pub num_players: u8,
    pub players: Vec<A2SPlayerInfo>,
}

impl SourceQueryResponse for A2SPlayerReply {
    fn packet_header() -> QueryHeader {
        QueryHeader::A2SPlayerReply
    }
}

impl Into<Vec<u8>> for A2SPlayerReply {
    fn into(self) -> Vec<u8> {
        let mut data: Vec<u8> = Vec::with_capacity(1 + 1 + self.players.len() * 10);

        let header: u8 = self.header.into();
        data.push(header);
        data.push(self.num_players);

        for player in self.players {
            let bytes: Vec<u8> = player.into();
            data.extend(bytes);
        }

        data
    }
}

impl TryFrom<&[u8]> for A2SPlayerReply {
    type Error = std::io::Error;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let mut data = value;

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

            let name =
                match String::from_utf8(data.iter().take_while(|&&c| c != 0).cloned().collect()) {
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

#[derive(Debug, Clone, PartialEq)]
#[repr(C)]
pub struct A2SPlayerInfo {
    pub index: u8,
    pub name: String,
    pub score: i32,
    pub duration: f32,
}

impl Into<Vec<u8>> for A2SPlayerInfo {
    fn into(self) -> Vec<u8> {
        let mut data: Vec<u8> = Vec::with_capacity(1 + self.name.len() + 1 + 4 + 4);

        data.push(self.index);
        data.extend(self.name.as_bytes());
        data.push(0x00);
        data.extend(self.score.to_le_bytes().iter());
        data.extend(self.duration.to_le_bytes().iter());

        data
    }
}
