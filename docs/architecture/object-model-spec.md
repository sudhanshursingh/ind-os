# Ind OS Kernel Object Model Specification (SP-14)

**Document ID:** `INDOS-SPEC-OBJ-01`  
**Governing Standard:** `Ind_OS_Development_Bible_v1.0.0.0.md` (§8.8, §14)  
**TCB Subsystems:** `kernel/src/object/`, `kernel/src/capability/`  
**Review Status:** RATIFIED (Phase 2 Core Kernel Sprint SP-14 Baseline)  

---

## 1. Architectural Overview & Objectives

In Ind OS, all privileged state and shared hardware/system primitives are encapsulated within first-class **Kernel Objects** managed by a centralized, typed **Kernel Object Table** (`KObjectTable`). In strict compliance with:
* **Rule N-04 (Security is enforced, never requested):** User applications and subsystems hold zero direct pointers to kernel data structures.
* **Rule N-05 (Zero ambient authority):** All interactions with kernel objects are mediated through unforgeable, capability-based handles referencing validated entries in `KObjectTable`.

The object model defines the foundational objects that Phase 3 capabilities will reference:
1. **Process Object (`ProcessObject`):** Represents an address-space-isolated execution domain, its lifecycle state, scheduling priority, and CPU statistics.
2. **Endpoint Object (`EndpointObject`):** Represents a synchronous IPC communication channel supporting zero-copy rendezvous message passing and capability delegation.
3. **Memory Region Object (`MemoryRegionObject`):** Represents a physical memory aperture, cache attributes (Normal Cacheable, Device MMIO), page size, and access bounds.
4. **IRQ Object (`IrqObject`):** Represents a hardware interrupt line (SPI/PPI) bound to a user-mode driver or notification endpoint.

---

## 2. Machine Invariants

1. **`INV-OBJ-001` (Generation Soundness & Anti-ABA):**
   Every entry in `KObjectTable` pairs an index with a monotonic 16-bit generation counter. When an object is reclaimed, the generation increments. Stale capability handles pointing to a reused slot with an outdated generation are rejected with `EBADF`, preventing use-after-free and ABA aliasing.
   $$\text{lookup}(id) \implies (id.\text{generation} == \text{table}[id.\text{index}].\text{generation}) \lor \text{Err}(\text{EBADF})$$

2. **`INV-OBJ-002` (Reference Count Soundness):**
   Objects maintain a non-negative reference count (`ref_count`). An object remains alive as long as $\text{ref\_count} > 0$. When $\text{ref\_count}$ transitions to 0 via `release()`, the object is deterministically reclaimed and its slot generation incremented.

3. **`INV-OBJ-003` (Type Isolation & Strong Typing):**
   Objects are tagged with an explicit `KObjectType`. Casting or invoking an object under an incompatible type is strictly rejected with `EINVAL`.

4. **`INV-OBJ-004` (Deterministic Bounded Allocation):**
   The Object Table is pre-allocated with a fixed maximum capacity (`MAX_KOBJECTS = 128`), ensuring $O(1)$ allocation/deallocation latency and zero kernel heap exhaustion.

---

## 3. Object Identifier (`KObjectId`) Structure

An unforgeable 32-bit compact token representing an object reference:

```text
 31             24 23             16 15                           0
+-----------------+-----------------+-------------------------------+
|  Type (8 bits)  | Gen (8 bits)    |      Table Index (16 bits)    |
+-----------------+-----------------+-------------------------------+
```

* `index` (`u16`): Offset into `KObjectTable.slots` ($0 \le \text{index} < \text{MAX\_KOBJECTS}$).
* `generation` (`u16`): Current lifecycle generation of the slot.
* `obj_type` (`KObjectType`): Tagged object type (`Process`, `Endpoint`, `MemoryRegion`, `Irq`).

---

## 4. Kernel Object Definitions

### 4.1 Process Object (`ProcessObject`)
* `pid: ProcessId` — Unique process identifier.
* `name: &'static str` — Process display name.
* `priority: Priority` — Multi-level scheduler priority class.
* `state: ProcessState` — Execution lifecycle state.
* `ttbr0: usize` — Physical address of process L0 page table root.

### 4.2 Endpoint Object (`EndpointObject`)
* `ep_id: u32` — IPC endpoint identifier.
* `state: EndpointState` — Synchronous rendezvous state (`Idle`, `SenderWaiting`, `ReceiverWaiting`).
* `messages_transferred: u64` — Throughput telemetry.

### 4.3 Memory Region Object (`MemoryRegionObject`)
* `base: usize` — Physical base address.
* `size: usize` — Size in bytes (must be 4KB frame-aligned).
* `is_device: bool` — Device MMIO (`nGnRE`) vs. Normal Cacheable RAM.
* `is_writable: bool` — Write permission flag.
* `is_executable: bool` — Execution permission flag (UXN/PXN).

### 4.4 IRQ Object (`IrqObject`)
* `irq_num: u32` — Hardware interrupt vector (GICv2 SPI/PPI).
* `is_edge: bool` — Edge-triggered vs. Level-triggered.
* `is_masked: bool` — Interrupt mask state.
* `target_pid: Option<ProcessId>` — Bound recipient process for userspace driver delivery.

---

## 5. Object Table Operations (`KObjectTable`)

| Method | Signature | Description |
| :--- | :--- | :--- |
| `create_process` | `(pid, name, prio, ttbr0) -> Result<KObjectId, i64>` | Allocates slot, initializes `ProcessObject`, sets `ref_count = 1`. |
| `create_endpoint` | `(ep_id) -> Result<KObjectId, i64>` | Allocates slot, initializes `EndpointObject`, sets `ref_count = 1`. |
| `create_memory_region` | `(base, size, dev, wr, exec) -> Result<KObjectId, i64>` | Allocates slot, initializes `MemoryRegionObject`, sets `ref_count = 1`. |
| `create_irq` | `(irq_num, is_edge) -> Result<KObjectId, i64>` | Allocates slot, initializes `IrqObject`, sets `ref_count = 1`. |
| `retain` | `(id: KObjectId) -> Result<(), i64>` | Increments `ref_count` if `id` generation matches. |
| `release` | `(id: KObjectId) -> Result<bool, i64>` | Decrements `ref_count`; if 0, reclaims slot and increments generation. |
| `get` | `(id: KObjectId) -> Result<&KObject, i64>` | Type-checked object resolution verifying generation. |
| `get_mut` | `(id: KObjectId) -> Result<&mut KObject, i64>` | Mutable type-checked resolution verifying generation. |
