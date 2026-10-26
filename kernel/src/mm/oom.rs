//! Deterministic Out-of-Memory (OOM) Killer for IndK
//! Implements INV-OOM-001 (strictly algorithmic, deterministic, non-model-controlled OOM policy per Bible §8.10).

use crate::log_info;
use crate::task::scheduler::Priority;

pub const MAX_OOM_CANDIDATES: usize = 32;

/// Reason why a process is strictly immune from OOM termination
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OomImmunityReason {
    /// Kernel idle task or init daemon (pid <= 1)
    SystemCritical,
    /// Hard real-time hardware priority task
    RealtimePriority,
    /// Explicit kernel resilience immunity
    ExplicitImmunity,
}

impl OomImmunityReason {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::SystemCritical => "SystemCritical (PID <= 1)",
            Self::RealtimePriority => "RealtimePriority (Hardware/Audio Spine)",
            Self::ExplicitImmunity => "ExplicitImmunity (Protected Daemon)",
        }
    }
}

/// Candidate process tracked by OOM killer
#[derive(Clone, Copy, Debug)]
pub struct OomCandidate {
    pub pid: u32,
    pub priority: Priority,
    pub allocated_frames: usize,
    pub is_immune: bool,
    pub name: &'static str,
}

impl OomCandidate {
    pub const fn new(pid: u32, priority: Priority, allocated_frames: usize, is_immune: bool, name: &'static str) -> Self {
        Self {
            pid,
            priority,
            allocated_frames,
            is_immune,
            name,
        }
    }
}

/// Deterministic OOM Killer Subsystem
pub struct OomKiller {
    candidates: [Option<OomCandidate>; MAX_OOM_CANDIDATES],
    candidate_count: usize,
    total_kills: usize,
}

impl OomKiller {
    pub const fn new() -> Self {
        Self {
            candidates: [None; MAX_OOM_CANDIDATES],
            candidate_count: 0,
            total_kills: 0,
        }
    }

    /// Registers or updates a process candidate for OOM tracking
    pub fn register_candidate(&mut self, candidate: OomCandidate) -> Result<(), &'static str> {
        // Check if PID already exists
        for i in 0..self.candidate_count {
            if let Some(ref mut existing) = self.candidates[i] {
                if existing.pid == candidate.pid {
                    *existing = candidate;
                    return Ok(());
                }
            }
        }

        if self.candidate_count >= MAX_OOM_CANDIDATES {
            return Err("OOM candidate table full");
        }

        self.candidates[self.candidate_count] = Some(candidate);
        self.candidate_count += 1;
        Ok(())
    }

    /// Removes a process from OOM tracking (e.g. upon normal exit)
    pub fn unregister_candidate(&mut self, pid: u32) {
        let mut found_idx = None;
        for i in 0..self.candidate_count {
            if let Some(c) = self.candidates[i] {
                if c.pid == pid {
                    found_idx = Some(i);
                    break;
                }
            }
        }

        if let Some(idx) = found_idx {
            for i in idx..self.candidate_count - 1 {
                self.candidates[i] = self.candidates[i + 1];
            }
            self.candidates[self.candidate_count - 1] = None;
            self.candidate_count -= 1;
        }
    }

    /// Computes deterministic badness score for a candidate process (INV-OOM-001)
    /// Returns Err(OomImmunityReason) if the candidate is immune.
    pub fn compute_badness(candidate: &OomCandidate, total_frames: usize) -> Result<u32, OomImmunityReason> {
        // 1. Core system immunity (PID <= 1)
        if candidate.pid <= 1 {
            return Err(OomImmunityReason::SystemCritical);
        }

        // 2. Real-time priority immunity
        if candidate.priority == Priority::Realtime {
            return Err(OomImmunityReason::RealtimePriority);
        }

        // 3. Explicit resilience immunity
        if candidate.is_immune {
            return Err(OomImmunityReason::ExplicitImmunity);
        }

        // Base memory usage score: (allocated_frames * 1000) / total_frames
        let base_score = if total_frames > 0 {
            ((candidate.allocated_frames * 1000) / total_frames) as u32
        } else {
            0
        };

        // Priority penalty: lower priority processes receive higher penalty
        let priority_penalty = match candidate.priority {
            Priority::Idle => 400,
            Priority::BatchIntelligence => 300,
            Priority::Normal => 100,
            Priority::Interactive => 0,
            Priority::Realtime => 0, // Handled above by immunity
        };

        Ok(base_score + priority_penalty)
    }

    /// Evaluates all candidates and deterministically selects the optimal victim (INV-OOM-001)
    /// Tie-breaking order:
    /// 1. Higher badness score
    /// 2. Higher allocated frames
    /// 3. Higher PID
    pub fn select_victim(&self, total_frames: usize) -> Option<(OomCandidate, u32)> {
        let mut best_candidate: Option<OomCandidate> = None;
        let mut best_score: u32 = 0;

        for i in 0..self.candidate_count {
            if let Some(ref candidate) = self.candidates[i] {
                match Self::compute_badness(candidate, total_frames) {
                    Ok(score) => {
                        let should_replace = match best_candidate {
                            None => true,
                            Some(ref current_best) => {
                                if score > best_score {
                                    true
                                } else if score == best_score {
                                    // Tie-break 1: larger memory footprint
                                    if candidate.allocated_frames > current_best.allocated_frames {
                                        true
                                    } else if candidate.allocated_frames == current_best.allocated_frames {
                                        // Tie-break 2: higher PID
                                        candidate.pid > current_best.pid
                                    } else {
                                        false
                                    }
                                } else {
                                    false
                                }
                            }
                        };

                        if should_replace {
                            best_candidate = Some(*candidate);
                            best_score = score;
                        }
                    }
                    Err(reason) => {
                        // Process is immune, skip
                        let _ = reason;
                    }
                }
            }
        }

        best_candidate.map(|c| (c, best_score))
    }

    /// Deterministically terminates the selected victim and returns its descriptor (INV-OOM-001)
    pub fn kill_victim(&mut self, total_frames: usize) -> Option<OomCandidate> {
        if let Some((victim, score)) = self.select_victim(total_frames) {
            log_info!(
                "oom",
                "INV-OOM-001: Deterministic OOM Killer terminating victim PID {} ('{}', priority: {:?}, frames: {}, badness: {})",
                victim.pid,
                victim.name,
                victim.priority,
                victim.allocated_frames,
                score
            );
            self.unregister_candidate(victim.pid);
            self.total_kills += 1;
            Some(victim)
        } else {
            None
        }
    }

    pub fn total_kills(&self) -> usize {
        self.total_kills
    }

    pub fn candidate_count(&self) -> usize {
        self.candidate_count
    }

    pub fn clear(&mut self) {
        self.candidates = [None; MAX_OOM_CANDIDATES];
        self.candidate_count = 0;
    }
}
