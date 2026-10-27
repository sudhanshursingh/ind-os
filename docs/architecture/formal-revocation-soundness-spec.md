# Sprint SP-117 Specification: Formal Verification of Capability Revocation Soundness

## 1. Context & Objectives
- **Milestone:** Phase 10 — Hardening, Verification, Beta Readiness (Bible §8.17)
- **Sprint:** SP-117 — Formal verification push: Capability revocation soundness
- **Primary Requirement:** `REQ-SEC-106` (Formal verification of capability-core invariants)
- **Constitutional Invariant:** `INV-FORMAL-002` (Revocation Soundness & Cascade Completeness)

## 2. Mathematical Definition
Let $T = (V, E)$ be the capability derivation tree rooted at $C_{\text{root}}$.
For any capability node $u \in V$, let $\text{Descendants}(u) = \{ v \in V \mid u \rightsquigarrow v \}$.
When a revocation action $\text{Revoke}(u)$ is executed:
$$\forall v \in \{ u \} \cup \text{Descendants}(u), \quad \text{State}(v) \leftarrow \text{Revoked}$$
Soundness Guarantee:
$$\forall v \in \{ u \} \cup \text{Descendants}(u), \quad \text{Access}(v) = \bot$$
No dangling capability handle can retain authority once its ancestor has been revoked.

## 3. Invariants
- `INV-FORMAL-002`: Instantaneous cascade revocation: all descendants of revoked capability $u$ transition to `Revoked` state within the same transaction.
- `INV-FORMAL-002a`: Zero Authority on Revocation: invoking any revoked capability returns `Err(CapabilityRevoked)`.
- `INV-FORMAL-002b`: Sibling isolation: revoking node $u$ does NOT affect non-descendant sibling branches.
