use std::convert::TryFrom;
use std::io::{Read, Seek};
use std::rc::Rc;

use crate::stringpool::StringPool;
use crate::{read_u16, read_u32, read_u8, ParseError};

#[derive(Debug)]
pub(crate) struct ResourceValue {
    pub(crate) size: u16,
    pub(crate) res: u8,
    pub(crate) data_type: ResourceValueType,
    pub(crate) data: u32,
}

impl ResourceValue {
    pub(crate) fn read_from_file<F: Read + Seek>(input: &mut F) -> Result<Self, ParseError> {
        let size = read_u16(input)?;
        let res = read_u8(input)?;
        let data_type = ResourceValueType::try_from(read_u8(input)?).unwrap();
        let data = read_u32(input)?;

        Ok(Self {
            size,
            res,
            data_type,
            data,
        })
    }

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

#[repr(u8)]
#[derive(Debug, PartialEq)]
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

impl TryFrom<u8> for ResourceValueType {
    type Error = ();

    fn try_from(v: u8) -> Result<Self, Self::Error> {
        match v {
            x if x == Self::Null as u8 => Ok(Self::Null),
            x if x == Self::Reference as u8 => Ok(Self::Reference),
            x if x == Self::Attribute as u8 => Ok(Self::Attribute),
            x if x == Self::String as u8 => Ok(Self::String),
            x if x == Self::Float as u8 => Ok(Self::Float),
            x if x == Self::Dimension as u8 => Ok(Self::Dimension),
            x if x == Self::Fraction as u8 => Ok(Self::Fraction),
            x if x == Self::Dec as u8 => Ok(Self::Dec),
            x if x == Self::Hex as u8 => Ok(Self::Hex),
            x if x == Self::Boolean as u8 => Ok(Self::Boolean),
            x if x == Self::ColorArgb8 as u8 => Ok(Self::ColorArgb8),
            x if x == Self::ColorRgb8 as u8 => Ok(Self::ColorRgb8),
            x if x == Self::ColorArgb4 as u8 => Ok(Self::ColorArgb4),
            x if x == Self::ColorRgb4 as u8 => Ok(Self::ColorRgb4),
            _ => Err(()),
        }
    }
}
