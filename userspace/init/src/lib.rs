//! Ind OS Root Userspace Init System Library (PID 1)
//!
//! Constitutional References:
//! - REQ-USE-201: Init system: service manifests, dependency graph, crash-restart with backoff, boot logs
//! - INV-INIT-001: Deterministic Dependency Ordering
//! - INV-INIT-002: Crash-Restart Containment & Exponential Backoff
//! - Rule N-01: Zero external crates
//! - Rule N-04: Zero unsafe code in new modules

#![no_std]

pub mod graph;
pub mod logger;
pub mod manifest;
pub mod supervisor;
pub mod g5_hw;
pub mod hw_matrix;

pub use graph::{DependencyGraph, InitError, MAX_SERVICES};
pub use logger::{BootStageRecord, StructuredBootLogger, MAX_BOOT_LOGS};
pub use manifest::{RestartPolicy, Runlevel, ServiceManifest, BASELINE_MANIFESTS};
pub use supervisor::{ServiceProcess, ServiceState, ServiceSupervisor};
pub use g5_hw::{
    G5HwCoordinator, G5HwError, G5ProcessInfo, ProcessStatus, CAP_DISPLAY_RENDER, CAP_IPC_SYNC,
    CAP_LOG_RECORD, CAP_STORAGE_MOUNT, CAP_SYS_ADMIN, CAP_TOUCH_INPUT, G5_TOTAL_SERVICES,
    IPC_RING_TEST_CYCLES,
};
pub use hw_matrix::{
    HwMatrixAuditor, HwMatrixEntry, HwMatrixError, HwStatus, CAP_AUDIT_HARDWARE,
    OUT_OF_SCOPE_COUNT, PARTIAL_COUNT, TOTAL_MATRIX_ENTRIES, WORKS_COUNT,
};
pub mod recovery_audit;
pub use recovery_audit::{
    RecoveryAuditEngine, RecoveryError, RecoveryMode, RecoveryStatus, CAP_RECOVERY_ADMIN,
    QUALCOMM_EDL_PID, QUALCOMM_EDL_VID, SAHARA_HELLO_CMD, STOCK_IMAGE_CRC32,
};
pub mod endurance_soak;
pub use endurance_soak::{
    EnduranceSoakEngine, SoakError, SoakMetrics, CAP_STRESS_TEST, MASK_AIS, MASK_ALL_SUBSYSTEMS,
    MASK_AUDIO, MASK_DISPLAY, MASK_NETWORK, MASK_SENSORS, MASK_UFS, MAX_PERMITTED_TEMP_MC,
    SOAK_TOTAL_CYCLES,
};

/// Central Init Subsystem Controller coordinating bring-up and process supervision.
pub struct InitSubsystem {
    pub graph: DependencyGraph,
    pub supervisor: ServiceSupervisor,
    pub logger: StructuredBootLogger,
}

impl InitSubsystem {
    pub const fn new() -> Self {
        Self {
            graph: DependencyGraph::new(),
            supervisor: ServiceSupervisor::new(),
            logger: StructuredBootLogger::new(),
        }
    }

    /// Loads baseline system manifests into the dependency graph and supervisor.
    pub fn load_baseline(&mut self) -> Result<(), InitError> {
        for &manifest in &BASELINE_MANIFESTS {
            self.graph.add_service(manifest)?;
            self.supervisor.register(manifest)?;
        }
        Ok(())
    }

    /// Resolves topological startup sequence across registered services (`INV-INIT-001`).
    pub fn resolve_startup_order(&self, order: &mut [&'static str; MAX_SERVICES]) -> Result<usize, InitError> {
        self.graph.resolve_order(order)
    }
}
