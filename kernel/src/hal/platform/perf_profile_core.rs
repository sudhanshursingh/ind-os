//! Kernel Hardware Performance Profiling & Hotspot Analyzer (SP-110, REQ-PERF-101..107)
//!
//! Standards: Bible §8.16, §9.4, §16.1..16.4; INV-PERF-PROF-001..003
//! Rule N-01: Zero external crates

pub const KERNEL_CAP_PERF_PROFILE: u64 = 0x0000_0000_0000_0080;
pub const KERNEL_MAX_PERF_HOTSPOTS: usize = 20;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KernelPerfProfileError {
    PermissionDenied,
    HotspotCatalogFull,
    InvalidMetricId,
    SampleUnderflow,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KernelPerfMetricSample {
    pub metric_id: u32,
    pub p50_us: u64,
    pub p95_us: u64,
    pub p99_us: u64,
    pub samples: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KernelPerfHotspot {
    pub id: u8,
    pub name: [u8; 24],
    pub cpu_share_permille: u16,
    pub target_reduction_pct: u8,
}

impl KernelPerfHotspot {
    pub const fn new(id: u8, name_bytes: &[u8], cpu_share_permille: u16, target_reduction_pct: u8) -> Self {
        let mut name = [0u8; 24];
        let mut i = 0;
        while i < name_bytes.len() && i < 24 {
            name[i] = name_bytes[i];
            i += 1;
        }
        Self {
            id,
            name,
            cpu_share_permille,
            target_reduction_pct,
        }
    }
}

pub struct KernelPerfProfileHarness {
    hotspots: [Option<KernelPerfHotspot>; KERNEL_MAX_PERF_HOTSPOTS],
    hotspot_count: usize,
    samples_collected: u32,
}

impl KernelPerfProfileHarness {
    pub const fn new() -> Self {
        Self {
            hotspots: [None; KERNEL_MAX_PERF_HOTSPOTS],
            hotspot_count: 0,
            samples_collected: 0,
        }
    }

    pub fn record_sample(&mut self, sample: KernelPerfMetricSample, caps: u64) -> Result<(), KernelPerfProfileError> {
        if (caps & KERNEL_CAP_PERF_PROFILE) == 0 {
            return Err(KernelPerfProfileError::PermissionDenied);
        }
        if sample.samples < 10 {
            return Err(KernelPerfProfileError::SampleUnderflow);
        }
        if sample.metric_id == 0 || sample.metric_id > 107 {
            return Err(KernelPerfProfileError::InvalidMetricId);
        }
        self.samples_collected += sample.samples;
        Ok(())
    }

    pub fn register_hotspot(&mut self, hotspot: KernelPerfHotspot, caps: u64) -> Result<(), KernelPerfProfileError> {
        if (caps & KERNEL_CAP_PERF_PROFILE) == 0 {
            return Err(KernelPerfProfileError::PermissionDenied);
        }
        if self.hotspot_count >= KERNEL_MAX_PERF_HOTSPOTS {
            return Err(KernelPerfProfileError::HotspotCatalogFull);
        }
        self.hotspots[self.hotspot_count] = Some(hotspot);
        self.hotspot_count += 1;
        Ok(())
    }

    pub fn hotspot_count(&self) -> usize {
        self.hotspot_count
    }

    pub fn samples_collected(&self) -> u32 {
        self.samples_collected
    }
}
