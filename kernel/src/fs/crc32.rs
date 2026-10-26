//! CRC32C (Castagnoli) Pure Safe Rust Implementation
//! Polynomial: 0x82F63B78 (reflected 0x1EDC6F41)
//! Compliant with Bible §8.10 (SP-25), Rule N-01, Rule N-04, and Invariant INV-FS-001.

const CRC32C_POLYNOMIAL: u32 = 0x82F63B78;

/// Computes CRC32C checksum over a byte slice using the standard Castagnoli polynomial
pub fn crc32c(data: &[u8]) -> u32 {
    let mut crc: u32 = 0xFFFF_FFFF;

    for &byte in data {
        crc ^= byte as u32;
        for _ in 0..8 {
            if (crc & 1) != 0 {
                crc = (crc >> 1) ^ CRC32C_POLYNOMIAL;
            } else {
                crc >>= 1;
            }
        }
    }

    !crc
}

/// Updates an existing CRC32C with additional bytes
pub fn crc32c_update(initial: u32, data: &[u8]) -> u32 {
    let mut crc = !initial;

    for &byte in data {
        crc ^= byte as u32;
        for _ in 0..8 {
            if (crc & 1) != 0 {
                crc = (crc >> 1) ^ CRC32C_POLYNOMIAL;
            } else {
                crc >>= 1;
            }
        }
    }

    !crc
}
