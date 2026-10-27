//! Kernel Anomaly Sentinel Verification Harness (SP-48)
//!
//! Constitutional References:
//! - Bible §6.3, §8.12: Anomaly sentinels v1 (permission-use and storage-access baselines; alerts via trusted UI only)
//! - REQ-AI-107: Anomaly sentinels v1
//! - INV-SENTINEL-001: Statistical baseline tracking
//! - INV-SENTINEL-002: Trusted UI alert path only
//! - Rule N-01: Zero external crates
//! - Rule N-04: Zero unsafe code in new modules

use crate::userspace::logd::{KernelLogBus, KernelLogLevel};

pub const KERNEL_MAX_SENTINEL_ALERTS: usize = 8;
pub const KERNEL_DEFAULT_BASELINE_THRESHOLD_FACTOR: u32 = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KernelAnomalyType {
    PermissionSpike,
    StorageAccessBurst,
    UnexpectedNetworkFlow,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KernelAnomalyAlert {
    pub alert_id: u32,
    pub anomaly_type: KernelAnomalyType,
    pub observed_rate: u32,
    pub baseline_rate: u32,
    pub message: &'static str,
    pub routed_to_trusted_ui: bool,
}

pub struct KernelAnomalySentinelV1 {
    perm_baseline: u32,
    storage_baseline: u32,
    alerts: [Option<KernelAnomalyAlert>; KERNEL_MAX_SENTINEL_ALERTS],
    alert_count: usize,
    next_alert_id: u32,
}

impl KernelAnomalySentinelV1 {
    pub const fn new(initial_perm_baseline: u32, initial_storage_baseline: u32) -> Self {
        Self {
            perm_baseline: initial_perm_baseline,
            storage_baseline: initial_storage_baseline,
            alerts: [None; KERNEL_MAX_SENTINEL_ALERTS],
            alert_count: 0,
            next_alert_id: 8001,
        }
    }

    pub fn inspect_permission_rate(&mut self, current_rate: u32, log_bus: &mut KernelLogBus) -> Option<KernelAnomalyAlert> {
        if current_rate >= self.perm_baseline * KERNEL_DEFAULT_BASELINE_THRESHOLD_FACTOR {
            let id = self.next_alert_id;
            self.next_alert_id += 1;

            let alert = KernelAnomalyAlert {
                alert_id: id,
                anomaly_type: KernelAnomalyType::PermissionSpike,
                observed_rate: current_rate,
                baseline_rate: self.perm_baseline,
                message: "Permission invocation rate exceeded safe baseline threshold (3x)",
                routed_to_trusted_ui: true, // Dispatched to trusted UI only (INV-SENTINEL-002)
            };

            if self.alert_count < KERNEL_MAX_SENTINEL_ALERTS {
                self.alerts[self.alert_count] = Some(alert);
                self.alert_count += 1;
            }

            // Log security alert to structured bus
            log_bus.ingest(1000, KernelLogLevel::SecurityAudit, 9, id, "Anomaly Sentinel: permission spike detected");

            Some(alert)
        } else {
            None
        }
    }

    pub fn inspect_storage_rate(&mut self, bytes_per_sec_kb: u32, log_bus: &mut KernelLogBus) -> Option<KernelAnomalyAlert> {
        if bytes_per_sec_kb >= self.storage_baseline * KERNEL_DEFAULT_BASELINE_THRESHOLD_FACTOR {
            let id = self.next_alert_id;
            self.next_alert_id += 1;

            let alert = KernelAnomalyAlert {
                alert_id: id,
                anomaly_type: KernelAnomalyType::StorageAccessBurst,
                observed_rate: bytes_per_sec_kb,
                baseline_rate: self.storage_baseline,
                message: "Bulk storage I/O rate exceeded historical baseline",
                routed_to_trusted_ui: true,
            };

            if self.alert_count < KERNEL_MAX_SENTINEL_ALERTS {
                self.alerts[self.alert_count] = Some(alert);
                self.alert_count += 1;
            }

            log_bus.ingest(1000, KernelLogLevel::SecurityAudit, 9, id, "Anomaly Sentinel: storage burst detected");

            Some(alert)
        } else {
            None
        }
    }

    pub fn get_alert(&self, alert_id: u32) -> Option<&KernelAnomalyAlert> {
        for i in 0..self.alert_count {
            if let Some(ref a) = self.alerts[i] {
                if a.alert_id == alert_id {
                    return Some(a);
                }
            }
        }
        None
    }

    pub fn alert_count(&self) -> usize {
        self.alert_count
    }
}
