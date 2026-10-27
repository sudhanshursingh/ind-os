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
pub mod tls;
pub mod dns;
pub mod dhcp;
pub mod firewall;

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
pub use tls::{
    CipherSuite, TlsCertificateValidator, TlsHandshakeEngine, TlsRecordHeader, TlsRecordType,
    TlsSessionState, TlsStream, TLS_RECORD_HEADER_LEN, TLS_VERSION_1_3,
};
pub use dns::{DnsHeader, DnsResolver, DNS_HEADER_LEN, DNS_PORT};
pub use dhcp::{
    DhcpClient, DhcpLease, DhcpMessageType, DhcpState, DHCP_CLIENT_PORT, DHCP_MAGIC_COOKIE,
    DHCP_SERVER_PORT,
};
pub use firewall::{
    FilterAction, FirewallEngine, FirewallRule, PacketDirection, CAP_NET_FIREWALL_ADMIN,
    MAX_FIREWALL_RULES,
};
pub mod update;
pub use update::{
    SlotId, SlotState, UpdateError, UpdateManager, UpdateManifest, UpdatePolicy,
    AUTHORIZED_UPDATE_SIGNATURE,
};
pub mod ceremony;
pub use ceremony::{
    CeremonyError, CustodianKey, CustodianStatus, ThresholdSigningEngine,
    MAX_CUSTODIANS, REQUIRED_THRESHOLD,
};
pub mod sentinel;
pub use sentinel::{
    FlowClass, LeakAlert, LeakAnomalyType, NetworkFlowDescriptor, NetworkLeakSentinel,
    SentinelAction, LEAK_BURST_THRESHOLD_FACTOR, MAX_LEAK_ALERTS, MAX_QUARANTINED_IPS,
};





