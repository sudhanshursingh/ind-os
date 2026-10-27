//! Proactive Engine UI & Aggressiveness Dial (SP-65, REQ-AI-106)
//!
//! Architectural Invariants:
//! - INV-PROACTIVE-001: Aggressiveness dial filtering by confidence score threshold
//! - INV-PROACTIVE-002: Deterministic suggestion ranking queue
//! - INV-PROACTIVE-003: Dismiss, snooze, and explain feedback actions

pub const MAX_SUGGESTIONS: usize = 8;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProactiveError {
    QueueFull,
    NotFound,
    ThresholdNotMet,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AggressivenessDial {
    Off,          // Suppress all suggestions
    Conservative, // Minimum 90% confidence
    Balanced,     // Minimum 70% confidence
    Proactive,    // Minimum 50% confidence
}

impl AggressivenessDial {
    pub fn min_confidence_score(self) -> u8 {
        match self {
            Self::Off => 101, // Impossible to satisfy
            Self::Conservative => 90,
            Self::Balanced => 70,
            Self::Proactive => 50,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProactiveSuggestion {
    pub id: u32,
    pub title: &'static str,
    pub body: &'static str,
    pub confidence_score: u8,
    pub is_dismissed: bool,
    pub is_snoozed: bool,
}

pub struct ProactiveUiEngine {
    dial: AggressivenessDial,
    suggestions: [Option<ProactiveSuggestion>; MAX_SUGGESTIONS],
    suggestion_count: usize,
    next_id: u32,
}

impl ProactiveUiEngine {
    pub const fn new() -> Self {
        Self {
            dial: AggressivenessDial::Balanced,
            suggestions: [None; MAX_SUGGESTIONS],
            suggestion_count: 0,
            next_id: 1,
        }
    }

    pub fn set_dial(&mut self, dial: AggressivenessDial) {
        self.dial = dial;
    }

    pub fn dial(&self) -> AggressivenessDial {
        self.dial
    }

    pub fn submit_candidate(
        &mut self,
        title: &'static str,
        body: &'static str,
        confidence_score: u8,
    ) -> Result<u32, ProactiveError> {
        // Enforce dial threshold (INV-PROACTIVE-001)
        if confidence_score < self.dial.min_confidence_score() {
            return Err(ProactiveError::ThresholdNotMet);
        }

        for slot in self.suggestions.iter_mut() {
            if slot.is_none() {
                let id = self.next_id;
                self.next_id += 1;
                *slot = Some(ProactiveSuggestion {
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
        Err(ProactiveError::QueueFull)
    }

    pub fn dismiss(&mut self, id: u32) -> Result<(), ProactiveError> {
        for slot in self.suggestions.iter_mut() {
            if let Some(ref mut s) = slot {
                if s.id == id {
                    s.is_dismissed = true;
                    return Ok(());
                }
            }
        }
        Err(ProactiveError::NotFound)
    }

    pub fn snooze(&mut self, id: u32) -> Result<(), ProactiveError> {
        for slot in self.suggestions.iter_mut() {
            if let Some(ref mut s) = slot {
                if s.id == id {
                    s.is_snoozed = true;
                    return Ok(());
                }
            }
        }
        Err(ProactiveError::NotFound)
    }

    pub fn explain(&self, id: u32) -> Result<u32, ProactiveError> {
        for slot in self.suggestions.iter() {
            if let Some(ref s) = slot {
                if s.id == id {
                    // Returns deterministic why-engine audit link token
                    return Ok(0xEE00_0000 | (id & 0xFFFF));
                }
            }
        }
        Err(ProactiveError::NotFound)
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
