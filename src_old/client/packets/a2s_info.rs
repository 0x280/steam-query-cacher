use super::{packet::{SimplePacket, QueryPacket}, headers::QueryHeader};

///////////////////////////////////////////////////////////////////////////
/// A2S_INFO
/// https://developer.valvesoftware.com/wiki/Server_queries#A2S_INFO
///////////////////////////////////////////////////////////////////////////

pub type A2SInfo = SimplePacket<_A2SInfo>;

impl A2SInfo {
    pub fn new(challenge: Option<i32>) -> Self {
        Self::construct(_A2SInfo::new(challenge))
    }
}

#[derive(Debug, Clone)]
#[repr(C)]
pub struct _A2SInfo {
    pub header: QueryHeader,
    pub payload: String,
    pub challenge: Option<i32>,
}

impl _A2SInfo {
    pub fn new(challenge: Option<i32>) -> Self {
        Self {
            header: QueryHeader::A2SInfo,
            payload: "Source Engine Query".to_string(),
            challenge,
        }
    }
}

impl QueryPacket for _A2SInfo {}

impl TryFrom<Vec<u8>> for _A2SInfo {
    type Error = std::io::Error;

    fn try_from(data: Vec<u8>) -> Result<Self, Self::Error> {
        if data.len() != Self::SIZE && data.len() != Self::SIZE - 4 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid packet size",
            ));
        }

        let header = match QueryHeader::try_from(data[0] as u8) {
            Ok(header) => header,
            Err(_) => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Invalid query header",
                ))
            }
        };
        if header != QueryHeader::A2SInfo {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Invalid packet header: {:?}", header),
            ));
        }
        let payload = match String::from_utf8(data[1..].to_vec()) {
            Ok(payload) => payload,
            Err(_) => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Invalid payload",
                ))
            }
        };
        Ok(Self {
            header,
            payload,
            challenge: None,
        })
    }
}

impl Into<Vec<u8>> for _A2SInfo {
    fn into(self) -> Vec<u8> {
        let mut data = Vec::with_capacity(Self::SIZE);
        let header: u8 = self.header.into();
        data.push(header);
        data.extend(self.payload.as_bytes());
        data.push(0);
        if let Some(challenge) = self.challenge {
            data.extend(&challenge.to_le_bytes());
        }
        data
    }
}

///////////////////////////////////////////////////////////////////////////
/// A2S_INFO_REPLY
/// https://developer.valvesoftware.com/wiki/Server_queries#A2S_INFO
///////////////////////////////////////////////////////////////////////////

pub type A2SInfoReply = SimplePacket<_A2SInfoReply>;

#[derive(Debug, Clone)]
#[repr(C)]
pub struct _A2SInfoReply {
    pub header: QueryHeader,
    pub protocol: u8,
    pub name: String,
    pub map: String,
    pub folder: String,
    pub game: String,
    pub id: i16,
    pub players: u8,
    pub max_players: u8,
    pub bots: u8,
    pub server_type: u8,
    pub environment: u8,
    pub visibility: u8,
    pub vac: u8,
    pub version: String,
    pub edf: u8,
    pub port: Option<i16>,
    pub steam_id: Option<i64>,
    pub source_tv_port: Option<i16>,
    pub source_tv_name: Option<String>,
    pub keywords: Option<String>,
    pub game_id: Option<i64>,
}

impl QueryPacket for _A2SInfoReply {}

impl TryFrom<Vec<u8>> for _A2SInfoReply {
    type Error = std::io::Error;

    fn try_from(data: Vec<u8>) -> Result<Self, Self::Error> {
        let header = match QueryHeader::try_from(data[0] as u8) {
            Ok(header) => header,
            Err(_) => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Invalid query header",
                ))
            }
        };
        if header != QueryHeader::A2SInfoReply {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Invalid packet header: {:?}", header),
            ));
        }

        let mut data = &data[1..];

        let protocol: u8 = data[0];
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

        let map = match String::from_utf8(data.iter().take_while(|&&c| c != 0).cloned().collect()) {
            Ok(map) => map,
            Err(_) => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Invalid map",
                ))
            }
        };
        data = &data[map.len() + 1..];

        let folder = match String::from_utf8(data.iter().take_while(|&&c| c != 0).cloned().collect()) {
            Ok(folder) => folder,
            Err(_) => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Invalid folder",
                ))
            }
        };
        data = &data[folder.len() + 1..];

        let game = match String::from_utf8(data.iter().take_while(|&&c| c != 0).cloned().collect()) {
            Ok(game) => game,
            Err(_) => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Invalid game",
                ))
            }
        };
        data = &data[game.len() + 1..];

        let id = i16::from_le_bytes([data[0], data[1]]);
        data = &data[2..];

        let players = data[0];
        data = &data[1..];

        let max_players = data[0];
        data = &data[1..];

        let bots = data[0];
        data = &data[1..];

        let server_type = data[0];
        data = &data[1..];

        let environment = data[0];
        data = &data[1..];

        let visibility = data[0];
        data = &data[1..];

        let vac = data[0];
        data = &data[1..];

        let version = match String::from_utf8(data.iter().take_while(|&&c| c != 0).cloned().collect()) {
            Ok(version) => version,
            Err(_) => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Invalid version",
                ))
            }
        };
        data = &data[version.len() + 1..];

        let edf = data[0];
        data = &data[1..];

        let mut port = None;
        let mut steam_id = None;
        let mut source_tv_port = None;
        let mut source_tv_name = None;
        let mut keywords = None;
        let mut game_id = None;

        if edf & 0x80 != 0 {
            port = Some(i16::from_le_bytes([data[0], data[1]]));
            data = &data[2..];
        }

        if edf & 0x10 != 0 {
            steam_id = Some(i64::from_le_bytes([
                data[0], data[1], data[2], data[3], data[4], data[5], data[6], data[7],
            ]));
            data = &data[8..];
        }

        if edf & 0x40 != 0 {
            source_tv_port = Some(i16::from_le_bytes([data[0], data[1]]));
            data = &data[2..];

            source_tv_name = match String::from_utf8(data.iter().take_while(|&&c| c != 0).cloned().collect()) {
                Ok(source_tv_name) => Some(source_tv_name),
                Err(_) => {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "Invalid source tv name",
                    ))
                }
            };
            data = &data[source_tv_name.as_ref().unwrap().len() + 1..];
        }

        if edf & 0x20 != 0 {
            keywords = match String::from_utf8(data.iter().take_while(|&&c| c != 0).cloned().collect()) {
                Ok(keywords) => Some(keywords),
                Err(_) => {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "Invalid keywords",
                    ))
                }
            };
            data = &data[keywords.as_ref().unwrap().len() + 1..];
        }

        if edf & 0x02 != 0 {
            game_id = Some(i64::from_le_bytes([
                data[0], data[1], data[2], data[3], data[4], data[5], data[6], data[7],
            ]));
            // data = &data[8..];
        }

        Ok(Self {
            header,
            protocol,
            name,
            map,
            folder,
            game,
            id,
            players,
            max_players,
            bots,
            server_type,
            environment,
            visibility,
            vac,
            version,
            edf,
            port,
            steam_id,
            source_tv_port,
            source_tv_name,
            keywords,
            game_id,
        })
    }
}

impl Into<Vec<u8>> for _A2SInfoReply {
    fn into(self) -> Vec<u8> {
        let mut data = Vec::with_capacity(Self::SIZE);

        let header: u8 = self.header.into();
        data.push(header);

        data.push(self.protocol);

        data.extend(self.name.as_bytes());
        data.push(0);

        data.extend(self.map.as_bytes());
        data.push(0);

        data.extend(self.folder.as_bytes());
        data.push(0);

        data.extend(self.game.as_bytes());
        data.push(0);

        data.extend(&self.id.to_le_bytes());

        data.push(self.players);

        data.push(self.max_players);

        data.push(self.bots);

        data.push(self.server_type);

        data.push(self.environment);

        data.push(self.visibility);

        data.push(self.vac);

        data.extend(self.version.as_bytes());

        data.push(self.edf);

        if let Some(port) = self.port {
            data.extend(&port.to_le_bytes());
        }

        if let Some(steam_id) = self.steam_id {
            data.extend(&steam_id.to_le_bytes());
        }

        if let Some(source_tv_port) = self.source_tv_port {
            data.extend(&source_tv_port.to_le_bytes());
        }

        if let Some(source_tv_name) = self.source_tv_name {
            data.extend(source_tv_name.as_bytes());
            data.push(0);
        }

        if let Some(keywords) = self.keywords {
            data.extend(keywords.as_bytes());
            data.push(0);
        }

        if let Some(game_id) = self.game_id {
            data.extend(&game_id.to_le_bytes());
        }

        data
    }
}
