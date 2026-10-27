//! Network Stack Core Types (SP-51, REQ-KRN-110)
//!
//! Architectural Invariants:
//! - INV-NET-001: Sockets as capability tokens (Zero ambient network authority)
//! - INV-NET-002: Bounded packet parsing and zero heap allocation in fast path
//! - INV-NET-003: Non-bypassable kernel socket table and port namespace isolation

pub const CAP_NET_SOCKET: u32  = 0x0001;
pub const CAP_NET_BIND: u32    = 0x0002;
pub const CAP_NET_LISTEN: u32  = 0x0004;
pub const CAP_NET_CONNECT: u32 = 0x0008;
pub const CAP_NET_SEND: u32    = 0x0010;
pub const CAP_NET_RECV: u32    = 0x0020;
pub const CAP_NET_RAW: u32     = 0x0040;

pub const DEFAULT_MTU: usize = 1500;
pub const MAX_SOCKETS: usize = 16;
pub const SOCKET_BUF_SIZE: usize = 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MacAddress(pub [u8; 6]);

impl MacAddress {
    pub const fn new(bytes: [u8; 6]) -> Self {
        Self(bytes)
    }

    pub const fn broadcast() -> Self {
        Self([0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF])
    }

    pub const fn zero() -> Self {
        Self([0x00, 0x00, 0x00, 0x00, 0x00, 0x00])
    }

    pub fn is_broadcast(&self) -> bool {
        self.0 == [0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ipv4Address(pub [u8; 4]);

impl Ipv4Address {
    pub const fn new(a: u8, b: u8, c: u8, d: u8) -> Self {
        Self([a, b, c, d])
    }

    pub const fn loopback() -> Self {
        Self([127, 0, 0, 1])
    }

    pub const fn any() -> Self {
        Self([0, 0, 0, 0])
    }

    pub const fn broadcast() -> Self {
        Self([255, 255, 255, 255])
    }

    pub const fn as_u32(&self) -> u32 {
        ((self.0[0] as u32) << 24)
            | ((self.0[1] as u32) << 16)
            | ((self.0[2] as u32) << 8)
            | (self.0[3] as u32)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EtherType {
    Ipv4 = 0x0800,
    Arp  = 0x0806,
    Unknown = 0xFFFF,
}

impl From<u16> for EtherType {
    fn from(val: u16) -> Self {
        match val {
            0x0800 => EtherType::Ipv4,
            0x0806 => EtherType::Arp,
            _ => EtherType::Unknown,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IpProtocol {
    Icmp = 1,
    Tcp  = 6,
    Udp  = 17,
    Unknown = 255,
}

impl From<u8> for IpProtocol {
    fn from(val: u8) -> Self {
        match val {
            1 => IpProtocol::Icmp,
            6 => IpProtocol::Tcp,
            17 => IpProtocol::Udp,
            _ => IpProtocol::Unknown,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SocketType {
    Stream,   // TCP
    Datagram, // UDP
    Raw,      // Raw Ethernet / IP
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TcpState {
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
pub enum NetError {
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
