# Ind OS Architecture Specification: Anomaly Sentinels v1 (SP-48)

## Document Information
- **Standard**: Ind OS Development Bible v1.0.0.0 §6.3, §8.12
- **Requirement**: REQ-AI-107, Bible §8.12 (Anomaly sentinels v1: permission-use and storage-access baselines; alerts via trusted UI only)
- **Invariants**:
  - `INV-SENTINEL-001`: Statistical Baseline Tracking — Permission invocation and storage throughput rates are tracked against statistical running averages without granting the sentinel ambient authority to alter capability grants.
  - `INV-SENTINEL-002`: Trusted UI Alert Path Only — Detected anomalies (e.g. rate-limit bursts, unexpected bulk storage reads) produce alerts dispatched exclusively to the secure, isolated Trusted UI channel.
- **Rule Set**: Rule N-01 (100% pure Rust), Rule N-04 (Zero unsafe).

---

## 1. Scope & Sentinel Operation

```
           +---------------------------------------------+
           |           Observed Subsystems               |
           |     (Storage, Network, Hardware Devices)    |
           +----------------------+----------------------+
                                  | Rate Events
           +----------------------v----------------------+
           |             Anomaly Sentinel v1             |
           |  +---------------------------------------+  |
           |  | Baseline Sliding Window               |  |
           |  | Rate Threshold Monitor                |  |
           |  | Spike Detector                        |  |
           |  +-------------------+-------------------+  |
           +----------------------|----------------------+
                                  | Anomaly Alarm
           +----------------------v----------------------+
           |             Trusted UI Only                 |
           |    (Prompt user / Alert without silent ban) |
           +---------------------------------------------+
```

---

## 2. Sentinel Data Structures

```rust
pub enum AnomalyType {
    PermissionSpike,
    StorageAccessBurst,
    UnexpectedNetworkFlow,
}

pub struct AnomalyAlert {
    pub alert_id: u32,
    pub anomaly_type: AnomalyType,
    pub observed_rate: u32,
    pub baseline_rate: u32,
    pub message: &'static str,
    pub routed_to_trusted_ui: bool,
}
```

---

## 3. Kernel Verification Stages (187–191)
- `187_SENTINEL_V1_INIT`: Anomaly sentinel engine v1 initialized.
- `188_SENTINEL_V1_BASELINE_TRACKING`: Statistical permission & storage baseline tracking verified.
- `189_SENTINEL_V1_ANOMALY_DETECTION`: Rapid permission spike & excessive access detection verified.
- `190_SENTINEL_V1_TRUSTED_UI_ALARM`: Out-of-band alarm dispatch to trusted UI only verified.
- `191_SP48_SENTINEL_READY`: Sprint SP-48 Milestone reached.
