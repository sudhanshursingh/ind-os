# Sprint SP-121 Specification: Production Operations & Rollback-Safe A/B Update Pipeline

## 1. Context & Objectives
- **Milestone:** Phase 10 — Hardening, Verification, Beta Readiness (Bible §8.17)
- **Sprint:** SP-121 — Production operations & update infrastructure (Gate G10 prerequisite)
- **Primary Requirement:** `REQ-OPS-101` (Dual-slot atomic OTA update pipeline with rollback)
- **Constitutional Invariant:** `INV-PRODOPS-001` (Cryptographic Update Integrity & Automatic Fallback)

## 2. Architecture & Operational Protocol
1. **Cryptographic Signing Ceremony:** Releases are cryptographically signed using an in-tree ED25519 root key. The kernel verifies the manifest digest against hardcoded trust anchors before staging.
2. **Atomic Inactive Slot Staging:** The payload is written into the inactive partition slot (`boot_b` or `boot_a`) without modifying the active partition.
3. **Watchdog-Protected Boot Trial:**
   - Active slot points to new image with `tries_remaining = 3`.
   - Microkernel must execute self-tests and assert `mark_boot_successful()` within 30 seconds.
   - If watchdog timer expires or kernel panics, Qualcomm ABL automatically reverts back to the known-good slot.

## 3. Invariants
- `INV-PRODOPS-001`: Staging corrupted or unsigned payloads MUST fail verification before modifying boot metadata.
- `INV-PRODOPS-001a`: Boot failure decrements `tries_remaining` and triggers automatic rollback when exhausted.
