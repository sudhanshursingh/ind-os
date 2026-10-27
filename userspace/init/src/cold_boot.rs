//! Cold-Boot Stability & Reproducibility Verification Engine
//!
//! Standards: Bible §6.3, §8.15, Table 19.2, REQ-QA-201, INV-BOOT-COLD-001..003
//! Rule N-01: Zero external crates in TCB
//! Rule N-04: 100% safe Rust in userspace

pub const CAP_COLD_BOOT_VERIFY: u32 = 0x0000_1070;
pub const MIN_COLD_BOOT_RUNS: usize = 10;
pub const MAX_BOOT_JITTER_MS: u32 = 50;
pub const EXPECTED_RESET_REASON_POR: u32 = 0x0000_0001;
pub const KERNEL_IMAGE_CRC32: u32 = 0x5C01_D92E;
pub const NOMINAL_COLD_BOOT_TIME_MS: u32 = 820;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColdBootError {
    PermissionDenied,
    RunCountInsufficient,
    JitterExceeded,
    ChecksumMismatch,
    InvalidResetReason,
    ArrayCapacityOverflow,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ColdBootRecord {
    pub run_index: u32,
    pub boot_time_ms: u32,
    pub reset_reason: u32,
    pub crc32: u32,
    pub success: bool,
}

impl ColdBootRecord {
    pub const fn empty() -> Self {
        Self {
            run_index: 0,
            boot_time_ms: 0,
            reset_reason: 0,
            crc32: 0,
            success: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ColdBootSummary {
    pub total_runs: usize,
    pub successful_runs: usize,
    pub avg_time_ms: u32,
    pub max_jitter_ms: u32,
    pub all_passed: bool,
}

impl ColdBootSummary {
    pub const fn empty() -> Self {
        Self {
            total_runs: 0,
            successful_runs: 0,
            avg_time_ms: 0,
            max_jitter_ms: 0,
            all_passed: false,
        }
    }
}

pub struct ColdBootEngine {
    records: [ColdBootRecord; MIN_COLD_BOOT_RUNS],
    recorded_count: usize,
    summary: ColdBootSummary,
}

impl ColdBootEngine {
    pub const fn new() -> Self {
        Self {
            records: [ColdBootRecord::empty(); MIN_COLD_BOOT_RUNS],
            recorded_count: 0,
            summary: ColdBootSummary::empty(),
        }
    }

    pub fn inspect_reset_reason(&self, cap: u32, raw_reason: u32) -> Result<u32, ColdBootError> {
        if cap != CAP_COLD_BOOT_VERIFY {
            return Err(ColdBootError::PermissionDenied);
        }
        if raw_reason != EXPECTED_RESET_REASON_POR {
            return Err(ColdBootError::InvalidResetReason);
        }
        Ok(raw_reason)
    }

    pub fn record_cold_boot(
        &mut self,
        cap: u32,
        run_idx: u32,
        boot_time_ms: u32,
        crc32: u32,
    ) -> Result<(), ColdBootError> {
        if cap != CAP_COLD_BOOT_VERIFY {
            return Err(ColdBootError::PermissionDenied);
        }
        if crc32 != KERNEL_IMAGE_CRC32 {
            return Err(ColdBootError::ChecksumMismatch);
        }
        if self.recorded_count >= MIN_COLD_BOOT_RUNS {
            return Err(ColdBootError::ArrayCapacityOverflow);
        }

        self.records[self.recorded_count] = ColdBootRecord {
            run_index: run_idx,
            boot_time_ms,
            reset_reason: EXPECTED_RESET_REASON_POR,
            crc32,
            success: true,
        };
        self.recorded_count += 1;
        Ok(())
    }

    pub fn evaluate_stability(&mut self, cap: u32) -> Result<ColdBootSummary, ColdBootError> {
        if cap != CAP_COLD_BOOT_VERIFY {
            return Err(ColdBootError::PermissionDenied);
        }
        if self.recorded_count < MIN_COLD_BOOT_RUNS {
            return Err(ColdBootError::RunCountInsufficient);
        }

        let mut sum_ms: u64 = 0;
        let mut min_ms: u32 = u32::MAX;
        let mut max_ms: u32 = 0;
        let mut success_count = 0;

        let mut i = 0;
        while i < self.recorded_count {
            let r = &self.records[i];
            if r.success {
                success_count += 1;
            }
            sum_ms += r.boot_time_ms as u64;
            if r.boot_time_ms < min_ms {
                min_ms = r.boot_time_ms;
            }
            if r.boot_time_ms > max_ms {
                max_ms = r.boot_time_ms;
            }
            i += 1;
        }

        let avg = (sum_ms / (self.recorded_count as u64)) as u32;
        let jitter = max_ms.saturating_sub(min_ms);

        if jitter > MAX_BOOT_JITTER_MS {
            return Err(ColdBootError::JitterExceeded);
        }

        self.summary = ColdBootSummary {
            total_runs: self.recorded_count,
            successful_runs: success_count,
            avg_time_ms: avg,
            max_jitter_ms: jitter,
            all_passed: success_count == MIN_COLD_BOOT_RUNS,
        };

        Ok(self.summary)
    }

    pub fn summary(&self) -> &ColdBootSummary {
        &self.summary
    }

    pub fn is_verified(&self) -> bool {
        self.summary.all_passed && self.summary.total_runs >= MIN_COLD_BOOT_RUNS
    }
}
