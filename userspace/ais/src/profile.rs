//! Ind OS Dynamic Hardware Profile Manager (REQ-AI-112, RFC-INDOS-2026-001)
//!
//! Standards:
//! - INV-AIS-003: Deterministic telemetry-driven hardware profile tiering
//! - Rule N-01: Zero external crates
//! - Rule N-04: Zero unsafe code in new modules

use crate::types::{HardwareTier, InferenceConfig, QuantizationType};

pub struct IndHardwareProfile;

impl IndHardwareProfile {
    /// Selects the optimal inference configuration based on discovered physical RAM (in Megabytes).
    ///
    /// Tiers (per REQ-AI-112):
    /// - Budget (<= 6144 MB): Q4_K_M 1.5B, 2048 context, 128 MB KV cache, <= 1250 MB ceiling
    /// - Standard (6145 MB .. 12288 MB): Q4_K_M 3B, 4096 context, 256 MB KV cache, <= 3000 MB ceiling
    /// - Flagship (> 12288 MB): Q8_0 8B, 8192 context, 512 MB KV cache, <= 6000 MB ceiling
    pub fn select_profile(ram_mb: u32) -> InferenceConfig {
        if ram_mb <= 6144 {
            InferenceConfig {
                tier: HardwareTier::Budget,
                quantization: QuantizationType::Q4_K_M,
                parameter_billions_x10: 15, // 1.5B
                context_tokens: 2048,
                kv_cache_mb: 128,
                ram_limit_mb: 1250,
            }
        } else if ram_mb <= 12288 {
            InferenceConfig {
                tier: HardwareTier::Standard,
                quantization: QuantizationType::Q4_K_M,
                parameter_billions_x10: 30, // 3.0B
                context_tokens: 4096,
                kv_cache_mb: 256,
                ram_limit_mb: 3000,
            }
        } else {
            InferenceConfig {
                tier: HardwareTier::Flagship,
                quantization: QuantizationType::Q8_0,
                parameter_billions_x10: 80, // 8.0B
                context_tokens: 8192,
                kv_cache_mb: 512,
                ram_limit_mb: 6000,
            }
        }
    }
}
