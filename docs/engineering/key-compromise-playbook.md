# Key Compromise Response Playbook (SP-55, REQ-SEC-108)
**Phase:** Phase 7 — Network + Updates + Service Completion  
**Sprint:** SP-55 — Key Ceremony Tooling  
**Document ID:** `INDOS-PLAYBOOK-SEC-001`  
**Status:** **ACTIVE & RATIFIED**  
**Classification:** Operational Security Response  

---

## 1. Trigger Conditions

1. Suspected or confirmed physical theft / unauthorized access to a custodian HSM.
2. Cryptographic anomaly or anomalous release signing attempt detected by audit sentinels.
3. Compromise declaration by any designated key custodian.

---

## 2. Emergency Response Protocol

1. **Step 1: Immediate Revocation Dispatch:**
   - Any surviving custodian invokes `ThresholdSigner::revoke_custodian(custodian_id)`.
   - The compromised key ID is placed onto the immediate revocation list.
   - All subsequent release manifests signed by the revoked key are invalidated.

2. **Step 2: Quorum Integrity Maintenance:**
   - Because $n = 3, k = 2$, revocation of a single key still leaves 2 valid keys available.
   - The remaining 2 custodians can sign an emergency key rotation manifest.

3. **Step 3: Reconstitution Ceremony:**
   - Rotate backup key pair for the affected custodian slot.
   - Publish transparency log update signed by the 2 valid custodians.
   - Update client trust anchors via authenticated A/B update (`SP-54`).
