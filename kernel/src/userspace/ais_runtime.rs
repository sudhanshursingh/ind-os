//! Kernel AIS Model Runtime Sandbox & Hardware Profile Verification Harness (SP-41)
//!
//! Constitutional References:
//! - REQ-AI-101: Model runtime sandbox
//! - REQ-AI-112: Dynamic hardware profile manager (RFC-INDOS-2026-001)
//! - INV-AIS-001: Signed-model loader only
//! - INV-AIS-002: Bounded capability sandbox & resource quota
//! - INV-AIS-003: Deterministic hardware profile tiering
//! - Rule N-01: Zero external crates
//! - Rule N-04: Zero unsafe code in new modules

pub const KERNEL_CAP_AIS_LOAD: u32 = 0x10;
pub const KERNEL_CAP_AIS_INFER: u32 = 0x20;
pub const KERNEL_AIS_AUTHORIZED_SIGNATURE: u32 = 0x516E_A150;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KernelHardwareTier {
    Budget,
    Standard,
    Flagship,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KernelInferenceConfig {
    pub tier: KernelHardwareTier,
    pub parameter_billions_x10: u32,
    pub context_tokens: u32,
    pub ram_limit_mb: u32,
}

pub struct KernelHardwareProfile;

impl KernelHardwareProfile {
    pub fn select_profile(ram_mb: u32) -> KernelInferenceConfig {
        if ram_mb <= 6144 {
            KernelInferenceConfig {
                tier: KernelHardwareTier::Budget,
                parameter_billions_x10: 15, // 1.5B
                context_tokens: 2048,
                ram_limit_mb: 1250,
            }
        } else if ram_mb <= 12288 {
            KernelInferenceConfig {
                tier: KernelHardwareTier::Standard,
                parameter_billions_x10: 30, // 3.0B
                context_tokens: 4096,
                ram_limit_mb: 3000,
            }
        } else {
            KernelInferenceConfig {
                tier: KernelHardwareTier::Flagship,
                parameter_billions_x10: 80, // 8.0B
                context_tokens: 8192,
                ram_limit_mb: 6000,
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KernelModelError {
    SignatureRequired,
    InvalidSignature,
    IntegrityMismatch,
    InsufficientQuota,
    PermissionDenied,
}

pub struct KernelModelSandbox {
    pub config: KernelInferenceConfig,
    pub model_loaded: bool,
    pub model_checksum: u32,
    pub allocated_bytes: usize,
}

impl KernelModelSandbox {
    pub fn new(config: KernelInferenceConfig) -> Self {
        Self {
            config,
            model_loaded: false,
            model_checksum: 0,
            allocated_bytes: 0,
        }
    }

    pub fn load_signed_model(
        &mut self,
        required_ram_mb: u32,
        weights_checksum: u32,
        signature_token: u32,
        actual_weights: &[u8],
        caller_cap: u32,
    ) -> Result<(), KernelModelError> {
        if (caller_cap & KERNEL_CAP_AIS_LOAD) == 0 {
            return Err(KernelModelError::PermissionDenied);
        }
        if signature_token == 0 {
            return Err(KernelModelError::SignatureRequired);
        }
        if signature_token != KERNEL_AIS_AUTHORIZED_SIGNATURE {
            return Err(KernelModelError::InvalidSignature);
        }
        if required_ram_mb > self.config.ram_limit_mb {
            return Err(KernelModelError::InsufficientQuota);
        }

        // Compute checksum
        let mut hash: u32 = 0x811c9dc5;
        for &b in actual_weights {
            hash ^= b as u32;
            hash = hash.wrapping_mul(0x01000193);
        }

        if hash != weights_checksum {
            return Err(KernelModelError::IntegrityMismatch);
        }

        self.model_loaded = true;
        self.model_checksum = hash;
        Ok(())
    }

    pub fn infer(
        &mut self,
        prompt: &str,
        output_buf: &mut [u8],
        caller_cap: u32,
    ) -> Result<usize, KernelModelError> {
        if (caller_cap & KERNEL_CAP_AIS_INFER) == 0 {
            return Err(KernelModelError::PermissionDenied);
        }
        if !self.model_loaded {
            return Err(KernelModelError::PermissionDenied);
        }

        let prefix = b"AIS-PROPOSAL:";
        let p_bytes = prompt.as_bytes();
        let total = prefix.len() + p_bytes.len();
        if output_buf.len() < total {
            return Err(KernelModelError::InsufficientQuota);
        }

        for i in 0..prefix.len() {
            output_buf[i] = prefix[i];
        }
        for i in 0..p_bytes.len() {
            output_buf[prefix.len() + i] = p_bytes[i];
        }

        self.allocated_bytes = total;
        Ok(total)
    }
}
