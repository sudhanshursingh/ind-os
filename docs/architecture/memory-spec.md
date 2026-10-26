# IndK Memory Discovery & Virtual Memory Architecture Specification
**Document ID:** `INDOS-SPEC-MEM-01`  
**Phase Target:** Phase 1 & 2 (SP-06, SP-07, SP-10) $\rightarrow$ Gates G1, G2  
**Requirement Mapping:** `REQ-KRN-102`, `REQ-KRN-103`  
**Platform:** ARM64 (48-bit Virtual Addressing, 4 KB Page Granularity)

---

## 1. Scope & Objectives

This specification defines physical memory discovery from the Device Tree, the early page frame allocator (bitmap / buddy system), kernel heap architecture, and the two-level virtual address space layout (Kernel / User split).

---

## 2. Memory Map & Address Space Layout

IndK enforces a strict **Higher-Half Kernel** design:
- **User Address Space:** `0x0000_0000_0000_0000` to `0x0000_7FFF_FFFF_FFFF` (Lower 128 TB, governed by `TTBR0_EL1`).
- **Kernel Address Space:** `0xFFFF_8000_0000_0000` to `0xFFFF_FFFF_FFFF_FFFF` (Upper 128 TB, governed by `TTBR1_EL1`).

```text
0xFFFF_FFFF_FFFF_FFFF ┌─────────────────────────────────────────┐
                      │ High Fixmaps & Core Peripherals (UART,GIC)│
0xFFFF_FFFF_8000_0000 ├─────────────────────────────────────────┤
                      │ Kernel Dynamic Heap (Intrusive Allocator) │
0xFFFF_8000_4000_0000 ├─────────────────────────────────────────┤
                      │ Kernel Text, Rodata, BSS, Stack         │
0xFFFF_8000_0000_0000 ├─────────────────────────────────────────┤
                      │ Physical Memory Direct Map (PhysLinear) │
                      └─────────────────────────────────────────┘
                      ===========================================
                      GUARD REGION (Unmapped Fault Zone)
                      ===========================================
0x0000_7FFF_FFFF_FFFF ┌─────────────────────────────────────────┐
                      │ User Stack (grows down, Guard Page)     │
                      ├─────────────────────────────────────────┤
                      │ User Heap / Anonymous Mappings          │
                      ├─────────────────────────────────────────┤
                      │ User Binary Text & Data (ELF Segments)   │
0x0000_0000_0040_0000 ├─────────────────────────────────────────┤
                      │ NULL-Pointer Trap Zone (0 to 4MB)       │
0x0000_0000_0000_0000 └─────────────────────────────────────────┘
```

---

## 3. Page Frame Allocator & Kernel Heap

1. **Early Frame Allocator (`BootBitmapAllocator`):**
   - Discovers physical RAM bounds by parsing `/memory@.../reg` in the DTB.
   - Represents physical memory as a 4 KB page bitmask.
   - Marks reserved DTB, kernel ELF segments, and memory-mapped I/O regions as unavailable.
2. **Kernel Heap Allocator (`KernelHeap`):**
   - Slab / Intrusive linked-list allocator managing kernel heap space.
   - Enforces zero allocation in interrupt context.
   - Every physical frame allocation failure returns a typed `MemoryError::OutOfMemory`, never an unchecked null pointer.

---

## 4. Invariants (Machine-Verifiable)

- **INV-MEM-001:** Physical address `0x0` and virtual address `0x0` are **unmapped** (null dereference causes an immediate page fault exception).
- **INV-MEM-002:** User-space code cannot execute kernel address space pages, and kernel code cannot execute user-space pages (`PXN` and `UXN` bits enforced).
- **INV-MEM-003:** Physical page allocations are identity-audited: double-free or freeing an unallocated page triggers an immediate kernel panic.
- **INV-MEM-004:** Kernel page tables are strictly marked `Read-Only` after initial boot mapping configuration.
