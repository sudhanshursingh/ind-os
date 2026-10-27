//! Kernel Network Services Verification Harness: DNS, DHCP, Firewall (SP-53)
//!
//! Architectural Invariants:
//! - INV-FW-001: Default-deny inbound posture on all network interfaces
//! - INV-FW-002: Capability-gated rule modification (CAP_NET_FIREWALL_ADMIN)
//! - INV-DHCP-001: Discover/Offer/Request/Ack state machine
//! - INV-DNS-001: Transaction ID verification and spoof rejection

use crate::userspace::net_core::{
    KernelIpProtocol, KernelIpv4Address, KernelMacAddress, KernelNetError,
};

pub const KERNEL_CAP_NET_FIREWALL_ADMIN: u32 = 0x0100;
pub const KERNEL_MAX_FIREWALL_RULES: usize = 32;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KernelPacketDirection {
    Inbound,
    Outbound,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KernelFilterAction {
    Allow,
    Deny,
}

#[derive(Debug, Clone, Copy)]
pub struct KernelFirewallRule {
    pub id: u32,
    pub direction: KernelPacketDirection,
    pub protocol: KernelIpProtocol,
    pub remote_ip: Option<KernelIpv4Address>,
    pub port_range_start: u16,
    pub port_range_end: u16,
    pub action: KernelFilterAction,
}

pub struct KernelFirewallEngine {
    rules: [Option<KernelFirewallRule>; KERNEL_MAX_FIREWALL_RULES],
    next_id: u32,
}

impl KernelFirewallEngine {
    pub const fn new() -> Self {
        Self {
            rules: [None; KERNEL_MAX_FIREWALL_RULES],
            next_id: 1,
        }
    }

    pub fn add_rule(
        &mut self,
        admin_cap: u32,
        direction: KernelPacketDirection,
        protocol: KernelIpProtocol,
        remote_ip: Option<KernelIpv4Address>,
        port_start: u16,
        port_end: u16,
        action: KernelFilterAction,
    ) -> Result<u32, KernelNetError> {
        if (admin_cap & KERNEL_CAP_NET_FIREWALL_ADMIN) == 0 {
            return Err(KernelNetError::PermissionDenied);
        }

        for slot in self.rules.iter_mut() {
            if slot.is_none() {
                let id = self.next_id;
                self.next_id += 1;
                *slot = Some(KernelFirewallRule {
                    id,
                    direction,
                    protocol,
                    remote_ip,
                    port_range_start: port_start,
                    port_range_end: port_end,
                    action,
                });
                return Ok(id);
            }
        }
        Err(KernelNetError::QueueFull)
    }

    pub fn remove_rule(&mut self, admin_cap: u32, rule_id: u32) -> Result<(), KernelNetError> {
        if (admin_cap & KERNEL_CAP_NET_FIREWALL_ADMIN) == 0 {
            return Err(KernelNetError::PermissionDenied);
        }

        for slot in self.rules.iter_mut() {
            if let Some(r) = slot {
                if r.id == rule_id {
                    *slot = None;
                    return Ok(());
                }
            }
        }
        Err(KernelNetError::SocketNotFound)
    }

    pub fn filter_packet(
        &self,
        direction: KernelPacketDirection,
        protocol: KernelIpProtocol,
        remote_ip: KernelIpv4Address,
        port: u16,
    ) -> KernelFilterAction {
        for slot in self.rules.iter() {
            if let Some(r) = slot {
                if r.direction == direction
                    && (r.protocol == protocol || r.protocol == KernelIpProtocol::Unknown)
                    && (r.remote_ip.is_none() || r.remote_ip == Some(remote_ip))
                    && port >= r.port_range_start
                    && port <= r.port_range_end
                {
                    return r.action;
                }
            }
        }

        // Posture default: Inbound DENY, Outbound ALLOW (INV-FW-001)
        match direction {
            KernelPacketDirection::Inbound => KernelFilterAction::Deny,
            KernelPacketDirection::Outbound => KernelFilterAction::Allow,
        }
    }
}

pub struct KernelDhcpHarness;

impl KernelDhcpHarness {
    pub fn simulate_lease(
        mac: KernelMacAddress,
        xid: u32,
    ) -> (KernelIpv4Address, KernelIpv4Address, KernelIpv4Address) {
        let _ = mac;
        let _ = xid;
        (
            KernelIpv4Address([10, 0, 2, 15]), // Assigned IP
            KernelIpv4Address([10, 0, 2, 2]),  // Gateway
            KernelIpv4Address([10, 0, 2, 3]),  // DNS
        )
    }
}

pub struct KernelDnsHarness;

impl KernelDnsHarness {
    pub fn resolve(
        query_domain: &str,
        tx_id: u16,
        resp_tx_id: u16,
    ) -> Result<KernelIpv4Address, KernelNetError> {
        if tx_id != resp_tx_id {
            return Err(KernelNetError::PermissionDenied); // Spoofed response rejected
        }

        if query_domain == "api.indos.internal" {
            Ok(KernelIpv4Address([10, 0, 2, 50]))
        } else if query_domain == "update.indos.internal" {
            Ok(KernelIpv4Address([10, 0, 2, 51]))
        } else {
            Err(KernelNetError::SocketNotFound)
        }
    }
}
