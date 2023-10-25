use std::fmt::{Debug, Display};

use super::headers::PacketHeader;

pub trait QueryPacket: TryFrom<Vec<u8>> + Into<Vec<u8>> + Debug {
    const SIZE: usize = std::mem::size_of::<Self>();
}

#[derive(Debug)]
pub struct SimplePacket<T>
where
    T: QueryPacket,
{
    pub header: PacketHeader,
    pub payload: T,
}

impl<T> SimplePacket<T>
where
    T: QueryPacket,
{
    pub fn construct(payload: T) -> Self {
        Self {
            header: PacketHeader::SimplePacket,
            payload,
        }
    }
}

impl<T> TryFrom<Vec<u8>> for SimplePacket<T>
where
    T: QueryPacket,
    <T as TryFrom<Vec<u8>>>::Error: Debug + Display,
{
    type Error = std::io::Error;

    fn try_from(data: Vec<u8>) -> Result<Self, Self::Error> {
        let header: PacketHeader = PacketHeader::try_from(data.as_slice())?;
        if header != PacketHeader::SimplePacket {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Invalid packet header: {:?}", header),
            ));
        }
        let payload = match T::try_from(data[4..].to_vec()) {
            Ok(payload) => payload,
            Err(e) => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("Invalid payload: {}", e),
                ));
            }
        };
        Ok(Self { header, payload })
    }
}

impl<T> Into<Vec<u8>> for SimplePacket<T>
where
    T: QueryPacket,
{
    fn into(self) -> Vec<u8> {
        let mut data = Vec::with_capacity(T::SIZE + 1);
        let header: i32 = self.header.into();
        data.extend(header.to_le_bytes().iter());
        data.extend(self.payload.into());
        data
    }
}

impl<T> Into<Vec<u8>> for &SimplePacket<T> where T: QueryPacket {
    fn into(self) -> Vec<u8> {
        let mut data = Vec::with_capacity(T::SIZE + 1);
        let header: i32 = self.header.into();
        data.extend(header.to_le_bytes().iter());
        data.extend(self.payload.into());
        data
    }
}
