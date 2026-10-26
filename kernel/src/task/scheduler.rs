//! Preemptive Multi-Level Priority Scheduler for IndK
//! Implements priority classes, run queues, preemption quantum tracking, and sched_hint per Bible §8.8 (SP-12).
//! Compliant with INDOS-SPEC-SCHED-01 and machine invariants INV-SCHED-001..004.

use alloc::vec::Vec;
use crate::log_info;
use crate::sync::Spinlock;
use crate::task::process::{Process, ProcessId, ProcessState, UserContext};

pub const DEFAULT_TIME_QUANTUM_TICKS: usize = 1; // 1 tick = 10ms at 100 Hz (INDOS-SPEC-SCHED-01 §2)
pub const MAX_PRIORITY_LEVELS: usize = 5;

/// Scheduler Priority Classes (INDOS-SPEC-SCHED-01 §2)
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    Idle = 0,
    BatchIntelligence = 1,
    Normal = 2,
    Interactive = 3,
    Realtime = 4,
}

impl Priority {
    pub const REALTIME_HARDWARE: Self = Self::Realtime;
    pub const INTERACTIVE_USER: Self = Self::Interactive;
    pub const NORMAL_SERVICES: Self = Self::Normal;
    pub const BATCH_INTELLIGENCE: Self = Self::BatchIntelligence;
    pub const IDLE: Self = Self::Idle;

    pub fn as_usize(self) -> usize {
        self as usize
    }
}

/// Advisory scheduling hints (INDOS-SPEC-SCHED-01 §3 & Bible §8.8)
#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SchedHint {
    YieldNow = 0,
    LatencySensitive = 1,
    ThroughputOriented = 2,
    BackgroundWork = 3,
    AiInferenceBoost = 4,
}

impl SchedHint {
    pub fn from_u32(val: u32) -> Option<Self> {
        match val {
            0 => Some(Self::YieldNow),
            1 => Some(Self::LatencySensitive),
            2 => Some(Self::ThroughputOriented),
            3 => Some(Self::BackgroundWork),
            4 => Some(Self::AiInferenceBoost),
            _ => None,
        }
    }
}

/// Preemptive Multi-Level Priority Scheduler
pub struct Scheduler {
    /// Separate run queues for each priority class (highest priority = Realtime = 4)
    run_queues: [Vec<Process>; MAX_PRIORITY_LEVELS],
    current: Option<Process>,
    ticks_remaining: usize,
    total_context_switches: u64,
    total_preemptions: u64,
    total_yields: u64,
    total_hints_received: u64,
}

impl Scheduler {
    pub const fn new() -> Self {
        Self {
            run_queues: [
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
            ],
            current: None,
            ticks_remaining: DEFAULT_TIME_QUANTUM_TICKS,
            total_context_switches: 0,
            total_preemptions: 0,
            total_yields: 0,
            total_hints_received: 0,
        }
    }

    /// Adds a newly created process to the appropriate priority run queue
    pub fn add_process(&mut self, mut process: Process, priority: Priority) {
        process.priority = priority;
        process.state = ProcessState::Ready;
        self.run_queues[priority.as_usize()].push(process);
    }

    /// Picks the first ready process to begin execution
    pub fn pick_first(&mut self) -> Option<Process> {
        self.pick_next_process()
    }

    /// Returns the currently active process, if any
    pub fn current_process(&self) -> Option<&Process> {
        self.current.as_ref()
    }

    /// Sets the currently active process
    pub fn set_current(&mut self, proc: Option<Process>) {
        self.current = proc;
    }

    /// Checks if any ready processes are queued
    pub fn has_ready_processes(&self) -> bool {
        self.run_queues.iter().any(|q| !q.is_empty()) || self.current.is_some()
    }

    /// Selects the next ready process from the highest populated priority queue
    fn pick_next_process(&mut self) -> Option<Process> {
        for priority_idx in (0..MAX_PRIORITY_LEVELS).rev() {
            if !self.run_queues[priority_idx].is_empty() {
                return Some(self.run_queues[priority_idx].remove(0));
            }
        }
        None
    }

    /// Timer tick handler called from 100 Hz Generic Timer interrupt
    /// Returns true if the current process's quantum expired and a reschedule is needed
    pub fn on_timer_tick(&mut self) -> bool {
        if self.current.is_some() {
            if self.ticks_remaining > 1 {
                self.ticks_remaining -= 1;
                false
            } else {
                self.ticks_remaining = DEFAULT_TIME_QUANTUM_TICKS;
                self.total_preemptions += 1;
                true
            }
        } else {
            false
        }
    }

    /// Performs preemptive context switch or voluntary yield
    /// Replaces the active TrapFrame register context with the next scheduled process
    pub fn schedule_next(&mut self, current_ctx: &mut UserContext) -> bool {
        // If there's an active process that hasn't exited, requeue it
        if let Some(mut prev) = self.current.take() {
            if !prev.is_exited() {
                prev.context = *current_ctx;
                prev.state = ProcessState::Ready;
                let prio = prev.priority.as_usize();
                self.run_queues[prio].push(prev);
            }
        }

        // Pick next ready process
        if let Some(mut next) = self.pick_next_process() {
            next.state = ProcessState::Running;
            next.address_space.activate();
            *current_ctx = next.context;
            log_info!("sched", "Context switch: activating PID {} ('{}', priority: {:?})", next.pid, next.name, next.priority);
            self.current = Some(next);
            self.ticks_remaining = DEFAULT_TIME_QUANTUM_TICKS;
            self.total_context_switches += 1;
            true
        } else {
            false
        }
    }

    /// Voluntary yield issued by SYS_YIELD
    pub fn yield_current(&mut self, current_ctx: &mut UserContext) -> bool {
        self.total_yields += 1;
        self.schedule_next(current_ctx)
    }

    /// Process exit handler issued by SYS_EXIT
    /// Returns true if another process was activated, or false if run queues are exhausted
    pub fn exit_current(&mut self, exit_code: i32, current_ctx: &mut UserContext) -> bool {
        if let Some(mut cur) = self.current.take() {
            cur.state = ProcessState::Exited(exit_code);
            cur.exit_code = Some(exit_code);
            log_info!("sched", "Process {} ('{}') exited with status {}", cur.pid, cur.name, exit_code);
        }

        if let Some(mut next) = self.pick_next_process() {
            next.state = ProcessState::Running;
            next.address_space.activate();
            *current_ctx = next.context;
            log_info!("sched", "Context switch on exit: activating PID {} ('{}')", next.pid, next.name);
            self.current = Some(next);
            self.ticks_remaining = DEFAULT_TIME_QUANTUM_TICKS;
            self.total_context_switches += 1;
            true
        } else {
            false
        }
    }

    /// Advisory scheduling hint stub (INDOS-SPEC-SCHED-01 §3 & Bible §8.8)
    pub fn sched_hint(&mut self, pid: ProcessId, hint: SchedHint) {
        self.total_hints_received += 1;
        log_info!("sched", "Advisory sched_hint {:?} received for PID {}", hint, pid);
    }

    /// Scheduler metrics for validation audit
    pub fn metrics(&self) -> (u64, u64, u64, u64) {
        (
            self.total_context_switches,
            self.total_preemptions,
            self.total_yields,
            self.total_hints_received,
        )
    }
}

pub static SCHEDULER: Spinlock<Scheduler> = Spinlock::new(Scheduler::new());
