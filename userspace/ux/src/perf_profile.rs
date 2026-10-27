//! Full-System Performance Profile & Hotspot Analysis (SP-110, REQ-PERF-101..107)
//!
//! Standards: Bible §8.16, §9.4, §16.1..16.4; INV-PERF-PROF-001..003
//! Rule N-01: Zero external crates
//! Rule N-04: Zero unsafe code

pub const CAP_PERF_PROFILE: u64 = 0x0000_0000_0000_0080;
pub const MAX_PERF_HOTSPOTS: usize = 20;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PerfProfileError {
    PermissionDenied,
    HotspotCatalogFull,
    InvalidMetricId,
    SampleUnderflow,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PerfMetricSample {
    pub metric_id: u32,
    pub p50_us: u64,
    pub p95_us: u64,
    pub p99_us: u64,
    pub samples: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PerfHotspot {
    pub id: u8,
    pub name: [u8; 24],
    pub cpu_share_permille: u16, // per-mille (0.1%)
    pub target_reduction_pct: u8,
}

impl PerfHotspot {
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

pub struct PerfProfileHarness {
    hotspots: [Option<PerfHotspot>; MAX_PERF_HOTSPOTS],
    hotspot_count: usize,
    samples_collected: u32,
}

impl PerfProfileHarness {
    pub const fn new() -> Self {
        Self {
            hotspots: [None; MAX_PERF_HOTSPOTS],
            hotspot_count: 0,
            samples_collected: 0,
        }
    }

    pub fn record_sample(&mut self, sample: PerfMetricSample, caps: u64) -> Result<(), PerfProfileError> {
        if (caps & CAP_PERF_PROFILE) == 0 {
            return Err(PerfProfileError::PermissionDenied);
        }
        if sample.samples < 10 {
            return Err(PerfProfileError::SampleUnderflow);
        }
        if sample.metric_id == 0 || sample.metric_id > 107 {
            return Err(PerfProfileError::InvalidMetricId);
        }
        self.samples_collected += sample.samples;
        Ok(())
    }

    pub fn register_hotspot(&mut self, hotspot: PerfHotspot, caps: u64) -> Result<(), PerfProfileError> {
        if (caps & CAP_PERF_PROFILE) == 0 {
            return Err(PerfProfileError::PermissionDenied);
        }
        if self.hotspot_count >= MAX_PERF_HOTSPOTS {
            return Err(PerfProfileError::HotspotCatalogFull);
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
