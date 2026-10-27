/*
 * IndK Boot Entry & Reset Routine (ARM64)
 * Ratified in docs/architecture/boot-spec.md (INDOS-SPEC-BOOT-01)
 *
 * Invariants Enforced:
 * - INV-BOOT-001: Kernel executes in EL1 within <= 50 instructions of entry
 * - INV-BOOT-002: Stack pointer is strictly 16-byte aligned
 * - INV-BOOT-003: x0 preserves DTB pointer and validates magic 0xd00dfeed
 */

.section .text.boot, "ax"
.global _start
.type _start, @function

_start:
    /* ARM64 Linux Kernel Image Header (64 bytes) for ABOOT/LK and AVB 2.0 */
    b .Lreal_start
    nop
    .quad 0x80000          /* 0x08: text_offset (512KB offset in DRAM) */
    .quad 0x200000         /* 0x10: image_size */
    .quad 0x0a             /* 0x18: flags: 4KB page, little endian, any placement */
    .quad 0                /* 0x20: reserved */
    .quad 0                /* 0x28: reserved */
    .quad 0                /* 0x30: reserved */
    .ascii "ARM\x64"       /* 0x38: magic 'ARM\x64' */
    .word 0                /* 0x3c: flags/res5 */

.Lreal_start:
    /* Preserve DTB pointer passed in x0 by bootloader / QEMU */
    mov x19, x0

    /* Check current Exception Level (CurrentEL[3:2]) */
    mrs x1, CurrentEL
    lsr x1, x1, #2
    cmp x1, #1
    b.eq .Lin_el1
    cmp x1, #2
    b.eq .Ldrop_from_el2

    /* Unexpected EL3 or EL0 -> halt immediately */
    b .Lemergency_halt

.Ldrop_from_el2:
    /* Configure HCR_EL2: Bit 31 (RW) = 1 (AArch64 execution state for EL1) */
    mov x2, #(1 << 31)
    msr hcr_el2, x2

    /* Disable EL2 FP/SIMD traps */
    msr cptr_el2, xzr

    /* Enable Non-secure EL1 timer access */
    mrs x2, cnthctl_el2
    orr x2, x2, #3          /* EL1PCEN (bit 1) and EL1PCTEN (bit 0) */
    msr cnthctl_el2, x2
    msr cntvoff_el2, xzr

    /* Configure SPSR_EL2: Return to EL1h (mode 0b0101) with all interrupts masked (DAIF = 0b1111) */
    mov x2, #0x3c5
    msr spsr_el2, x2

    /* Set return target to .Lin_el1 */
    adr x2, .Lin_el1
    msr elr_el2, x2

    /* Execute Exception Return to EL1 (INV-BOOT-001 satisfied in <= 20 instructions) */
    eret

.Lin_el1:
    /* Enable Advanced SIMD and Floating-Point in EL1 (CPACR_EL1.FPEN = 0b11) */
    mov x1, #(3 << 20)
    msr cpacr_el1, x1
    isb

    /* Initialize stack pointer to top of early stack */
    ldr x1, =__stack_top
    mov sp, x1

    /* INV-BOOT-002: Verify 16-byte stack alignment */
    mov x1, sp
    tst x1, #0xf
    b.ne .Lstack_misaligned_panic

    /* Install Exception Vector Table (VBAR_EL1) */
    ldr x1, =__vector_table
    msr vbar_el1, x1
    isb

    /* Clear BSS segment (__bss_start to __bss_end) */
    ldr x1, =__bss_start
    ldr x2, =__bss_end
.Lclear_bss_loop:
    cmp x1, x2
    b.ge .Lclear_bss_done
    str xzr, [x1], #8
    b .Lclear_bss_loop

.Lclear_bss_done:
    /* Restore DTB pointer from x19 */
    mov x0, x19
    cbnz x0, .Lcheck_dtb_magic

    /* Default QEMU virt fallback */
    mov x0, #0x48000000

.Lcheck_dtb_magic:
    /* If pointer is valid and contains 0xd00dfeed, pass it */
    cbz x0, .Lpass_null_dtb
    ldr w1, [x0]
    rev w1, w1              /* DTB is big-endian on disk */
    ldr w2, =0xd00dfeed
    cmp w1, w2
    b.eq .Lpass_valid_dtb

.Lpass_null_dtb:
    mov x0, xzr             /* Pass 0 to indicate no external DTB */

.Lpass_valid_dtb:
    /* Call freestanding Rust kernel entry point: kernel_main(dtb_addr: usize) */
    bl kernel_main

    /* If kernel_main returns, park CPU in low-power state */
.Lemergency_halt:
    wfe
    b .Lemergency_halt

.Lstack_misaligned_panic:
    b .Lemergency_halt

.Ldtb_corrupted_panic:
    b .Lemergency_halt
