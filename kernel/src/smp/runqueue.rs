//! IndOS Per-CPU Run Queues & Task Affinity
//! Compliant with Bible §8.10 (SP-28), INDOS-SPEC-SMP-01, Rule N-01, Rule N-04, and REQ-KRN-109.

use crate::smp::core::CpuId;
use crate::task::process::ProcessId;
use crate::task::scheduler::Priority;

pub const MAX_TASKS_PER_QUEUE: usize = 16;

/// Bitmask defining processor core affinity constraints (REQ-KRN-109)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AffinityMask(pub u8);

impl AffinityMask {
    pub const ALL: Self = Self(0x0F);
    pub const CPU0: Self = Self(1 << 0);
    pub const CPU1: Self = Self(1 << 1);
    pub const CPU2: Self = Self(1 << 2);
    pub const CPU3: Self = Self(1 << 3);

    pub const fn from_bits(bits: u8) -> Self {
        Self(bits & 0x0F)
    }

    pub const fn is_allowed_on(&self, cpu_id: CpuId) -> bool {
        (self.0 & (1 << cpu_id)) != 0
    }
}

/// Lightweight schedulable task entry within a per-CPU run queue
#[derive(Clone, Copy, Debug)]
pub struct ScheduledTask {
    pub pid: ProcessId,
    pub priority: Priority,
    pub affinity: AffinityMask,
    pub name: [u8; 16],
    pub name_len: usize,
}

impl ScheduledTask {
    pub const fn empty() -> Self {
        Self {
            pid: 0,
            priority: Priority::Idle,
            affinity: AffinityMask::ALL,
            name: [0u8; 16],
            name_len: 0,
        }
    }

    pub fn new(pid: ProcessId, priority: Priority, affinity: AffinityMask, name: &str) -> Self {
        let mut task = Self::empty();
        task.pid = pid;
        task.priority = priority;
        task.affinity = affinity;
        let bytes = name.as_bytes();
        let len = if bytes.len() > 16 { 16 } else { bytes.len() };
        task.name[..len].copy_from_slice(&bytes[..len]);
        task.name_len = len;
        task
    }

    pub fn name(&self) -> &str {
        core::str::from_utf8(&self.name[..self.name_len]).unwrap_or("")
    }
}

/// Decentralized per-CPU run queue (INV-SMP-002)
#[derive(Clone, Copy)]
pub struct PerCpuRunQueue {
    pub cpu_id: CpuId,
    pub tasks: [Option<ScheduledTask>; MAX_TASKS_PER_QUEUE],
    pub count: usize,
}

impl PerCpuRunQueue {
    pub const fn new(cpu_id: CpuId) -> Self {
        Self {
            cpu_id,
            tasks: [None; MAX_TASKS_PER_QUEUE],
            count: 0,
        }
    }

    /// Enqueues a task onto this CPU's local run queue, strictly enforcing affinity (REQ-KRN-109)
    pub fn enqueue(&mut self, task: ScheduledTask) -> Result<(), &'static str> {
        // Enforce affinity constraints
        if !task.affinity.is_allowed_on(self.cpu_id) {
            return Err("EINVAL: Task affinity forbids execution on this CPU (REQ-KRN-109)");
        }

        if self.count >= MAX_TASKS_PER_QUEUE {
            return Err("ENOMEM: Per-CPU run queue full");
        }

        for i in 0..MAX_TASKS_PER_QUEUE {
            if self.tasks[i].is_none() {
                self.tasks[i] = Some(task);
                self.count += 1;
                return Ok(());
            }
        }
        Err("ENOMEM: No free slot in run queue")
    }

    /// Dequeues the highest priority task ready to execute on this core
    pub fn dequeue_highest_priority(&mut self) -> Option<ScheduledTask> {
        if self.count == 0 {
            return None;
        }

        let mut best_idx = None;
        let mut highest_prio = Priority::Idle;

        for i in 0..MAX_TASKS_PER_QUEUE {
            if let Some(ref t) = self.tasks[i] {
                if best_idx.is_none() || t.priority > highest_prio {
                    highest_prio = t.priority;
                    best_idx = Some(i);
                }
            }
        }

        if let Some(idx) = best_idx {
            let task = self.tasks[idx].take();
            self.count -= 1;
            task
        } else {
            None
        }
    }

    /// Returns count of pending tasks in this CPU queue
    pub fn task_count(&self) -> usize {
        self.count
    }
}
