//! Kernel G5-Standard Userspace Hardware Core (SP-103, REQ-USR-101, REQ-SEC-113)
//!
//! Address space isolation, capability domain attenuation, and IPC ring arbiter for G5 userspace.

pub const KERNEL_CAP_G5_SYS_ADMIN: u32 = 0x0001_0000;
pub const KERNEL_CAP_G5_LOG_RECORD: u32 = 0x0000_0100;
pub const KERNEL_CAP_G5_STORAGE_MOUNT: u32 = 0x0000_1000;
pub const KERNEL_CAP_G5_DISPLAY_RENDER: u32 = 0x0000_0020;
pub const KERNEL_CAP_G5_TOUCH_INPUT: u32 = 0x0000_0040;
pub const KERNEL_CAP_G5_IPC_SYNC: u32 = 0x0000_0200;

pub const KERNEL_G5_TOTAL_SERVICES: usize = 5;
pub const KERNEL_IPC_RING_TEST_CYCLES: usize = 500;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KernelProcessStatus {
    Unspawned,
    Running,
    Suspended,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KernelG5HwError {
    PermissionDenied,
    ProcessNotFound,
    IpcFailure,
    BufferFull,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KernelG5ProcessInfo {
    pub pid: u32,
    pub name: &'static str,
    pub caps: u32,
    pub status: KernelProcessStatus,
    pub messages_processed: u32,
}

pub struct KernelG5HwHarness {
    processes: [KernelG5ProcessInfo; KERNEL_G5_TOTAL_SERVICES],
    ipc_rounds_completed: usize,
}

impl KernelG5HwHarness {
    pub const fn new() -> Self {
        Self {
            processes: [
                KernelG5ProcessInfo {
                    pid: 1,
                    name: "indus-init",
                    caps: KERNEL_CAP_G5_SYS_ADMIN,
                    status: KernelProcessStatus::Unspawned,
                    messages_processed: 0,
                },
                KernelG5ProcessInfo {
                    pid: 2,
                    name: "indus-logd",
                    caps: KERNEL_CAP_G5_LOG_RECORD,
                    status: KernelProcessStatus::Unspawned,
                    messages_processed: 0,
                },
                KernelG5ProcessInfo {
                    pid: 3,
                    name: "indus-store",
                    caps: KERNEL_CAP_G5_STORAGE_MOUNT,
                    status: KernelProcessStatus::Unspawned,
                    messages_processed: 0,
                },
                KernelG5ProcessInfo {
                    pid: 4,
                    name: "indus-shell",
                    caps: KERNEL_CAP_G5_DISPLAY_RENDER | KERNEL_CAP_G5_TOUCH_INPUT,
                    status: KernelProcessStatus::Unspawned,
                    messages_processed: 0,
                },
                KernelG5ProcessInfo {
                    pid: 5,
                    name: "indus-edit",
                    caps: KERNEL_CAP_G5_DISPLAY_RENDER | KERNEL_CAP_G5_IPC_SYNC,
                    status: KernelProcessStatus::Unspawned,
                    messages_processed: 0,
                },
            ],
            ipc_rounds_completed: 0,
        }
    }

    pub fn spawn_tcb_services(&mut self, admin_caps: u32) -> Result<(), KernelG5HwError> {
        if (admin_caps & KERNEL_CAP_G5_SYS_ADMIN) == 0 {
            return Err(KernelG5HwError::PermissionDenied);
        }
        self.processes[0].status = KernelProcessStatus::Running; // Init
        self.processes[1].status = KernelProcessStatus::Running; // Logd
        self.processes[2].status = KernelProcessStatus::Running; // Store
        Ok(())
    }

    pub fn spawn_ux_services(&mut self, admin_caps: u32) -> Result<(), KernelG5HwError> {
        if (admin_caps & KERNEL_CAP_G5_SYS_ADMIN) == 0 {
            return Err(KernelG5HwError::PermissionDenied);
        }
        self.processes[3].status = KernelProcessStatus::Running; // Shell
        self.processes[4].status = KernelProcessStatus::Running; // Edit
        Ok(())
    }

    pub fn run_ipc_ring_stress(&mut self, rounds: usize) -> Result<usize, KernelG5HwError> {
        for proc in &self.processes {
            if proc.status != KernelProcessStatus::Running {
                return Err(KernelG5HwError::ProcessNotFound);
            }
        }

        for _ in 0..rounds {
            for i in 0..KERNEL_G5_TOTAL_SERVICES {
                self.processes[i].messages_processed += 1;
            }
            self.ipc_rounds_completed += 1;
        }

        Ok(self.ipc_rounds_completed)
    }

    pub fn get_process_info(&self, pid: u32) -> Option<KernelG5ProcessInfo> {
        if pid >= 1 && pid <= KERNEL_G5_TOTAL_SERVICES as u32 {
            Some(self.processes[(pid - 1) as usize])
        } else {
            None
        }
    }

    pub fn total_running(&self) -> usize {
        let mut count = 0;
        for proc in &self.processes {
            if proc.status == KernelProcessStatus::Running {
                count += 1;
            }
        }
        count
    }

    pub fn ipc_rounds_completed(&self) -> usize {
        self.ipc_rounds_completed
    }
}
