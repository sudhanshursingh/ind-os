//! Userspace DevMgr Ambient Light & Proximity Sensor Driver (SP-90, REQ-HW-110, REQ-SNS-102)
//!
//! Architectural Invariants:
//! - INV-ALS-001: ALS/Prox I2C bus MMIO base mapping (0x0888_4000, addr 0x39)
//! - INV-ALS-002: Photometric lux calculation and optical gain scaling
//! - INV-ALS-003: Proximity threshold hysteresis event dispatch

pub const HANOIP_ALS_I2C_BASE: u64 = 0x0888_4000;
pub const ALS_PROX_I2C_ADDR: u8 = 0x39;
pub const PROX_THRESHOLD_HIGH: u16 = 50;
pub const PROX_THRESHOLD_LOW: u16 = 25;
pub const CAP_SENSOR_READ: u32 = 0x0800_0000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProximityState {
    Far,
    Near,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlsGain {
    Gain1x,
    Gain8x,
    Gain64x,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlsProxError {
    InvalidMmioBase,
    PermissionDenied,
    SensorNotReady,
    InvalidReading,
}

pub struct SensorAlsProxHarness {
    mmio_mapped: bool,
    prox_state: ProximityState,
    current_lux: u32,
    samples_processed: usize,
}

impl SensorAlsProxHarness {
    pub const fn new() -> Self {
        Self {
            mmio_mapped: false,
            prox_state: ProximityState::Far,
            current_lux: 0,
            samples_processed: 0,
        }
    }

    pub fn map_mmio(&mut self, base: u64, addr: u8) -> Result<(), AlsProxError> {
        if base != HANOIP_ALS_I2C_BASE || addr != ALS_PROX_I2C_ADDR {
            return Err(AlsProxError::InvalidMmioBase);
        }
        self.mmio_mapped = true;
        Ok(())
    }

    pub fn sample_lux(&mut self, raw_counts: u16, gain: AlsGain) -> Result<u32, AlsProxError> {
        if !self.mmio_mapped {
            return Err(AlsProxError::InvalidMmioBase);
        }
        let scale = match gain {
            AlsGain::Gain1x => 10,
            AlsGain::Gain8x => 5,
            AlsGain::Gain64x => 1,
        };
        let lux = (raw_counts as u32) * scale / 4;
        self.current_lux = lux;
        self.samples_processed += 1;
        Ok(lux)
    }

    pub fn update_proximity(&mut self, raw_prox: u16) -> Result<ProximityState, AlsProxError> {
        if !self.mmio_mapped {
            return Err(AlsProxError::InvalidMmioBase);
        }
        if raw_prox >= PROX_THRESHOLD_HIGH {
            self.prox_state = ProximityState::Near;
        } else if raw_prox <= PROX_THRESHOLD_LOW {
            self.prox_state = ProximityState::Far;
        }
        // If between LOW and HIGH, retain current hysteresis state
        Ok(self.prox_state)
    }

    pub fn is_mmio_mapped(&self) -> bool {
        self.mmio_mapped
    }

    pub fn proximity_state(&self) -> ProximityState {
        self.prox_state
    }

    pub fn current_lux(&self) -> u32 {
        self.current_lux
    }

    pub fn samples_processed(&self) -> usize {
        self.samples_processed
    }
}
