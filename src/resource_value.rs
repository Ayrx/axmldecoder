use deku::prelude::*;

use std::convert::TryFrom;
use std::rc::Rc;

use crate::stringpool::StringPool;

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
