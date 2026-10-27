//! Kernel Network Leak Sentinel Verification Harness (SP-56, REQ-AI-107 network half)
//!
//! Architectural Invariants:
//! - INV-LEAK-001: Strict classification of outbound flows (consented vs unconsented)
//! - INV-LEAK-002: Out-of-band sentinel alerting routed to trusted UI
//! - INV-LEAK-003: Policy-guided enforcement (zero silent ambient leakage)
//! - INV-LEAK-004: Statistical baseline monitoring for volume bursts and beaconing

pub const KERNEL_MAX_LEAK_ALERTS: usize = 16;
pub const KERNEL_MAX_QUARANTINED_IPS: usize = 8;
pub const KERNEL_LEAK_BURST_THRESHOLD_FACTOR: usize = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KernelFlowClass {
    Consented,
    SystemCritical,
    Unconsented,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KernelLeakAnomalyType {
    UnconsentedEgress,
    VolumeBurst,
    CovertBeaconing,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KernelSentinelAction {
    Allow,
    AlertOnly,
    HoldForConsent,
    Quarantine,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KernelLeakAlert {
    pub alert_id: u32,
    pub anomaly_type: KernelLeakAnomalyType,
    pub dst_ip: [u8; 4],
    pub dst_port: u16,
    pub bytes_observed: usize,
    pub baseline_bytes: usize,
    pub routed_to_trusted_ui: bool,
    pub action_taken: KernelSentinelAction,
}

#[derive(Debug, Clone, Copy)]
pub struct KernelNetworkFlowDescriptor {
    pub src_port: u16,
    pub dst_ip: [u8; 4],
    pub dst_port: u16,
    pub protocol: u8,
    pub bytes_sent: usize,
    pub has_consent_token: bool,
    pub is_system_critical: bool,
}

pub struct KernelNetworkLeakSentinel {
    baseline_bytes_per_flow: usize,
    alerts: [Option<KernelLeakAlert>; KERNEL_MAX_LEAK_ALERTS],
    alert_count: usize,
    next_alert_id: u32,
    quarantined_ips: [Option<[u8; 4]>; KERNEL_MAX_QUARANTINED_IPS],
    quarantine_count: usize,
}

impl KernelNetworkLeakSentinel {
    pub const fn new(baseline_bytes_per_flow: usize) -> Self {
        Self {
            baseline_bytes_per_flow,
            alerts: [None; KERNEL_MAX_LEAK_ALERTS],
            alert_count: 0,
            next_alert_id: 9001,
            quarantined_ips: [None; KERNEL_MAX_QUARANTINED_IPS],
            quarantine_count: 0,
        }
    }

    pub fn classify_flow(&self, flow: &KernelNetworkFlowDescriptor) -> KernelFlowClass {
        if flow.is_system_critical {
            KernelFlowClass::SystemCritical
        } else if flow.has_consent_token {
            KernelFlowClass::Consented
        } else {
            KernelFlowClass::Unconsented
        }
    }

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

    pub fn inspect_outbound_flow(
        &mut self,
        flow: &KernelNetworkFlowDescriptor,
    ) -> KernelSentinelAction {
        if self.is_quarantined(flow.dst_ip) {
            return KernelSentinelAction::Quarantine;
        }

        let classification = self.classify_flow(flow);

        // Anomaly 1: Unconsented outbound egress (INV-LEAK-001)
        if classification == KernelFlowClass::Unconsented {
            let alert_id = self.next_alert_id;
            self.next_alert_id += 1;

            let alert = KernelLeakAlert {
                alert_id,
                anomaly_type: KernelLeakAnomalyType::UnconsentedEgress,
                dst_ip: flow.dst_ip,
                dst_port: flow.dst_port,
                bytes_observed: flow.bytes_sent,
                baseline_bytes: self.baseline_bytes_per_flow,
                routed_to_trusted_ui: true, // INV-LEAK-002
                action_taken: KernelSentinelAction::HoldForConsent,
            };

            self.record_alert(alert);
            self.add_quarantine(flow.dst_ip);
            return KernelSentinelAction::HoldForConsent;
        }

        // Anomaly 2: Consented volume burst (>3x baseline) (INV-LEAK-004)
        if classification == KernelFlowClass::Consented
            && flow.bytes_sent >= self.baseline_bytes_per_flow * KERNEL_LEAK_BURST_THRESHOLD_FACTOR
        {
            let alert_id = self.next_alert_id;
            self.next_alert_id += 1;

            let alert = KernelLeakAlert {
                alert_id,
                anomaly_type: KernelLeakAnomalyType::VolumeBurst,
                dst_ip: flow.dst_ip,
                dst_port: flow.dst_port,
                bytes_observed: flow.bytes_sent,
                baseline_bytes: self.baseline_bytes_per_flow,
                routed_to_trusted_ui: true, // INV-LEAK-002
                action_taken: KernelSentinelAction::AlertOnly,
            };

            self.record_alert(alert);
            return KernelSentinelAction::AlertOnly;
        }

        KernelSentinelAction::Allow
    }

    fn record_alert(&mut self, alert: KernelLeakAlert) {
        if self.alert_count < KERNEL_MAX_LEAK_ALERTS {
            self.alerts[self.alert_count] = Some(alert);
            self.alert_count += 1;
        } else {
            self.alerts[0] = Some(alert);
        }
    }

    fn add_quarantine(&mut self, ip: [u8; 4]) {
        if self.quarantine_count < KERNEL_MAX_QUARANTINED_IPS {
            self.quarantined_ips[self.quarantine_count] = Some(ip);
            self.quarantine_count += 1;
        }
    }

    pub fn alert_count(&self) -> usize {
        self.alert_count
    }

    pub fn alerts(&self) -> &[Option<KernelLeakAlert>; KERNEL_MAX_LEAK_ALERTS] {
        &self.alerts
    }

    pub fn baseline_bytes(&self) -> usize {
        self.baseline_bytes_per_flow
    }
}
