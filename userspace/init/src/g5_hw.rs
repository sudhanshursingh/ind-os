//! Ind OS G5-Standard Userspace Hardware Coordinator (SP-103)
//!
//! Standards: Bible §6.3, §8.15, Table 19.2; REQ-USR-101, REQ-SEC-113
//! Invariants: INV-G5-HW-001..003
//! Rule N-01: Zero external crates
//! Rule N-04: Pure memory-safe Rust

pub const CAP_SYS_ADMIN: u64 = 0x0001_0000_0000_0000;
pub const CAP_LOG_RECORD: u64 = 0x0000_0000_0000_0100;
pub const CAP_STORAGE_MOUNT: u64 = 0x0000_0000_0001_0000;
pub const CAP_DISPLAY_RENDER: u64 = 0x0000_0000_0000_0020;
pub const CAP_TOUCH_INPUT: u64 = 0x0000_0000_0000_0040;
pub const CAP_IPC_SYNC: u64 = 0x0000_0000_0000_0200;

pub const G5_TOTAL_SERVICES: usize = 5;
pub const IPC_RING_TEST_CYCLES: usize = 500;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessStatus {
    Unspawned,
    Running,
    Suspended,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum G5HwError {
    PermissionDenied,
    ProcessNotFound,
    IpcFailure,
    BufferFull,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct G5ProcessInfo {
    pub pid: u32,
    pub name: &'static str,
    pub caps: u64,
    pub status: ProcessStatus,
    pub messages_processed: u32,
}

pub struct G5HwCoordinator {
    processes: [G5ProcessInfo; G5_TOTAL_SERVICES],
    ipc_rounds_completed: usize,
}

impl G5HwCoordinator {
    pub const fn new() -> Self {
        Self {
            processes: [
                G5ProcessInfo {
                    pid: 1,
                    name: "indus-init",
                    caps: CAP_SYS_ADMIN,
                    status: ProcessStatus::Unspawned,
                    messages_processed: 0,
                },
                G5ProcessInfo {
                    pid: 2,
                    name: "indus-logd",
                    caps: CAP_LOG_RECORD,
                    status: ProcessStatus::Unspawned,
                    messages_processed: 0,
                },
                G5ProcessInfo {
                    pid: 3,
                    name: "indus-store",
                    caps: CAP_STORAGE_MOUNT,
                    status: ProcessStatus::Unspawned,
                    messages_processed: 0,
                },
                G5ProcessInfo {
                    pid: 4,
                    name: "indus-shell",
                    caps: CAP_DISPLAY_RENDER | CAP_TOUCH_INPUT,
                    status: ProcessStatus::Unspawned,
                    messages_processed: 0,
                },
                G5ProcessInfo {
                    pid: 5,
                    name: "indus-edit",
                    caps: CAP_DISPLAY_RENDER | CAP_IPC_SYNC,
                    status: ProcessStatus::Unspawned,
                    messages_processed: 0,
                },
            ],
            ipc_rounds_completed: 0,
        }
    }

    pub fn spawn_tcb_services(&mut self, admin_caps: u64) -> Result<(), G5HwError> {
        if (admin_caps & CAP_SYS_ADMIN) == 0 {
            return Err(G5HwError::PermissionDenied);
        }
        self.processes[0].status = ProcessStatus::Running; // Init
        self.processes[1].status = ProcessStatus::Running; // Logd
        self.processes[2].status = ProcessStatus::Running; // Store
        Ok(())
    }

    pub fn spawn_ux_services(&mut self, admin_caps: u64) -> Result<(), G5HwError> {
        if (admin_caps & CAP_SYS_ADMIN) == 0 {
            return Err(G5HwError::PermissionDenied);
        }
        self.processes[3].status = ProcessStatus::Running; // Shell
        self.processes[4].status = ProcessStatus::Running; // Edit
        Ok(())
    }

    pub fn run_ipc_ring_stress(&mut self, rounds: usize) -> Result<usize, G5HwError> {
        for proc in &self.processes {
            if proc.status != ProcessStatus::Running {
                return Err(G5HwError::ProcessNotFound);
            }
        }

        for _ in 0..rounds {
            for i in 0..G5_TOTAL_SERVICES {
                self.processes[i].messages_processed += 1;
            }
            self.ipc_rounds_completed += 1;
        }

        Ok(self.ipc_rounds_completed)
    }

    pub fn get_process_info(&self, pid: u32) -> Option<G5ProcessInfo> {
        if pid >= 1 && pid <= G5_TOTAL_SERVICES as u32 {
            Some(self.processes[(pid - 1) as usize])
        } else {
            None
        }
    }

    pub fn total_running(&self) -> usize {
        let mut count = 0;
        for proc in &self.processes {
            if proc.status == ProcessStatus::Running {
                count += 1;
            }
        }
        count
    }

    pub fn ipc_rounds_completed(&self) -> usize {
        self.ipc_rounds_completed
    }
}
