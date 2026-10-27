//! Ind OS Cold Boot Animation State Machine & Procedural Timeline Engine
//!
//! Constitutional Invariants:
//! - INV-ANIM-001: Total animation duration <= 2800ms
//! - INV-ANIM-002: Zero dynamic heap allocation in rendering loop
//! - Rule N-01: Zero external crates
//! - Rule N-04: Zero unsafe code in userspace

pub const TOTAL_ANIM_DURATION_MS: u32 = 2800;
pub const STAGE1_END_MS: u32 = 800;
pub const STAGE2_END_MS: u32 = 1800;
pub const STAGE3_END_MS: u32 = 2500;
pub const STAGE4_END_MS: u32 = 2800;

pub const COLOR_ANIM_BG: u32 = 0xFF0A0D12;
pub const COLOR_CYAN_CIRCUIT: u32 = 0xFF00F0FF;
pub const COLOR_GOLD_MANDALA: u32 = 0xFFFFB300;
pub const COLOR_WHITE_TEXT: u32 = 0xFFFFFFFF;
pub const COLOR_ORANGE_BRAND: u32 = 0xFFFF6D00;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BootAnimStage {
    SiliconIgnition,  // 0.0s -> 0.8s
    MascotAssembly,   // 0.8s -> 1.8s
    SovereigntyPulse, // 1.8s -> 2.5s
    ShellTransition,  // 2.5s -> 2.8s
    Complete,         // > 2.8s
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MascotTransform {
    pub center_x: i32,
    pub center_y: i32,
    pub scale_permille: u32, // 1000 = 1.0x, 200 = 0.2x
    pub opacity_permille: u32,
}

pub struct BootAnimationEngine {
    current_time_ms: u32,
}

impl BootAnimationEngine {
    pub const fn new() -> Self {
        Self {
            current_time_ms: 0,
        }
    }

    pub fn set_time(&mut self, ms: u32) {
        self.current_time_ms = ms;
    }

    pub fn current_stage(&self) -> BootAnimStage {
        if self.current_time_ms < STAGE1_END_MS {
            BootAnimStage::SiliconIgnition
        } else if self.current_time_ms < STAGE2_END_MS {
            BootAnimStage::MascotAssembly
        } else if self.current_time_ms < STAGE3_END_MS {
            BootAnimStage::SovereigntyPulse
        } else if self.current_time_ms <= STAGE4_END_MS {
            BootAnimStage::ShellTransition
        } else {
            BootAnimStage::Complete
        }
    }

    /// Computes the normalized stage progress [0..1000] (permille)
    pub fn stage_progress_permille(&self) -> u32 {
        match self.current_stage() {
            BootAnimStage::SiliconIgnition => {
                (self.current_time_ms * 1000) / STAGE1_END_MS
            }
            BootAnimStage::MascotAssembly => {
                let elapsed = self.current_time_ms - STAGE1_END_MS;
                let duration = STAGE2_END_MS - STAGE1_END_MS;
                (elapsed * 1000) / duration
            }
            BootAnimStage::SovereigntyPulse => {
                let elapsed = self.current_time_ms - STAGE2_END_MS;
                let duration = STAGE3_END_MS - STAGE2_END_MS;
                (elapsed * 1000) / duration
            }
            BootAnimStage::ShellTransition => {
                let elapsed = self.current_time_ms - STAGE3_END_MS;
                let duration = STAGE4_END_MS - STAGE3_END_MS;
                (elapsed * 1000) / duration
            }
            BootAnimStage::Complete => 1000,
        }
    }

    /// Radius of expanding sovereignty shockwave pulse in pixels (0..600)
    pub fn sovereignty_pulse_radius(&self) -> u32 {
        if self.current_stage() == BootAnimStage::SovereigntyPulse {
            let progress = self.stage_progress_permille();
            (progress * 600) / 1000
        } else {
            0
        }
    }

    /// Computes Gaja mascot center position and scale during Stage 4 morph
    pub fn mascot_transform(&self) -> MascotTransform {
        match self.current_stage() {
            BootAnimStage::SiliconIgnition => MascotTransform {
                center_x: 540,
                center_y: 1100,
                scale_permille: 500,
                opacity_permille: 300,
            },
            BootAnimStage::MascotAssembly | BootAnimStage::SovereigntyPulse => MascotTransform {
                center_x: 540,
                center_y: 1100,
                scale_permille: 1000,
                opacity_permille: 1000,
            },
            BootAnimStage::ShellTransition => {
                let progress = self.stage_progress_permille(); // 0..1000
                // Interpolate center_x from 540 -> 60
                let cx = 540 - ((480 * progress as i32) / 1000);
                // Interpolate center_y from 1100 -> 45
                let cy = 1100 - ((1055 * progress as i32) / 1000);
                // Interpolate scale from 1000 -> 180 (shrink into status bar)
                let scale = 1000 - ((820 * progress) / 1000);
                MascotTransform {
                    center_x: cx,
                    center_y: cy,
                    scale_permille: scale,
                    opacity_permille: 1000,
                }
            }
            BootAnimStage::Complete => MascotTransform {
                center_x: 60,
                center_y: 45,
                scale_permille: 180,
                opacity_permille: 1000,
            },
        }
    }

    pub fn is_complete(&self) -> bool {
        self.current_time_ms >= STAGE4_END_MS
    }
}
