#![no_std]

//! Ind OS Network Stack Core Library (SP-51, REQ-KRN-110)
//!
//! Architectural Invariants:
//! - INV-NET-001: Sockets as capability tokens (Zero ambient network authority)
//! - INV-NET-002: Bounded packet parsing and zero heap allocation in fast path
//! - INV-NET-003: Non-bypassable kernel socket table and port namespace isolation

pub mod types;
pub mod ethernet;
pub mod ipv4;
pub mod udp;
pub mod tcp;
pub mod socket;

pub use types::{
    EtherType, Ipv4Address, IpProtocol, MacAddress, NetError, SocketType, TcpState,
    CAP_NET_BIND, CAP_NET_CONNECT, CAP_NET_LISTEN, CAP_NET_RAW, CAP_NET_RECV, CAP_NET_SEND,
    CAP_NET_SOCKET, DEFAULT_MTU, MAX_SOCKETS, SOCKET_BUF_SIZE,
};

pub use ethernet::{EthernetHeader, ETHERNET_HEADER_LEN};
pub use ipv4::{Ipv4Header, IPV4_MIN_HEADER_LEN};
pub use udp::{UdpHeader, UDP_HEADER_LEN};
pub use tcp::{
    TcpHeader, TcpStateMachine, TCP_FLAG_ACK, TCP_FLAG_FIN, TCP_FLAG_PSH, TCP_FLAG_RST,
    TCP_FLAG_SYN, TCP_FLAG_URG, TCP_MIN_HEADER_LEN,
};
pub use socket::{Socket, SocketCapToken, SocketTable};
