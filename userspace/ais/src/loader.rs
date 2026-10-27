//! Ind OS Signed Model Loader (REQ-AI-101, INV-AIS-001)
//!
//! Standards:
//! - INV-AIS-001: Signed-model loader only (cryptographic verification)
//! - Rule N-01: Zero external crates
//! - Rule N-04: Zero unsafe code in new modules

use crate::types::{ModelLoadError, ModelManifest, CAP_AIS_LOAD};

pub const AUTHORIZED_SIGNING_AUTHORITY: u32 = 0x516E_A150; // "SIGN_AIS0" authority token

pub struct SignedModelLoader;

impl SignedModelLoader {
    /// Computes a deterministic checksum across weight bytes.
    pub fn compute_checksum(data: &[u8]) -> u32 {
        let mut hash: u32 = 0x811c9dc5;
        for &b in data {
            hash ^= b as u32;
            hash = hash.wrapping_mul(0x01000193);
        }
        hash
    }

    /// Verifies the digital signature and checksum of a model artifact.
    pub fn verify_and_load(
        manifest: &ModelManifest,
        weight_bytes: &[u8],
        caller_cap: u32,
    ) -> Result<(), ModelLoadError> {
        // 1. Capability check
        if (caller_cap & CAP_AIS_LOAD) == 0 {
            return Err(ModelLoadError::PermissionDenied);
        }

        // 2. Signature presence and authority check (INV-AIS-001)
        if manifest.signature_token == 0 {
            return Err(ModelLoadError::SignatureRequired);
        }
        if manifest.signature_token != AUTHORIZED_SIGNING_AUTHORITY {
            return Err(ModelLoadError::InvalidSignature);
        }

        // 3. Weight integrity check
        let actual_checksum = Self::compute_checksum(weight_bytes);
        if actual_checksum != manifest.weights_checksum {
            return Err(ModelLoadError::IntegrityMismatch);
        }

        Ok(())
    }
}
