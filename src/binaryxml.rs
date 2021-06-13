use std::convert::TryFrom;
use std::io::{Read, Seek};

use crate::resource_value::ResourceValue;
use crate::{read_u16, read_u32, ParseError};
use num_enum::TryFromPrimitive;

#[repr(u16)]
#[derive(Debug, PartialEq, Clone, TryFromPrimitive)]
pub(crate) enum ResourceType {
    NullType = 0x000,
    StringPool = 0x0001,
    Table = 0x0002,
    Xml = 0x0003,
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

#[repr(C)]
#[derive(Clone, Debug)]
pub(crate) struct ChunkHeader {
    pub(crate) typ: ResourceType,
    pub(crate) header_size: u16,
    pub(crate) size: u32,
}

impl ChunkHeader {
    pub(crate) fn read_from_file<F: Read + Seek>(input: &mut F) -> Result<Self, ParseError> {
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

#[allow(clippy::enum_variant_names)]
#[derive(Debug)]
pub(crate) enum XmlElement {
    XmlStartNameSpace(XmlStartNameSpace),
    XmlEndNameSpace(XmlEndNameSpace),
    XmlStartElement(XmlStartElement),
    XmlEndElement(XmlEndElement),
    XmlCdata(XmlCdata),
}

pub(crate) fn parse_resource_map<F: Read + Seek>(
    input: &mut F,
    header: &ChunkHeader,
) -> Result<Vec<u32>, ParseError> {
    let id_count = (header.size - u32::from(header.header_size)) / 4;

    let mut ids = Vec::with_capacity(usize::try_from(id_count).unwrap());
    for _ in 0..id_count {
        ids.push(read_u32(input)?);
    }

    Ok(ids)
}

#[derive(Debug)]
pub(crate) struct XmlNodeHeader {
    pub(crate) chunk_header: ChunkHeader,
    pub(crate) line_no: u32,
    pub(crate) comment: u32,
}

impl XmlNodeHeader {
    fn read_from_file<F: Read + Seek>(
        input: &mut F,
        chunk_header: &ChunkHeader,
    ) -> Result<Self, ParseError> {
        let chunk_header = chunk_header.clone();
        let line_no = read_u32(input)?;
        let comment = read_u32(input)?;

        let header = Self {
            chunk_header,
            line_no,
            comment,
        };

        Ok(header)
    }
}

#[derive(Debug)]
pub(crate) struct XmlStartNameSpace {
    pub(crate) header: XmlNodeHeader,
    pub(crate) prefix: u32,
    pub(crate) uri: u32,
}

impl XmlStartNameSpace {
    pub(crate) fn read_from_file<F: Read + Seek>(
        input: &mut F,
        chunk_header: &ChunkHeader,
    ) -> Result<Self, ParseError> {
        let header = XmlNodeHeader::read_from_file(input, &chunk_header)?;
        let prefix = read_u32(input)?;
        let uri = read_u32(input)?;

        let node = Self {
            header,
            prefix,
            uri,
        };

        // println!("{:?}", node);
        Ok(node)
    }
}

#[derive(Debug)]
pub(crate) struct XmlEndNameSpace {
    pub(crate) header: XmlNodeHeader,
    pub(crate) prefix: u32,
    pub(crate) uri: u32,
}

impl XmlEndNameSpace {
    pub(crate) fn read_from_file<F: Read + Seek>(
        input: &mut F,
        chunk_header: &ChunkHeader,
    ) -> Result<Self, ParseError> {
        let header = XmlNodeHeader::read_from_file(input, &chunk_header)?;
        let prefix = read_u32(input)?;
        let uri = read_u32(input)?;

        let node = Self {
            header,
            prefix,
            uri,
        };

        // println!("{:?}", node);
        Ok(node)
    }
}

#[derive(Debug)]
pub(crate) struct XmlAttrExt {
    pub(crate) ns: u32,
    pub(crate) name: u32,
    pub(crate) attribute_start: u16,
    pub(crate) attribute_size: u16,
    pub(crate) attribute_count: u16,
    pub(crate) id_index: u16,
    pub(crate) class_index: u16,
    pub(crate) style_index: u16,
}

impl XmlAttrExt {
    fn read_from_file<F: Read + Seek>(input: &mut F) -> Result<Self, ParseError> {
        let ns = read_u32(input)?;
        let name = read_u32(input)?;

        let attribute_start = read_u16(input)?;
        let attribute_size = read_u16(input)?;
        let attribute_count = read_u16(input)?;
        let id_index = read_u16(input)?;
        let class_index = read_u16(input)?;
        let style_index = read_u16(input)?;

        let header = Self {
            ns,
            name,
            attribute_start,
            attribute_size,
            attribute_count,
            id_index,
            class_index,
            style_index,
        };

        Ok(header)
    }
}

#[derive(Debug)]
pub(crate) struct XmlAttribute {
    pub(crate) ns: u32,
    pub(crate) name: u32,
    pub(crate) typed_value: ResourceValue,
}

impl XmlAttribute {
    fn read_from_file<F: Read + Seek>(input: &mut F) -> Result<Self, ParseError> {
        let ns = read_u32(input)?;
        let name = read_u32(input)?;
        read_u32(input)?; // raw_value stored in the chunk. There does not seem to be any value in keeping it around since `typed_value` is available...
        let typed_value = ResourceValue::read_from_file(input)?;

        let attr = Self {
            ns,
            name,
            typed_value,
        };

        Ok(attr)
    }
}

#[derive(Debug)]
pub(crate) struct XmlStartElement {
    pub(crate) header: XmlNodeHeader,
    pub(crate) attr_ext: XmlAttrExt,
    pub(crate) attributes: Vec<XmlAttribute>,
}

impl XmlStartElement {
    pub(crate) fn read_from_file<F: Read + Seek>(
        input: &mut F,
        chunk_header: &ChunkHeader,
    ) -> Result<Self, ParseError> {
        let header = XmlNodeHeader::read_from_file(input, &chunk_header)?;
        let attr_ext = XmlAttrExt::read_from_file(input)?;

        let mut attributes = Vec::with_capacity(attr_ext.attribute_count.into());
        for _ in 0..attr_ext.attribute_count {
            attributes.push(XmlAttribute::read_from_file(input)?);
        }

        let node = Self {
            header,
            attr_ext,
            attributes,
        };

        // println!("{:?}", node);
        Ok(node)
    }
}

#[derive(Debug)]
pub(crate) struct XmlEndElement {
    pub(crate) header: XmlNodeHeader,
    pub(crate) ns: u32,
    pub(crate) name: u32,
}

impl XmlEndElement {
    pub(crate) fn read_from_file<F: Read + Seek>(
        input: &mut F,
        chunk_header: &ChunkHeader,
    ) -> Result<Self, ParseError> {
        let header = XmlNodeHeader::read_from_file(input, &chunk_header)?;
        let ns = read_u32(input)?;
        let name = read_u32(input)?;

        let node = Self { header, ns, name };

        // println!("{:?}", node);
        Ok(node)
    }
}

#[derive(Debug)]
pub(crate) struct XmlCdata {
    pub(crate) header: XmlNodeHeader,
    pub(crate) data: u32,
    pub(crate) typed_data: ResourceValue,
}

impl XmlCdata {
    pub(crate) fn read_from_file<F: Read + Seek>(
        input: &mut F,
        chunk_header: &ChunkHeader,
    ) -> Result<Self, ParseError> {
        let header = XmlNodeHeader::read_from_file(input, &chunk_header)?;
        let data = read_u32(input)?;
        let typed_data = ResourceValue::read_from_file(input)?;

        let node = Self {
            header,
            data,
            typed_data,
        };

        // println!("{:?}", node);
        Ok(node)
    }
}
