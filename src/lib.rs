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
//!Please file an issue with the relevant binary `AndroidManifest.xml` if
//!if any issues are encountered.

mod binaryxml;
mod resource_value;
mod stringpool;
mod xml;

use byteorder::ByteOrder;
use byteorder::LittleEndian;
use std::io::{Read, Seek};
use thiserror::Error;

use crate::binaryxml::{
    parse_resource_map, ChunkHeader, ResourceType, XmlCdata, XmlElement, XmlEndElement,
    XmlEndNameSpace, XmlStartElement, XmlStartNameSpace,
};
use crate::stringpool::StringPool;

pub use crate::xml::{Cdata, Element, Node, XmlDocument};

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("invalid file")]
    InvalidFile,

    #[error("missing StringPool chunk")]
    MissingStringPoolChunk,

    #[error("missing ResourceMap chunk")]
    MissingResourceMapChunk,

    #[error("StringPool missing index: {0}")]
    StringNotFound(u32),

    #[error("Namespace missing: {0}")]
    NamespaceNotFound(String),

    #[error("ResourceMap missing index: {0}")]
    ResourceIdNotFound(u32),

    #[error("Unknown resource string: {0}")]
    UnknownResourceString(u32),

    #[error(transparent)]
    Utf8StringParseError(std::string::FromUtf8Error),

    #[error(transparent)]
    Utf16StringParseError(std::string::FromUtf16Error),

    #[error(transparent)]
    IoError(std::io::Error),
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

    XmlDocument::new(
        elements,
        string_pool.ok_or(ParseError::MissingStringPoolChunk)?,
        resource_map.ok_or(ParseError::MissingResourceMapChunk)?,
    )
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
        let mut examples = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        examples.push("examples");

        for entry in std::fs::read_dir(examples).unwrap() {
            let entry = entry.unwrap();
            let mut f = File::open(entry.path()).unwrap();
            parse(&mut f).expect(&format!("{} failed to parse", entry.path().display()));
        }
    }
}
