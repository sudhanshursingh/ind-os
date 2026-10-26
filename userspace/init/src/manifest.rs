//! Service Manifest Definitions for Ind OS Init (PID 1)
//!
//! Constitutional References:
//! - REQ-USE-201: Service manifests, dependency graph, crash-restart with backoff
//! - Rule N-01: Zero external crates
//! - Rule N-04: Zero unsafe code in new modules

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RestartPolicy {
    Always,
    OnFailure,
    Never,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Runlevel {
    CoreSecurity = 0,     // Audit bus, policy engine, consent store
    StorageDrivers = 1,   // VFS, IndFS, block device
    SystemServices = 2,   // Device manager, network stack, telemetry
    InteractiveShell = 3, // Shell, user terminal session
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ServiceManifest {
    pub name: &'static str,
    pub binary_path: &'static str,
    pub runlevel: Runlevel,
    pub restart_policy: RestartPolicy,
    pub max_retries: u32,
    pub initial_backoff_ms: u64,
    pub max_backoff_ms: u64,
    pub required_caps: u32,
    pub dependencies: &'static [&'static str],
}

impl ServiceManifest {
    pub const fn new(
        name: &'static str,
        binary_path: &'static str,
        runlevel: Runlevel,
        restart_policy: RestartPolicy,
        max_retries: u32,
        initial_backoff_ms: u64,
        max_backoff_ms: u64,
        required_caps: u32,
        dependencies: &'static [&'static str],
    ) -> Self {
        Self {
            name,
            binary_path,
            runlevel,
            restart_policy,
            max_retries,
            initial_backoff_ms,
            max_backoff_ms,
            required_caps,
            dependencies,
        }
    }

    pub fn validate(&self) -> bool {
        if self.name.is_empty() || self.binary_path.is_empty() {
            return false;
        }
        if self.initial_backoff_ms == 0 || self.max_backoff_ms < self.initial_backoff_ms {
            return false;
        }
        true
    }
}

/// Baseline system service manifests for Phase 5 Developer OS bring-up.
pub static BASELINE_MANIFESTS: [ServiceManifest; 4] = [
    ServiceManifest::new(
        "auditd",
        "/bin/auditd",
        Runlevel::CoreSecurity,
        RestartPolicy::Always,
        5,
        10,
        1000,
        0x0001,
        &[],
    ),
    ServiceManifest::new(
        "vfs-indfs",
        "/bin/indfs",
        Runlevel::StorageDrivers,
        RestartPolicy::Always,
        5,
        20,
        2000,
        0x0002,
        &["auditd"],
    ),
    ServiceManifest::new(
        "devmgr",
        "/bin/devmgr",
        Runlevel::SystemServices,
        RestartPolicy::Always,
        3,
        20,
        2000,
        0x0004,
        &["vfs-indfs"],
    ),
    ServiceManifest::new(
        "shell",
        "/bin/shell",
        Runlevel::InteractiveShell,
        RestartPolicy::Always,
        10,
        50,
        5000,
        0x0008,
        &["devmgr", "vfs-indfs"],
    ),
];
