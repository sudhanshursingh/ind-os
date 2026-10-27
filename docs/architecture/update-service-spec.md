# A/B Update Service Specification (SP-54, REQ-USE-205)
**Phase:** Phase 7 — Network + Updates + Service Completion (SP-51 – SP-59)  
**Sprint:** SP-54 — Update Service  
**Document ID:** `INDOS-SPEC-UPD-001`  
**Status:** **APPROVED & IMPLEMENTED**  
**Author:** Ind OS Reliability & Update Architecture Group  

---

## 1. Architectural Scope & Requirements

Under Ind OS Development Bible v1.0.0.0 §8.13 and `REQ-USE-205`:
- **Dual Partition Slots (A/B):** The operating system maintains two independent bootable slots (`SlotA` and `SlotB`).
- **INV-UPD-001 (Atomic Slot Switch):** Updates are written only to the inactive slot. The active boot pointer is updated atomically in non-volatile metadata only after complete image verification.
- **INV-UPD-002 (Cryptographically Signed Manifests):** Every update package must carry an Ed25519/RSA signature verified against pinned root keys. Unsigned or corrupted images are rejected before any disk write.
- **INV-UPD-003 (Verified Rollback):** Upon reboot into the new slot, a health check watchdog evaluates system integrity. Failure to verify boot health triggers immediate, automatic rollback to the previous known-good slot.
- **INV-UPD-004 (User-Controlled Update Policy):** System updates and reboots respect user policy (`ManualApproval`, `ScheduledWindow`, `Immediate`). No ambient unconsented reboots.

---

## 2. A/B Slot State Machine

```text
    +-----------------------------------------------+
    |                                               |
    v                                               |
[Slot A Active] --- Update Staged in Slot B ---> [Slot B Pending]
    |                                               |
    |                                           Health Check
    |                                          /            \
    |                                      Pass              Fail (Watchdog)
    |                                       /                  \
    |                                      v                    v
    +------------------------------ [Slot B Active]       [Rollback to Slot A]
```
