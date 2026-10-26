//! IndOS SMP CPU Management & Core Bring-Up
//! Compliant with Bible §8.10 (SP-28), INDOS-SPEC-SMP-01, Rule N-01, Rule N-04, and REQ-KRN-109.

use crate::task::ProcessId;

pub type CpuId = u8;
pub const MAX_CPUS: usize = 4;

/// Operational lifecycle state of a CPU core
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CpuState {
    Offline,
    Starting,
    Online,
    Halted,
}

/// Core descriptor tracking topology, architectural registers, and execution state
#[derive(Clone, Copy, Debug)]
pub struct CpuDescriptor {
    pub cpu_id: CpuId,
    pub mpidr: u64,
    pub is_primary: bool,
    pub state: CpuState,
    pub ticks: u64,
    pub current_pid: Option<ProcessId>,
}

impl CpuDescriptor {
    pub const fn empty(cpu_id: CpuId) -> Self {
        Self {
            cpu_id,
            mpidr: 0,
            is_primary: false,
            state: CpuState::Offline,
            ticks: 0,
            current_pid: None,
        }
    }
}

/// Central CPU topology manager orchestrating multi-core bring-up and health tracking
pub struct CpuManager {
    pub cpus: [CpuDescriptor; MAX_CPUS],
    pub online_count: usize,
}

impl CpuManager {
    pub const fn new() -> Self {
        Self {
            cpus: [
                CpuDescriptor::empty(0),
                CpuDescriptor::empty(1),
                CpuDescriptor::empty(2),
                CpuDescriptor::empty(3),
            ],
            online_count: 0,
        }
    }

    /// Initializes primary bootstrap processor (CPU 0)
    pub fn init_primary(&mut self, mpidr: u64) {
        self.cpus[0].mpidr = mpidr;
        self.cpus[0].is_primary = true;
        self.cpus[0].state = CpuState::Online;
        self.online_count = 1;
    }

    /// Brings up a secondary core via architectural handshake (INV-SMP-001)
    pub fn bringup_secondary(&mut self, cpu_id: CpuId, mpidr: u64) -> Result<(), &'static str> {
        let id = cpu_id as usize;
        if id >= MAX_CPUS {
            return Err("EINVAL: Invalid CPU ID (must be 0..3)");
        }
        if self.cpus[id].state == CpuState::Online {
            return Err("EBUSY: CPU core already online");
        }

        // Transition: Offline -> Starting -> Online
        self.cpus[id].mpidr = mpidr;
        self.cpus[id].is_primary = false;
        self.cpus[id].state = CpuState::Starting;

        // Perform simulated architectural core handshake
        self.cpus[id].state = CpuState::Online;
        self.online_count += 1;
        Ok(())
    }

    /// Checks if a core is online
    pub fn is_online(&self, cpu_id: CpuId) -> bool {
        let id = cpu_id as usize;
        if id < MAX_CPUS {
            self.cpus[id].state == CpuState::Online
        } else {
            false
        }
    }

    /// Retrieves CPU descriptor
    pub fn get_cpu(&self, cpu_id: CpuId) -> Option<&CpuDescriptor> {
        let id = cpu_id as usize;
        if id < MAX_CPUS {
            Some(&self.cpus[id])
        } else {
            None
        }
    }
}
