//! Userspace Subsystems & Init Management for IndK
//!
//! Constitutional References:
//! - REQ-USE-201: Init system: service manifests, dependency graph, crash-restart with backoff, boot logs
//! - Rule N-01: Zero external crates
//! - Rule N-04: Zero unsafe code in new modules

pub mod init;

pub use init::{
    BootStageRecord, InitError, KernelInitSubsystem, RestartPolicy, Runlevel,
    ServiceManifest, ServiceProcess, ServiceState, KERNEL_BASELINE_SERVICES,
    MAX_INIT_BOOT_LOGS, MAX_INIT_SERVICES,
};
