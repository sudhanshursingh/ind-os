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
    cbnz x0, .Lvalidate_dtb

    /* Fallback for direct QEMU virt bare-metal boot: DTB placed at 0x48000000 */
    mov x0, #0x48000000

.Lvalidate_dtb:
    /* INV-BOOT-003: Validate DTB pointer and magic 0xd00dfeed */
    cbz x0, .Ldtb_corrupted_panic
    ldr w1, [x0]
    rev w1, w1              /* DTB is big-endian on disk */
    ldr w2, =0xd00dfeed
    cmp w1, w2
    b.ne .Ldtb_corrupted_panic

    /* Call freestanding Rust kernel entry point: kernel_main(dtb_addr: usize) */
    bl kernel_main

    /* If kernel_main returns, park CPU */
.Lemergency_halt:
    wfe
    b .Lemergency_halt

.Lstack_misaligned_panic:
    /* Emergency UART indicator 'S' (Stack fault) at QEMU PL011 (0x09000000) */
    mov x1, #0x09000000
    mov w2, #'S'
    str w2, [x1]
    mov w2, #'\n'
    str w2, [x1]
    b .Lemergency_halt

.Ldtb_corrupted_panic:
    /* Emergency UART indicator 'D' (DTB fault) at QEMU PL011 (0x09000000) */
    mov x21, #0x09000000
    mov w22, #'D'
    str w22, [x21]
    mov w22, #'['
    str w22, [x21]

    /* Print x0 hex (16 hex digits) */
    mov x23, #60
.Lprint_x0_loop:
    lsr x24, x0, x23
    and x24, x24, #0xf
    cmp x24, #10
    b.lt .Lprint_x0_dec
    add w22, w24, #('a' - 10)
    b .Lprint_x0_emit
.Lprint_x0_dec:
    add w22, w24, #'0'
.Lprint_x0_emit:
    str w22, [x21]
    subs x23, x23, #4
    b.ge .Lprint_x0_loop

    mov w22, #','
    str w22, [x21]

    /* Print w1 hex (8 hex digits) */
    mov x23, #28
.Lprint_w1_loop:
    lsr x24, x1, x23
    and x24, x24, #0xf
    cmp x24, #10
    b.lt .Lprint_w1_dec
    add w22, w24, #('a' - 10)
    b .Lprint_w1_emit
.Lprint_w1_dec:
    add w22, w24, #'0'
.Lprint_w1_emit:
    str w22, [x21]
    subs x23, x23, #4
    b.ge .Lprint_w1_loop

    mov w22, #']'
    str w22, [x21]
    mov w22, #'\n'
    str w22, [x21]
    b .Lemergency_halt
