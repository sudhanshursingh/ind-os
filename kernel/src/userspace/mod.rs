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
pub mod feedback_loop;
pub use feedback_loop::{
    KernelFeedbackArbiter, KernelHintCategory, KernelTelemetryHint,
    KERNEL_CONSENT_UI_PRIORITY, KERNEL_MAX_AI_ADVISORY_PRIORITY, KERNEL_REALTIME_BASE_PRIORITY,
};
pub mod sentinel_v1;
pub use sentinel_v1::{
    KernelAnomalyAlert, KernelAnomalySentinelV1, KernelAnomalyType,
    KERNEL_DEFAULT_BASELINE_THRESHOLD_FACTOR, KERNEL_MAX_SENTINEL_ALERTS,
};
pub mod ais_quality;
pub use ais_quality::{
    KernelEvaluationSummary, KernelIntentQualityHarness, KernelIntentTestCase,
    KERNEL_EVAL_CORPUS_SIZE, KERNEL_MIN_COMPLETION_RATE_PERCENT,
};
pub mod ais_redteam;
pub use ais_redteam::{
    KernelAttackType, KernelRedTeamAttackScenario, KernelRedTeamHarness,
};
pub mod net_core;
pub use net_core::{
    KernelEtherType, KernelIpProtocol, KernelIpv4Address, KernelMacAddress, KernelNetError,
    KernelNetStack, KernelSocket, KernelSocketCapToken, KernelSocketTable, KernelSocketType,
    KernelTcpState, KERNEL_CAP_NET_BIND, KERNEL_CAP_NET_CONNECT, KERNEL_CAP_NET_LISTEN,
    KERNEL_CAP_NET_RAW, KERNEL_CAP_NET_RECV, KERNEL_CAP_NET_SEND, KERNEL_CAP_NET_SOCKET,
    KERNEL_DEFAULT_MTU, KERNEL_MAX_SOCKETS, KERNEL_SOCKET_BUF_SIZE,
};
pub mod tls_core;
pub use tls_core::{
    KernelCipherSuite, KernelTlsHarness, KERNEL_TLS_VERSION_1_3,
};
pub mod net_services;
pub use net_services::{
    KernelDhcpHarness, KernelDnsHarness, KernelFilterAction, KernelFirewallEngine,
    KernelFirewallRule, KernelPacketDirection, KERNEL_CAP_NET_FIREWALL_ADMIN,
    KERNEL_MAX_FIREWALL_RULES,
};
pub mod update_service;
pub use update_service::{
    KernelSlotId, KernelSlotState, KernelUpdateError, KernelUpdateManager, KernelUpdateManifest,
    KernelUpdatePolicy, KERNEL_AUTHORIZED_UPDATE_SIGNATURE,
};
pub mod ceremony_core;
pub use ceremony_core::{
    KernelCeremonyError, KernelCustodianKey, KernelCustodianStatus, KernelThresholdSigningEngine,
    KERNEL_MAX_CUSTODIANS, KERNEL_REQUIRED_THRESHOLD,
};
pub mod leak_sentinel;
pub use leak_sentinel::{
    KernelFlowClass, KernelLeakAlert, KernelLeakAnomalyType, KernelNetworkFlowDescriptor,
    KernelNetworkLeakSentinel, KernelSentinelAction, KERNEL_LEAK_BURST_THRESHOLD_FACTOR,
    KERNEL_MAX_LEAK_ALERTS, KERNEL_MAX_QUARANTINED_IPS,
};
pub mod net_interop;
pub use net_interop::{
    KernelFileTransferFrame, KernelFileTransferOp, KernelInteropError, KernelLoopbackBus,
    KernelNetRemoteShellSession, KernelNetworkFileTransferEngine, KERNEL_NETSHELL_AUTH_SECRET,
    KERNEL_NFTP_PAYLOAD_SIZE, KERNEL_REQUIRED_FILE_READ_CAP, KERNEL_REQUIRED_FILE_WRITE_CAP,
};
pub mod net_soak;
pub use net_soak::{
    KernelNetFuzzScenario, KernelNetSoakHarness, KernelNetSoakReport,
    KERNEL_DEFAULT_STORM_PACKET_COUNT,
};
pub mod net_reliability;
pub use net_reliability::{
    KernelNetAttackResult, KernelNetAttackType, KernelNetReliabilityHarness,
};








