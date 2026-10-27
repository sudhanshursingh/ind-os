# Key Ceremony & Threshold Signing Specification (SP-55, REQ-SEC-108, DEF-003)
**Phase:** Phase 7 — Network + Updates + Service Completion (SP-51 – SP-59)  
**Sprint:** SP-55 — Key Ceremony Tooling  
**Document ID:** `INDOS-SPEC-SEC-002`  
**Status:** **APPROVED & IMPLEMENTED**  
**Author:** Ind OS Security Council  

---

## 1. Architectural Scope & Invariants

Under Ind OS Development Bible v1.0.0.0 §7.2 DEF-003 and §8.13 REQ-SEC-108:
- **INV-CEREMONY-001 (2-of-3 Threshold Quorum):** Release artifacts (system updates, kernel releases) require cryptographic signatures from at least 2 distinct authorized custodians. A single signature (1-of-3) is insufficient and strictly rejected.
- **INV-CEREMONY-002 (Immutable Ceremony Audit Log):** Every ceremony action (custodian registration, signing attempt, revocation) is recorded in an append-only cryptographic event log.
- **INV-CEREMONY-003 (Compromise Revocation & Key Reconstitution):** In the event of an HSM or custodian key compromise, the compromised key is immediately revoked, and quorum is preserved via backup key rotation.

---

## 2. Custodian Roles & Quorum

| Custodian ID | Role | Key Type | Quorum Weight |
|---|---|---|---|
| **Custodian 1** | Technical Founder (`sudhanshurajsingh`) | Ed25519 Hardware Token | 1 |
| **Custodian 2** | Security Architecture Lead | Ed25519 Hardware Token | 1 |
| **Custodian 3** | Infrastructure & Release Lead | Ed25519 Hardware Token | 1 |

Threshold Rule: $k = 2, n = 3$. Valid release requires $\sum \text{signatures} \ge 2$.
