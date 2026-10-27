//! Kernel Network Stack Core Verification & Runtime Harness (SP-51, REQ-KRN-110)
//!
//! Architectural Invariants:
//! - INV-NET-001: Sockets as capability tokens (Zero ambient network authority)
//! - INV-NET-002: Bounded packet parsing and zero heap allocation in fast path
//! - INV-NET-003: Non-bypassable kernel socket table and port namespace isolation

pub const KERNEL_CAP_NET_SOCKET: u32  = 0x0001;
pub const KERNEL_CAP_NET_BIND: u32    = 0x0002;
pub const KERNEL_CAP_NET_LISTEN: u32  = 0x0004;
pub const KERNEL_CAP_NET_CONNECT: u32 = 0x0008;
pub const KERNEL_CAP_NET_SEND: u32    = 0x0010;
pub const KERNEL_CAP_NET_RECV: u32    = 0x0020;
pub const KERNEL_CAP_NET_RAW: u32     = 0x0040;

pub const KERNEL_DEFAULT_MTU: usize = 1500;
pub const KERNEL_MAX_SOCKETS: usize = 16;
pub const KERNEL_SOCKET_BUF_SIZE: usize = 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KernelMacAddress(pub [u8; 6]);

impl KernelMacAddress {
    pub const fn new(bytes: [u8; 6]) -> Self {
        Self(bytes)
    }

    pub const fn broadcast() -> Self {
        Self([0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF])
    }

    pub const fn zero() -> Self {
        Self([0x00, 0x00, 0x00, 0x00, 0x00, 0x00])
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KernelIpv4Address(pub [u8; 4]);

impl KernelIpv4Address {
    pub const fn new(a: u8, b: u8, c: u8, d: u8) -> Self {
        Self([a, b, c, d])
    }

    pub const fn loopback() -> Self {
        Self([127, 0, 0, 1])
    }

    pub const fn any() -> Self {
        Self([0, 0, 0, 0])
    }

    pub const fn as_u32(&self) -> u32 {
        ((self.0[0] as u32) << 24)
            | ((self.0[1] as u32) << 16)
            | ((self.0[2] as u32) << 8)
            | (self.0[3] as u32)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KernelEtherType {
    Ipv4 = 0x0800,
    Arp  = 0x0806,
    Unknown = 0xFFFF,
}

impl From<u16> for KernelEtherType {
    fn from(val: u16) -> Self {
        match val {
            0x0800 => KernelEtherType::Ipv4,
            0x0806 => KernelEtherType::Arp,
            _ => KernelEtherType::Unknown,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KernelIpProtocol {
    Icmp = 1,
    Tcp  = 6,
    Udp  = 17,
    Unknown = 255,
}

impl From<u8> for KernelIpProtocol {
    fn from(val: u8) -> Self {
        match val {
            1 => KernelIpProtocol::Icmp,
            6 => KernelIpProtocol::Tcp,
            17 => KernelIpProtocol::Udp,
            _ => KernelIpProtocol::Unknown,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KernelSocketType {
    Stream,   // TCP
    Datagram, // UDP
    Raw,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KernelTcpState {
    Closed,
    Listen,
    SynSent,
    SynReceived,
    Established,
    FinWait1,
    FinWait2,
    CloseWait,
    Closing,
    LastAck,
    TimeWait,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KernelNetError {
    BufferTooShort,
    InvalidChecksum,
    InvalidVersion,
    InvalidHeaderLength,
    PortInUse,
    SocketNotFound,
    PermissionDenied,
    ConnectionRefused,
    ConnectionReset,
    StateMismatch,
    QueueFull,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KernelSocketCapToken {
    pub socket_id: u32,
    pub rights: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct KernelSocket {
    pub id: u32,
    pub sock_type: KernelSocketType,
    pub state: KernelTcpState,
    pub local_ip: KernelIpv4Address,
    pub local_port: u16,
    pub remote_ip: KernelIpv4Address,
    pub remote_port: u16,
    pub rx_buf: [u8; KERNEL_SOCKET_BUF_SIZE],
    pub rx_len: usize,
    pub tx_buf: [u8; KERNEL_SOCKET_BUF_SIZE],
    pub tx_len: usize,
}

impl KernelSocket {
    pub const fn empty(id: u32) -> Self {
        Self {
            id,
            sock_type: KernelSocketType::Stream,
            state: KernelTcpState::Closed,
            local_ip: KernelIpv4Address::any(),
            local_port: 0,
            remote_ip: KernelIpv4Address::any(),
            remote_port: 0,
            rx_buf: [0u8; KERNEL_SOCKET_BUF_SIZE],
            rx_len: 0,
            tx_buf: [0u8; KERNEL_SOCKET_BUF_SIZE],
            tx_len: 0,
        }
    }
}

pub struct KernelSocketTable {
    sockets: [Option<KernelSocket>; KERNEL_MAX_SOCKETS],
    next_id: u32,
}

impl KernelSocketTable {
    pub const fn new() -> Self {
        Self {
            sockets: [None; KERNEL_MAX_SOCKETS],
            next_id: 1,
        }
    }

    pub fn create_socket(
        &mut self,
        sock_type: KernelSocketType,
        cap_rights: u32,
    ) -> Result<KernelSocketCapToken, KernelNetError> {
        if (cap_rights & KERNEL_CAP_NET_SOCKET) == 0 {
            return Err(KernelNetError::PermissionDenied);
        }

        for slot in self.sockets.iter_mut() {
            if slot.is_none() {
                let id = self.next_id;
                self.next_id += 1;

                let mut s = KernelSocket::empty(id);
                s.sock_type = sock_type;
                *slot = Some(s);

                return Ok(KernelSocketCapToken {
                    socket_id: id,
                    rights: cap_rights,
                });
            }
        }
        Err(KernelNetError::QueueFull)
    }

    pub fn bind(
        &mut self,
        token: &KernelSocketCapToken,
        ip: KernelIpv4Address,
        port: u16,
    ) -> Result<(), KernelNetError> {
        if (token.rights & KERNEL_CAP_NET_BIND) == 0 {
            return Err(KernelNetError::PermissionDenied);
        }

        for slot in self.sockets.iter() {
            if let Some(s) = slot {
                if s.local_port == port && s.id != token.socket_id {
                    return Err(KernelNetError::PortInUse);
                }
            }
        }

        let sock = self.get_mut(token.socket_id)?;
        sock.local_ip = ip;
        sock.local_port = port;
        Ok(())
    }

    pub fn listen(&mut self, token: &KernelSocketCapToken) -> Result<(), KernelNetError> {
        if (token.rights & KERNEL_CAP_NET_LISTEN) == 0 {
            return Err(KernelNetError::PermissionDenied);
        }

        let sock = self.get_mut(token.socket_id)?;
        if sock.sock_type != KernelSocketType::Stream {
            return Err(KernelNetError::StateMismatch);
        }
        sock.state = KernelTcpState::Listen;
        Ok(())
    }

    pub fn connect(
        &mut self,
        token: &KernelSocketCapToken,
        remote_ip: KernelIpv4Address,
        remote_port: u16,
    ) -> Result<(), KernelNetError> {
        if (token.rights & KERNEL_CAP_NET_CONNECT) == 0 {
            return Err(KernelNetError::PermissionDenied);
        }

        let sock = self.get_mut(token.socket_id)?;
        sock.remote_ip = remote_ip;
        sock.remote_port = remote_port;

        if sock.sock_type == KernelSocketType::Stream {
            sock.state = KernelTcpState::SynSent;
        }
        Ok(())
    }

    pub fn send(&mut self, token: &KernelSocketCapToken, data: &[u8]) -> Result<usize, KernelNetError> {
        if (token.rights & KERNEL_CAP_NET_SEND) == 0 {
            return Err(KernelNetError::PermissionDenied);
        }

        let sock = self.get_mut(token.socket_id)?;
        if sock.sock_type == KernelSocketType::Stream && sock.state != KernelTcpState::Established {
            return Err(KernelNetError::StateMismatch);
        }

        let to_copy = if data.len() > KERNEL_SOCKET_BUF_SIZE {
            KERNEL_SOCKET_BUF_SIZE
        } else {
            data.len()
        };

        sock.tx_buf[..to_copy].copy_from_slice(&data[..to_copy]);
        sock.tx_len = to_copy;
        Ok(to_copy)
    }

    pub fn recv(&mut self, token: &KernelSocketCapToken, out_buf: &mut [u8]) -> Result<usize, KernelNetError> {
        if (token.rights & KERNEL_CAP_NET_RECV) == 0 {
            return Err(KernelNetError::PermissionDenied);
        }

        let sock = self.get_mut(token.socket_id)?;
        let len = sock.rx_len;
        if len == 0 {
            return Ok(0);
        }

        let to_read = if len > out_buf.len() {
            out_buf.len()
        } else {
            len
        };

        out_buf[..to_read].copy_from_slice(&sock.rx_buf[..to_read]);
        sock.rx_len = 0;
        Ok(to_read)
    }

    pub fn get(&self, socket_id: u32) -> Result<&KernelSocket, KernelNetError> {
        for slot in self.sockets.iter() {
            if let Some(s) = slot {
                if s.id == socket_id {
                    return Ok(s);
                }
            }
        }
        Err(KernelNetError::SocketNotFound)
    }

    pub fn get_mut(&mut self, socket_id: u32) -> Result<&mut KernelSocket, KernelNetError> {
        for slot in self.sockets.iter_mut() {
            if let Some(s) = slot {
                if s.id == socket_id {
                    return Ok(s);
                }
            }
        }
        Err(KernelNetError::SocketNotFound)
    }

    pub fn count(&self) -> usize {
        let mut c = 0;
        for s in self.sockets.iter() {
            if s.is_some() {
                c += 1;
            }
        }
        c
    }
}

pub struct KernelNetStack {
    pub mac: KernelMacAddress,
    pub ip: KernelIpv4Address,
    pub socket_table: KernelSocketTable,
}

impl KernelNetStack {
    pub fn new(mac: KernelMacAddress, ip: KernelIpv4Address) -> Self {
        Self {
            mac,
            ip,
            socket_table: KernelSocketTable::new(),
        }
    }

    pub fn compute_ip_checksum(header_bytes: &[u8]) -> u16 {
        let mut sum: u32 = 0;
        let mut i = 0;
        let len = header_bytes.len();

        while i + 1 < len {
            let word = ((header_bytes[i] as u32) << 8) | (header_bytes[i + 1] as u32);
            sum = sum.wrapping_add(word);
            i += 2;
        }

        if i < len {
            sum = sum.wrapping_add((header_bytes[i] as u32) << 8);
        }

        while (sum >> 16) != 0 {
            sum = (sum & 0xFFFF) + (sum >> 16);
        }

        !(sum as u16)
    }

    /// Complete 3-way TCP handshake simulation between two endpoints
    pub fn simulate_tcp_handshake(
        &mut self,
        server_token: &KernelSocketCapToken,
        client_token: &KernelSocketCapToken,
    ) -> Result<(), KernelNetError> {
        let server = self.socket_table.get_mut(server_token.socket_id)?;
        if server.state != KernelTcpState::Listen {
            return Err(KernelNetError::StateMismatch);
        }

        let client = self.socket_table.get_mut(client_token.socket_id)?;
        if client.state != KernelTcpState::SynSent {
            return Err(KernelNetError::StateMismatch);
        }

        // Server receives SYN -> SynReceived
        let server = self.socket_table.get_mut(server_token.socket_id)?;
        server.state = KernelTcpState::SynReceived;

        // Client receives SYN-ACK -> Established
        let client = self.socket_table.get_mut(client_token.socket_id)?;
        client.state = KernelTcpState::Established;

        // Server receives ACK -> Established
        let server = self.socket_table.get_mut(server_token.socket_id)?;
        server.state = KernelTcpState::Established;

        Ok(())
    }
}
