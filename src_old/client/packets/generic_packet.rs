use super::{packet::{SimplePacket, QueryPacket}, headers::QueryHeader};

pub type GenericPacket = SimplePacket<_GenericPacket>;
#[derive(Debug)]
#[repr(C)]
pub struct _GenericPacket {
    pub header: QueryHeader,
    pub payload: Vec<u8>,
}
impl QueryPacket for _GenericPacket {}
impl TryFrom<Vec<u8>> for _GenericPacket {
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
        let payload = data[1..].to_vec();
        Ok(Self { header, payload })
    }
}
impl Into<Vec<u8>> for _GenericPacket {
    fn into(self) -> Vec<u8> {
        let mut data = Vec::with_capacity(self.payload.len() + 1);
        let header: u8 = self.header.into();
        data.push(header);
        data.extend(self.payload);
        data
    }
}
