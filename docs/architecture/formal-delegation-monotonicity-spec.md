# Sprint SP-116 Specification: Formal Verification of Capability Delegation Monotonicity

## 1. Context & Objectives
- **Milestone:** Phase 10 — Hardening, Verification, Beta Readiness (Bible §8.17)
- **Sprint:** SP-116 — Formal verification push: Capability delegation monotonicity
- **Primary Requirement:** `REQ-SEC-106` (Formal verification of capability-core invariants)
- **Constitutional Invariant:** `INV-FORMAL-001` (Monotonic Attenuation)

## 2. Mathematical Definition
Let $\mathcal{R}$ be the bounded lattice of capability permission rights $(\mathcal{P}(\text{Rights}), \subseteq, \cap, \cup, \emptyset, \mathcal{U})$.
For any capability node $C_p$ (parent) granting rights to a newly created child node $C_c$:
$$\mathcal{R}(C_c) \subseteq \mathcal{R}(C_p)$$
Rights amplification is strictly prohibited:
$$\forall r \in \mathcal{R}(C_c), \quad r \in \mathcal{R}(C_p)$$
If $r \notin \mathcal{R}(C_p)$, the delegation operation MUST reject with `Err(CapabilityAmplificationViolation)` and abort the transition with zero state change.

## 3. Invariants
- `INV-FORMAL-001`: For any delegation from parent to child, $\mathcal{R}_{\text{child}} \subseteq \mathcal{R}_{\text{parent}}$. No combination of mask, transfer, or derivation operations can grant rights exceeding the parent's current rights.
- `INV-FORMAL-001a`: Capability type identity preservation ($\text{Type}(C_c) == \text{Type}(C_p)$).
- `INV-FORMAL-001b`: Generation monotonically strictly increments ($\text{Gen}(C_c) = \text{Gen}(C_p) + 1$).

## 4. Verification Bounds
- Automated model checks run over $2^{16}$ randomized mask permutations.
- Mathematical inductive invariant: $\forall k \ge 0, \mathcal{R}(C_{k+1}) \subseteq \mathcal{R}(C_k) \implies \mathcal{R}(C_n) \subseteq \mathcal{R}(C_0)$.
