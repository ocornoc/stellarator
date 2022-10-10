use nom::{IResult, bytes::complete::*};

mod rle;

pub const MAGIC_BYTES0: &[u8] = b"Stella BINARY ";
pub const MAGIC_BYTES1: &[u8] = &[0xB8, 0xA5, 0xA9, 0x6A];

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct HumanMetadata {
    pub metadata: Vec<u8>,
    pub name: Option<String>,
    pub description: Option<String>,
}

impl HumanMetadata {
    fn parse_name_only(bytes: &[u8]) -> IResult<&[u8], Self> {
        let (bytes, metadata) = tag(&[0x00])(bytes)?;
        let (bytes, name) = take_till(|byte| byte == 0x00)(bytes)?;
        let (bytes, _) = tag(&[0x00])(bytes)?;
        Ok((bytes, HumanMetadata {
            metadata: metadata.to_vec(),
            name: {
                let name = String::from_utf8_lossy(name).to_string();
                if name.is_empty() {
                    None
                } else {
                    Some(name)
                }
            },
            description: None,
        }))
    }

    fn parse_description_only(bytes: &[u8]) -> IResult<&[u8], Self> {
        let (bytes, _) = tag(&[0x37, 0x00])(bytes)?;
        let (bytes, description) = take_till(|byte| byte == 0x00)(bytes)?;
        let (bytes, _) = tag(&[0x00])(bytes)?;
        Ok((bytes, HumanMetadata {
            metadata: Vec::new(),
            name: None,
            description: {
                let description = String::from_utf8_lossy(description).to_string();
                if description.is_empty() {
                    None
                } else {
                    Some(description)
                }
            },
        }))
    }

    fn parse_name_description(bytes: &[u8]) -> IResult<&[u8], Self> {
        let (mut real_bytes, mut human) = HumanMetadata::parse_name_only(bytes)?;
        if let Ok((bytes, human_desc)) = HumanMetadata::parse_description_only(bytes) {
            real_bytes = bytes;
            human.description = human_desc.description;
        }
        Ok((real_bytes, human))
    }

    fn parse(bytes: &[u8]) -> IResult<&[u8], Self> {
        let (bytes, _) = tag(&[0x33])(bytes)?;
        let (bytes, mut human) = HumanMetadata::parse_name_description(bytes)?;
        human.metadata.insert(0, 0x33);
        Ok((bytes, human))
    }
}

impl std::fmt::Display for HumanMetadata {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Human metadata section: ")?;
        write_bytes(&self.metadata, f)?;
        if let Some(name) = self.name.as_ref() {
            writeln!(f, "Name: {name}")?;
        }
        if let Some(description) = self.description.as_ref() {
            writeln!(f, "Description: {description}")?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct StelData {
    pub metadata0: Vec<u8>,
    pub metadata1: Vec<u8>,
    pub human_metadata: Option<HumanMetadata>,
    raw_leftover: Vec<u8>,
}

impl StelData {
    pub fn parse(bytes: &[u8]) -> Result<Self, nom::Err<nom::error::Error<&[u8]>>> {
        let mut human_metadata = None;
        // match and strip the magic bytes
        let (bytes, _) = tag(MAGIC_BYTES0)(bytes)?;
        let (bytes, metadata0) = take(14_usize)(bytes)?;
        let (bytes, _) = tag(MAGIC_BYTES1)(bytes)?;
        let (mut bytes, metadata1) = take_till(|byte| byte == 0x33)(bytes)?;
        if let Ok((new_bytes, new_human_metadata)) = HumanMetadata::parse(bytes) {
            bytes = new_bytes;
            human_metadata = Some(new_human_metadata);
        }
        Ok(StelData {
            metadata0: metadata0.to_vec(),
            metadata1: metadata1.to_vec(),
            human_metadata,
            raw_leftover: bytes.to_vec(),
        })
    }
}

impl std::fmt::Display for StelData {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "First metadata section: ")?;
        write_bytes(&self.metadata0, f)?;
        write!(f, "Second metadata section: ")?;
        write_bytes(&self.metadata1, f)?;
        if let Some(human) = self.human_metadata.as_ref() {
            write!(f, "{human}")?;
        } else {
            writeln!(f, "Failed to parse human metadata.")?;
        }
        writeln!(f, "Leftover:")?;
        write_bytes(&self.raw_leftover, f)
    }
}

fn write_bytes(bytes: &[u8], f: &mut std::fmt::Formatter) -> std::fmt::Result {
    for &byte in bytes {
        write!(f, "{byte:02X}")?;
    }

    writeln!(f)
}
