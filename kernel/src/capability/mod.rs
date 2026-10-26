//! Capability-Based Security and Access Control System for IndK
//! Implements unforgeable capability tokens, monotonic attenuation (INV-CAP-001),
//! revocation soundness (INV-CAP-002), object-table reference coherence (INV-CAP-003),
//! and per-process namespace isolation (INV-CAP-004) per Bible §8.9 / §14.

pub mod namespace;
pub mod rights;
pub mod table;

pub use namespace::{CapNamespace, NamespaceRegistry, MAX_NAMESPACES};
pub use rights::CapRights;
pub use table::{CapHandle, CapTable, CapType, Capability, MAX_CAPABILITIES};

use crate::sync::Spinlock;

/// Global registry of per-process Capability Namespaces
pub static NAMESPACE_REGISTRY: Spinlock<NamespaceRegistry> =
    Spinlock::new(NamespaceRegistry::new());
