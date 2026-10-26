//! Symmetric Multi-Processing (SMP) Subsystem for IndOS
//! Compliant with Bible §8.10 (SP-28), INDOS-SPEC-SMP-01, Rule N-01, Rule N-04, and REQ-KRN-109.

pub mod core;
pub mod ipi;
pub mod runqueue;

pub use core::{CpuDescriptor, CpuId, CpuManager, CpuState, MAX_CPUS};
pub use ipi::{IpiBroker, IpiKind, IpiMessage};
pub use runqueue::{AffinityMask, PerCpuRunQueue, ScheduledTask, MAX_TASKS_PER_QUEUE};

use crate::sync::Spinlock;

/// Global CPU topology manager singleton
pub static CPU_MANAGER: Spinlock<CpuManager> = Spinlock::new(CpuManager::new());

/// Global IPI Broker singleton
pub static IPI_BROKER: Spinlock<IpiBroker> = Spinlock::new(IpiBroker::new());

/// Global decentralized per-CPU run queues
pub static PER_CPU_QUEUES: Spinlock<[PerCpuRunQueue; MAX_CPUS]> = Spinlock::new([
    PerCpuRunQueue::new(0),
    PerCpuRunQueue::new(1),
    PerCpuRunQueue::new(2),
    PerCpuRunQueue::new(3),
]);
