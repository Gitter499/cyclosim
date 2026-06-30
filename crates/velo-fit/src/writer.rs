//! Low-level FIT binary writer.

use crate::crc::fit_crc16;

/// FIT base types (Profile.xlsx).
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BaseType {
    Enum = 0x00,
    Sint8 = 0x01,
    Uint8 = 0x02,
    Sint16 = 0x03,
    Uint16 = 0x04,
    Sint32 = 0x05,
    Uint32 = 0x06,
    String = 0x07,
    Float32 = 0x08,
    Float64 = 0x09,
    Uint8z = 0x0A,
    Uint16z = 0x0B,
    Uint32z = 0x0C,
    Byte = 0x0D,
    Sint64 = 0x8E,
    Uint64 = 0x8F,
}

impl BaseType {
    pub fn size(self) -> u8 {
        match self {
            BaseType::Enum | BaseType::Sint8 | BaseType::Uint8 | BaseType::Uint8z | BaseType::Byte => 1,
            BaseType::Sint16 | BaseType::Uint16 | BaseType::Uint16z => 2,
            BaseType::Sint32 | BaseType::Uint32 | BaseType::Uint32z | BaseType::Float32 => 4,
            BaseType::Float64 | BaseType::Sint64 | BaseType::Uint64 => 8,
            BaseType::String => 0,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct FieldDef {
    pub num: u8,
    pub size: u8,
    pub base_type: BaseType,
}

#[derive(Default)]
pub struct FitWriter {
    data: Vec<u8>,
    defined: [bool; 16],
}

impl FitWriter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn data_bytes(&self) -> &[u8] {
        &self.data
    }

    pub fn write_definition(&mut self, local_type: u8, global_num: u16, fields: &[FieldDef]) {
        assert!(local_type < 16);
        let header = 0x40 | (local_type & 0x0F);
        self.data.push(header);
        self.data.push(0); // reserved
        self.data.push(0); // little-endian
        self.data.extend_from_slice(&global_num.to_le_bytes());
        self.data.push(fields.len() as u8);
        for f in fields {
            self.data.push(f.num);
            self.data.push(f.size);
            self.data.push(f.base_type as u8);
        }
        self.defined[local_type as usize] = true;
    }

    pub fn begin_data(&mut self, local_type: u8) -> FitMessageWriter<'_> {
        assert!(local_type < 16);
        assert!(self.defined[local_type as usize], "message type {local_type} not defined");
        let header = local_type & 0x0F;
        self.data.push(header);
        FitMessageWriter {
            writer: self,
            local_type,
        }
    }

    pub fn finish(mut self) -> Vec<u8> {
        let data_size = self.data.len() as u32;
        let mut file = Vec::with_capacity(14 + self.data.len() + 2);

        // 14-byte header.
        file.push(14); // header size
        file.push(0x10); // protocol v1.0
        file.extend_from_slice(&350u16.to_le_bytes()); // profile v3.50
        file.extend_from_slice(&data_size.to_le_bytes());
        file.extend_from_slice(b".FIT");
        let header_crc = fit_crc16(&file);
        file.extend_from_slice(&header_crc.to_le_bytes());

        file.append(&mut self.data);
        let file_crc = fit_crc16(&file);
        file.extend_from_slice(&file_crc.to_le_bytes());
        file
    }
}

pub struct FitMessageWriter<'a> {
    writer: &'a mut FitWriter,
    #[allow(dead_code)]
    local_type: u8,
}

impl FitMessageWriter<'_> {
    pub fn write_u8(&mut self, v: u8) {
        self.writer.data.push(v);
    }

    pub fn write_u16(&mut self, v: u16) {
        self.writer.data.extend_from_slice(&v.to_le_bytes());
    }

    pub fn write_u32(&mut self, v: u32) {
        self.writer.data.extend_from_slice(&v.to_le_bytes());
    }

    pub fn write_i32(&mut self, v: i32) {
        self.writer.data.extend_from_slice(&v.to_le_bytes());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_starts_with_fit_magic() {
        let mut w = FitWriter::new();
        w.write_definition(
            0,
            0,
            &[FieldDef {
                num: 0,
                size: 1,
                base_type: BaseType::Enum,
            }],
        );
        {
            let mut m = w.begin_data(0);
            m.write_u8(4);
        }
        let bytes = w.finish();
        assert_eq!(&bytes[8..12], b".FIT");
        assert!(bytes.len() > 16);
    }
}
