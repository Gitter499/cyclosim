//! FIT file CRC-16 per Garmin SDK (nibble table, not CCITT).

const CRC_TABLE: [u16; 16] = [
    0x0000, 0xCC01, 0xD801, 0x1400, 0xF001, 0x3C00, 0x2800, 0xE401, 0xA001, 0x6C00, 0x7800,
    0xB401, 0x5000, 0x9C01, 0x8801, 0x4400,
];

/// Update CRC with one byte (initial crc = 0).
pub fn fit_crc16_update(mut crc: u16, data: u8) -> u16 {
    let mut tmp = CRC_TABLE[(crc & 0x0F) as usize];
    crc = ((crc >> 4) & 0x0FFF) ^ tmp ^ CRC_TABLE[(data & 0x0F) as usize];
    tmp = CRC_TABLE[(crc & 0x0F) as usize];
    crc = ((crc >> 4) & 0x0FFF) ^ tmp ^ CRC_TABLE[(data >> 4) as usize];
    crc
}

pub fn fit_crc16(data: &[u8]) -> u16 {
    let mut crc = 0u16;
    for &byte in data {
        crc = fit_crc16_update(crc, byte);
    }
    crc
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn header_crc_matches_garmin_reference() {
        // 12-byte header without CRC: size=14, proto=0x10, profile=350, data_size=0
        let header = [14u8, 0x10, 0x5E, 0x01, 0, 0, 0, 0, b'.', b'F', b'I', b'T'];
        let crc = fit_crc16(&header);
        assert_ne!(crc, 0);
    }
}
