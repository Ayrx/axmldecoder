//!Decoder for the binary XML format used by Android.
//!
//!This library implements the minimal amount of parsing required obtain
//!useful information from a binary `AndroidManifest.xml`. It does not
//!support parsing generic binary XML documents and does not have
//!support for decoding resource identifiers. In return, the compiled
//!footprint of the library is _much_ lighter as it does not have to
//!link in Android's `resources.arsc` file.
//!
//!For a full-featured Rust binary XML parser,
//![abxml-rs](https://github.com/SUPERAndroidAnalyzer/abxml-rs)
//!is highly recommended if it is acceptable to link a 30MB `resources.arsc`
//!file into your compiled binary.
//!
//!The following features will be implemented in the future:
//!* Support for binary XML documents using UTF-8 encoding for the string pool
//!* Support for binary XML documents with UTF-16 strings longer than 32767
//!characters
//!
//!Please file an issue with the relevant binary `AndroidManifest.xml` if
//!if any issues are encountered.

mod binaryxml;
mod resource_value;
mod stringpool;
mod xml;

use byteorder::ByteOrder;
use byteorder::LittleEndian;
use std::convert::TryFrom;
use std::io::{Read, Seek};
use thiserror::Error;

use crate::binaryxml::{
    XmlCdata, XmlElement, XmlEndElement, XmlEndNameSpace, XmlStartElement, XmlStartNameSpace,
};
use crate::stringpool::StringPool;

pub use crate::xml::{Element, XmlDocument};

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("invalid file")]
    InvalidFile,

    #[error("i/o error")]
    IoError(std::io::Error),
}

#[repr(u16)]
#[derive(Debug, PartialEq, Clone)]
enum ResourceType {
    // Might be an error in the code?
    // NullType = 0x001,
    StringPool = 0x0001,
    Table = 0x0002,
    Xml = 0x0003,

    // Might be an error in the code?
    // XmlFirstChunk = 0x0100,
    XmlStartNameSpace = 0x0100,
    XmlEndNameSpace = 0x101,
    XmlStartElement = 0x0102,
    XmlEndElement = 0x0103,
    XmlCdata = 0x0104,
    XmlLastChunk = 0x017f,
    XmlResourceMap = 0x0180,

    TablePackage = 0x0200,
    TableType = 0x0201,
    TableTypeSpec = 0x0202,
    TableLibrary = 0x0203,
}

impl TryFrom<u16> for ResourceType {
    type Error = ();

    fn try_from(v: u16) -> Result<Self, Self::Error> {
        match v {
            x if x == Self::StringPool as u16 => Ok(Self::StringPool),
            x if x == Self::Table as u16 => Ok(Self::Table),
            x if x == Self::Xml as u16 => Ok(Self::Xml),
            x if x == Self::XmlStartNameSpace as u16 => Ok(Self::XmlStartNameSpace),
            x if x == Self::XmlEndNameSpace as u16 => Ok(Self::XmlEndNameSpace),
            x if x == Self::XmlStartElement as u16 => Ok(Self::XmlStartElement),
            x if x == Self::XmlEndElement as u16 => Ok(Self::XmlEndElement),
            x if x == Self::XmlCdata as u16 => Ok(Self::XmlCdata),
            x if x == Self::XmlLastChunk as u16 => Ok(Self::XmlLastChunk),
            x if x == Self::XmlResourceMap as u16 => Ok(Self::XmlResourceMap),
            x if x == Self::TablePackage as u16 => Ok(Self::TablePackage),
            x if x == Self::TableType as u16 => Ok(Self::TableType),
            x if x == Self::TableTypeSpec as u16 => Ok(Self::TableTypeSpec),
            x if x == Self::TableLibrary as u16 => Ok(Self::TableLibrary),
            _ => Err(()),
        }
    }
}

#[repr(C)]
#[derive(Clone, Debug)]
struct ChunkHeader {
    typ: ResourceType,
    header_size: u16,
    size: u32,
}

impl ChunkHeader {
    fn read_from_file<F: Read + Seek>(input: &mut F) -> Result<Self, ParseError> {
        let typ = ResourceType::try_from(read_u16(input)?).map_err(|_| ParseError::InvalidFile)?;
        let header_size = read_u16(input)?;
        let size = read_u32(input)?;

        let header = ChunkHeader {
            typ,
            header_size,
            size,
        };

        Ok(header)
    }
}

///Parses an Android binary XML and returns a [XmlDocument] object.
///
///```rust
///use axmldecoder::parse;
///# use axmldecoder::ParseError;
///# let manifest_file = "examples/AndroidManifest.xml";
///let mut f = std::fs::File::open(manifest_file).unwrap();
///parse(&mut f)?;
///# Ok::<(), ParseError>(())
///```
pub fn parse<F: Read + Seek>(input: &mut F) -> Result<XmlDocument, ParseError> {
    let header = ChunkHeader::read_from_file(input)?;

    if header.typ != ResourceType::Xml {
        return Err(ParseError::InvalidFile);
    }

    let mut elements = Vec::new();
    let mut string_pool = None;
    let mut resource_map = None;

    loop {
        let header = ChunkHeader::read_from_file(input);
        if let Err(ParseError::IoError(_)) = &header {
            break;
        }
        let header = header?;

        match header.typ {
            ResourceType::StringPool => {
                string_pool = Some(StringPool::read_from_file(input, &header)?);
            }
            ResourceType::XmlResourceMap => {
                resource_map = Some(parse_resource_map(input, &header)?);
            }
            ResourceType::XmlStartNameSpace => {
                elements.push(XmlElement::XmlStartNameSpace(
                    XmlStartNameSpace::read_from_file(input, &header)?,
                ));
            }
            ResourceType::XmlEndNameSpace => {
                elements.push(XmlElement::XmlEndNameSpace(
                    XmlEndNameSpace::read_from_file(input, &header)?,
                ));
            }
            ResourceType::XmlStartElement => {
                elements.push(XmlElement::XmlStartElement(
                    XmlStartElement::read_from_file(input, &header)?,
                ));
            }
            ResourceType::XmlEndElement => {
                elements.push(XmlElement::XmlEndElement(XmlEndElement::read_from_file(
                    input, &header,
                )?));
            }
            ResourceType::XmlCdata => {
                elements.push(XmlElement::XmlCdata(XmlCdata::read_from_file(
                    input, &header,
                )?));
            }
            _ => return Err(ParseError::InvalidFile),
        }
    }

    Ok(XmlDocument::new(
        elements,
        string_pool.unwrap(),
        resource_map.unwrap(),
    ))
}

fn parse_resource_map<F: Read + Seek>(
    input: &mut F,
    header: &ChunkHeader,
) -> Result<Vec<u32>, ParseError> {
    let id_count = (header.size - header.header_size as u32) / 4;

    let mut ids = Vec::with_capacity(id_count as usize);
    for _ in 0..id_count {
        ids.push(read_u32(input)?);
    }

    Ok(ids)
}

fn read_u8<F: Read + Seek>(input: &mut F) -> Result<u8, ParseError> {
    let mut buf = [0; 1];
    input.read_exact(&mut buf).map_err(ParseError::IoError)?;

    Ok(buf[0])
}

fn read_u16<F: Read + Seek>(input: &mut F) -> Result<u16, ParseError> {
    let mut buf = [0; 2];
    input.read_exact(&mut buf).map_err(ParseError::IoError)?;

    Ok(LittleEndian::read_u16(&buf))
}

fn read_u32<F: Read + Seek>(input: &mut F) -> Result<u32, ParseError> {
    let mut buf = [0; 4];
    input.read_exact(&mut buf).map_err(ParseError::IoError)?;

    Ok(LittleEndian::read_u32(&buf))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::path::PathBuf;

    #[test]
    fn test_parse() {
        let mut example = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        example.push("examples");
        example.push(".tmp");

        example.set_file_name("AndroidManifest.xml");
        let mut f = File::open(example).unwrap();
        parse(&mut f).unwrap();
    }
}
