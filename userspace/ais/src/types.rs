//! Ind OS AIS Types & Architectural Constants (SP-41)
//!
//! Standards:
//! - REQ-AI-101: Model runtime sandbox
//! - REQ-AI-112: Dynamic hardware profile manager

pub const CAP_AIS_LOAD: u32 = 0x10;
pub const CAP_AIS_INFER: u32 = 0x20;
pub const CAP_AIS_ADMIN: u32 = 0x40;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HardwareTier {
    Budget,   // <= 6 GB RAM
    Standard, // 6 GB - 12 GB RAM
    Flagship, // >= 12 GB RAM
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub enum QuantizationType {
    Q4_K_M,
    Q8_0,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InferenceConfig {
    pub tier: HardwareTier,
    pub quantization: QuantizationType,
    pub parameter_billions_x10: u32,
    pub context_tokens: u32,
    pub kv_cache_mb: u32,
    pub ram_limit_mb: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ModelManifest {
    pub model_id: u32,
    pub version: u32,
    pub weights_checksum: u32,
    pub signature_token: u32,
    pub required_ram_mb: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelLoadError {
    SignatureRequired,
    InvalidSignature,
    IntegrityMismatch,
    InsufficientQuota,
    PermissionDenied,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InferenceError {
    ModelNotLoaded,
    TimeBudgetExceeded,
    MemoryQuotaExceeded,
    PermissionDenied,
    BufferOverflow,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SandboxStatus {
    Uninitialized,
    ModelLoaded,
    Inferring,
    Idle,
    QuotaViolated,
}
