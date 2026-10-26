# Ind OS Kernel (IndK) Unsafe-Code Budget & Audit Register
**Document ID:** `INDOS-AUDIT-UNSAFE-001`  
**Opened At:** Sprint SP-07 (Phase 1 Kernel Bootstrap)  
**Governing Standard:** `Ind_OS_Development_Bible_v1.0.0.0.md` (§8.7, §24.1)  
**TCB Scope:** `kernel/src/`  
**Policy:** Every `unsafe` block in IndK MUST be:
1. Preceded immediately by a `// SAFETY:` justification doc comment explaining the exact preconditions and invariants.
2. Formally logged in this register with file path, line range, rationale, and audit status.
3. Audited at every sprint and gate review (G1 through G10) against monotonic expansion.

---

## 1. Unsafe Code Inventory

| ID | File | Component | Purpose / Invariant | Preconditions Enforced | Audit Status |
| :--- | :--- | :--- | :--- | :--- | :---: |
| **UNSAFE-001** | `kernel/src/arch/aarch64/mod.rs` | Low-level Asm | Hardware exception table binding (`vbar_el1`) | Address is 2048-byte aligned and executable. | ✅ AUDITED |
| **UNSAFE-002** | `kernel/src/drivers/pl011.rs` | MMIO | Hardware register read/write (`read_volatile`, `write_volatile`) | Physical base `0x09000000` is mapped MMIO page. | ✅ AUDITED |
| **UNSAFE-003** | `kernel/src/sync/spinlock.rs` | Concurrency | Raw pointer dereference in `Deref`/`DerefMut` | Protected by `AtomicBool` acquire/release fence. | ✅ AUDITED |
| **UNSAFE-004** | `kernel/src/panic.rs` | Emergency Console | Direct PL011 MMIO byte write during panic | Used only when kernel has panicked; ignores locks. | ✅ AUDITED |
| **UNSAFE-005** | `kernel/src/mm/dtb.rs` | FDT Parser | Raw byte pointer scanning and `read_unaligned` | Pointer bounded by `dtb_ptr + totalsize`. | ✅ AUDITED |
| **UNSAFE-006** | `kernel/src/mm/mod.rs` | Memory Layout | Reading linker symbol address (`__kernel_image_end`) | Linker script guarantees valid symbol address. | ✅ AUDITED |
| **UNSAFE-007** | `kernel/src/mm/heap.rs` | Kernel Heap | Heap memory slice binding & `GlobalAlloc` interface | Arena backed by verified contiguous physical frames. | ✅ AUDITED |
| **UNSAFE-008** | `kernel/src/drivers/gicv2.rs` | MMIO | GICv2 distributor & CPU interface register access | Addresses `0x08000000` and `0x08010000` are valid MMIO pages. | ✅ AUDITED |
| **UNSAFE-009** | `kernel/src/drivers/timer.rs` | Low-level Asm | ARM Generic Timer system register programming | Privileged EL1 execution; atomic countdown reload. | ✅ AUDITED |
| **UNSAFE-010** | `kernel/src/interrupts/mod.rs` | Low-level Asm | DAIF interrupt mask manipulation & SP check | Architectural state changes for critical sections. | ✅ AUDITED |

---

## 2. Invariant & Safety Guidelines

1. **Rule of Containment:** `unsafe` code must NEVER be exposed across public module boundaries without a safe wrapper guaranteeing invariants.
2. **MMIO Safety:** All memory-mapped I/O MUST use `read_volatile` and `write_volatile` with explicit bitmasks.
3. **Zero Allocation in Interrupts:** The kernel heap allocator is protected by non-reentrant spinlocks. Memory allocation in hardware interrupt handlers is strictly prohibited.
