//! DHCP Protocol Client & Lease State Machine (SP-53, INV-DHCP-001)
//!
//! Architectural Invariants:
//! - INV-DHCP-001: Discover/Offer/Request/Ack transaction validation without hardcoded ambient leases

use crate::types::{Ipv4Address, MacAddress, NetError};

pub const DHCP_CLIENT_PORT: u16 = 68;
pub const DHCP_SERVER_PORT: u16 = 67;
pub const DHCP_MAGIC_COOKIE: u32 = 0x63825363;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DhcpMessageType {
    Discover = 1,
    Offer = 2,
    Request = 3,
    Ack = 5,
    Nak = 6,
    Unknown = 255,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DhcpState {
    Init,
    Selecting,
    Requesting,
    Bound,
}

#[derive(Debug, Clone, Copy)]
pub struct DhcpLease {
    pub assigned_ip: Ipv4Address,
    pub subnet_mask: Ipv4Address,
    pub gateway: Ipv4Address,
    pub dns_server: Ipv4Address,
    pub lease_time_secs: u32,
    pub xid: u32,
}

impl DhcpLease {
    pub const fn empty() -> Self {
        Self {
            assigned_ip: Ipv4Address::any(),
            subnet_mask: Ipv4Address::any(),
            gateway: Ipv4Address::any(),
            dns_server: Ipv4Address::any(),
            lease_time_secs: 0,
            xid: 0,
        }
    }
}

pub struct DhcpClient {
    pub state: DhcpState,
    pub mac: MacAddress,
    pub xid: u32,
    pub lease: DhcpLease,
}

impl DhcpClient {
    pub fn new(mac: MacAddress, xid: u32) -> Self {
        Self {
            state: DhcpState::Init,
            mac,
            xid,
            lease: DhcpLease::empty(),
        }
    }

    /// Build DHCP Discover packet
    pub fn build_discover(&mut self, buf: &mut [u8]) -> Result<usize, NetError> {
        if buf.len() < 240 {
            return Err(NetError::BufferTooShort);
        }

        buf[0] = 1; // BootRequest
        buf[1] = 1; // 10Mb Ethernet
        buf[2] = 6; // HLEN 6 bytes MAC
        buf[3] = 0; // HOPS

        buf[4] = (self.xid >> 24) as u8;
        buf[5] = (self.xid >> 16) as u8;
        buf[6] = (self.xid >> 8) as u8;
        buf[7] = (self.xid & 0xFF) as u8;

        // Client MAC at offset 28
        buf[28..34].copy_from_slice(&self.mac.0);

        // Magic Cookie at offset 236
        buf[236] = 0x63;
        buf[237] = 0x82;
        buf[238] = 0x53;
        buf[239] = 0x63;

        self.state = DhcpState::Selecting;
        Ok(240)
    }

    /// Process DHCP Offer from server
    pub fn handle_offer(
        &mut self,
        offer_ip: Ipv4Address,
        offer_xid: u32,
    ) -> Result<(), NetError> {
        if self.state != DhcpState::Selecting || offer_xid != self.xid {
            return Err(NetError::StateMismatch);
        }
        self.lease.assigned_ip = offer_ip;
        self.state = DhcpState::Requesting;
        Ok(())
    }

    /// Process DHCP Ack from server and finalize lease
    pub fn handle_ack(
        &mut self,
        ack_xid: u32,
        gateway: Ipv4Address,
        dns: Ipv4Address,
        lease_time: u32,
    ) -> Result<DhcpLease, NetError> {
        if self.state != DhcpState::Requesting || ack_xid != self.xid {
            return Err(NetError::StateMismatch);
        }

        self.lease.gateway = gateway;
        self.lease.dns_server = dns;
        self.lease.subnet_mask = Ipv4Address::new(255, 255, 255, 0);
        self.lease.lease_time_secs = lease_time;
        self.lease.xid = ack_xid;
        self.state = DhcpState::Bound;

        Ok(self.lease)
    }
}
