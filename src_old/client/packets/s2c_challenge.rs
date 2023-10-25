use super::{
    headers::{QueryHeader, PacketHeader},
    packet::{QueryPacket, SimplePacket},
};

pub type S2CChallenge = SimplePacket<_S2CChallenge>;

impl S2CChallenge {
    pub fn new(challenge: i32) -> Self {
        Self {
            header: PacketHeader::SimplePacket,
            payload: _S2CChallenge {
                header: QueryHeader::S2CChallenge,
                challenge,
            },
        }
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct _S2CChallenge {
    pub header: QueryHeader,
    pub challenge: i32,
}

impl _S2CChallenge {
    const SIZE: usize = 5;
}

impl QueryPacket for _S2CChallenge {}

impl TryFrom<Vec<u8>> for _S2CChallenge {
    type Error = std::io::Error;

    fn try_from(data: Vec<u8>) -> Result<Self, Self::Error> {
        if data.len() < Self::SIZE {
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
        let challenge: i32 = match i32::from_le_bytes([data[1], data[2], data[3], data[4]]) {
            0 => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Invalid challenge",
                ))
            }
            challenge => challenge,
        };
        Ok(Self { header, challenge })
    }
}

impl Into<Vec<u8>> for _S2CChallenge {
    fn into(self) -> Vec<u8> {
        let mut data = Vec::with_capacity(Self::SIZE);
        let header: u8 = self.header.into();
        data.push(header);
        data.extend(self.challenge.to_le_bytes().iter());
        data
    }
}
