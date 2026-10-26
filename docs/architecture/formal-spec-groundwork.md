# Ind OS Architecture Specification: Phase 3 Formal-Spec Groundwork

**Document Identifier:** `INDOS-SPEC-FORMAL-01`  
**Classification:** Core Kernel Architecture / Security Spine Formal Foundations  
**Status:** Approved & Machine-Checkable Groundwork (Sprint SP-23 Prerequisite)  
**Governing Standard:** Ind OS Development Bible v1.0.0.0 (§8.9, §10, §14)  
**Target Gate:** Gate G3 Hard-Stop Readiness  

---

## 1. Executive Summary

This specification establishes the mathematical foundations and machine-checkable invariant models for the Ind OS Phase 3 Security Spine. As mandated by the Development Bible (§8.9) and Gate G3 criteria (§19.2), the microkernel's security properties must be rigorously defined through formal predicate logic to support machine verification alongside runtime test evidence.

The formal model captures the six foundational invariants of Phase 3:
1. **`INV-CAP-001`**: Monotonic Rights Attenuation
2. **`INV-CAP-002`**: Revocation Soundness & Lineage Preservation
3. **`INV-SEC-001`**: Zero Ambient Authority
4. **`INV-IPC-001` / `INV-IPC-002`**: Zero-Copy Rendezvous & Deterministic Latency
5. **`INV-AUD-001`**: Cryptographic Append-Only Audit Integrity
6. **`INV-CON-001`**: Unspoofable Consent Surface & Input Mediation

---

## 2. Invariant Formal Definitions

### 2.1 `INV-CAP-001`: Monotonic Rights Attenuation
Let $\mathcal{R} = \{ \text{READ}, \text{WRITE}, \text{EXECUTE}, \text{GRANT}, \text{TRANSFER} \} \subset \mathcal{P}(\text{Rights})$.  
Let $c = (T, R, B, \text{pid}, s)$ be a capability token with rights set $R \subseteq \mathcal{R}$.  
When process $p$ mints or derives child token $c'$ from parent $c$, the derivation relation $\text{Attenuate}(c, c')$ satisfies:

$$\forall c, c' : \text{Derive}(c \to c') \implies (c'.R \subseteq c.R) \iff (c'.R \ \& \ \sim c.R) = \emptyset$$

**Lemma 1.1 (Non-Escalation).** No transitive delegation chain $c_0 \to c_1 \to \dots \to c_k$ can possess authority not present in the root capability:
$$\forall k \ge 0, \quad c_k.R \subseteq c_0.R$$

---

### 2.2 `INV-CAP-002`: Revocation Soundness
Let $\mathcal{C}$ be the universe of all allocated capability slots across all process namespaces $\mathcal{N}_p$.  
Let $\text{Descendants}(c) = \{ c' \in \mathcal{C} \mid c \rightsquigarrow c' \}$ be the transitive closure of delegation lineage from $c$.  
The revocation operation $\text{Revoke}(c)$ satisfies:

$$\text{Revoke}(c) \implies \forall c' \in (\{c\} \cup \text{Descendants}(c)) : \text{Valid}(c') = \text{false} \land \text{RefCount}(\text{Target}(c')) \leftarrow \text{RefCount}(\text{Target}(c')) - 1$$

**Lemma 2.1 (No Dangling References).** For any revoked slot $c$, access attempts deterministically yield `EBADF` or `EPERM`:
$$\forall c' \in \text{Descendants}(c), \quad \text{Access}(c') = \bot$$

---

### 2.3 `INV-SEC-001`: Zero Ambient Authority
Let $\Sigma$ be the kernel state, and $\mathcal{A}$ be the set of system call operations.  
A transition $\Sigma \xrightarrow{\text{syscall}(p, op, \text{args})} \Sigma'$ is valid if and only if an explicit, valid capability handle $h$ is presented that authorizes $op$:

$$\text{ValidTransition}(p, op, \text{args}) \iff \exists h \in \mathcal{N}_p : \text{Valid}(h) \land \text{Permits}(h, op, \text{args})$$

$$\forall p \in \text{Processes}, \quad \text{AmbientAuthority}(p) = \emptyset$$

---

### 2.4 `INV-AUD-001`: Cryptographic Append-Only Audit Integrity
Let $\mathcal{E} = [E_0, E_1, \dots, E_N]$ be the sequence of recorded audit events.  
Let $\mathcal{H}(m)$ be the NIST FIPS 180-4 SHA-256 compression function.  
The audit log state satisfies:

$$E_0.\text{prev\_hash} = \mathbf{0}^{32}, \quad E_0.\text{entry\_hash} = \mathcal{H}(\text{PreImage}(E_0))$$

$$\forall i \in [1, N], \quad E_i.\text{prev\_hash} = E_{i-1}.\text{entry\_hash} \land E_i.\text{entry\_hash} = \mathcal{H}(\text{PreImage}(E_i))$$

**Lemma 4.1 (Tamper-Evidence).** If any byte $b \in E_j$ ($j \le N$) is mutated to $b'$, the set of valid downstream hashes is destroyed:
$$E_j \ne E'_j \implies \exists k \ge j : E_k.\text{prev\_hash} \ne E_{k-1}.\text{entry\_hash} \lor E_k.\text{entry\_hash} \ne \mathcal{H}(E'_k)$$

---

### 2.5 `INV-CON-001`: Unspoofable Consent Surface
Let $\mathcal{S}$ be the display surface state, and $\mathcal{I}$ be the hardware input routing state.  
When a consent prompt $P$ is active ($\text{PromptActive} = \text{true}$):

$$\mathcal{S}.\text{active\_plane} = \text{PlaneId::HardwareIsolated} \land \mathcal{I}.\text{exclusive\_owner} = \text{KernelPID} (0)$$

$$\forall p \ne 0 : \text{DrawOverlay}(p) = \text{Err}(\text{EPERM}) \land \text{InjectInput}(p) = \text{Err}(\text{EPERM})$$

---

## 3. Machine-Checkable Verification Strategy

1. **Pre-Mint Attenuation Predicate:** Formally integrated into [`kernel/src/capability/rights.rs`](file:///Users/dhan/IndOS/kernel/src/capability/rights.rs) (`can_attenuate_to`).
2. **Lineage Cascading Engine:** Formally integrated into [`kernel/src/capability/namespace.rs`](file:///Users/dhan/IndOS/kernel/src/capability/namespace.rs) (`revoke_child_lineage`).
3. **AST Static AST Scanners:** Verifying zero raw ambient syscalls via [`ci/check_zero_ambient_authority.py`](file:///Users/dhan/IndOS/ci/check_zero_ambient_authority.py).
4. **Mathematical Model Check:** Machine-checked in [`ci/verify_sp23.py`](file:///Users/dhan/IndOS/ci/verify_sp23.py) during Gate G3 assembly.
