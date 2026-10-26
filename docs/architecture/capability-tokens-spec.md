# Ind OS Capability Tokens & Namespace Specification (SP-17)

**Document ID:** `INDOS-SPEC-CAP-02`  
**Governing Standard:** `Ind_OS_Development_Bible_v1.0.0.0.md` (§8.9, §10, §14), `docs/architecture/phase-3-security-spine-roadmap.md`  
**TCB Subsystems:** `kernel/src/capability/`, `kernel/src/object/`, `kernel/src/task/`  
**Review Status:** RATIFIED (Phase 3 Security Spine Baseline)  

---

## 1. Architectural Overview & Objectives

In Ind OS, capabilities are the sole, unforgeable currency of authority. In strict compliance with:
* **Rule N-04 (Security is enforced, never requested):** Access to any kernel resource requires presenting a capability with the required `CapRights`.
* **Rule N-05 (Zero ambient authority):** All hardware, memory, and synchronization primitives are accessed exclusively through capabilities; raw pointers or integer IDs from userspace are treated as untrusted and unprivileged.

Sprint SP-17 establishes:
1. **Unforgeable Token Architecture:** Capabilities encapsulate typed references to kernel objects (`KObjectId`), explicit rights bitflags (`CapRights`), and unforgeable 64-bit badges.
2. **Object-Table Integration:** A bidirectional binding between `CapTable` / `CapNamespace` and `KObjectTable`. Every capability slot referencing an object increments that object's `ref_count`; revocation or destruction of capability slots releases object references, ensuring zero memory leaks and deterministic Anti-ABA generation increments.
3. **Per-Process Capability Namespaces (`CapNamespace` / CSpace):** Process authority is strictly partitioned into distinct CSpaces. Capability handles (`CapHandle`) are process-local indices ($0 \le \text{handle} < 64$), preventing cross-process handle confusion.
4. **Mint, Grant, and Revoke Operations:**
   * **Mint:** Derives a child token within the same or target namespace with attenuated rights.
   * **Grant:** Cross-process capability transfer during IPC rendezvous.
   * **Revoke:** Cascading recursive invalidation of a capability and all descendant tokens derived from it.

---

## 2. Machine Invariants

1. **`INV-CAP-001` (Monotonic Attenuation):**
   Derived capability rights must strictly be a subset of the source capability's rights:
   $$\forall R_{\text{child}}, R_{\text{parent}} : (R_{\text{child}} \ \& \ \sim R_{\text{parent}}) = 0$$
   Any attempt to expand authority during minting or delegation is rejected with `Err(EPERM)`.

2. **`INV-CAP-002` (Revocation Soundness):**
   Revoking a capability invalidates its slot and cascades recursively to all transitive child and grandchild capabilities across delegation trees.

3. **`INV-CAP-003` (Object-Table Reference Coherence):**
   The reference count of any `KObject` in `KObjectTable` must strictly equal the number of active capabilities referencing that object across all namespaces:
   $$\text{ref\_count}(O) = \sum_{P \in \text{Processes}} \text{count}(P.\text{cspace}, O)$$

4. **`INV-CAP-004` (Namespace Isolation):**
   Capability handles are strictly process-local integer indices. Attempting to present a foreign process's handle without an explicit cross-process delegation returns `Err(EBADF)`.

---

## 3. Data Structures & Namespace Layout

### 3.1 Capability Token (`Capability`)

```text
+---------------------------------------------------------------+
| cap_type: CapType                                             |
|   - Object(KObjectId)                                         |
|   - Endpoint(u32)                                             |
|   - Notification(u32)                                         |
|   - Process(ProcessId)                                        |
|   - MemoryRegion { base: usize, size: usize }                 |
|   - SyscallBitmap(u64)                                        |
+---------------------------------------------------------------+
| rights: CapRights (READ, WRITE, GRANT, CALL)                  |
+---------------------------------------------------------------+
| badge: u64 (unforgeable capability identifier / token)        |
+---------------------------------------------------------------+
| is_valid: bool                                                |
+---------------------------------------------------------------+
| parent_slot: Option<CapHandle> (lineage tracking for revoke)   |
+---------------------------------------------------------------+
```

### 3.2 Capability Namespace (`CapNamespace`)

Each user process owns a dedicated `CapNamespace`:
* `process_id: ProcessId` — Owner process ID.
* `table: CapTable` — Bounded array of 64 capability slots.
* `badge_mask: u64` — Badge attenuation mask.

---

## 4. Lifecycle Operations

| Operation | Signature | Description |
| :--- | :--- | :--- |
| `mint_object` | `(src_handle, target_rights, badge, &mut KObjectTable)` | Mints derived capability, attenuates rights, retains `KObject` (`ref_count += 1`). |
| `revoke` | `(handle, &mut KObjectTable)` | Recursively invalidates capability tree, calls `table.release()` for all revoked object capabilities (`ref_count -= 1`). |
| `delegate` | `(src_handle, target_rights, target_ns, &mut KObjectTable)` | Transfers capability to another namespace with attenuated rights, retaining underlying object. |
| `lookup` | `(handle)` | Validates handle within calling process's namespace, returning typed reference. |
