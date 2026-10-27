//! Kernel Ambient Light & Proximity Sensor Driver (SP-90, REQ-HW-110, REQ-SNS-102)
//!
//! Architectural Invariants:
//! - INV-ALS-001: ALS/Prox I2C bus MMIO base mapping (0x0888_4000, addr 0x39)
//! - INV-ALS-002: Photometric lux calculation and optical gain scaling
//! - INV-ALS-003: Proximity threshold hysteresis event dispatch

pub const KERNEL_HANOIP_ALS_I2C_BASE: u64 = 0x0888_4000;
pub const KERNEL_ALS_PROX_I2C_ADDR: u8 = 0x39;
pub const KERNEL_PROX_THRESHOLD_HIGH: u16 = 50;
pub const KERNEL_PROX_THRESHOLD_LOW: u16 = 25;
pub const KERNEL_CAP_SENSOR_READ: u32 = 0x0800_0000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KernelProximityState {
    Far,
    Near,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KernelAlsGain {
    Gain1x,
    Gain8x,
    Gain64x,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KernelAlsProxError {
    InvalidMmioBase,
    PermissionDenied,
    SensorNotReady,
    InvalidReading,
}

pub struct KernelSensorAlsProxHarness {
    mmio_mapped: bool,
    prox_state: KernelProximityState,
    current_lux: u32,
    samples_processed: usize,
}

impl KernelSensorAlsProxHarness {
    pub const fn new() -> Self {
        Self {
            mmio_mapped: false,
            prox_state: KernelProximityState::Far,
            current_lux: 0,
            samples_processed: 0,
        }
    }

    pub fn map_mmio(&mut self, base: u64, addr: u8) -> Result<(), KernelAlsProxError> {
        if base != KERNEL_HANOIP_ALS_I2C_BASE || addr != KERNEL_ALS_PROX_I2C_ADDR {
            return Err(KernelAlsProxError::InvalidMmioBase);
        }
        self.mmio_mapped = true;
        Ok(())
    }

    pub fn sample_lux(&mut self, raw_counts: u16, gain: KernelAlsGain) -> Result<u32, KernelAlsProxError> {
        if !self.mmio_mapped {
            return Err(KernelAlsProxError::InvalidMmioBase);
        }
        let scale = match gain {
            KernelAlsGain::Gain1x => 10,
            KernelAlsGain::Gain8x => 5,
            KernelAlsGain::Gain64x => 1,
        };
        let lux = (raw_counts as u32) * scale / 4;
        self.current_lux = lux;
        self.samples_processed += 1;
        Ok(lux)
    }

    pub fn update_proximity(&mut self, raw_prox: u16) -> Result<KernelProximityState, KernelAlsProxError> {
        if !self.mmio_mapped {
            return Err(KernelAlsProxError::InvalidMmioBase);
        }
        if raw_prox >= KERNEL_PROX_THRESHOLD_HIGH {
            self.prox_state = KernelProximityState::Near;
        } else if raw_prox <= KERNEL_PROX_THRESHOLD_LOW {
            self.prox_state = KernelProximityState::Far;
        }
        Ok(self.prox_state)
    }

    pub fn is_mmio_mapped(&self) -> bool {
        self.mmio_mapped
    }

    pub fn proximity_state(&self) -> KernelProximityState {
        self.prox_state
    }

    pub fn current_lux(&self) -> u32 {
        self.current_lux
    }

    pub fn samples_processed(&self) -> usize {
        self.samples_processed
    }
}
