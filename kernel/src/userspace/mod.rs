//! Userspace Subsystems & Init Management for IndK
//!
//! Constitutional References:
//! - REQ-USE-201: Init system: service manifests, dependency graph, crash-restart with backoff, boot logs
//! - Rule N-01: Zero external crates
//! - Rule N-04: Zero unsafe code in new modules

pub mod init;
pub mod core_libs;
pub mod devmgr;
pub mod shell;
pub mod logd;
pub mod store;
pub mod devx;
pub mod qa;
pub mod quality;

pub use init::{
    BootStageRecord, InitError, KernelInitSubsystem, RestartPolicy, Runlevel,
    ServiceManifest, ServiceProcess, ServiceState, KERNEL_BASELINE_SERVICES,
    MAX_INIT_BOOT_LOGS, MAX_INIT_SERVICES,
};
pub use core_libs::{
    AllocError, KernelCapToken, KernelIpcClient, KernelUserHeap,
    CAP_RIGHTS_DELEGATE as CORE_CAP_DELEGATE, CAP_RIGHTS_READ as CORE_CAP_READ,
    CAP_RIGHTS_WRITE as CORE_CAP_WRITE, IpcError as CoreIpcError,
};
pub use devmgr::{
    KernelBusType, KernelDevError, KernelDeviceCapToken, KernelDeviceId,
    KernelDeviceManager, KernelDeviceType, KernelDmaCapToken, KernelHardwareDescriptor,
    KernelHotplugBus, KernelHotplugEvent, KernelIrqCapToken, KernelMmioCapToken,
};
pub use shell::{
    KernelCommandInvocation, KernelShellExecError, KernelShellParseError,
    KernelShellParser, KernelShellSession, KERNEL_SHELL_OUTPUT_BUF_SIZE,
};
pub use logd::{
    KernelDiagnosticCause, KernelDiagnosticRecord, KernelLogBus, KernelLogLevel,
    KernelLogMessage, KernelRedactionFilter, KernelWhyEngine, KERNEL_LOG_CAPACITY,
};
pub use store::{
    KernelConsentRecord, KernelConsentState, KernelSettingEntry,
    KernelSettingsAndConsentStore, KernelStoreError,
    CAP_STORE_ADMIN as KERNEL_CAP_STORE_ADMIN, CAP_STORE_READ as KERNEL_CAP_STORE_READ,
};
pub use devx::{
    KernelEditError, KernelRemoteShellBridge, KernelTextBuffer,
};
pub use qa::{
    KernelJourneyReport, KernelJourneyStatus, KernelSoakHarness, KernelSoakReport,
    KernelUserJourneyRunner,
};
pub use quality::{
    KernelManifestAudit, KernelQualityAuditor, KernelResourceAudit,
};
pub mod ais_runtime;
pub use ais_runtime::{
    KernelHardwareProfile, KernelHardwareTier, KernelInferenceConfig, KernelModelError,
    KernelModelSandbox, KERNEL_AIS_AUTHORIZED_SIGNATURE, KERNEL_CAP_AIS_INFER,
    KERNEL_CAP_AIS_LOAD,
};
pub mod intent_broker;
pub use intent_broker::{
    KernelActionCategory, KernelIntentBroker, KernelSchemaDescriptor, KernelSchemaRegistry,
    KernelTypedProposal,
};
pub mod policy_v2;
pub use policy_v2::{
    KernelDryRunReport, KernelPolicyEngineV2, KernelPolicyLifecycleState, KernelStandingPolicy,
    MAX_STANDING_POLICIES,
};
pub mod consent_v2;
pub use consent_v2::{
    KernelConsentBrokerV2, KernelConsentCard, KernelConsentRiskTier, KernelUserConsentDecision,
    MAX_PENDING_CONSENT_CARDS,
};
pub mod memory_store;
pub use memory_store::{
    KernelMemoryItem, KernelMemoryScope, KernelMemoryStatus, KernelMemoryStore,
    KERNEL_MAX_MEMORY_ITEMS,
};
pub mod explain_v1;
pub use explain_v1::{
    KernelExplanationEngineV1, KernelExplanationReport, KernelProvenanceRecord,
    KERNEL_MAX_PROVENANCE_RECORDS,
};
