//! Ind OS Core Foundation Library (libindus / indus-core)
//!
//! Constitutional References:
//! - REQ-USE-202: Core libraries: alloc, collections, IPC bindings with capability-first ergonomics (no_std-friendly)
//! - INV-LIB-001: Safe Bounded no_std Allocation & Collections
//! - INV-LIB-002: Capability-First IPC Ergonomics with Zero Ambient Authority
//! - Rule N-01: Zero external crates
//! - Rule N-04: Zero unsafe code in new modules

#![no_std]

pub mod alloc;
pub mod collections;
pub mod ipc;

pub use alloc::{AllocError, UserHeapAllocator, USER_HEAP_SIZE};
pub use collections::{CollectionError, RingBuffer, StaticMap, StaticString, StaticVec};
pub use ipc::{
    CapabilityToken, IpcChannel, IpcClient, IpcError, IpcMessage,
    CAP_RIGHTS_DELEGATE, CAP_RIGHTS_READ, CAP_RIGHTS_REVOKE, CAP_RIGHTS_WRITE,
};
