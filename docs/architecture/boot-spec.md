# IndK Boot Entry & Reset Architecture Specification
**Document ID:** `INDOS-SPEC-BOOT-01`  
**Phase Target:** Phase 1 (SP-04 – SP-09) $\rightarrow$ Gate G1  
**Requirement Mapping:** `REQ-KRN-101`, `REQ-KRN-102`  
**Platform:** ARM64 QEMU `virt` (Cortex-A53/A57/A72, EL1/EL2) $\rightarrow$ Qualcomm Snapdragon 732G (`hanoip`)

---

## 1. Scope & Objectives

This specification defines the machine-level entry point, Exception Level (EL) handling, early architectural initialization, vector table installation, and Device Tree Blob (DTB) handoff for the from-scratch Ind OS Kernel (`IndK`).

In accordance with **Rule N-01 (From scratch means from scratch)** and **DEC-001**, all boot routines are written in clean ARM64 assembly (`boot.s`) without external C shims or foreign bootloader code.

---

## 2. Machine State at Entry (Direct QEMU `-kernel` Path)

Per the ratified decision in `docs/architecture/qemu-boot-path-spike.md`:
1. **Entry Address:** `0x40080000` (physical RAM offset on QEMU `virt`).
2. **Registers on Entry:**
   - `x0`: Physical memory address of the system Device Tree Blob (`dtb_addr`).
   - `x1`, `x2`, `x3`: Reserved by architecture protocol (set to 0 by loader).
   - `pc`: Set to `_start` symbol (`kernel/arch/aarch64/boot.s`).
   - `sp`: Uninitialized (must be set to kernel early stack immediately).
3. **Paging / MMU:** Disabled (identity mapped flat physical memory).
4. **Caches:** Instruction/Data caches disabled.

---

## 3. Exception Level (EL) Transition & Vector Table

IndK must operate securely in **EL1** (Kernel mode), supporting transitions from **EL2** (Hypervisor mode) if booted under virtualization:

```text
       +-----------------------+
       |   Reset / Boot Entry  | (EL2 or EL1)
       +-----------------------+
                   |
           CurrentEL check
                   |
         +---------+---------+
         |                   |
      (Is EL2)            (Is EL1)
         |                   |
  Configure HCR_EL2          |
  (EL1 AArch64 execution)    |
  Drop to EL1 via ERET       |
         |                   |
         +---------> <-------+
                   |
       +-----------------------+
       |  Set SP_EL1 to Stack  |
       +-----------------------+
                   |
       +-----------------------+
       |  Install VBAR_EL1     | (Exception Vector Table)
       +-----------------------+
                   |
       +-----------------------+
       |  Validate DTB & Jump  | (Call kernel_main(dtb_ptr))
       +-----------------------+
```

### 3.1 Vector Table Layout (`VBAR_EL1`)
The vector table consists of 16 entries (each aligned to 128 bytes, total size 2,048 bytes):
- **Current EL with SP0:** Synchronous, IRQ, FIQ, SError.
- **Current EL with SPx:** Synchronous, IRQ, FIQ, SError.
- **Lower EL using AArch64:** Synchronous (Syscalls), IRQ, FIQ, SError.
- **Lower EL using AArch32:** Synchronous, IRQ, FIQ, SError.

---

## 4. Invariants (Machine-Verifiable)

- **INV-BOOT-001:** `IndK` executes in **EL1** within $\le 50$ instructions of entry.
- **INV-BOOT-002:** The early stack pointer `sp` is strictly aligned to a 16-byte boundary.
- **INV-BOOT-003:** Device Tree Blob pointer `x0` is preserved across all EL transitions and validated against the DTB magic constant `0xd00dfeed`.
- **INV-BOOT-004:** Any unexpected synchronous exception or SError during early boot triggers a structured panic with CPU register dumps printed to the early PL011 UART.
