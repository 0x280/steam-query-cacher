
///////////////////////////////////////////////////////////////////////////
/// A2S_RULES
/// https://developer.valvesoftware.com/wiki/Server_queries#A2S_RULES
///////////////////////////////////////////////////////////////////////////

use super::{headers::{QueryHeader, PacketHeader}, packet::{SimplePacket, QueryPacket}};

pub type A2SRules = SimplePacket<_A2SRules>;

impl A2SRules {
    pub fn new(challenge: Option<i32>) -> Self {
        Self {
            header: PacketHeader::SimplePacket,
            payload: _A2SRules::new(challenge),
        }
    }
}

#[derive(Debug, Clone)]
#[repr(C)]
pub struct _A2SRules {
    pub header: QueryHeader,
    pub challenge: i32,
}

impl _A2SRules {
    pub fn new(challenge: Option<i32>) -> Self {
        Self {
            header: QueryHeader::A2SRules,
            challenge: challenge.unwrap_or(-1), // -1 is the value to use to request a challenge
        }
    }
}

impl QueryPacket for _A2SRules {}

impl TryFrom<Vec<u8>> for _A2SRules {
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

        if header != QueryHeader::A2SRules {
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

impl Into<Vec<u8>> for _A2SRules {
    fn into(self) -> Vec<u8> {
        let mut data: Vec<u8> = Vec::new();
        data.push(self.header.into());
        data.extend_from_slice(&self.challenge.to_le_bytes());
        data
    }
}

///////////////////////////////////////////////////////////////////////////
/// A2S_RULES
/// https://developer.valvesoftware.com/wiki/Server_queries#A2S_RULES
///////////////////////////////////////////////////////////////////////////

pub type A2SRulesReply = SimplePacket<_A2SRulesReply>;

#[derive(Debug, Clone)]
#[repr(C)]
pub struct A2SRule {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone)]
#[repr(C)]
pub struct _A2SRulesReply {
    pub header: QueryHeader,
    pub num_rules: i16,
    pub rules: Vec<A2SRule>,
}

impl QueryPacket for _A2SRulesReply {}

impl TryFrom<Vec<u8>> for _A2SRulesReply {
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

        if header != QueryHeader::A2SRulesReply {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Invalid header for A2SRulesReply: {:?}", header),
            ));
        }

        let num_rules = i16::from_le_bytes(match data[0..2].try_into() {
            Ok(num_rules) => num_rules,
            Err(_) => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Invalid num_rules",
                ))
            }
        });
        data = &data[2..];

        let mut rules: Vec<A2SRule> = Vec::with_capacity(num_rules as usize);

        for _ in 0..num_rules {
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

            let value = match String::from_utf8(data.iter().take_while(|&&c| c != 0).cloned().collect()) {
                Ok(value) => value,
                Err(_) => {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "Invalid value",
                    ))
                }
            };
            data = &data[value.len() + 1..];

            rules.push(A2SRule { name, value });
        }

        Ok(Self { header, num_rules, rules })
    }
}

impl Into<Vec<u8>> for _A2SRulesReply {
    fn into(self) -> Vec<u8> {
        let mut data: Vec<u8> = Vec::new();
        data.push(self.header.into());
        data.extend_from_slice(&self.num_rules.to_le_bytes());
        data
    }
}
