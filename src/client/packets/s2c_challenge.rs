use super::{QueryHeader, SourceChallenge, SourceQueryResponse};

#[derive(Debug, Clone, PartialEq)]
pub struct S2CChallenge {
    pub header: QueryHeader,
    pub challenge: SourceChallenge,
}

impl S2CChallenge {
    pub fn new(challenge: SourceChallenge) -> Self {
        Self {
            header: QueryHeader::S2CChallenge,
            challenge,
        }
    }
}

impl SourceQueryResponse for S2CChallenge {
    const SIZE: usize = std::mem::size_of::<Self>();

    fn packet_header() -> QueryHeader {
        QueryHeader::S2CChallenge
    }
}

impl Into<Vec<u8>> for S2CChallenge {
    fn into(self) -> Vec<u8> {
        let mut data: Vec<u8> = Vec::with_capacity(Self::SIZE);

        let header: u8 = self.header.into();
        data.push(header);
        data.extend(self.challenge.to_le_bytes().iter());

        data
    }
}

impl From<&[u8]> for S2CChallenge {
    fn from(buf: &[u8]) -> Self {
        let header = QueryHeader::try_from(buf[0]).unwrap();
        let challenge = i32::from_le_bytes([buf[1], buf[2], buf[3], buf[4]]);

        Self { header, challenge }
    }
}
