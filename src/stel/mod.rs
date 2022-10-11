use nom::{IResult, bytes::complete::*};

mod rle;

pub const MAGIC_BYTES0: &[u8] = b"Stella BINARY ";
pub const MAGIC_BYTES1: &[u8] = &[0xB8, 0xA5, 0xA9, 0x6A];

fn parse_c_str(bytes: &[u8]) -> IResult<&[u8], String> {
    let (bytes, s) = take_till(|byte| byte == 0x00)(bytes)?;
    let (bytes, _) = tag(&[0x00])(bytes)?;
    Ok((bytes, String::from_utf8_lossy(s).to_string()))
}

#[derive(Debug, PartialEq, Eq, Clone, Hash, Default)]
pub struct HumanMetadata {
    pub name: Option<String>,
    pub description: Option<String>,
    pub field3a: Option<String>,
    pub website_link: Option<String>,
}

impl HumanMetadata {
    fn parse_tagged_string<'a>(
        tag_bytes: &'a [u8],
    ) -> impl Fn(&'a [u8]) -> IResult<&[u8], String> + 'a {
        let tag = tag(tag_bytes);
        move |bytes| {
            let (bytes, _) = tag(bytes)?;
            parse_c_str(bytes)
        }
    }

    fn parse_name(bytes: &[u8]) -> IResult<&[u8], Self> {
        let (bytes, name) = HumanMetadata::parse_tagged_string(&[0x33, 0x00])(bytes)?;
        Ok((bytes, HumanMetadata {
            name: Some(name),
            ..Default::default()
        }))
    }

    fn parse_description(bytes: &[u8]) -> IResult<&[u8], Self> {
        let (bytes, description) = HumanMetadata::parse_tagged_string(&[0x37, 0x00])(bytes)?;
        Ok((bytes, HumanMetadata {
            description: Some(description),
            ..Default::default()
        }))
    }

    fn parse_field3a(bytes: &[u8]) -> IResult<&[u8], Self> {
        let (bytes, field3a) = HumanMetadata::parse_tagged_string(&[0x3A, 0x00])(bytes)?;
        Ok((bytes, HumanMetadata {
            field3a: Some(field3a),
            ..Default::default()
        }))
    }

    fn parse_website_link(bytes: &[u8]) -> IResult<&[u8], Self> {
        let (bytes, website_link) = HumanMetadata::parse_tagged_string(&[0x8E, 0x00])(bytes)?;
        Ok((bytes, HumanMetadata {
            website_link: Some(website_link),
            ..Default::default()
        }))
    }

    fn parse(bytes: &[u8]) -> IResult<&[u8], Self> {
        let (mut bytes, mut human) = HumanMetadata::parse_name(bytes)?;
        if let Ok((new_bytes, new_human)) = HumanMetadata::parse_description(bytes) {
            bytes = new_bytes;
            human.description = new_human.description;
        }
        if let Ok((new_bytes, new_human)) = HumanMetadata::parse_field3a(bytes) {
            bytes = new_bytes;
            human.field3a = new_human.field3a;
        }
        if let Ok((new_bytes, new_human)) = HumanMetadata::parse_website_link(bytes) {
            bytes = new_bytes;
            human.website_link = new_human.website_link;
        }
        Ok((bytes, human))
    }
}

impl std::fmt::Display for HumanMetadata {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        writeln!(f, "Human metadata section: ")?;
        if let Some(name) = self.name.as_ref() {
            writeln!(f, "Name: {name}")?;
        }
        if let Some(description) = self.description.as_ref() {
            writeln!(f, "Description: {description}")?;
        }
        if let Some(field3a) = self.field3a.as_ref() {
            writeln!(f, "Field 3A: {field3a}")?;
        }
        if let Some(website_link) = self.website_link.as_ref() {
            writeln!(f, "Website link: {website_link}")?;
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
