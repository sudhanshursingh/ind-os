//! Kernel Magnetometer Driver Core (SP-91, REQ-HW-110, REQ-SNS-103)
//!
//! Target: Asahi Kasei AK09918 3-Axis Electronic Compass
//! Architectural Invariants:
//! - INV-MAG-001: AK09918 I2C bus MMIO mapping (0x0888_8000, 7-bit addr 0x0C)
//! - INV-MAG-002: Chip identification verification (WIA1=0x48, WIA2=0x0C)
//! - INV-MAG-003: 3-Axis magnetic field reading & integer heading calculation (0..=359 deg)

pub const KERNEL_HANOIP_MAG_I2C_BASE: u64 = 0x0888_8000;
pub const KERNEL_AK09918_I2C_ADDR: u8 = 0x0C;
pub const KERNEL_AK09918_WIA1_COMPANY: u8 = 0x48;
pub const KERNEL_AK09918_WIA2_DEVICE: u8 = 0x0C;
pub const KERNEL_CAP_SENSOR_READ: u32 = 0x0800_0000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KernelMagReading {
    pub x: i16,
    pub y: i16,
    pub z: i16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KernelMagError {
    InvalidMmioBase,
    InvalidChipId,
    SensorNotReady,
    PermissionDenied,
}

pub struct KernelSensorMagHarness {
    mmio_mapped: bool,
    chip_id_verified: bool,
    last_reading: Option<KernelMagReading>,
    last_heading_deg: u16,
    samples_count: usize,
}

impl KernelSensorMagHarness {
    pub const fn new() -> Self {
        Self {
            mmio_mapped: false,
            chip_id_verified: false,
            last_reading: None,
            last_heading_deg: 0,
            samples_count: 0,
        }
    }

    pub fn map_mmio(&mut self, base: u64, addr: u8) -> Result<(), KernelMagError> {
        if base != KERNEL_HANOIP_MAG_I2C_BASE || addr != KERNEL_AK09918_I2C_ADDR {
            return Err(KernelMagError::InvalidMmioBase);
        }
        self.mmio_mapped = true;
        Ok(())
    }

    pub fn verify_chip_id(&mut self, wia1: u8, wia2: u8) -> Result<(), KernelMagError> {
        if !self.mmio_mapped {
            return Err(KernelMagError::InvalidMmioBase);
        }
        if wia1 != KERNEL_AK09918_WIA1_COMPANY || wia2 != KERNEL_AK09918_WIA2_DEVICE {
            return Err(KernelMagError::InvalidChipId);
        }
        self.chip_id_verified = true;
        Ok(())
    }

    pub fn read_mag_and_heading(&mut self, x: i16, y: i16, z: i16) -> Result<(KernelMagReading, u16), KernelMagError> {
        if !self.mmio_mapped {
            return Err(KernelMagError::InvalidMmioBase);
        }
        if !self.chip_id_verified {
            return Err(KernelMagError::SensorNotReady);
        }

        let heading = kernel_compute_heading_deg(x, y);
        let reading = KernelMagReading { x, y, z };

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
pub fn kernel_compute_heading_deg(x: i16, y: i16) -> u16 {
    if x == 0 && y == 0 {
        return 0;
    }
    let x32 = x as i32;
    let y32 = y as i32;

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
