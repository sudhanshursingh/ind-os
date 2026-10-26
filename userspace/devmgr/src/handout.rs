//! Ind OS Capability Handout Engine
//!
//! Standards:
//! - REQ-USE-203: Capability handout per device manager
//! - INV-DEV-001: Zero ambient authority (scoped access)
//! - INV-DEV-002: Atomic capability revocation upon device departure

use crate::types::{DeviceError, DeviceId, HardwareDescriptor};

pub const MAX_HANDOUT_TOKENS: usize = 32;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MmioCapToken {
    pub base: usize,
    pub size: usize,
    pub is_valid: bool,
}

impl MmioCapToken {
    pub const fn new(base: usize, size: usize) -> Self {
        Self {
            base,
            size,
            is_valid: true,
        }
    }

    pub fn check_access(&self, offset: usize, len: usize) -> Result<(), DeviceError> {
        if !self.is_valid {
            return Err(DeviceError::AccessDenied);
        }
        if offset + len > self.size {
            return Err(DeviceError::AccessDenied);
        }
        Ok(())
    }

    pub fn revoke(&mut self) {
        self.is_valid = false;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IrqCapToken {
    pub irq_line: u32,
    pub is_valid: bool,
}

impl IrqCapToken {
    pub const fn new(irq_line: u32) -> Self {
        Self {
            irq_line,
            is_valid: true,
        }
    }

    pub fn check_handle(&self, line: u32) -> Result<(), DeviceError> {
        if !self.is_valid || self.irq_line != line {
            return Err(DeviceError::AccessDenied);
        }
        Ok(())
    }

    pub fn revoke(&mut self) {
        self.is_valid = false;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DmaCapToken {
    pub buffer_size: usize,
    pub is_valid: bool,
}

impl DmaCapToken {
    pub const fn new(buffer_size: usize) -> Self {
        Self {
            buffer_size,
            is_valid: true,
        }
    }

    pub fn check_dma(&self, len: usize) -> Result<(), DeviceError> {
        if !self.is_valid || len > self.buffer_size {
            return Err(DeviceError::AccessDenied);
        }
        Ok(())
    }

    pub fn revoke(&mut self) {
        self.is_valid = false;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DeviceCapToken {
    pub token_id: u32,
    pub device_id: DeviceId,
    pub mmio: MmioCapToken,
    pub irq: IrqCapToken,
    pub dma: DmaCapToken,
    pub is_valid: bool,
}

impl DeviceCapToken {
    pub const fn new(
        token_id: u32,
        device_id: DeviceId,
        mmio: MmioCapToken,
        irq: IrqCapToken,
        dma: DmaCapToken,
    ) -> Self {
        Self {
            token_id,
            device_id,
            mmio,
            irq,
            dma,
            is_valid: true,
        }
    }

    pub fn revoke(&mut self) {
        self.is_valid = false;
        self.mmio.revoke();
        self.irq.revoke();
        self.dma.revoke();
    }

    pub fn is_active(&self) -> bool {
        self.is_valid && self.mmio.is_valid && self.irq.is_valid
    }
}

pub struct CapabilityHandoutRegistry {
    tokens: [Option<DeviceCapToken>; MAX_HANDOUT_TOKENS],
    next_token_id: u32,
}

impl CapabilityHandoutRegistry {
    pub const fn new() -> Self {
        Self {
            tokens: [None; MAX_HANDOUT_TOKENS],
            next_token_id: 1,
        }
    }

    pub fn mint_token(
        &mut self,
        device_id: DeviceId,
        desc: &HardwareDescriptor,
    ) -> Result<DeviceCapToken, DeviceError> {
        let token_id = self.next_token_id;
        self.next_token_id += 1;

        let mmio = MmioCapToken::new(desc.mmio_base, desc.mmio_size);
        let irq = IrqCapToken::new(desc.irq_line);
        let dma = DmaCapToken::new(desc.dma_buffer_size);

        let token = DeviceCapToken::new(token_id, device_id, mmio, irq, dma);

        for slot in self.tokens.iter_mut() {
            if slot.is_none() {
                *slot = Some(token);
                return Ok(token);
            }
        }
        Err(DeviceError::BufferFull)
    }

    pub fn revoke_for_device(&mut self, device_id: DeviceId) -> usize {
        let mut count = 0;
        for slot in self.tokens.iter_mut() {
            if let Some(ref mut tok) = slot {
                if tok.device_id == device_id && tok.is_valid {
                    tok.revoke();
                    count += 1;
                }
            }
        }
        count
    }

    pub fn get_token(&self, token_id: u32) -> Option<&DeviceCapToken> {
        for slot in self.tokens.iter() {
            if let Some(ref tok) = slot {
                if tok.token_id == token_id {
                    return Some(tok);
                }
            }
        }
        None
    }
}
