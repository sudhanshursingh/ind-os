//! In-Kernel Capability-Gated Firewall (SP-53, INV-FW-001, INV-FW-002)
//!
//! Architectural Invariants:
//! - INV-FW-001: Default-deny inbound posture on all network interfaces
//! - INV-FW-002: Capability-gated rule modification (CAP_NET_FIREWALL_ADMIN)

use crate::types::{IpProtocol, Ipv4Address, NetError};

pub const CAP_NET_FIREWALL_ADMIN: u32 = 0x0100;
pub const MAX_FIREWALL_RULES: usize = 32;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PacketDirection {
    Inbound,
    Outbound,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterAction {
    Allow,
    Deny,
}

#[derive(Debug, Clone, Copy)]
pub struct FirewallRule {
    pub id: u32,
    pub direction: PacketDirection,
    pub protocol: IpProtocol,
    pub remote_ip: Option<Ipv4Address>,
    pub port_range_start: u16,
    pub port_range_end: u16,
    pub action: FilterAction,
}

pub struct FirewallEngine {
    rules: [Option<FirewallRule>; MAX_FIREWALL_RULES],
    next_id: u32,
}

impl FirewallEngine {
    pub const fn new() -> Self {
        Self {
            rules: [None; MAX_FIREWALL_RULES],
            next_id: 1,
        }
    }

    /// Add rule requiring CAP_NET_FIREWALL_ADMIN capability (INV-FW-002)
    pub fn add_rule(
        &mut self,
        admin_cap: u32,
        direction: PacketDirection,
        protocol: IpProtocol,
        remote_ip: Option<Ipv4Address>,
        port_start: u16,
        port_end: u16,
        action: FilterAction,
    ) -> Result<u32, NetError> {
        if (admin_cap & CAP_NET_FIREWALL_ADMIN) == 0 {
            return Err(NetError::PermissionDenied);
        }

        for slot in self.rules.iter_mut() {
            if slot.is_none() {
                let id = self.next_id;
                self.next_id += 1;
                *slot = Some(FirewallRule {
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
        Err(NetError::QueueFull)
    }

    /// Remove rule requiring CAP_NET_FIREWALL_ADMIN capability (INV-FW-002)
    pub fn remove_rule(&mut self, admin_cap: u32, rule_id: u32) -> Result<(), NetError> {
        if (admin_cap & CAP_NET_FIREWALL_ADMIN) == 0 {
            return Err(NetError::PermissionDenied);
        }

        for slot in self.rules.iter_mut() {
            if let Some(r) = slot {
                if r.id == rule_id {
                    *slot = None;
                    return Ok(());
                }
            }
        }
        Err(NetError::SocketNotFound)
    }

    /// Evaluate packet against firewall policy (INV-FW-001)
    /// Default-Deny for Inbound traffic, Default-Allow for Outbound traffic
    pub fn filter_packet(
        &self,
        direction: PacketDirection,
        protocol: IpProtocol,
        remote_ip: Ipv4Address,
        port: u16,
    ) -> FilterAction {
        for slot in self.rules.iter() {
            if let Some(r) = slot {
                if r.direction == direction
                    && (r.protocol == protocol || r.protocol == IpProtocol::Unknown)
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
            PacketDirection::Inbound => FilterAction::Deny,
            PacketDirection::Outbound => FilterAction::Allow,
        }
    }
}
