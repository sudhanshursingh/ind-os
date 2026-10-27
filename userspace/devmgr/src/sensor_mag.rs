//! Userspace DevMgr Magnetometer Driver (SP-91, REQ-HW-110, REQ-SNS-103)
//!
//! Target: Asahi Kasei AK09918 3-Axis Electronic Compass
//! Architectural Invariants:
//! - INV-MAG-001: AK09918 I2C bus MMIO mapping (0x0888_8000, 7-bit addr 0x0C)
//! - INV-MAG-002: Chip identification verification (WIA1=0x48, WIA2=0x0C)
//! - INV-MAG-003: 3-Axis magnetic field reading & integer heading calculation (0..=359 deg)

pub const HANOIP_MAG_I2C_BASE: u64 = 0x0888_8000;
pub const AK09918_I2C_ADDR: u8 = 0x0C;
pub const AK09918_WIA1_COMPANY: u8 = 0x48;
pub const AK09918_WIA2_DEVICE: u8 = 0x0C;
pub const CAP_SENSOR_READ: u32 = 0x0800_0000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MagReading {
    pub x: i16,
    pub y: i16,
    pub z: i16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MagError {
    InvalidMmioBase,
    InvalidChipId,
    SensorNotReady,
    PermissionDenied,
}

pub struct SensorMagHarness {
    mmio_mapped: bool,
    chip_id_verified: bool,
    last_reading: Option<MagReading>,
    last_heading_deg: u16,
    samples_count: usize,
}

impl SensorMagHarness {
    pub const fn new() -> Self {
        Self {
            mmio_mapped: false,
            chip_id_verified: false,
            last_reading: None,
            last_heading_deg: 0,
            samples_count: 0,
        }
    }

    pub fn map_mmio(&mut self, base: u64, addr: u8) -> Result<(), MagError> {
        if base != HANOIP_MAG_I2C_BASE || addr != AK09918_I2C_ADDR {
            return Err(MagError::InvalidMmioBase);
        }
        self.mmio_mapped = true;
        Ok(())
    }

    pub fn verify_chip_id(&mut self, wia1: u8, wia2: u8) -> Result<(), MagError> {
        if !self.mmio_mapped {
            return Err(MagError::InvalidMmioBase);
        }
        if wia1 != AK09918_WIA1_COMPANY || wia2 != AK09918_WIA2_DEVICE {
            return Err(MagError::InvalidChipId);
        }
        self.chip_id_verified = true;
        Ok(())
    }

    pub fn read_mag_and_heading(&mut self, x: i16, y: i16, z: i16) -> Result<(MagReading, u16), MagError> {
        if !self.mmio_mapped {
            return Err(MagError::InvalidMmioBase);
        }
        if !self.chip_id_verified {
            return Err(MagError::SensorNotReady);
        }

        let heading = compute_heading_deg(x, y);
        let reading = MagReading { x, y, z };

        self.last_reading = Some(reading);
        self.last_heading_deg = heading;
        self.samples_count += 1;

        Ok((reading, heading))
    }

    pub fn is_mmio_mapped(&self) -> bool {
        self.mmio_mapped
    }

    pub fn is_chip_id_verified(&self) -> bool {
        self.chip_id_verified
    }

    pub fn last_heading_deg(&self) -> u16 {
        self.last_heading_deg
    }

    pub fn samples_count(&self) -> usize {
        self.samples_count
    }
}

/// Compute compass heading in degrees [0..359] from magnetic X and Y using integer arithmetic.
pub fn compute_heading_deg(x: i16, y: i16) -> u16 {
    if x == 0 && y == 0 {
        return 0;
    }
    // Fixed-point atan2 approximation in degrees
    // Azimuth angle = atan2(y, x) converted to degrees 0..359
    let x32 = x as i32;
    let y32 = y as i32;

    // Use standard integer octant approximation:
    // angle in range [0..45] deg: ~ (45 * num) / den
    let abs_x = if x32 < 0 { -x32 } else { x32 };
    let abs_y = if y32 < 0 { -y32 } else { y32 };

    let angle: i32;
    if abs_x >= abs_y {
        if abs_x == 0 {
            angle = 0;
        } else {
            angle = (45 * abs_y) / abs_x;
        }
    } else {
        angle = 90 - (45 * abs_x) / abs_y;
    }

    // Determine quadrant:
    // Q1: x >= 0, y >= 0 -> angle
    // Q2: x < 0,  y >= 0 -> 180 - angle
    // Q3: x < 0,  y < 0  -> 180 + angle
    // Q4: x >= 0, y < 0  -> 360 - angle
    let deg = if x32 >= 0 && y32 >= 0 {
        angle
    } else if x32 < 0 && y32 >= 0 {
        180 - angle
    } else if x32 < 0 && y32 < 0 {
        180 + angle
    } else {
        360 - angle
    };

    let normalized = ((deg % 360) + 360) % 360;
    normalized as u16
}
