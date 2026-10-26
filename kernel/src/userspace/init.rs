//! Kernel-Side Init Process & Service Manager Subsystem for IndK
//!
//! Constitutional References:
//! - REQ-USE-201: Init system: service manifests, dependency graph, crash-restart with backoff, boot logs
//! - INV-INIT-001: Deterministic Dependency Ordering
//! - INV-INIT-002: Crash-Restart Containment & Exponential Backoff
//! - Rule N-01: Zero external crates
//! - Rule N-04: Zero unsafe code in new modules

pub const MAX_INIT_SERVICES: usize = 16;
pub const MAX_INIT_BOOT_LOGS: usize = 32;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RestartPolicy {
    Always,
    OnFailure,
    Never,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Runlevel {
    CoreSecurity = 0,
    StorageDrivers = 1,
    SystemServices = 2,
    InteractiveShell = 3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServiceState {
    Stopped,
    Starting,
    Running,
    Crashed,
    BackingOff,
    Dead,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InitError {
    DependencyCycleDetected,
    MissingDependency,
    MaxServicesExceeded,
    InvalidManifest,
    ServiceNotFound,
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

pub static KERNEL_BASELINE_SERVICES: [ServiceManifest; 4] = [
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ServiceProcess {
    pub manifest: ServiceManifest,
    pub state: ServiceState,
    pub pid: Option<u32>,
    pub retry_count: u32,
    pub current_backoff_ms: u64,
    pub total_crashes: u32,
}

impl ServiceProcess {
    pub const fn new(manifest: ServiceManifest) -> Self {
        Self {
            manifest,
            state: ServiceState::Stopped,
            pid: None,
            retry_count: 0,
            current_backoff_ms: manifest.initial_backoff_ms,
            total_crashes: 0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BootStageRecord {
    pub timestamp_ticks: u64,
    pub service_name: &'static str,
    pub old_state: ServiceState,
    pub new_state: ServiceState,
    pub exit_code: Option<i32>,
    pub message: &'static str,
}

pub struct KernelInitSubsystem {
    services: [Option<ServiceProcess>; MAX_INIT_SERVICES],
    count: usize,
    records: [Option<BootStageRecord>; MAX_INIT_BOOT_LOGS],
    record_count: usize,
}

impl KernelInitSubsystem {
    pub const fn new() -> Self {
        Self {
            services: [None; MAX_INIT_SERVICES],
            count: 0,
            records: [None; MAX_INIT_BOOT_LOGS],
            record_count: 0,
        }
    }

    pub fn load_baseline(&mut self) -> Result<(), InitError> {
        for &manifest in &KERNEL_BASELINE_SERVICES {
            self.register_service(manifest)?;
        }
        Ok(())
    }

    pub fn register_service(&mut self, manifest: ServiceManifest) -> Result<usize, InitError> {
        if !manifest.validate() {
            return Err(InitError::InvalidManifest);
        }
        if self.count >= MAX_INIT_SERVICES {
            return Err(InitError::MaxServicesExceeded);
        }
        let idx = self.count;
        self.services[idx] = Some(ServiceProcess::new(manifest));
        self.count += 1;
        Ok(idx)
    }

    pub fn get_service(&self, name: &str) -> Option<&ServiceProcess> {
        for slot in self.services.iter().take(self.count) {
            if let Some(p) = slot {
                if p.manifest.name == name {
                    return Some(p);
                }
            }
        }
        None
    }

    pub fn service_count(&self) -> usize {
        self.count
    }

    /// Resolves topological startup ordering using in-degree elimination (Kahn's algorithm, `INV-INIT-001`).
    pub fn resolve_order(&self, order: &mut [&'static str; MAX_INIT_SERVICES]) -> Result<usize, InitError> {
        for i in 0..self.count {
            if let Some(proc) = self.services[i] {
                for &dep in proc.manifest.dependencies {
                    if self.get_service(dep).is_none() {
                        return Err(InitError::MissingDependency);
                    }
                }
            }
        }

        let mut in_degree = [0usize; MAX_INIT_SERVICES];
        for i in 0..self.count {
            if let Some(proc) = self.services[i] {
                in_degree[i] = proc.manifest.dependencies.len();
            }
        }

        let mut queue = [0usize; MAX_INIT_SERVICES];
        let mut q_head = 0;
        let mut q_tail = 0;

        for i in 0..self.count {
            if in_degree[i] == 0 {
                queue[q_tail] = i;
                q_tail += 1;
            }
        }

        let mut sorted_count = 0;
        while q_head < q_tail {
            let u_idx = queue[q_head];
            q_head += 1;

            let u_name = self.services[u_idx].unwrap().manifest.name;
            order[sorted_count] = u_name;
            sorted_count += 1;

            for v_idx in 0..self.count {
                if let Some(v_proc) = self.services[v_idx] {
                    for &dep in v_proc.manifest.dependencies {
                        if dep == u_name {
                            in_degree[v_idx] -= 1;
                            if in_degree[v_idx] == 0 {
                                queue[q_tail] = v_idx;
                                q_tail += 1;
                            }
                        }
                    }
                }
            }
        }

        if sorted_count != self.count {
            return Err(InitError::DependencyCycleDetected);
        }

        Ok(sorted_count)
    }

    pub fn start_service(&mut self, idx: usize, pid: u32) -> Result<(), InitError> {
        let (s_name, old) = {
            let proc = self.services.get_mut(idx).and_then(|p| p.as_mut()).ok_or(InitError::ServiceNotFound)?;
            let old = proc.state;
            proc.state = ServiceState::Running;
            proc.pid = Some(pid);
            (proc.manifest.name, old)
        };
        self.log_transition(100, s_name, old, ServiceState::Running, None, "Service spawned into EL0");
        Ok(())
    }

    /// Handles service crash and calculates exponential backoff (`INV-INIT-002`).
    pub fn handle_crash(&mut self, idx: usize, exit_code: i32) -> Result<ServiceState, InitError> {
        let (s_name, old, new_state) = {
            let proc = self.services.get_mut(idx).and_then(|p| p.as_mut()).ok_or(InitError::ServiceNotFound)?;
            let old = proc.state;
            proc.total_crashes += 1;
            proc.retry_count += 1;
            proc.pid = None;

            match proc.manifest.restart_policy {
                RestartPolicy::Never => {
                    proc.state = ServiceState::Dead;
                }
                RestartPolicy::OnFailure | RestartPolicy::Always => {
                    if proc.retry_count > proc.manifest.max_retries && proc.manifest.restart_policy == RestartPolicy::OnFailure {
                        proc.state = ServiceState::Dead;
                    } else {
                        let shift = proc.retry_count.saturating_sub(1).min(10);
                        let computed_backoff = proc.manifest.initial_backoff_ms.saturating_mul(1 << shift);
                        proc.current_backoff_ms = computed_backoff.min(proc.manifest.max_backoff_ms);
                        proc.state = ServiceState::BackingOff;
                    }
                }
            }

            (proc.manifest.name, old, proc.state)
        };

        self.log_transition(200, s_name, old, new_state, Some(exit_code), "Service crashed; supervisor backoff engaged");
        Ok(new_state)
    }

    pub fn restart_service(&mut self, idx: usize, new_pid: u32) -> Result<(), InitError> {
        let (s_name, old) = {
            let proc = self.services.get_mut(idx).and_then(|p| p.as_mut()).ok_or(InitError::ServiceNotFound)?;
            if proc.state == ServiceState::Dead {
                return Err(InitError::InvalidManifest);
            }
            let old = proc.state;
            proc.state = ServiceState::Running;
            proc.pid = Some(new_pid);
            (proc.manifest.name, old)
        };
        self.log_transition(300, s_name, old, ServiceState::Running, None, "Service recovered and restarted");
        Ok(())
    }

    pub fn log_transition(
        &mut self,
        timestamp_ticks: u64,
        service_name: &'static str,
        old_state: ServiceState,
        new_state: ServiceState,
        exit_code: Option<i32>,
        message: &'static str,
    ) {
        let rec = BootStageRecord {
            timestamp_ticks,
            service_name,
            old_state,
            new_state,
            exit_code,
            message,
        };
        let idx = self.record_count % MAX_INIT_BOOT_LOGS;
        self.records[idx] = Some(rec);
        self.record_count += 1;
    }

    pub fn record_count(&self) -> usize {
        self.record_count
    }

    pub fn get_record(&self, idx: usize) -> Option<&BootStageRecord> {
        if idx >= self.record_count.min(MAX_INIT_BOOT_LOGS) {
            return None;
        }
        self.records[idx].as_ref()
    }

    pub fn get_process(&self, idx: usize) -> Option<&ServiceProcess> {
        self.services.get(idx).and_then(|p| p.as_ref())
    }
}
