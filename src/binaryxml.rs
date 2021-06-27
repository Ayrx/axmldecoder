use crate::stringpool::StringPool;
use deku::prelude::*;
use std::convert::TryFrom;
use std::rc::Rc;

#[derive(Debug, DekuRead)]
pub(crate) struct BinaryXmlDocument {
    pub(crate) header: ChunkHeader,
    pub(crate) string_pool: StringPool,
    pub(crate) resource_map: ResourceMap,
    #[deku(bytes_read = "header.size - 
            u32::try_from(header.header_size).unwrap() - 
            string_pool.header.chunk_header.size - 
            resource_map.header.size")]
    pub(crate) elements: Vec<XmlNode>,
}

#[repr(u16)]
#[derive(Debug, PartialEq, Clone, Copy, DekuRead, DekuWrite)]
#[deku(type = "u16")]
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
#[derive(Clone, Debug, DekuRead, DekuWrite)]
pub(crate) struct ChunkHeader {
    pub(crate) typ: ResourceType,
    pub(crate) header_size: u16,
    pub(crate) size: u32,
}

#[derive(Debug, DekuRead, DekuWrite)]
pub(crate) struct ResourceMap {
    pub(crate) header: ChunkHeader,
    #[deku(count = "(header.size - u32::from(header.header_size)) / 4")]
    pub(crate) resource_ids: Vec<u32>,
}

#[derive(Debug, DekuRead, DekuWrite)]
pub(crate) struct XmlNode {
    pub(crate) header: XmlNodeHeader,
    #[deku(ctx = "header.chunk_header.typ")]
    pub(crate) element: XmlNodeType,
}

#[allow(clippy::enum_variant_names)]
#[derive(Debug, DekuRead, DekuWrite)]
#[deku(ctx = "typ: ResourceType", id = "typ")]
pub(crate) enum XmlNodeType {
    #[deku(id = "ResourceType::XmlStartNameSpace")]
    XmlStartNameSpace(XmlStartNameSpace),
    #[deku(id = "ResourceType::XmlEndNameSpace")]
    XmlEndNameSpace(XmlEndNameSpace),
    #[deku(id = "ResourceType::XmlStartElement")]
    XmlStartElement(XmlStartElement),
    #[deku(id = "ResourceType::XmlEndElement")]
    XmlEndElement(XmlEndElement),
    #[deku(id = "ResourceType::XmlCdata")]
    XmlCdata(XmlCdata),
}

#[derive(Debug, DekuRead, DekuWrite)]
pub(crate) struct XmlNodeHeader {
    pub(crate) chunk_header: ChunkHeader,
    pub(crate) line_no: u32,
    pub(crate) comment: u32,
}

#[derive(Debug, DekuRead, DekuWrite)]
pub(crate) struct XmlStartNameSpace {
    pub(crate) prefix: u32,
    pub(crate) uri: u32,
}

#[derive(Debug, DekuRead, DekuWrite)]
pub(crate) struct XmlEndNameSpace {
    pub(crate) prefix: u32,
    pub(crate) uri: u32,
}

#[derive(Debug, DekuRead, DekuWrite)]
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

#[derive(Debug, DekuRead, DekuWrite)]
pub(crate) struct ResourceValue {
    pub(crate) size: u16,
    pub(crate) res: u8,
    pub(crate) data_type: ResourceValueType,
    pub(crate) data: u32,
}

impl ResourceValue {
    pub(crate) fn get_value(&self, string_pool: &StringPool) -> Rc<String> {
        match &self.data_type {
            ResourceValueType::String => string_pool
                .get(usize::try_from(self.data).unwrap())
                .unwrap(),
            ResourceValueType::Dec => Rc::new(self.data.to_string()),
            ResourceValueType::Hex => Rc::new(format!("0x{}", self.data)),
            ResourceValueType::Boolean => Rc::new(match self.data {
                0 => "false".to_string(),
                _ => "true".to_string(),
            }),
            n => Rc::new(format!("ResourceValueType::{:?}/{}", n, self.data)),
        }
    }
}

#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
#[deku(type = "u8")]
pub(crate) enum ResourceValueType {
    Null = 0x00,
    Reference = 0x01,
    Attribute = 0x02,
    String = 0x03,
    Float = 0x04,
    Dimension = 0x05,
    Fraction = 0x06,
    Dec = 0x10,
    Hex = 0x11,
    Boolean = 0x12,
    ColorArgb8 = 0x1c,
    ColorRgb8 = 0x1d,
    ColorArgb4 = 0x1e,
    ColorRgb4 = 0x1f,
}

#[derive(Debug, DekuRead, DekuWrite)]
pub(crate) struct XmlAttribute {
    pub(crate) ns: u32,
    pub(crate) name: u32,
    pub(crate) raw_value: u32,
    pub(crate) typed_value: ResourceValue,
}

#[derive(Debug, DekuRead, DekuWrite)]
pub(crate) struct XmlStartElement {
    pub(crate) attr_ext: XmlAttrExt,
    #[deku(count = "attr_ext.attribute_count")]
    pub(crate) attributes: Vec<XmlAttribute>,
}

#[derive(Debug, DekuRead, DekuWrite)]
pub(crate) struct XmlEndElement {
    pub(crate) ns: u32,
    pub(crate) name: u32,
}

#[derive(Debug, DekuRead, DekuWrite)]
pub(crate) struct XmlCdata {
    pub(crate) data: u32,
    pub(crate) typed_data: ResourceValue,
}
