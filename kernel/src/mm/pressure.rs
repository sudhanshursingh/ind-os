//! Memory Pressure Detection & Multi-Tier Cache Reclamation for IndK
//! Implements multi-tier pressure monitoring and reclamation hooks per INDOS-SPEC-OOM-01 and INV-OOM-002.

use crate::log_info;

pub const MAX_RECLAIMERS: usize = 8;

/// Memory Pressure Tiers per INDOS-SPEC-OOM-01 §3
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum MemoryPressureLevel {
    /// >= 25% free RAM: Adequate headroom, zero reclamation
    Normal = 0,
    /// 10% .. 25% free RAM: Moderate constraint, voluntary cache dropping
    Warning = 1,
    /// 2% .. 10% free RAM: Severe constraint, involuntary aggressive pruning
    Critical = 2,
    /// < 2% free RAM: Physical exhaustion, deterministic OOM kill required
    Oom = 3,
}

impl MemoryPressureLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Normal => "Normal",
            Self::Warning => "Warning",
            Self::Critical => "Critical",
            Self::Oom => "Oom",
        }
    }
}

/// Type of reclamation action demanded by the pressure subsystem
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReclamationType {
    /// Voluntary: drop clean page caches, stale directory entries, unused vnode buffers
    Voluntary,
    /// Involuntary: aggressively shrink slab allocations, discard non-essential caches
    Involuntary,
}

/// Type alias for reclamation callback functions: fn(ReclamationType) -> reclaimed_frames
pub type ReclaimFn = fn(ReclamationType) -> usize;

/// Memory Pressure Manager
pub struct MemoryPressureManager {
    current_level: MemoryPressureLevel,
    reclaimers: [Option<ReclaimFn>; MAX_RECLAIMERS],
    reclaimer_count: usize,
    total_reclaimed_frames: usize,
}

impl MemoryPressureManager {
    pub const fn new() -> Self {
        Self {
            current_level: MemoryPressureLevel::Normal,
            reclaimers: [None; MAX_RECLAIMERS],
            reclaimer_count: 0,
            total_reclaimed_frames: 0,
        }
    }

    /// Evaluates memory pressure tier given free frames and total frames
    /// Thresholds: Normal (>= 25%), Warning (10%..25%), Critical (2%..10%), OOM (< 2%)
    pub fn compute_level(free_frames: usize, total_frames: usize) -> MemoryPressureLevel {
        if total_frames == 0 {
            return MemoryPressureLevel::Oom;
        }

        // Percentage computation scaled by 1000 for integer precision (e.g. 250 = 25.0%)
        let free_permille = (free_frames * 1000) / total_frames;

        if free_permille >= 250 {
            MemoryPressureLevel::Normal
        } else if free_permille >= 100 {
            MemoryPressureLevel::Warning
        } else if free_permille >= 20 {
            MemoryPressureLevel::Critical
        } else {
            MemoryPressureLevel::Oom
        }
    }

    /// Registers a cache reclamation callback
    pub fn register_reclaimer(&mut self, cb: ReclaimFn) -> Result<(), &'static str> {
        if self.reclaimer_count >= MAX_RECLAIMERS {
            return Err("Reclaimer table exhausted");
        }
        self.reclaimers[self.reclaimer_count] = Some(cb);
        self.reclaimer_count += 1;
        Ok(())
    }

    /// Triggers registered reclaimer callbacks based on memory pressure tier
    pub fn trigger_reclamation(&mut self, level: MemoryPressureLevel) -> usize {
        let reclaim_type = match level {
            MemoryPressureLevel::Normal => return 0,
            MemoryPressureLevel::Warning => ReclamationType::Voluntary,
            MemoryPressureLevel::Critical | MemoryPressureLevel::Oom => ReclamationType::Involuntary,
        };

        let mut frames_freed = 0;
        for i in 0..self.reclaimer_count {
            if let Some(cb) = self.reclaimers[i] {
                frames_freed += cb(reclaim_type);
            }
        }

        self.total_reclaimed_frames += frames_freed;
        frames_freed
    }

    /// Updates current pressure level and performs reclamation if necessary (INV-OOM-002)
    pub fn update(&mut self, free_frames: usize, total_frames: usize) -> (MemoryPressureLevel, usize) {
        let new_level = Self::compute_level(free_frames, total_frames);
        let mut freed = 0;

        if new_level != self.current_level {
            log_info!(
                "mm_pressure",
                "Memory pressure transition: {} -> {} (free frames: {}/{})",
                self.current_level.as_str(),
                new_level.as_str(),
                free_frames,
                total_frames
            );
            self.current_level = new_level;
        }

        if new_level >= MemoryPressureLevel::Warning {
            freed = self.trigger_reclamation(new_level);
        }

        (new_level, freed)
    }

    pub fn current_level(&self) -> MemoryPressureLevel {
        self.current_level
    }

    pub fn total_reclaimed_frames(&self) -> usize {
        self.total_reclaimed_frames
    }

    pub fn reclaimer_count(&self) -> usize {
        self.reclaimer_count
    }
}
