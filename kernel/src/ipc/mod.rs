//! Inter-Process Communication (IPC) Subsystem for IndK
//! Implements synchronous rendezvous endpoints, message passing, and capability delegation per Bible §8.8 / §10.

pub mod endpoint;

pub use endpoint::{
    EndpointId, EndpointRegistry, EndpointState, IpcEndpoint, IpcMessage, IpcRecvResult,
    IpcSendResult, MAX_ENDPOINTS,
};

use crate::sync::Spinlock;

/// Global Endpoint Registry protected by spinlock
pub static ENDPOINT_REGISTRY: Spinlock<EndpointRegistry> = Spinlock::new(EndpointRegistry::new());
