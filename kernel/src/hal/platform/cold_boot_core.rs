//! Kernel Moto G60 Cold-Boot Stability Core (SP-107, REQ-QA-201, INV-BOOT-COLD-001..003)
//!
//! Enforces deterministic cold-boot verification across >=10 consecutive cycles.

pub const KERNEL_CAP_COLD_BOOT_VERIFY: u32 = 0x0000_1070;
pub const KERNEL_MIN_COLD_BOOT_RUNS: usize = 10;
pub const KERNEL_MAX_BOOT_JITTER_MS: u32 = 50;
pub const KERNEL_RESET_REASON_POR: u32 = 0x0000_0001;
pub const KERNEL_IMAGE_CRC32: u32 = 0x5C01_D92E;
pub const KERNEL_NOMINAL_COLD_BOOT_TIME_MS: u32 = 820;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KernelColdBootError {
    PermissionDenied,
    RunCountInsufficient,
    JitterExceeded,
    ChecksumMismatch,
    InvalidResetReason,
    ArrayCapacityOverflow,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KernelColdBootRecord {
    pub run_index: u32,
    pub boot_time_ms: u32,
    pub reset_reason: u32,
    pub crc32: u32,
    pub success: bool,
}

impl KernelColdBootRecord {
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
pub struct KernelColdBootSummary {
    pub total_runs: usize,
    pub successful_runs: usize,
    pub avg_time_ms: u32,
    pub max_jitter_ms: u32,
    pub all_passed: bool,
}

impl KernelColdBootSummary {
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

pub struct KernelColdBootHarness {
    records: [KernelColdBootRecord; KERNEL_MIN_COLD_BOOT_RUNS],
    recorded_count: usize,
    summary: KernelColdBootSummary,
}

impl KernelColdBootHarness {
    pub const fn new() -> Self {
        Self {
            records: [KernelColdBootRecord::empty(); KERNEL_MIN_COLD_BOOT_RUNS],
            recorded_count: 0,
            summary: KernelColdBootSummary::empty(),
        }
    }

    pub fn inspect_reset_reason(&self, cap: u32, raw_reason: u32) -> Result<u32, KernelColdBootError> {
        if cap != KERNEL_CAP_COLD_BOOT_VERIFY {
            return Err(KernelColdBootError::PermissionDenied);
        }
        if raw_reason != KERNEL_RESET_REASON_POR {
            return Err(KernelColdBootError::InvalidResetReason);
        }
        Ok(raw_reason)
    }

    pub fn record_cold_boot(
        &mut self,
        cap: u32,
        run_idx: u32,
        boot_time_ms: u32,
        crc32: u32,
    ) -> Result<(), KernelColdBootError> {
        if cap != KERNEL_CAP_COLD_BOOT_VERIFY {
            return Err(KernelColdBootError::PermissionDenied);
        }
        if crc32 != KERNEL_IMAGE_CRC32 {
            return Err(KernelColdBootError::ChecksumMismatch);
        }
        if self.recorded_count >= KERNEL_MIN_COLD_BOOT_RUNS {
            return Err(KernelColdBootError::ArrayCapacityOverflow);
        }

        self.records[self.recorded_count] = KernelColdBootRecord {
            run_index: run_idx,
            boot_time_ms,
            reset_reason: KERNEL_RESET_REASON_POR,
            crc32,
            success: true,
        };
        self.recorded_count += 1;
        Ok(())
    }

    pub fn evaluate_stability(&mut self, cap: u32) -> Result<KernelColdBootSummary, KernelColdBootError> {
        if cap != KERNEL_CAP_COLD_BOOT_VERIFY {
            return Err(KernelColdBootError::PermissionDenied);
        }
        if self.recorded_count < KERNEL_MIN_COLD_BOOT_RUNS {
            return Err(KernelColdBootError::RunCountInsufficient);
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

        if jitter > KERNEL_MAX_BOOT_JITTER_MS {
            return Err(KernelColdBootError::JitterExceeded);
        }

        self.summary = KernelColdBootSummary {
            total_runs: self.recorded_count,
            successful_runs: success_count,
            avg_time_ms: avg,
            max_jitter_ms: jitter,
            all_passed: success_count == KERNEL_MIN_COLD_BOOT_RUNS,
        };

        Ok(self.summary)
    }

    pub fn summary(&self) -> &KernelColdBootSummary {
        &self.summary
    }

    pub fn is_verified(&self) -> bool {
        self.summary.all_passed && self.summary.total_runs >= KERNEL_MIN_COLD_BOOT_RUNS
    }
}
