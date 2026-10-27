//! Kernel Performance Budget Verification Harness (SP-67, REQ-PERF Series)
//!
//! Architectural Invariants:
//! - INV-PERF-001: Frame-time budget (<16.6ms at p95)
//! - INV-PERF-002: App cold-start latency (<=250ms)
//! - INV-PERF-003: Per-app resident memory budget (<=16MB standard baseline)

pub const KERNEL_MAX_FRAME_TIME_US: u32 = 16_600;
pub const KERNEL_MAX_COLD_START_MS: u32 = 250;
pub const KERNEL_MAX_APP_MEMORY_KB: usize = 16384;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KernelPerfBudgetError {
    FrameTimeExceeded,
    ColdStartDeadlineMissed,
    MemoryBudgetExceeded,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KernelPerfBudgetRecord {
    pub p95_frame_time_us: u32,
    pub cold_start_ms: u32,
    pub app_memory_kb: usize,
}

pub struct KernelPerfBudgetAuditor {
    records: [Option<KernelPerfBudgetRecord>; 8],
    count: usize,
}

impl KernelPerfBudgetAuditor {
    pub const fn new() -> Self {
        Self {
            records: [None; 8],
            count: 0,
        }
    }

    pub fn audit_frame_time(&self, frame_time_us: u32) -> Result<(), KernelPerfBudgetError> {
        if frame_time_us > KERNEL_MAX_FRAME_TIME_US {
            return Err(KernelPerfBudgetError::FrameTimeExceeded);
        }
        Ok(())
    }

    pub fn audit_cold_start(&self, cold_start_ms: u32) -> Result<(), KernelPerfBudgetError> {
        if cold_start_ms > KERNEL_MAX_COLD_START_MS {
            return Err(KernelPerfBudgetError::ColdStartDeadlineMissed);
        }
        Ok(())
    }

    pub fn audit_memory_allocation(&self, memory_kb: usize) -> Result<(), KernelPerfBudgetError> {
        if memory_kb > KERNEL_MAX_APP_MEMORY_KB {
            return Err(KernelPerfBudgetError::MemoryBudgetExceeded);
        }
        Ok(())
    }

    pub fn record_benchmark(
        &mut self,
        p95_frame_time_us: u32,
        cold_start_ms: u32,
        app_memory_kb: usize,
    ) -> Result<(), KernelPerfBudgetError> {
        self.audit_frame_time(p95_frame_time_us)?;
        self.audit_cold_start(cold_start_ms)?;
        self.audit_memory_allocation(app_memory_kb)?;

        for slot in self.records.iter_mut() {
            if slot.is_none() {
                *slot = Some(KernelPerfBudgetRecord {
                    p95_frame_time_us,
                    cold_start_ms,
                    app_memory_kb,
                });
                self.count += 1;
                return Ok(());
            }
        }
        Ok(())
    }

    pub fn benchmark_count(&self) -> usize {
        self.count
    }
}
