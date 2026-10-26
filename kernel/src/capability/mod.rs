//! Capability-Based Security and Access Control System for IndK
//! Implements unforgeable capability tokens, monotonic attenuation (INV-CAP-001),
//! and revocation soundness (INV-CAP-002) per Bible §8.8 / §10.

pub mod rights;
pub mod table;

pub use rights::CapRights;
pub use table::{CapHandle, CapType, Capability, CapTable, MAX_CAPABILITIES};
