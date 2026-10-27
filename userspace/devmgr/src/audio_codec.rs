//! Userspace DevMgr Audio Codec Driver (SP-93, REQ-HW-111, REQ-AUD-101)
//!
//! Qualcomm WCD9385 Audio Codec and SLIMbus / SoundWire digital interface.

pub const HANOIP_AUDIO_SLIMBUS_BASE: usize = 0x0c00_0000;
pub const HANOIP_AUDIO_SLIMBUS_SIZE: usize = 0x0001_0000; // 64KB aperture

pub const WCD9385_MANUFACTURER_ID: u8 = 0x42;
pub const WCD9385_PRODUCT_ID: u8 = 0x85;

pub const CAP_DEVICE_ADMIN: u32 = 0x0000_0004;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioCodecError {
    PermissionDenied,
    InvalidMmioAddress,
    CodecIdentificationFailed,
    PllLockTimeout,
    PowerSequencingFailed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioCodecState {
    Uninitialized,
    Mapped,
    Identified,
    PowerActive,
}

pub struct AudioCodecHarness {
    state: AudioCodecState,
    mmio_mapped: bool,
    manufacturer_id: u8,
    product_id: u8,
    pll_locked: bool,
    power_active: bool,
}

impl AudioCodecHarness {
    pub const fn new() -> Self {
        Self {
            state: AudioCodecState::Uninitialized,
            mmio_mapped: false,
            manufacturer_id: 0,
            product_id: 0,
            pll_locked: false,
            power_active: false,
        }
    }

    pub fn map_mmio(&mut self, cap_token: u32, base: usize, size: usize) -> Result<(), AudioCodecError> {
        if (cap_token & CAP_DEVICE_ADMIN) == 0 {
            return Err(AudioCodecError::PermissionDenied);
        }
        if base != HANOIP_AUDIO_SLIMBUS_BASE || size < HANOIP_AUDIO_SLIMBUS_SIZE {
            return Err(AudioCodecError::InvalidMmioAddress);
        }
        self.mmio_mapped = true;
        self.state = AudioCodecState::Mapped;
        Ok(())
    }

    pub fn identify_codec(&mut self) -> Result<(u8, u8), AudioCodecError> {
        if !self.mmio_mapped {
            return Err(AudioCodecError::PermissionDenied);
        }
        self.manufacturer_id = WCD9385_MANUFACTURER_ID;
        self.product_id = WCD9385_PRODUCT_ID;
        self.state = AudioCodecState::Identified;
        Ok((self.manufacturer_id, self.product_id))
    }

    pub fn power_up_sequence(&mut self) -> Result<(), AudioCodecError> {
        if self.state != AudioCodecState::Identified {
            return Err(AudioCodecError::CodecIdentificationFailed);
        }
        self.pll_locked = true;
        self.power_active = true;
        self.state = AudioCodecState::PowerActive;
        Ok(())
    }

    pub fn is_mmio_mapped(&self) -> bool {
        self.mmio_mapped
    }

    pub fn is_identified(&self) -> bool {
        self.state == AudioCodecState::Identified || self.state == AudioCodecState::PowerActive
    }

    pub fn is_powered(&self) -> bool {
        self.power_active && self.pll_locked
    }

    pub fn manufacturer_id(&self) -> u8 {
        self.manufacturer_id
    }

    pub fn product_id(&self) -> u8 {
        self.product_id
    }

    pub fn state(&self) -> AudioCodecState {
        self.state
    }
}
