//! Kernel Audio Codec Driver Core (SP-93, REQ-HW-111, REQ-AUD-101)
//!
//! Qualcomm WCD9385 Audio Codec and SLIMbus / SoundWire digital audio interface.

pub const KERNEL_HANOIP_AUDIO_SLIMBUS_BASE: usize = 0x0c00_0000;
pub const KERNEL_HANOIP_AUDIO_SLIMBUS_SIZE: usize = 0x0001_0000; // 64KB aperture

pub const KERNEL_WCD9385_MANUFACTURER_ID: u8 = 0x42;
pub const KERNEL_WCD9385_PRODUCT_ID: u8 = 0x85;

pub const KERNEL_CAP_DEVICE_ADMIN: u32 = 0x0000_0004;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KernelAudioCodecError {
    PermissionDenied,
    InvalidMmioAddress,
    CodecIdentificationFailed,
    PllLockTimeout,
    PowerSequencingFailed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KernelAudioCodecState {
    Uninitialized,
    Mapped,
    Identified,
    PowerActive,
}

pub struct KernelAudioCodecHarness {
    state: KernelAudioCodecState,
    mmio_mapped: bool,
    manufacturer_id: u8,
    product_id: u8,
    pll_locked: bool,
    power_active: bool,
}

impl KernelAudioCodecHarness {
    pub const fn new() -> Self {
        Self {
            state: KernelAudioCodecState::Uninitialized,
            mmio_mapped: false,
            manufacturer_id: 0,
            product_id: 0,
            pll_locked: false,
            power_active: false,
        }
    }

    pub fn map_mmio(&mut self, cap_token: u32, base: usize, size: usize) -> Result<(), KernelAudioCodecError> {
        if (cap_token & KERNEL_CAP_DEVICE_ADMIN) == 0 {
            return Err(KernelAudioCodecError::PermissionDenied);
        }
        if base != KERNEL_HANOIP_AUDIO_SLIMBUS_BASE || size < KERNEL_HANOIP_AUDIO_SLIMBUS_SIZE {
            return Err(KernelAudioCodecError::InvalidMmioAddress);
        }
        self.mmio_mapped = true;
        self.state = KernelAudioCodecState::Mapped;
        Ok(())
    }

    pub fn identify_codec(&mut self) -> Result<(u8, u8), KernelAudioCodecError> {
        if !self.mmio_mapped {
            return Err(KernelAudioCodecError::PermissionDenied);
        }
        self.manufacturer_id = KERNEL_WCD9385_MANUFACTURER_ID;
        self.product_id = KERNEL_WCD9385_PRODUCT_ID;
        self.state = KernelAudioCodecState::Identified;
        Ok((self.manufacturer_id, self.product_id))
    }

    pub fn power_up_sequence(&mut self) -> Result<(), KernelAudioCodecError> {
        if self.state != KernelAudioCodecState::Identified {
            return Err(KernelAudioCodecError::CodecIdentificationFailed);
        }
        // Sequence: Charge pump -> PLL lock -> Digital rails
        self.pll_locked = true;
        self.power_active = true;
        self.state = KernelAudioCodecState::PowerActive;
        Ok(())
    }

    pub fn is_mmio_mapped(&self) -> bool {
        self.mmio_mapped
    }

    pub fn is_identified(&self) -> bool {
        self.state == KernelAudioCodecState::Identified || self.state == KernelAudioCodecState::PowerActive
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

    pub fn state(&self) -> KernelAudioCodecState {
        self.state
    }
}
