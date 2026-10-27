# Ind OS Architecture Specification: Gate G9 Acceptance Pack & Manifest

**Sprint:** SP-108  
**Milestone:** Phase 9 (Moto G60 Bring-Up)  
**Constitutional References:**
- `Ind_OS_Development_Bible_v1.0.0.0.md` §6.3, §8.15, §19.2 Table 19.2, §19.3, §19.4, §27.3
- PRD-001 §9.5: Gate G9 Acceptance Pack Assembly
- Rule N-01: Zero external crates in TCB
- Rule N-04: Zero unsafe code in new userspace modules
- Rule N-06 / PRD-001 §7.5: Recovery-first and verifiable hardware evidence

---

## 1. Executive Summary

Sprint SP-108 formally gathers, structures, cryptographic-hashes, and audits all evidence required for the **Gate G9 Hard-Stop Audit** per PRD-001 §9.5 and Bible §19.4. 

The acceptance pack serves as the immutable evidentiary basis for ratifying milestone **MOTO G60 BRING-UP / HARDWARE V0.1**.

---

## 2. Evidence Pack Structure (§19.4)

The Gate G9 evidence pack comprises exactly 9 core evidentiary artifacts located under `docs/evidence/G9/`:

| Section | Artifact | Purpose & Standard |
|---|---|---|
| **1. Manifest** | `docs/evidence/G9/manifest.json` | CycloneDX-aligned SHA-256 / CRC32 hashes of all Phase 9 deliverables. |
| **2. Build Provenance** | `docs/evidence/G9/build-provenance.md` | Build configuration, git commit sha, reproducible toolchain metadata. |
| **3. Test Logs** | `docs/evidence/G9/dod-test-logs.md` | Consolidated logs from all Phase 9 sprint DoD harnesses (`ci/verify_sp*.py`). |
| **4. Cold-Boot Reproduction** | `docs/evidence/golden-boot-sp107.log` | REQ-QA-201 proof of 10/10 consecutive cold boots on Moto G60 hardware ($16\text{ ms}$ jitter). |
| **5. Docs Diff** | `docs/evidence/G9/docs-diff-phase9.md` | Architecture and PRD documentation delta from Gate G8 to Gate G9. |
| **6. Hardware Matrix** | `docs/architecture/hardware-compatibility-matrix-v1.md` | PRD-001 §8.3 matrix covering all 15 SoC components (12 Works, 2 Partial, 1 Out-of-Scope). |
| **7. Recovery Route** | `docs/evidence/SP-105-validation.md` | Rule N-06 non-destructive recovery verification: Fastboot and Qualcomm EDL 9008 Sahara re-tested. |
| **8. DEC-001 Minute** | `docs/DECISIONS/gate-G9-dec001-review.md` | Minuted founder decision to **Continue** on from-scratch kernel path (§19.3). |
| **9. Signatures** | `docs/evidence/G9/gate-g9-signatures.md` | Two-person rule sign-off (Founder + Phase 9 Lead) per REQ-GOV-106. |

---

## 3. Invariants

| ID | Description | Enforcement Mechanism |
|---|---|---|
| **INV-G9-PACK-001** | **Complete Evidentiary Manifest** | All 9 required sections must be present and verified with valid cryptographic checksums (`0x98A14C2F`). |
| **INV-G9-PACK-002** | **TCB Crate Zero-External Hygiene** | Phase 9 SBOM audit must prove exactly 0 external crate dependencies in TCB (`kernel/Cargo.toml`). |
| **INV-G9-PACK-003** | **DEC-001 Re-Examination Protocol** | Founder minuted decision explicitly committed to `docs/DECISIONS/gate-G9-dec001-review.md` before ratification. |

---

## 4. API & Capability Model

Operations require the specific capability token `CAP_G9_PACK_ADMIN` (`0x0000_1080`).

```rust
pub const CAP_G9_PACK_ADMIN: u32 = 0x0000_1080;
pub const G9_TOTAL_EVIDENCE_ITEMS: usize = 9;
pub const G9_MANIFEST_HASH_CRC32: u32 = 0x98A1_4C2F;
pub const G9_SBOM_COMPONENT_COUNT: usize = 6;
pub const G9_TCB_EXTERNAL_CRATES: usize = 0;
```
