# Ind OS Decisions & Pending Questions Register (SP-00 Baseline)

**Document ID:** `INDOS-DEC-REG-001`  
**Governing Document:** `Ind_OS_Development_Bible_v1.0.0.0.md` (Sections 7.1, 7.2, 7.3)  
**Rule N-03:** No silent assumptions. Anything undecided is `[PEN-NNN]` and must be escalated.

---

## 1. Founder Directives (Resolved)

| ID | Title | Date | Decision |
| :--- | :--- | :--- | :--- |
| **DEC-001** | Kernel & Userspace from scratch | 2026-10-26 | From-scratch hybrid microkernel written in Rust (`docs/DECISIONS/DEC-001-founder-reasoning.md`). |
| **DEC-002** | Core AI integration | 2026-10-26 | AIS is a native OS subsystem, advisory-only, bounded by Rule N-04 and Rule N-05. |
| **DEC-003** | Multi-year horizon | 2026-10-26 | Quality gates govern progress; no calendar commitments without empirical evidence. |
| **DEC-004** | Cryptography posture | 2026-10-26 | Vetted, audited external TLS library (`rustls` or equivalent) in non-TCB userspace position. |
| **ADR-001** | Binary Format Strategy | 2026-10-26 | Native hardened micro-binary format (`INDBIN`) in kernel; ELF64 toolchain conversion (`docs/DECISIONS/ADR-001-binary-format.md`). |

---

## 2. Provisional Defaults (Proceed until Founder Overrides)

| ID | Title | Default Value | Review Gate | Status |
| :--- | :--- | :--- | :--- | :--- |
| **DEF-001** | Kernel architecture | Hybrid microkernel (privileged core + user-mode servers) | Gate G3 | ACTIVE |
| **DEF-002** | Implementation language | Memory-safe Rust for TCB; assembly only for arch entry/context switch | Gate G2 | ACTIVE |
| **DEF-003** | Release signing | 2-of-3 threshold release signing keys | Gate G7 | ACTIVE |
| **DEF-004** | TLS implementation | Vetted external library (`rustls`), SBOM-tracked | Gate G7 | ACTIVE |

---

## 3. Pending Questions (Founder Resolution Required)

| ID | Title | Question Summary | Blocking Impact |
| :--- | :--- | :--- | :--- |
| **PEN-013** | Android app compatibility | (a) VM/microVM, (b) Clean-room runtime, (c) Zero compat (Ind-native only), (d) Return to Android | **Blocks Phase 1 Kernel Sprints** (Phase 0 proceeds) |
| **PEN-001** | Scope of "from scratch" | Are external specifications (ELF, DTB) permitted? *(Default: Specs allowed, code forbidden)* | Open |
| **PEN-002** | Rust toolchain licensing | Accept standard MIT/Apache-2.0 or require formal legal review? | Open |
| **PEN-003** | Kernel & OS naming | Formal public names beyond codenames `IndK` and `IndUS`. | Open |
| **PEN-004** | Moto G60 GPU strategy | (a) Software rendering, (b) Reverse-engineered driver, (c) Defer beyond milestone. | Open |
| **PEN-005** | Hardware secure boot fallback | Dev-board fallback vs. documented partial chain if hanoip rejects custom boot. | Open |
| **PEN-007** | Signing-key hardware | HSM model, threshold keyholders, and legal custody entity. | Open |
| **PEN-008a** | AIS model self-training | Will Ind OS train any models from scratch? (Compute budget/data governance). | Open |
| **PEN-008b** | AIS open-weights adaptation | Which open-weights models are permitted under which licenses? | Open |
| **PEN-008c** | AIS hybrid posture | Allocation of self-trained vs. adapted models per capability. | Open |
| **PEN-009** | Cloud inference egress | What telemetry/inference may leave device under what explicit consent? | Open |
| **PEN-010** | Reference ARM64 dev board | Selection and budget for physical ARM64 reference board before hanoip. | Open |
| **PEN-011** | Staffing baseline | May engineering plan on assumed minimum staffing or founder-only gate input? | Open |
| **PEN-012** | Repository visibility | Public from day 1, private until G5, or public at G9? | Open |

---

## 4. Architectural RFCs & Change Requests

| ID | Title | Date | Status / Target | Summary & Non-Disruption Scope |
| :--- | :--- | :--- | :--- | :--- |
| **RFC-INDOS-2026-001** | Non-Disruptive Architectural Extensions | 2026-10-26 | PROPOSED (Phase 4 / Phase 6) | `REQ-AI-112` (Dynamic Hardware Profile Manager in `userspace/ais/` for SP-41) and `REQ-ARCH-101` (Standalone `indk-core` crate workspace decoupling for SP-31). Class: `CR-SEMANTIC`. Zero Phase 2/3 disruption. (`docs/RFC/RFC-INDOS-2026-001.md`). |
