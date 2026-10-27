//! Ind OS AIS Model Runtime Sandbox & Hardware Profile Subsystem (`indus-ais`)
//!
//! Constitutional References:
//! - REQ-AI-101: Model runtime sandbox
//! - REQ-AI-112: Dynamic hardware profile manager
//! - INV-AIS-001: Signed-model loader only
//! - INV-AIS-002: Bounded capability sandbox & resource quota
//! - INV-AIS-003: Deterministic hardware profile tiering
//! - Rule N-01: Zero external crates
//! - Rule N-04: Zero unsafe code in new modules

#![no_std]

pub mod loader;
pub mod profile;
pub mod sandbox;
pub mod schema;
pub mod intent;
pub mod policy;
pub mod consent;
pub mod types;

pub use loader::{SignedModelLoader, AUTHORIZED_SIGNING_AUTHORITY};
pub use profile::IndHardwareProfile;
pub use sandbox::{ModelRuntimeSandbox, MAX_INFERENCE_TICKS, MAX_WORKSPACE_BYTES};
pub use schema::{ActionCategory, SchemaDescriptor, SchemaError, SchemaRegistry, MAX_REGISTERED_SCHEMAS};
pub use intent::{IntentBroker, IntentError, IntentSource, TypedProposal};
pub use policy::{DryRunReport, PolicyLifecycleState, PolicyRegistry, PolicyStatus, RuleEffect, StandingPolicy};
pub use consent::{ConsentBrokerV2, ConsentCard, ConsentRiskTier, UserConsentDecision, MAX_PENDING_CONSENT_CARDS};
pub use types::{
    HardwareTier, InferenceConfig, InferenceError, ModelLoadError, ModelManifest,
    QuantizationType, SandboxStatus, CAP_AIS_ADMIN, CAP_AIS_INFER, CAP_AIS_LOAD,
};
