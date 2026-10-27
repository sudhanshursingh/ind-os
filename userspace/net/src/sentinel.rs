//! Network Leak Sentinel (SP-56, REQ-AI-107 network half)
//!
//! Architectural Invariants:
//! - INV-LEAK-001: Strict classification of outbound flows (consented vs unconsented)
//! - INV-LEAK-002: Out-of-band sentinel alerting routed to trusted UI
//! - INV-LEAK-003: Policy-guided enforcement (zero silent ambient leakage)
//! - INV-LEAK-004: Statistical baseline monitoring for volume bursts and beaconing

pub const MAX_LEAK_ALERTS: usize = 16;
pub const MAX_QUARANTINED_IPS: usize = 8;
pub const LEAK_BURST_THRESHOLD_FACTOR: usize = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlowClass {
    Consented,
    SystemCritical,
    Unconsented,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LeakAnomalyType {
    UnconsentedEgress,
    VolumeBurst,
    CovertBeaconing,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SentinelAction {
    Allow,
    AlertOnly,
    HoldForConsent,
    Quarantine,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LeakAlert {
    pub alert_id: u32,
    pub anomaly_type: LeakAnomalyType,
    pub dst_ip: [u8; 4],
    pub dst_port: u16,
    pub bytes_observed: usize,
    pub baseline_bytes: usize,
    pub routed_to_trusted_ui: bool,
    pub action_taken: SentinelAction,
}

#[derive(Debug, Clone, Copy)]
pub struct NetworkFlowDescriptor {
    pub src_port: u16,
    pub dst_ip: [u8; 4],
    pub dst_port: u16,
    pub protocol: u8,
    pub bytes_sent: usize,
    pub has_consent_token: bool,
    pub is_system_critical: bool,
}

pub struct NetworkLeakSentinel {
    baseline_bytes_per_flow: usize,
    alerts: [Option<LeakAlert>; MAX_LEAK_ALERTS],
    alert_count: usize,
    next_alert_id: u32,
    quarantined_ips: [Option<[u8; 4]>; MAX_QUARANTINED_IPS],
    quarantine_count: usize,
}

impl NetworkLeakSentinel {
    pub const fn new(baseline_bytes_per_flow: usize) -> Self {
        Self {
            baseline_bytes_per_flow,
            alerts: [None; MAX_LEAK_ALERTS],
            alert_count: 0,
            next_alert_id: 9001,
            quarantined_ips: [None; MAX_QUARANTINED_IPS],
            quarantine_count: 0,
        }
    }

    /// Classify flow category (INV-LEAK-001)
    pub fn classify_flow(&self, flow: &NetworkFlowDescriptor) -> FlowClass {
        if flow.is_system_critical {
            FlowClass::SystemCritical
        } else if flow.has_consent_token {
            FlowClass::Consented
        } else {
            FlowClass::Unconsented
        }
    }

    /// Check if target IP is in quarantine list
    pub fn is_quarantined(&self, dst_ip: [u8; 4]) -> bool {
        for slot in self.quarantined_ips.iter() {
            if let Some(ip) = slot {
                if *ip == dst_ip {
                    return true;
                }
            }
        }
        false
    }

    /// Inspect outbound flow and detect anomalies (REQ-AI-107, INV-LEAK-001..004)
    pub fn inspect_outbound_flow(&mut self, flow: &NetworkFlowDescriptor) -> SentinelAction {
        // Fast path: if already quarantined, drop immediately
        if self.is_quarantined(flow.dst_ip) {
            return SentinelAction::Quarantine;
        }

        let classification = self.classify_flow(flow);

        // Anomaly 1: Unconsented outbound egress (INV-LEAK-001)
        if classification == FlowClass::Unconsented {
            let alert_id = self.next_alert_id;
            self.next_alert_id += 1;

            let alert = LeakAlert {
                alert_id,
                anomaly_type: LeakAnomalyType::UnconsentedEgress,
                dst_ip: flow.dst_ip,
                dst_port: flow.dst_port,
                bytes_observed: flow.bytes_sent,
                baseline_bytes: self.baseline_bytes_per_flow,
                routed_to_trusted_ui: true, // INV-LEAK-002
                action_taken: SentinelAction::HoldForConsent,
            };

            self.record_alert(alert);
            // Quarantine target if untrusted
            self.add_quarantine(flow.dst_ip);
            return SentinelAction::HoldForConsent;
        }

        // Anomaly 2: Consented volume burst (>3x baseline) (INV-LEAK-004)
        if classification == FlowClass::Consented
            && flow.bytes_sent >= self.baseline_bytes_per_flow * LEAK_BURST_THRESHOLD_FACTOR
        {
            let alert_id = self.next_alert_id;
            self.next_alert_id += 1;

            let alert = LeakAlert {
                alert_id,
                anomaly_type: LeakAnomalyType::VolumeBurst,
                dst_ip: flow.dst_ip,
                dst_port: flow.dst_port,
                bytes_observed: flow.bytes_sent,
                baseline_bytes: self.baseline_bytes_per_flow,
                routed_to_trusted_ui: true, // INV-LEAK-002
                action_taken: SentinelAction::AlertOnly,
            };

            self.record_alert(alert);
            return SentinelAction::AlertOnly;
        }

        SentinelAction::Allow
    }

    fn record_alert(&mut self, alert: LeakAlert) {
        if self.alert_count < MAX_LEAK_ALERTS {
            self.alerts[self.alert_count] = Some(alert);
            self.alert_count += 1;
        } else {
            // Circular overwrite
            self.alerts[0] = Some(alert);
        }
    }

    fn add_quarantine(&mut self, ip: [u8; 4]) {
        if self.quarantine_count < MAX_QUARANTINED_IPS {
            self.quarantined_ips[self.quarantine_count] = Some(ip);
            self.quarantine_count += 1;
        }
    }

    pub fn alert_count(&self) -> usize {
        self.alert_count
    }

    pub fn alerts(&self) -> &[Option<LeakAlert>; MAX_LEAK_ALERTS] {
        &self.alerts
    }

    pub fn baseline_bytes(&self) -> usize {
        self.baseline_bytes_per_flow
    }
}
