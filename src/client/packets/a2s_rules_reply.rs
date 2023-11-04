use super::{QueryHeader, SourceQueryResponse};

#[derive(Debug, Clone)]
#[repr(C)]
pub struct A2SRule {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone)]
#[repr(C)]
pub struct A2SRulesReply {
    pub header: QueryHeader,
    pub num_rules: i16,
    pub rules: Vec<A2SRule>,
}

impl SourceQueryResponse for A2SRulesReply {
    fn packet_header() -> QueryHeader {
        QueryHeader::A2SRulesReply
    }
}

impl Into<Vec<u8>> for A2SRulesReply {
    fn into(self) -> Vec<u8> {
        let mut data: Vec<u8> = Vec::with_capacity(1 + 1 + self.rules.len() * 10);

        let header: u8 = self.header.into();
        data.push(header);
        data.extend(self.num_rules.to_le_bytes());

        for rule in self.rules {
            data.extend(rule.name.into_bytes());
            data.push(0x00);
            data.extend(rule.value.into_bytes());
            data.push(0x00);
        }

        data
    }
}

impl TryFrom<&[u8]> for A2SRulesReply {
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

            let value =
                match String::from_utf8(data.iter().take_while(|&&c| c != 0).cloned().collect()) {
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

        Ok(Self {
            header,
            num_rules,
            rules,
        })
    }
}
