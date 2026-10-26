//! Inter-Process Communication (IPC) Subsystem for IndK
//! Implements synchronous rendezvous endpoints, rights transfer, notifications,
//! and deadlines/timeouts per Bible §8.9 / §10 and INDOS-SPEC-IPC-02.

pub mod endpoint;
pub mod notification;

pub use endpoint::{
    CapFastCache, EndpointId, EndpointRegistry, EndpointState, IpcEndpoint, IpcMessage,
    IpcRecvResult, IpcSendResult, IPC_TIMEOUT_INFINITE, IPC_TIMEOUT_NONBLOCK, MAX_ENDPOINTS,
};
pub use notification::{
    NotificationId, NotificationObject, NotificationRegistry, NotificationState,
    MAX_NOTIFICATIONS,
};

use crate::sync::Spinlock;

/// Global Endpoint Registry protected by spinlock
pub static ENDPOINT_REGISTRY: Spinlock<EndpointRegistry> = Spinlock::new(EndpointRegistry::new());

/// Global Asynchronous Notification Registry protected by spinlock
pub static NOTIFICATION_REGISTRY: Spinlock<NotificationRegistry> =
    Spinlock::new(NotificationRegistry::new());
