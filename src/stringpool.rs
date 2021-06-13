use byteorder::ByteOrder;
use byteorder::LittleEndian;
use std::convert::TryFrom;
use std::io::{Read, Seek};
use std::rc::Rc;

use crate::binaryxml::ChunkHeader;
use crate::{read_u32, ParseError};

#[derive(Debug)]
pub(crate) struct StringPoolHeader {
    pub(crate) chunk_header: ChunkHeader,
    pub(crate) string_count: u32,
    pub(crate) style_count: u32,
    pub(crate) flags: u32,
    pub(crate) string_start: u32,
    pub(crate) style_start: u32,
}

impl StringPoolHeader {
    fn read_from_file<F: Read + Seek>(
        input: &mut F,
        chunk_header: &ChunkHeader,
    ) -> Result<Self, ParseError> {
        let chunk_header = chunk_header.clone();
        let string_count = read_u32(input)?;
        let style_count = read_u32(input)?;
        let flags = read_u32(input)?;

        let string_start = read_u32(input)?;
        let style_start = read_u32(input)?;

        let header = Self {
            chunk_header,
            string_count,
            style_count,
            flags,
            string_start,
            style_start,
        };

        Ok(header)
    }
}

#[derive(Debug)]
pub(crate) struct StringPool {
    pub(crate) header: StringPoolHeader,
    pub(crate) strings: Vec<Rc<String>>,
}

impl StringPool {
    pub(crate) fn read_from_file<F: Read + Seek>(
        input: &mut F,
        chunk_header: &ChunkHeader,
    ) -> Result<Self, ParseError> {
        let string_pool_header = StringPoolHeader::read_from_file(input, chunk_header)?;
        assert_eq!(string_pool_header.style_count, 0);

        let flag_is_utf8 = (string_pool_header.flags & (1 << 8)) != 0;

        const STRINGPOOL_HEADER_SIZE: usize = std::mem::size_of::<StringPoolHeader>();
        let s =
            usize::try_from(string_pool_header.chunk_header.size).unwrap() - STRINGPOOL_HEADER_SIZE;
        let mut string_pool_data = vec![0; s];

        input
            .read_exact(&mut string_pool_data)
            .map_err(ParseError::IoError)?;

        // Parse string offsets
        let num_offsets = usize::try_from(string_pool_header.string_count).unwrap();
        let offsets = parse_offsets(&string_pool_data, num_offsets);

        let string_data_start =
            usize::try_from(string_pool_header.string_start).unwrap() - STRINGPOOL_HEADER_SIZE;
        let string_data = &string_pool_data[string_data_start..];

        let mut strings =
            Vec::with_capacity(usize::try_from(string_pool_header.string_count).unwrap());

        let parse_fn = if flag_is_utf8 {
            parse_utf8_string
        } else {
            parse_utf16_string
        };

        for offset in offsets {
            strings.push(Rc::new(parse_fn(
                &string_data,
                usize::try_from(offset).unwrap(),
            )?));
        }

        Ok(Self {
            header: string_pool_header,
            strings,
        })
    }

    pub(crate) fn get(&self, i: usize) -> Option<Rc<String>> {
        if u32::try_from(i).unwrap() == u32::MAX {
            return None;
        }

        Some(self.strings.get(i)?.clone())
    }
}

fn parse_offsets(string_data: &[u8], count: usize) -> Vec<u32> {
    let mut offsets = Vec::with_capacity(count);

    for i in 0..count {
        let index = i * 4;
        let offset = LittleEndian::read_u32(&string_data[index..index + 4]);
        offsets.push(offset);
    }

    offsets
}

fn parse_utf16_string(string_data: &[u8], offset: usize) -> Result<String, ParseError> {
    let len = LittleEndian::read_u16(&string_data[offset..offset + 2]);

    // Handles the case where the string is > 32767 characters
    if is_high_bit_set_16(len) {
        unimplemented!()
    }

    // This needs to change if we ever implement support for long strings
    let string_start = offset + 2;

    let mut s = Vec::with_capacity(len.into());
    for i in 0..len {
        let index = string_start + usize::try_from(i * 2).unwrap();
        let char = LittleEndian::read_u16(&string_data[index..index + 2]);
        s.push(char);
    }

    let s = String::from_utf16(&s).map_err(ParseError::Utf16StringParseError)?;
    Ok(s)
}

fn is_high_bit_set_16(input: u16) -> bool {
    input & (1 << 15) != 0
}

fn parse_utf8_string(string_data: &[u8], offset: usize) -> Result<String, ParseError> {
    let len = string_data[offset + 1];

    // Handles the case where the length value has high bit set
    // Not quite clear if the UTF-8 encoding actually has this but
    // perform the check anyway...
    if is_high_bit_set_8(len) {
        unimplemented!()
    }

    // This needs to change if we ever implement support for long strings
    let string_start = offset + 2;

    let mut s = Vec::with_capacity(len.into());
    for i in 0..len {
        let index = string_start + usize::try_from(i).unwrap();
        let char = string_data[index];
        s.push(char);
    }

    let s = String::from_utf8(s).map_err(ParseError::Utf8StringParseError)?;
    Ok(s)
}

fn is_high_bit_set_8(input: u8) -> bool {
    input & (1 << 7) != 0
}
