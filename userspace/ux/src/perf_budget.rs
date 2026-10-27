//! Performance Budget Enforcement (SP-67, REQ-PERF Series)
//!
//! Architectural Invariants:
//! - INV-PERF-001: Frame-time budget (<16.6ms at p95)
//! - INV-PERF-002: App cold-start latency (<=250ms)
//! - INV-PERF-003: Per-app resident memory budget (<=16MB standard baseline)

pub const MAX_FRAME_TIME_US: u32 = 16_600; // 16.6ms (60 FPS)
pub const MAX_COLD_START_MS: u32 = 250;     // 250ms cold start deadline
pub const MAX_APP_MEMORY_KB: usize = 16384; // 16MB default G4 budget

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PerfBudgetError {
    FrameTimeExceeded,
    ColdStartDeadlineMissed,
    MemoryBudgetExceeded,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PerfBudgetRecord {
    pub p95_frame_time_us: u32,
    pub cold_start_ms: u32,
    pub app_memory_kb: usize,
}

pub struct PerfBudgetAuditor {
    records: [Option<PerfBudgetRecord>; 8],
    count: usize,
}

impl PerfBudgetAuditor {
    pub const fn new() -> Self {
        Self {
            records: [None; 8],
            count: 0,
        }
    }

    pub fn audit_frame_time(&self, frame_time_us: u32) -> Result<(), PerfBudgetError> {
        if frame_time_us > MAX_FRAME_TIME_US {
            return Err(PerfBudgetError::FrameTimeExceeded);
        }
        Ok(())
    }

    pub fn audit_cold_start(&self, cold_start_ms: u32) -> Result<(), PerfBudgetError> {
        if cold_start_ms > MAX_COLD_START_MS {
            return Err(PerfBudgetError::ColdStartDeadlineMissed);
        }
        Ok(())
    }

    pub fn audit_memory_allocation(&self, memory_kb: usize) -> Result<(), PerfBudgetError> {
        if memory_kb > MAX_APP_MEMORY_KB {
            return Err(PerfBudgetError::MemoryBudgetExceeded);
        }
        Ok(())
    }

    pub fn record_benchmark(
        &mut self,
        p95_frame_time_us: u32,
        cold_start_ms: u32,
        app_memory_kb: usize,
    ) -> Result<(), PerfBudgetError> {
        self.audit_frame_time(p95_frame_time_us)?;
        self.audit_cold_start(cold_start_ms)?;
        self.audit_memory_allocation(app_memory_kb)?;

        for slot in self.records.iter_mut() {
            if slot.is_none() {
                *slot = Some(PerfBudgetRecord {
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
