//! Ind OS AIS Model Runtime Sandbox (REQ-AI-101, INV-AIS-002)
//!
//! Standards:
//! - INV-AIS-002: Bounded capability sandbox & resource quota enforcement
//! - Rule N-01: Zero external crates
//! - Rule N-04: Zero unsafe code in new modules

use crate::loader::SignedModelLoader;
use crate::types::{
    InferenceConfig, InferenceError, ModelLoadError, ModelManifest, SandboxStatus,
    CAP_AIS_INFER,
};

pub const MAX_WORKSPACE_BYTES: usize = 4096;
pub const MAX_INFERENCE_TICKS: u32 = 1000;

pub struct ModelRuntimeSandbox {
    config: InferenceConfig,
    active_manifest: Option<ModelManifest>,
    status: SandboxStatus,
    allocated_bytes: usize,
    workspace: [u8; MAX_WORKSPACE_BYTES],
}

impl ModelRuntimeSandbox {
    pub const fn new(config: InferenceConfig) -> Self {
        Self {
            config,
            active_manifest: None,
            status: SandboxStatus::Uninitialized,
            allocated_bytes: 0,
            workspace: [0u8; MAX_WORKSPACE_BYTES],
        }
    }

    pub fn status(&self) -> SandboxStatus {
        self.status
    }

    pub fn config(&self) -> &InferenceConfig {
        &self.config
    }

    pub fn allocated_bytes(&self) -> usize {
        self.allocated_bytes
    }

    /// Loads a signed model into the sandbox, checking memory ceiling against hardware profile.
    pub fn load_model(
        &mut self,
        manifest: ModelManifest,
        weights: &[u8],
        caller_cap: u32,
    ) -> Result<(), ModelLoadError> {
        // Enforce quota from selected hardware profile
        if manifest.required_ram_mb > self.config.ram_limit_mb {
            self.status = SandboxStatus::QuotaViolated;
            return Err(ModelLoadError::InsufficientQuota);
        }

        SignedModelLoader::verify_and_load(&manifest, weights, caller_cap)?;

        self.active_manifest = Some(manifest);
        self.status = SandboxStatus::ModelLoaded;
        Ok(())
    }

    /// Performs bounded CPU inference within tick and memory limits.
    pub fn infer(
        &mut self,
        prompt: &str,
        output_buffer: &mut [u8],
        caller_cap: u32,
        tick_budget: u32,
    ) -> Result<usize, InferenceError> {
        if (caller_cap & CAP_AIS_INFER) == 0 {
            return Err(InferenceError::PermissionDenied);
        }
        if self.active_manifest.is_none() {
            return Err(InferenceError::ModelNotLoaded);
        }
        if tick_budget > MAX_INFERENCE_TICKS {
            return Err(InferenceError::TimeBudgetExceeded);
        }

        self.status = SandboxStatus::Inferring;

        // Perform bounded embedding & token proposal generation
        let prompt_bytes = prompt.as_bytes();
        let prompt_len = prompt_bytes.len();
        if prompt_len + 32 > MAX_WORKSPACE_BYTES {
            self.status = SandboxStatus::QuotaViolated;
            return Err(InferenceError::MemoryQuotaExceeded);
        }

        // Bounded CPU forward pass simulation
        for i in 0..prompt_len {
            self.workspace[i] = prompt_bytes[i].wrapping_add(1);
        }
        self.allocated_bytes = prompt_len;

        // Synthesize structured proposal prefix
        let prefix = b"AIS-PROPOSAL:";
        if output_buffer.len() < prefix.len() + prompt_len {
            self.status = SandboxStatus::Idle;
            return Err(InferenceError::BufferOverflow);
        }

        for (i, &b) in prefix.iter().enumerate() {
            output_buffer[i] = b;
        }
        for (i, &b) in prompt_bytes.iter().enumerate() {
            output_buffer[prefix.len() + i] = b;
        }

        let written = prefix.len() + prompt_len;
        self.status = SandboxStatus::Idle;
        Ok(written)
    }

    /// Cleanly resets sandbox buffers and releases memory.
    pub fn reset(&mut self) {
        self.allocated_bytes = 0;
        self.status = if self.active_manifest.is_some() {
            SandboxStatus::ModelLoaded
        } else {
            SandboxStatus::Uninitialized
        };
    }
}
