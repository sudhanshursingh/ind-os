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
pub mod memory;
pub mod explain;
pub mod feedback;
pub mod sentinel;
pub mod quality;
pub mod redteam;
pub mod types;
pub mod arm64_runtime;
pub mod thermal_governor;
pub mod intent_hw;

pub use loader::{SignedModelLoader, AUTHORIZED_SIGNING_AUTHORITY};
pub use profile::IndHardwareProfile;
pub use sandbox::{ModelRuntimeSandbox, MAX_INFERENCE_TICKS, MAX_WORKSPACE_BYTES};
pub use schema::{ActionCategory, SchemaDescriptor, SchemaError, SchemaRegistry, MAX_REGISTERED_SCHEMAS};
pub use intent::{IntentBroker, IntentError, IntentSource, TypedProposal};
pub use policy::{DryRunReport, PolicyLifecycleState, PolicyRegistry, PolicyStatus, RuleEffect, StandingPolicy};
pub use consent::{ConsentBrokerV2, ConsentCard, ConsentRiskTier, UserConsentDecision, MAX_PENDING_CONSENT_CARDS};
pub use memory::{MemoryItem, MemoryScope, MemoryStatus, PersonalMemoryStore, MAX_MEMORY_ITEMS};
pub use explain::{ExplanationEngineV1, ExplanationReport, ProvenanceRecord, MAX_PROVENANCE_RECORDS};
pub use feedback::{FeedbackArbiter, HintCategory, TelemetryHint, CONSENT_UI_PRIORITY, MAX_AI_ADVISORY_PRIORITY, REALTIME_BASE_PRIORITY};
pub use sentinel::{AnomalyAlert, AnomalySentinelV1, AnomalyType, DEFAULT_BASELINE_THRESHOLD_FACTOR, MAX_SENTINEL_ALERTS};
pub use quality::{EvaluationSummary, IntentQualityHarness, IntentTestCase, EVAL_CORPUS_SIZE, MIN_COMPLETION_RATE_PERCENT};
pub use redteam::{AttackType, RedTeamAttackScenario, RedTeamHarness};
pub use arm64_runtime::{
    AisArm64RuntimeHarness, AisRuntimeError, AisRuntimeState,
    AIS_MAX_RAM_BYTES, CAP_AIS_EXEC, TARGET_INFERENCE_LATENCY_MAX_MS,
};
pub use thermal_governor::{
    AisThermalGovernorHarness, AisThermalState,
    AIS_TEMP_EMERGENCY_HALT_MC, AIS_TEMP_NORMAL_MC,
    AIS_TEMP_RECOVERY_MC, AIS_TEMP_THROTTLE_MC,
};
pub use intent_hw::{
    HardwareIntentBrokerHarness, HardwareIntentError,
    HardwareIntentState, HardwareProposalCard,
    CAP_INTENT_DISPATCH,
};
pub use types::{
    HardwareTier, InferenceConfig, InferenceError, ModelLoadError, ModelManifest,
    QuantizationType, SandboxStatus, CAP_AIS_ADMIN, CAP_AIS_INFER, CAP_AIS_LOAD,
};
