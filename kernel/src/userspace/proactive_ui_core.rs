//! Kernel Proactive Engine UI Verification Harness (SP-65, REQ-AI-106)
//!
//! Architectural Invariants:
//! - INV-PROACTIVE-001: Aggressiveness dial filtering by confidence score threshold
//! - INV-PROACTIVE-002: Deterministic suggestion ranking queue
//! - INV-PROACTIVE-003: Dismiss, snooze, and explain feedback actions

pub const KERNEL_MAX_SUGGESTIONS: usize = 8;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KernelProactiveError {
    QueueFull,
    NotFound,
    ThresholdNotMet,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KernelAggressivenessDial {
    Off,
    Conservative,
    Balanced,
    Proactive,
}

impl KernelAggressivenessDial {
    pub fn min_confidence_score(self) -> u8 {
        match self {
            Self::Off => 101,
            Self::Conservative => 90,
            Self::Balanced => 70,
            Self::Proactive => 50,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KernelProactiveSuggestion {
    pub id: u32,
    pub title: &'static str,
    pub body: &'static str,
    pub confidence_score: u8,
    pub is_dismissed: bool,
    pub is_snoozed: bool,
}

pub struct KernelProactiveUiEngine {
    dial: KernelAggressivenessDial,
    suggestions: [Option<KernelProactiveSuggestion>; KERNEL_MAX_SUGGESTIONS],
    suggestion_count: usize,
    next_id: u32,
}

impl KernelProactiveUiEngine {
    pub const fn new() -> Self {
        Self {
            dial: KernelAggressivenessDial::Balanced,
            suggestions: [None; KERNEL_MAX_SUGGESTIONS],
            suggestion_count: 0,
            next_id: 1,
        }
    }

    pub fn set_dial(&mut self, dial: KernelAggressivenessDial) {
        self.dial = dial;
    }

    pub fn dial(&self) -> KernelAggressivenessDial {
        self.dial
    }

    pub fn submit_candidate(
        &mut self,
        title: &'static str,
        body: &'static str,
        confidence_score: u8,
    ) -> Result<u32, KernelProactiveError> {
        if confidence_score < self.dial.min_confidence_score() {
            return Err(KernelProactiveError::ThresholdNotMet);
        }

        for slot in self.suggestions.iter_mut() {
            if slot.is_none() {
                let id = self.next_id;
                self.next_id += 1;
                *slot = Some(KernelProactiveSuggestion {
                    id,
                    title,
                    body,
                    confidence_score,
                    is_dismissed: false,
                    is_snoozed: false,
                });
                self.suggestion_count += 1;
                return Ok(id);
            }
        }
        Err(KernelProactiveError::QueueFull)
    }

    pub fn dismiss(&mut self, id: u32) -> Result<(), KernelProactiveError> {
        for slot in self.suggestions.iter_mut() {
            if let Some(ref mut s) = slot {
                if s.id == id {
                    s.is_dismissed = true;
                    return Ok(());
                }
            }
        }
        Err(KernelProactiveError::NotFound)
    }

    pub fn snooze(&mut self, id: u32) -> Result<(), KernelProactiveError> {
        for slot in self.suggestions.iter_mut() {
            if let Some(ref mut s) = slot {
                if s.id == id {
                    s.is_snoozed = true;
                    return Ok(());
                }
            }
        }
        Err(KernelProactiveError::NotFound)
    }

    pub fn explain(&self, id: u32) -> Result<u32, KernelProactiveError> {
        for slot in self.suggestions.iter() {
            if let Some(ref s) = slot {
                if s.id == id {
                    return Ok(0xEE00_0000 | (id & 0xFFFF));
                }
            }
        }
        Err(KernelProactiveError::NotFound)
    }

    pub fn visible_count(&self) -> usize {
        let mut count = 0;
        for slot in self.suggestions.iter() {
            if let Some(ref s) = slot {
                if !s.is_dismissed && !s.is_snoozed {
                    count += 1;
                }
            }
        }
        count
    }

    pub fn suggestion_count(&self) -> usize {
        self.suggestion_count
    }
}
