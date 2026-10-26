//! Typed Kernel Object Definitions for IndK
//! Implements Process, Endpoint, Memory Region, and IRQ objects per Bible §8.8 / §14.

use crate::ipc::EndpointState;
use crate::task::scheduler::Priority;
use crate::task::{ProcessId, ProcessState};

/// First-class kernel object types
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum KObjectType {
    Process = 1,
    Endpoint = 2,
    MemoryRegion = 3,
    Irq = 4,
}

/// Process Kernel Object
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ProcessObject {
    pub pid: ProcessId,
    pub name: &'static str,
    pub priority: Priority,
    pub state: ProcessState,
    pub ttbr0: usize,
}

impl ProcessObject {
    pub const fn new(pid: ProcessId, name: &'static str, priority: Priority, ttbr0: usize) -> Self {
        Self {
            pid,
            name,
            priority,
            state: ProcessState::Ready,
            ttbr0,
        }
    }
}

/// Synchronous IPC Endpoint Kernel Object
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EndpointObject {
    pub ep_id: u32,
    pub state: EndpointState,
    pub messages_transferred: u64,
}

impl EndpointObject {
    pub const fn new(ep_id: u32) -> Self {
        Self {
            ep_id,
            state: EndpointState::Idle,
            messages_transferred: 0,
        }
    }
}

/// Memory Region Kernel Object
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MemoryRegionObject {
    pub base: usize,
    pub size: usize,
    pub is_device: bool,
    pub is_writable: bool,
    pub is_executable: bool,
}

impl MemoryRegionObject {
    pub const fn new(
        base: usize,
        size: usize,
        is_device: bool,
        is_writable: bool,
        is_executable: bool,
    ) -> Self {
        Self {
            base,
            size,
            is_device,
            is_writable,
            is_executable,
        }
    }
}

/// Hardware Interrupt (IRQ) Kernel Object
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IrqObject {
    pub irq_num: u32,
    pub is_edge: bool,
    pub is_masked: bool,
    pub target_pid: Option<ProcessId>,
}

impl IrqObject {
    pub const fn new(irq_num: u32, is_edge: bool) -> Self {
        Self {
            irq_num,
            is_edge,
            is_masked: true,
            target_pid: None,
        }
    }
}

/// Polymorphic container for all kernel object variants
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KObject {
    Process(ProcessObject),
    Endpoint(EndpointObject),
    MemoryRegion(MemoryRegionObject),
    Irq(IrqObject),
}

impl KObject {
    pub fn obj_type(&self) -> KObjectType {
        match self {
            Self::Process(_) => KObjectType::Process,
            Self::Endpoint(_) => KObjectType::Endpoint,
            Self::MemoryRegion(_) => KObjectType::MemoryRegion,
            Self::Irq(_) => KObjectType::Irq,
        }
    }
}
