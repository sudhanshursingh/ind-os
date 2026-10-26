//! Kernel Object Model Subsystem for IndK
//!
//! Implements Process, Endpoint, Memory Region, and IRQ kernel object tables
//! per Bible §8.8 / §14, enforcing:
//! - INV-OBJ-001: Generation Soundness & Anti-ABA protection
//! - INV-OBJ-002: Reference Count Soundness
//! - INV-OBJ-003: Strong Type Isolation
//! - INV-OBJ-004: Deterministic Bounded Allocation (MAX_KOBJECTS = 128)

pub mod id;
pub mod table;
pub mod types;

pub use id::KObjectId;
pub use table::{KObjectSlot, KObjectTable, MAX_KOBJECTS};
pub use types::{
    EndpointObject, IrqObject, KObject, KObjectType, MemoryRegionObject, ProcessObject,
};

use crate::sync::Spinlock;

/// Global Kernel Object Table
pub static OBJECT_TABLE: Spinlock<KObjectTable> = Spinlock::new(KObjectTable::new());
