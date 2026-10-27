# Ind OS Device Evidence Pack & Recovery Specification (SP-71)

## 1. Architectural Mandate
- **Governing Standard:** Ind OS Development Bible v1.0.0.0 §6.3, §6.4, §8.15, Table 19.2 (REQ-HW-102, Rule N-06 / PRD-001 §7.5, §9.2).
- **Target:** Moto G60 (`hanoip`, Snapdragon 732G) exact hardware identification, partition table mapping, stock ROM cryptographic hash pinning, and recovery image verification without destructive flashing.

---

## 2. Architectural Invariants

| ID | Description | Enforcement Mechanism |
| :--- | :--- | :--- |
| **INV-EVID-001** | Device Identity Integrity | Hardware descriptor verifies exact target: Moto G60 `hanoip`, SoC SM7150-AC, 6GB RAM, UFS 2.2 storage |
| **INV-EVID-002** | Partition Map & Cryptographic Pinning | A/B partition table (boot, dtbo, super, vbmeta) pinned with immutable SHA256 checksums |
| **INV-EVID-003** | Non-Destructive Recovery Validation | Known-good stock recovery image validated; restore procedure passes cryptographic signature check |

---

## 3. Data Structures & API Contracts

```rust
pub const MOTO_G60_DEVICE_CODENAME: &str = "hanoip";
pub const MOTO_G60_SOC_ID: &str = "SM7150-AC";

pub enum DeviceEvidenceError {
    DeviceMismatch,
    PartitionNotFound,
    HashVerificationFailed,
    TableFull,
}

pub struct PartitionDescriptor {
    pub name: &'static str,
    pub size_kb: u64,
    pub is_ab_slot: bool,
    pub sha256_hash: &'static str,
}

pub struct DeviceEvidenceHarness {
    device_verified: bool,
    partition_count: usize,
    recovery_image_verified: bool,
}
```
