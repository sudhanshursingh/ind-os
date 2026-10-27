//! Ind OS Kernel-AI Feedback Loop Subsystem (`indus-ais::feedback`)
//!
//! Standards:
//! - Bible §8.12: Kernel-AI feedback loop (telemetry -> sched/mem/power hints; provable bounds)
//! - INV-FEEDBACK-001: Provably bounded AI feedback hints
//! - INV-FEEDBACK-002: Advisory degradation under contention
//! - Rule N-01: Zero external crates
//! - Rule N-04: Zero unsafe code in new modules

pub const MAX_AI_ADVISORY_PRIORITY: u32 = 60;
pub const REALTIME_BASE_PRIORITY: u32 = 90;
pub const CONSENT_UI_PRIORITY: u32 = 95;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HintCategory {
    SchedulerPriority,
    MemoryPrealloc,
    PowerTier,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TelemetryHint {
    pub hint_id: u32,
    pub category: HintCategory,
    pub requested_val: u32,
    pub approved_val: u32,
    pub clamped: bool,
}

pub struct FeedbackArbiter {
    hints_processed: usize,
    next_hint_id: u32,
}

impl FeedbackArbiter {
    pub const fn new() -> Self {
        Self {
            hints_processed: 0,
            next_hint_id: 6001,
        }
    }

    pub fn submit_hint(&mut self, category: HintCategory, requested_val: u32) -> TelemetryHint {
        let id = self.next_hint_id;
        self.next_hint_id += 1;
        self.hints_processed += 1;

        let (approved_val, clamped) = match category {
            HintCategory::SchedulerPriority => {
                // Sched hints cannot exceed MAX_AI_ADVISORY_PRIORITY (60)
                // Realtime (90+) and Consent UI (95) are immune (INV-FEEDBACK-001)
                if requested_val > MAX_AI_ADVISORY_PRIORITY {
                    (MAX_AI_ADVISORY_PRIORITY, true)
                } else {
                    (requested_val, false)
                }
            }
            HintCategory::MemoryPrealloc => {
                // Max 32MB hint limit
                if requested_val > 32 {
                    (32, true)
                } else {
                    (requested_val, false)
                }
            }
            HintCategory::PowerTier => {
                if requested_val > 3 {
                    (3, true)
                } else {
                    (requested_val, false)
                }
            }
        };

        TelemetryHint {
            hint_id: id,
            category,
            requested_val,
            approved_val,
            clamped,
        }
    }

    pub fn hints_count(&self) -> usize {
        self.hints_processed
    }
}
