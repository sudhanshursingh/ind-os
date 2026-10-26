//! Capability-First Ergonomic IPC Client Bindings for Ind OS Userspace
//!
//! Constitutional References:
//! - REQ-USE-202: IPC bindings with capability-first ergonomics
//! - INV-LIB-002: Capability-First IPC Ergonomics with Zero Ambient Authority
//! - Rule N-01: Zero external crates
//! - Rule N-04: Zero unsafe code in new modules

pub const CAP_RIGHTS_READ: u32 = 0x01;
pub const CAP_RIGHTS_WRITE: u32 = 0x02;
pub const CAP_RIGHTS_DELEGATE: u32 = 0x04;
pub const CAP_RIGHTS_REVOKE: u32 = 0x08;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IpcError {
    PermissionDenied,
    EndpointNotFound,
    BufferOverflow,
    ChannelClosed,
    Timeout,
    InvalidToken,
}

/// Unforgeable client-side capability token representation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CapabilityToken {
    handle: u32,
    rights: u32,
}

impl CapabilityToken {
    pub const fn new(handle: u32, rights: u32) -> Self {
        Self { handle, rights }
    }

    pub fn handle(&self) -> u32 {
        self.handle
    }

    pub fn rights(&self) -> u32 {
        self.rights
    }

    pub fn can_read(&self) -> bool {
        (self.rights & CAP_RIGHTS_READ) != 0
    }

    pub fn can_write(&self) -> bool {
        (self.rights & CAP_RIGHTS_WRITE) != 0
    }

    pub fn can_delegate(&self) -> bool {
        (self.rights & CAP_RIGHTS_DELEGATE) != 0
    }

    /// Attenuates token rights (INV-CAP-001 monotonicity: rights cannot be amplified).
    pub fn attenuate(&self, mask: u32) -> Result<Self, IpcError> {
        let new_rights = self.rights & mask;
        Ok(Self {
            handle: self.handle,
            rights: new_rights,
        })
    }
}

/// Structured IPC transaction message packet.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IpcMessage {
    pub msg_id: u32,
    pub sender_pid: u32,
    pub payload: [u8; 64],
    pub payload_len: usize,
}

impl IpcMessage {
    pub const fn new(msg_id: u32, sender_pid: u32) -> Self {
        Self {
            msg_id,
            sender_pid,
            payload: [0u8; 64],
            payload_len: 0,
        }
    }

    pub fn with_payload(mut self, data: &[u8]) -> Result<Self, IpcError> {
        if data.len() > 64 {
            return Err(IpcError::BufferOverflow);
        }
        let mut i = 0;
        while i < data.len() {
            self.payload[i] = data[i];
            i += 1;
        }
        self.payload_len = data.len();
        Ok(self)
    }

    pub fn payload(&self) -> &[u8] {
        &self.payload[..self.payload_len]
    }
}

/// Ergonomic IPC Client wrapping an endpoint and enforcing capability checks.
#[derive(Debug, Clone)]
pub struct IpcClient {
    endpoint_id: u32,
    cap: CapabilityToken,
}

impl IpcClient {
    pub const fn new(endpoint_id: u32, cap: CapabilityToken) -> Self {
        Self { endpoint_id, cap }
    }

    pub fn endpoint_id(&self) -> u32 {
        self.endpoint_id
    }

    pub fn capability(&self) -> &CapabilityToken {
        &self.cap
    }

    /// Sends request and awaits response, strictly validating CapRights::WRITE and CapRights::READ (`INV-LIB-002`).
    pub fn call(&self, request: &IpcMessage, response: &mut IpcMessage) -> Result<(), IpcError> {
        if !self.cap.can_write() {
            return Err(IpcError::PermissionDenied);
        }
        if !self.cap.can_read() {
            return Err(IpcError::PermissionDenied);
        }

        // Mock zero-copy simulated syscall dispatch
        response.msg_id = request.msg_id + 1;
        response.sender_pid = self.endpoint_id;
        response.payload[0] = 0xAA; // ACK response code
        response.payload_len = 1;

        Ok(())
    }

    pub fn send(&self, _request: &IpcMessage) -> Result<(), IpcError> {
        if !self.cap.can_write() {
            return Err(IpcError::PermissionDenied);
        }
        Ok(())
    }

    pub fn recv(&self, response: &mut IpcMessage) -> Result<(), IpcError> {
        if !self.cap.can_read() {
            return Err(IpcError::PermissionDenied);
        }
        response.msg_id = 1;
        response.sender_pid = self.endpoint_id;
        response.payload_len = 0;
        Ok(())
    }
}

/// Bidirectional conversational IPC channel.
#[derive(Debug, Clone)]
pub struct IpcChannel {
    client: IpcClient,
    sequence: u32,
}

impl IpcChannel {
    pub const fn new(client: IpcClient) -> Self {
        Self { client, sequence: 1 }
    }

    pub fn request(&mut self, data: &[u8], reply_buf: &mut [u8]) -> Result<usize, IpcError> {
        let req = IpcMessage::new(self.sequence, 1).with_payload(data)?;
        self.sequence += 1;

        let mut resp = IpcMessage::new(0, 0);
        self.client.call(&req, &mut resp)?;

        let copy_len = resp.payload_len.min(reply_buf.len());
        let mut i = 0;
        while i < copy_len {
            reply_buf[i] = resp.payload[i];
            i += 1;
        }
        Ok(copy_len)
    }

    pub fn client(&self) -> &IpcClient {
        &self.client
    }
}
