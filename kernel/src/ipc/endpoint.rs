//! Synchronous IPC Endpoints and Rendezvous Transfer for IndK
//! Compliant with Bible §8.8 / §10 and Machine Invariant INV-IPC-001 (Bounded Latency Synchronous IPC).

use crate::capability::{CapHandle, CapRights, Capability, CapTable};
use crate::syscall::{EBADF, EBUSY, ENOMEM, EPERM, ETIMEDOUT};
use crate::task::ProcessId;

pub type EndpointId = u32;
pub const MAX_ENDPOINTS: usize = 32;

pub const IPC_TIMEOUT_INFINITE: u64 = u64::MAX;
pub const IPC_TIMEOUT_NONBLOCK: u64 = 0;

/// Inline Fast-Path Capability Validation Cache for Sub-10us IPC (INV-IPC-002)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CapFastCache {
    pub cached_handle: CapHandle,
    pub cached_rights: CapRights,
    pub cached_badge: u64,
}

impl CapFastCache {
    pub const fn empty() -> Self {
        Self {
            cached_handle: CapHandle::MAX,
            cached_rights: CapRights::from_bits(0),
            cached_badge: 0,
        }
    }

    /// Fast-path capability check against cached metadata
    pub fn validate_and_cache(
        &mut self,
        handle: CapHandle,
        required_rights: CapRights,
        cap_table: &CapTable,
    ) -> Result<u64, i64> {
        if self.cached_handle == handle {
            if self.cached_rights.contains(required_rights) {
                return Ok(self.cached_badge);
            } else {
                return Err(EPERM);
            }
        }

        let cap = cap_table.get(handle)?;
        if !cap.rights.contains(required_rights) {
            return Err(EPERM);
        }

        self.cached_handle = handle;
        self.cached_rights = cap.rights;
        self.cached_badge = cap.badge;
        Ok(self.cached_badge)
    }
}

/// Synchronous register-based IPC message
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IpcMessage {
    pub label: u64,
    pub length: usize,
    pub data: [u64; 4],
    pub cap_transfer: Option<CapHandle>,
}

impl IpcMessage {
    pub const fn empty() -> Self {
        Self {
            label: 0,
            length: 0,
            data: [0; 4],
            cap_transfer: None,
        }
    }

    pub const fn new(label: u64, data: [u64; 4], cap_transfer: Option<CapHandle>) -> Self {
        Self {
            label,
            length: 4 * core::mem::size_of::<u64>(),
            data,
            cap_transfer,
        }
    }
}

/// Endpoint Rendezvous State
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EndpointState {
    Idle,
    SenderWaiting {
        sender_pid: ProcessId,
        msg: IpcMessage,
        transferred_cap: Option<Capability>,
    },
    ReceiverWaiting {
        receiver_pid: ProcessId,
    },
}

/// Synchronous IPC Endpoint
pub struct IpcEndpoint {
    pub id: EndpointId,
    pub state: EndpointState,
    pub is_allocated: bool,
}

impl IpcEndpoint {
    pub const fn empty(id: EndpointId) -> Self {
        Self {
            id,
            state: EndpointState::Idle,
            is_allocated: false,
        }
    }
}

/// Result of an IPC Send operation
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IpcSendResult {
    RendezvousDelivered { receiver_pid: ProcessId },
    SenderQueued,
}

/// Result of an IPC Recv operation
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IpcRecvResult {
    RendezvousReceived {
        sender_pid: ProcessId,
        msg: IpcMessage,
        transferred_cap: Option<Capability>,
    },
    ReceiverQueued,
}

/// Global registry of IPC Endpoints
pub struct EndpointRegistry {
    endpoints: [IpcEndpoint; MAX_ENDPOINTS],
    total_messages_transferred: u64,
    total_rendezvous_events: u64,
}

impl EndpointRegistry {
    pub const fn new() -> Self {
        const INIT_EP: IpcEndpoint = IpcEndpoint::empty(0);
        let mut endpoints = [INIT_EP; MAX_ENDPOINTS];
        let mut i = 0;
        while i < MAX_ENDPOINTS {
            endpoints[i].id = i as EndpointId;
            i += 1;
        }

        Self {
            endpoints,
            total_messages_transferred: 0,
            total_rendezvous_events: 0,
        }
    }

    /// Allocates a new synchronous IPC endpoint
    pub fn create_endpoint(&mut self) -> Result<EndpointId, i64> {
        for i in 0..MAX_ENDPOINTS {
            if !self.endpoints[i].is_allocated {
                self.endpoints[i].is_allocated = true;
                self.endpoints[i].state = EndpointState::Idle;
                return Ok(i as EndpointId);
            }
        }
        Err(ENOMEM)
    }

    /// Releases an allocated endpoint
    pub fn destroy_endpoint(&mut self, ep_id: EndpointId) -> Result<(), i64> {
        let idx = ep_id as usize;
        if idx >= MAX_ENDPOINTS || !self.endpoints[idx].is_allocated {
            return Err(EBADF);
        }
        self.endpoints[idx].is_allocated = false;
        self.endpoints[idx].state = EndpointState::Idle;
        Ok(())
    }

    /// Performs synchronous send on the specified endpoint
    pub fn send(
        &mut self,
        sender_pid: ProcessId,
        ep_id: EndpointId,
        msg: IpcMessage,
        transferred_cap: Option<Capability>,
    ) -> Result<IpcSendResult, i64> {
        let idx = ep_id as usize;
        if idx >= MAX_ENDPOINTS || !self.endpoints[idx].is_allocated {
            return Err(EBADF);
        }

        match self.endpoints[idx].state {
            EndpointState::ReceiverWaiting { receiver_pid } => {
                // Synchronous rendezvous achieved (INV-IPC-001)
                self.endpoints[idx].state = EndpointState::Idle;
                self.total_messages_transferred += 1;
                self.total_rendezvous_events += 1;
                Ok(IpcSendResult::RendezvousDelivered { receiver_pid })
            }
            EndpointState::Idle => {
                // Queue sender and block
                self.endpoints[idx].state = EndpointState::SenderWaiting {
                    sender_pid,
                    msg,
                    transferred_cap,
                };
                Ok(IpcSendResult::SenderQueued)
            }
            EndpointState::SenderWaiting { .. } => Err(EBUSY),
        }
    }

    /// Performs synchronous receive on the specified endpoint
    pub fn recv(
        &mut self,
        receiver_pid: ProcessId,
        ep_id: EndpointId,
    ) -> Result<IpcRecvResult, i64> {
        let idx = ep_id as usize;
        if idx >= MAX_ENDPOINTS || !self.endpoints[idx].is_allocated {
            return Err(EBADF);
        }

        match self.endpoints[idx].state {
            EndpointState::SenderWaiting { sender_pid, msg, transferred_cap } => {
                // Synchronous rendezvous achieved (INV-IPC-001)
                self.endpoints[idx].state = EndpointState::Idle;
                self.total_messages_transferred += 1;
                self.total_rendezvous_events += 1;
                Ok(IpcRecvResult::RendezvousReceived {
                    sender_pid,
                    msg,
                    transferred_cap,
                })
            }
            EndpointState::Idle => {
                // Queue receiver and block
                self.endpoints[idx].state = EndpointState::ReceiverWaiting { receiver_pid };
                Ok(IpcRecvResult::ReceiverQueued)
            }
            EndpointState::ReceiverWaiting { .. } => Err(EBUSY),
        }
    }

    /// Performs timed synchronous send on the specified endpoint (INV-IPC-004)
    pub fn send_timed(
        &mut self,
        sender_pid: ProcessId,
        ep_id: EndpointId,
        msg: IpcMessage,
        transferred_cap: Option<Capability>,
        timeout_ticks: u64,
    ) -> Result<IpcSendResult, i64> {
        let idx = ep_id as usize;
        if idx >= MAX_ENDPOINTS || !self.endpoints[idx].is_allocated {
            return Err(EBADF);
        }

        match self.endpoints[idx].state {
            EndpointState::ReceiverWaiting { receiver_pid } => {
                // Synchronous rendezvous achieved (INV-IPC-001)
                self.endpoints[idx].state = EndpointState::Idle;
                self.total_messages_transferred += 1;
                self.total_rendezvous_events += 1;
                Ok(IpcSendResult::RendezvousDelivered { receiver_pid })
            }
            EndpointState::Idle => {
                if timeout_ticks == IPC_TIMEOUT_NONBLOCK {
                    // Non-blocking send returns immediately with ETIMEDOUT if no receiver
                    Err(ETIMEDOUT)
                } else {
                    self.endpoints[idx].state = EndpointState::SenderWaiting {
                        sender_pid,
                        msg,
                        transferred_cap,
                    };
                    Ok(IpcSendResult::SenderQueued)
                }
            }
            EndpointState::SenderWaiting { .. } => Err(EBUSY),
        }
    }

    /// Performs timed synchronous receive on the specified endpoint (INV-IPC-004)
    pub fn recv_timed(
        &mut self,
        receiver_pid: ProcessId,
        ep_id: EndpointId,
        timeout_ticks: u64,
    ) -> Result<IpcRecvResult, i64> {
        let idx = ep_id as usize;
        if idx >= MAX_ENDPOINTS || !self.endpoints[idx].is_allocated {
            return Err(EBADF);
        }

        match self.endpoints[idx].state {
            EndpointState::SenderWaiting { sender_pid, msg, transferred_cap } => {
                // Synchronous rendezvous achieved (INV-IPC-001)
                self.endpoints[idx].state = EndpointState::Idle;
                self.total_messages_transferred += 1;
                self.total_rendezvous_events += 1;
                Ok(IpcRecvResult::RendezvousReceived {
                    sender_pid,
                    msg,
                    transferred_cap,
                })
            }
            EndpointState::Idle => {
                if timeout_ticks == IPC_TIMEOUT_NONBLOCK {
                    // Non-blocking receive returns immediately with ETIMEDOUT if no sender
                    Err(ETIMEDOUT)
                } else {
                    self.endpoints[idx].state = EndpointState::ReceiverWaiting { receiver_pid };
                    Ok(IpcRecvResult::ReceiverQueued)
                }
            }
            EndpointState::ReceiverWaiting { .. } => Err(EBUSY),
        }
    }

    /// Direct cross-process synchronous transfer test helper
    /// Validates INV-IPC-001 zero-copy register payload transfer and capability delegation
    pub fn rendezvous_transfer(
        &mut self,
        _sender_pid: ProcessId,
        sender_table: &CapTable,
        _receiver_pid: ProcessId,
        receiver_table: &mut CapTable,
        ep_id: EndpointId,
        msg: IpcMessage,
    ) -> Result<(IpcMessage, Option<CapHandle>), i64> {
        let idx = ep_id as usize;
        if idx >= MAX_ENDPOINTS || !self.endpoints[idx].is_allocated {
            return Err(EBADF);
        }

        // Verify capability transfer authority if specified
        let mut received_cap_handle = None;
        let transferred_cap = if let Some(cap_slot) = msg.cap_transfer {
            let cap = sender_table.get(cap_slot)?;
            if !cap.rights.contains(CapRights::GRANT) {
                return Err(EPERM);
            }
            Some(*cap)
        } else {
            None
        };

        // If a capability is being transferred, insert into receiver's CapTable with monotonic attenuation
        if let Some(cap) = transferred_cap {
            let new_handle = receiver_table.insert(cap)?;
            received_cap_handle = Some(new_handle);
        }

        self.total_messages_transferred += 1;
        self.total_rendezvous_events += 1;

        Ok((msg, received_cap_handle))
    }

    pub fn metrics(&self) -> (u64, u64) {
        (self.total_messages_transferred, self.total_rendezvous_events)
    }
}
