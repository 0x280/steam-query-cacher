use super::{QueryHeader, SourceChallenge, SourceQueryRequest};

pub const A2S_INFO_REQUEST_PAYLOAD: &str = "Source Engine Query\0";

#[derive(Debug, Clone, PartialEq, Eq)]
#[repr(C)]
pub struct A2SInfo {
    pub header: QueryHeader,
    pub payload: String,
    pub challenge: Option<SourceChallenge>,
}

impl A2SInfo {
    pub fn new() -> Self {
        Self {
            header: QueryHeader::A2SInfo,
            payload: A2S_INFO_REQUEST_PAYLOAD.to_string(),
            challenge: None,
        }
    }

    pub fn with_challenge(challenge: SourceChallenge) -> Self {
        Self {
            header: QueryHeader::A2SInfo,
            payload: A2S_INFO_REQUEST_PAYLOAD.to_string(),
            challenge: Some(challenge),
        }
    }
}

impl SourceQueryRequest for A2SInfo {
    const SIZE: usize = 1 + A2S_INFO_REQUEST_PAYLOAD.len();

    fn set_challenge(&mut self, challenge: SourceChallenge) {
        self.challenge = Some(challenge);
    }
}

impl Into<Vec<u8>> for A2SInfo {
    fn into(self) -> Vec<u8> {
        let mut data: Vec<u8> = Vec::with_capacity(Self::SIZE);

        let header: u8 = self.header.into();
        data.push(header);
        data.extend(self.payload.as_bytes());

        if let Some(challenge) = self.challenge {
            data.extend(challenge.to_le_bytes().iter());
        }

        data
    }
}

impl TryFrom<&[u8]> for A2SInfo {
    type Error = std::io::Error;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if value.len() < Self::SIZE {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid packet size",
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
        if header != QueryHeader::A2SInfo {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid header",
            ));
        }
        let mut data = &value[1..];

        let payload =
            match String::from_utf8(data.iter().take_while(|&&c| c != 0).cloned().collect()) {
                Ok(map) => map,
                Err(_) => {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "Invalid payload",
                    ))
                }
            };
        data = &data[payload.len() + 1..];

        let mut challenge: Option<SourceChallenge> = None;
        if data.len() >= 4 {
            challenge = Some(SourceChallenge::from_le_bytes([
                data[0], data[1], data[2], data[3],
            ]));
        }

        Ok(Self {
            header,
            payload,
            challenge,
        })
    }
}
