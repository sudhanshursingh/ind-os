//! IndOS Inter-Processor Interrupts (IPI) & SGI Dispatcher
//! Compliant with Bible §8.10 (SP-28), INDOS-SPEC-SMP-01, Rule N-01, Rule N-04, and REQ-KRN-109.

use crate::smp::core::{CpuId, MAX_CPUS};

/// Inter-Processor Interrupt kinds mapped to ARM GIC SGIs
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IpiKind {
    Reschedule = 0,   // SGI 0: Trigger remote scheduler preemption
    TlbFlush = 1,     // SGI 1: Coordinate cross-core TLB invalidation
    FunctionCall = 2, // SGI 2: Remote kernel callback execution
    Halt = 3,         // SGI 3: Emergency CPU halt on panic
}

/// Message payload passed across core boundaries via SGI
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IpiMessage {
    pub source_cpu: CpuId,
    pub target_cpu: CpuId,
    pub kind: IpiKind,
    pub arg: u64,
}

/// Central IPI routing engine coordinating cross-core messaging (INV-SMP-003)
pub struct IpiBroker {
    pub mailboxes: [[Option<IpiMessage>; 4]; MAX_CPUS],
    pub pending_counts: [usize; MAX_CPUS],
    pub sent_counts: [u64; MAX_CPUS],
    pub handled_counts: [u64; MAX_CPUS],
}

impl IpiBroker {
    pub const fn new() -> Self {
        Self {
            mailboxes: [[None; 4]; MAX_CPUS],
            pending_counts: [0; MAX_CPUS],
            sent_counts: [0; MAX_CPUS],
            handled_counts: [0; MAX_CPUS],
        }
    }

    /// Dispatches an IPI to a designated target core (INV-SMP-003)
    pub fn send_ipi(
        &mut self,
        source_cpu: CpuId,
        target_cpu: CpuId,
        kind: IpiKind,
        arg: u64,
    ) -> Result<(), &'static str> {
        let target = target_cpu as usize;
        let source = source_cpu as usize;

        if target >= MAX_CPUS || source >= MAX_CPUS {
            return Err("EINVAL: Invalid CPU ID");
        }
        if self.pending_counts[target] >= 4 {
            return Err("EBUSY: Target CPU mailbox full");
        }

        let msg = IpiMessage {
            source_cpu,
            target_cpu,
            kind,
            arg,
        };

        for i in 0..4 {
            if self.mailboxes[target][i].is_none() {
                self.mailboxes[target][i] = Some(msg);
                self.pending_counts[target] += 1;
                self.sent_counts[source] += 1;
                return Ok(());
            }
        }
        Err("EBUSY: No free slot in target mailbox")
    }

    /// Receives and consumes the next pending IPI on the caller core
    pub fn receive_ipi(&mut self, target_cpu: CpuId) -> Option<IpiMessage> {
        let target = target_cpu as usize;
        if target >= MAX_CPUS || self.pending_counts[target] == 0 {
            return None;
        }

        for i in 0..4 {
            if let Some(msg) = self.mailboxes[target][i].take() {
                self.pending_counts[target] -= 1;
                self.handled_counts[target] += 1;
                return Some(msg);
            }
        }
        None
    }

    /// Returns count of pending IPIs on target core
    pub fn pending_count(&self, cpu_id: CpuId) -> usize {
        let id = cpu_id as usize;
        if id < MAX_CPUS {
            self.pending_counts[id]
        } else {
            0
        }
    }
}
