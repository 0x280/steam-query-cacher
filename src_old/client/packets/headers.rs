use num_enum::{IntoPrimitive, TryFromPrimitive};

#[derive(Debug, Clone, Copy, TryFromPrimitive, IntoPrimitive, PartialEq, Eq)]
#[repr(u8)]
pub enum QueryHeader {
    S2CChallenge = 0x41,
    A2SServerQueryGetChallenge = 0x57,
    A2SPlayer = 0x55,
    A2SPlayerReply = 0x44,
    A2SRules = 0x56,
    A2SRulesReply = 0x45,
    A2SInfo = 0x54,
    A2SInfoReply = 0x49,
    A2APing = 0x69,
    A2APingReply = 0x6A,
    GSInfo = 0x6D,
    GSInfoReply = 0x6E,
}

#[derive(Debug, Clone, Copy, TryFromPrimitive, IntoPrimitive, PartialEq, Eq)]
#[repr(i32)]
pub enum PacketHeader {
    SimplePacket = -1,
    MultiPacket = -2,
}

impl TryFrom<&[u8]> for PacketHeader {
    type Error = std::io::Error;

    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        let simple_packet: i32 = Self::SimplePacket.into();
        let multi_packet: i32 = Self::MultiPacket.into();
        let header = match i32::from_le_bytes(match data[0..4].try_into() {
            Ok(header) => header,
            Err(_) => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Invalid packet header",
                ))
            }
        }) {
            header if header == simple_packet => Self::SimplePacket,
            header if header == multi_packet => Self::MultiPacket,
            _ => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Invalid packet header",
                ))
            }
        };
        Ok(header)
    }
}
