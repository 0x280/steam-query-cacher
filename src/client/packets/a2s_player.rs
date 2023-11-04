use super::{QueryHeader, SourceChallenge, SourceQueryRequest};

#[derive(Debug, Clone, PartialEq, Eq)]
#[repr(C)]
pub struct A2SPlayer {
    pub header: QueryHeader,
    pub challenge: Option<SourceChallenge>,
}

impl A2SPlayer {
    pub fn with_challenge(challenge: SourceChallenge) -> Self {
        Self {
            header: QueryHeader::A2SPlayer,
            challenge: Some(challenge),
        }
    }
}

impl SourceQueryRequest for A2SPlayer {
    const SIZE: usize = 1;

    fn new() -> Self {
        Self {
            header: QueryHeader::A2SPlayer,
            challenge: None,
        }
    }

    fn set_challenge(&mut self, challenge: SourceChallenge) {
        self.challenge = Some(challenge);
    }
}

impl Into<Vec<u8>> for A2SPlayer {
    fn into(self) -> Vec<u8> {
        let mut data: Vec<u8> = Vec::with_capacity(Self::SIZE);

        let header: u8 = self.header.into();
        data.push(header);

        let challenge = self.challenge.unwrap_or(-1);
        data.extend(challenge.to_le_bytes().iter());

        data
    }
}

impl TryFrom<&[u8]> for A2SPlayer {
    type Error = std::io::Error;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if value.len() < Self::SIZE {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid A2SPlayer packet size",
            ));
        }

        let header: QueryHeader = match QueryHeader::try_from(value[0]) {
            Ok(header) => header,
            Err(_) => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Invalid query header",
                ))
            }
        };
        if header != QueryHeader::A2SPlayer {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid header",
            ));
        }

        let challenge = if value.len() > 1 {
            Some(SourceChallenge::from_le_bytes([value[1], value[2], value[3], value[4]]))
        } else {
            None
        };

        Ok(Self { header, challenge })
    }
}
