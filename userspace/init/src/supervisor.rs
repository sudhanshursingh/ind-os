//! Service Supervision & Crash-Restart State Machine for Ind OS Init
//!
//! Constitutional References:
//! - REQ-USE-201: Crash-restart with backoff
//! - INV-INIT-002: Crash-Restart Containment & Exponential Backoff
//! - Rule N-01: Zero external crates
//! - Rule N-04: Zero unsafe code

use crate::graph::{InitError, MAX_SERVICES};
use crate::manifest::{RestartPolicy, ServiceManifest};

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

pub struct ServiceSupervisor {
    processes: [Option<ServiceProcess>; MAX_SERVICES],
    count: usize,
}

impl ServiceSupervisor {
    pub const fn new() -> Self {
        Self {
            processes: [None; MAX_SERVICES],
            count: 0,
        }
    }

    pub fn register(&mut self, manifest: ServiceManifest) -> Result<usize, InitError> {
        if self.count >= MAX_SERVICES {
            return Err(InitError::MaxServicesExceeded);
        }
        let idx = self.count;
        self.processes[idx] = Some(ServiceProcess::new(manifest));
        self.count += 1;
        Ok(idx)
    }

    pub fn start(&mut self, idx: usize, pid: u32) -> Result<(), InitError> {
        let proc = self.processes.get_mut(idx).and_then(|p| p.as_mut()).ok_or(InitError::ServiceNotFound)?;
        proc.state = ServiceState::Running;
        proc.pid = Some(pid);
        Ok(())
    }

    /// Handles service termination/fault, applying exponential backoff calculation (`INV-INIT-002`).
    pub fn handle_crash(&mut self, idx: usize, _exit_code: i32) -> Result<ServiceState, InitError> {
        let proc = self.processes.get_mut(idx).and_then(|p| p.as_mut()).ok_or(InitError::ServiceNotFound)?;
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
                    // Exponential backoff: initial_backoff_ms * 2^(retry_count - 1), capped at max_backoff_ms
                    let shift = proc.retry_count.saturating_sub(1).min(10);
                    let computed_backoff = proc.manifest.initial_backoff_ms.saturating_mul(1 << shift);
                    proc.current_backoff_ms = computed_backoff.min(proc.manifest.max_backoff_ms);
                    proc.state = ServiceState::BackingOff;
                }
            }
        }

        Ok(proc.state)
    }

    pub fn restart(&mut self, idx: usize, new_pid: u32) -> Result<(), InitError> {
        let proc = self.processes.get_mut(idx).and_then(|p| p.as_mut()).ok_or(InitError::ServiceNotFound)?;
        if proc.state == ServiceState::Dead {
            return Err(InitError::InvalidManifest);
        }
        proc.state = ServiceState::Running;
        proc.pid = Some(new_pid);
        Ok(())
    }

    pub fn get_process(&self, idx: usize) -> Option<&ServiceProcess> {
        self.processes.get(idx).and_then(|p| p.as_ref())
    }

    pub fn process_count(&self) -> usize {
        self.count
    }
}
