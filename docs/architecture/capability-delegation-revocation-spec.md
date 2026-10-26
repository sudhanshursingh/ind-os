# Ind OS Capability Delegation, Revocation Propagation & Leak Prevention Specification (SP-18)

**Document ID:** `INDOS-SPEC-CAP-03`  
**Governing Standard:** `Ind_OS_Development_Bible_v1.0.0.0.md` (§8.9, §10, §14), `docs/architecture/phase-3-security-spine-roadmap.md` (§2.2, §3)  
**TCB Subsystems:** `kernel/src/capability/`, `kernel/src/object/`, `kernel/src/task/`  
**Review Status:** RATIFIED (Phase 3 Security Spine Baseline)  

---

## 1. Architectural Scope & Objectives

Sprint SP-18 advances the Ind OS capability subsystem from isolated per-process namespaces (SP-17) to multi-hop cross-process delegation chains, global revocation propagation, and comprehensive capability-leak prevention.

Under **Phase 3 Watchpoint 2 (§2.2)** and the core security requirements (`REQ-KRN-104`, `REQ-KRN-105`, `REQ-SEC-106`, `REQ-SEC-107`):
1. **Multi-Hop Delegation with Attenuation:**
   - Authority is passed across arbitrary multi-process chains ($P_1 \to P_2 \to P_3 \to \dots \to P_n$).
   - At each delegation boundary, capability rights shrink strictly monotonically ($R_{i+1} \subseteq R_i$). Rights expansion is rejected with `EPERM` ($O(1)$ verification).
   - Delegation requires the possession of `CapRights::GRANT` on the source capability. Presenting a capability without `GRANT` fails with `EPERM`.
2. **Global Cross-Process Revocation Propagation:**
   - Capabilities maintain cross-namespace lineage tracking pointers (`parent_pid`, `parent_handle`).
   - Revoking a capability in any namespace triggers a global transitive cascade that identifies and revokes all descendant tokens across all process namespaces in the system.
   - Non-recursive, bounded worklist traversal using stack-allocated bitmasks (`[u64; MAX_NAMESPACES]` = 128 bytes) eliminates recursion stack exhaustion and limits latency during mass revocation.
3. **Object-Table Reference Coherence & Anti-ABA Reclamation:**
   - Every delegation or derivation retaining a typed `KObjectId` increments the object's reference counter in `KObjectTable`.
   - Global revocation decrements reference counts for every invalidated slot.
   - Upon zero remaining capability references, the kernel object is deterministically destroyed and its generation counter incremented to prevent ABA reuse attacks (`INV-OBJ-001`).
4. **Capability-Leak Prevention & Process Termination:**
   - Destruction of a process namespace (`destroy_namespace`) triggers transitive global revocation of all capability trees rooted in that process, cleanly releasing all child delegations held by downstream processes.
   - Zero-leak invariant (`INV-CAP-LEAK-001`): active capability tokens and active kernel objects deterministically return to baseline upon cleanup.

---

## 2. Formal Machine Invariants

1. **`INV-CAP-001` (Monotonic Attenuation along Delegation Chains):**
   For any delegation sequence $C_0 \to C_1 \to \dots \to C_k$:
   $$\forall i \in \{0, \dots, k-1\} : (R_{C_{i+1}} \ \& \ \sim R_{C_i}) = 0 \quad \land \quad \text{GRANT} \in R_{C_i}$$
   Rights expansion or delegation without `GRANT` is rejected with `Err(EPERM)`.

2. **`INV-CAP-002` (Global Revocation Soundness & Propagation):**
   Let $\mathcal{D}(C)$ be the transitive set of all descendant capabilities derived or delegated from $C$ across all process namespaces. Revoking $C$ guarantees:
   $$\forall C' \in \{C\} \cup \mathcal{D}(C) : \text{is\_valid}(C') = \text{false} \quad \land \quad \text{lookup}(C') = \text{Err(EBADF)}$$
   Ancestors and unrelated branches remain untouched.

3. **`INV-CAP-003` (Object-Table Reference Coherence):**
   For any kernel object $O$:
   $$\text{ref\_count}(O) = \sum_{P \in \text{Processes}} \sum_{h \in \text{CSpace}(P)} \mathbf{1}_{\{ \text{target}(h) = O \}}$$
   Revocation decrements $\text{ref\_count}(O)$ for every invalidated slot; $\text{ref\_count}(O) = 0 \implies \text{destroy}(O)$.

4. **`INV-CAP-LEAK-001` (Zero Capability & Object Leakage):**
   Following the revocation of a delegation tree or termination of participating processes:
   $$\Delta N_{\text{caps}} = 0 \quad \land \quad \Delta N_{\text{objects}} = 0$$
   No orphan slots, dangling references, or uncollected object descriptors remain in the system.

---

## 3. Delegation Lineage & Worklist Bitmask Architecture

### 3.1 Lineage Fields in `Capability`

```text
+---------------------------------------------------------------+
| cap_type: CapType                                             |
| rights: CapRights                                             |
| badge: u64                                                    |
| is_valid: bool                                                |
| parent_slot: Option<u32>        (intra-table parent handle)   |
| parent_pid: Option<ProcessId>   (cross-namespace parent PID)  |
| parent_handle: Option<CapHandle>(cross-namespace parent handle)|
+---------------------------------------------------------------+
```

### 3.2 Bounded Non-Recursive Worklist Traversal

```text
                   NamespaceRegistry::revoke(pid, handle)
                                     │
                 ┌───────────────────┴───────────────────┐
                 │ to_revoke: [u64; 16] = 128 bytes      │
                 │ to_revoke[root_ns_idx] |= (1 << root) │
                 └───────────────────┬───────────────────┘
                                     │
                         ┌───────────▼───────────┐
                         │   Fixed-Point Loop    │◄────────────────┐
                         │ Scan all active slots │                 │
                         └───────────┬───────────┘                 │
                                     │                             │
                     Parent marked in to_revoke?                   │
                                   ┌─┴─┐                           │
                               YES │   │ NO                        │
                                   ▼   │                           │
                   to_revoke[ns] |= (1 << h)                       │
                   cascade = true; ────┘                           │
                                     │                             │
                         cascade == true? ─────────────────────────┘
                                     │ FALSE
                         ┌───────────▼───────────┐
                         │   Invalidate Slots    │
                         │ Release KObject Refs  │
                         └───────────────────────┘
```

---

## 4. Security Audit & Zero-Unsafe Budget

In compliance with Development Bible §24.1:
* The capability delegation, attenuation, and revocation propagation subsystems in `kernel/src/capability/` contain **0 `unsafe` blocks** (100% pure safe Rust).
* Cross-process lineage traversal operates on statically bounded tables (`MAX_NAMESPACES = 16`, `MAX_CAPABILITIES = 64`) with compile-time bounded execution loops.
