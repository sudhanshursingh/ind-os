/*
 * IndK Exception Vector Table & Low-Level Context Preservation (ARM64)
 * Ratified in docs/architecture/boot-spec.md & interrupt-spec.md
 *
 * Requirements:
 * - VBAR_EL1 must be aligned to 2,048 bytes (.align 11)
 * - Each vector entry is spaced by exactly 128 bytes (.align 7, offset 0x80 * i)
 * - Preserves all 31 general-purpose registers + system registers
 * - Calls Rust handle_early_exception(&ExceptionContext)
 */

.section .vectors, "ax"
.global __vector_table
.type __vector_table, @function

.align 11
__vector_table:

/* ----------------------------------------------------------------
 * 1. Current EL with SP0 (0x000 - 0x180)
 * ---------------------------------------------------------------- */
.align 7
vec_curr_el_sp0_sync:
    mov x0, #0x00
    b common_exception_entry

.align 7
vec_curr_el_sp0_irq:
    mov x0, #0x01
    b common_exception_entry

.align 7
vec_curr_el_sp0_fiq:
    mov x0, #0x02
    b common_exception_entry

.align 7
vec_curr_el_sp0_serror:
    mov x0, #0x03
    b common_exception_entry

/* ----------------------------------------------------------------
 * 2. Current EL with SPx (0x200 - 0x380)
 * ---------------------------------------------------------------- */
.align 7
vec_curr_el_spx_sync:
    mov x0, #0x10
    b common_exception_entry

.align 7
vec_curr_el_spx_irq:
    b common_irq_entry

.align 7
vec_curr_el_spx_fiq:
    mov x0, #0x12
    b common_exception_entry

.align 7
vec_curr_el_spx_serror:
    mov x0, #0x13
    b common_exception_entry

/* ----------------------------------------------------------------
 * 3. Lower EL using AArch64 (0x400 - 0x580)
 * ---------------------------------------------------------------- */
.align 7
vec_lower_el_a64_sync:
    b common_lower_el_sync_entry

.align 7
vec_lower_el_a64_irq:
    b common_irq_entry

.align 7
vec_lower_el_a64_fiq:
    mov x0, #0x22
    b common_exception_entry

.align 7
vec_lower_el_a64_serror:
    mov x0, #0x23
    b common_exception_entry

/* ----------------------------------------------------------------
 * 4. Lower EL using AArch32 (0x600 - 0x780)
 * ---------------------------------------------------------------- */
.align 7
vec_lower_el_a32_sync:
    mov x0, #0x30
    b common_exception_entry

.align 7
vec_lower_el_a32_irq:
    mov x0, #0x31
    b common_exception_entry

.align 7
vec_lower_el_a32_fiq:
    mov x0, #0x32
    b common_exception_entry

.align 7
vec_lower_el_a32_serror:
    mov x0, #0x33
    b common_exception_entry

/* ----------------------------------------------------------------
 * Common Exception Context Save and Dispatch
 * x0 contains the exception_id
 * ---------------------------------------------------------------- */
.section .text, "ax"
.type common_exception_entry, @function
common_exception_entry:
    /* Reserve stack space for ExceptionContext:
       31 registers (x0..x30) = 248 bytes
       sp, elr_el1, spsr_el1, esr_el1, far_el1 = 40 bytes
       exception_id, padding = 16 bytes
       Total = 304 bytes (16-byte aligned: 304 % 16 == 0)
    */
    sub sp, sp, #304

    /* Save general purpose registers x1..x29 (x0 is saved later after preserving exception_id) */
    stp x2, x3, [sp, #16]
    stp x4, x5, [sp, #32]
    stp x6, x7, [sp, #48]
    stp x8, x9, [sp, #64]
    stp x10, x11, [sp, #80]
    stp x12, x13, [sp, #96]
    stp x14, x15, [sp, #112]
    stp x16, x17, [sp, #128]
    stp x18, x19, [sp, #144]
    stp x20, x21, [sp, #160]
    stp x22, x23, [sp, #176]
    stp x24, x25, [sp, #192]
    stp x26, x27, [sp, #208]
    stp x28, x29, [sp, #224]

    /* Save x30 (link register) and original sp */
    mov x1, sp
    add x1, x1, #304        /* original sp */
    stp x30, x1, [sp, #240]

    /* Save exception system registers */
    mrs x1, elr_el1
    mrs x2, spsr_el1
    stp x1, x2, [sp, #256]

    mrs x3, esr_el1
    mrs x4, far_el1
    stp x3, x4, [sp, #272]

    /* Save exception ID */
    str x0, [sp, #288]
    str xzr, [sp, #296]

    /* For x0 in ExceptionContext, we store original x0 (or exception_id if x0 was clobbered) */
    str x0, [sp, #0]
    /* Restore x1 from caller or stack if needed, store at [sp, #8] */
    str x1, [sp, #8]

    /* Pass pointer to ExceptionContext in x0 */
    mov x0, sp
    bl handle_early_exception

    /* Restore registers and return */
    ldp x1, x2, [sp, #256]
    msr elr_el1, x1
    msr spsr_el1, x2

    ldp x30, x1, [sp, #240]
    ldp x28, x29, [sp, #224]
    ldp x26, x27, [sp, #208]
    ldp x24, x25, [sp, #192]
    ldp x22, x23, [sp, #176]
    ldp x20, x21, [sp, #160]
    ldp x18, x19, [sp, #144]
    ldp x16, x17, [sp, #128]
    ldp x14, x15, [sp, #112]
    ldp x12, x13, [sp, #96]
    ldp x10, x11, [sp, #80]
    ldp x8, x9, [sp, #64]
    ldp x6, x7, [sp, #48]
    ldp x4, x5, [sp, #32]
    ldp x2, x3, [sp, #16]
    ldp x0, x1, [sp, #0]

    add sp, sp, #304
    eret

/* ----------------------------------------------------------------
 * Common IRQ Context Save, Dispatch, and Return
 * ---------------------------------------------------------------- */
.type common_irq_entry, @function
common_irq_entry:
    /* Reserve 272 bytes on the stack (16-byte aligned: 272 % 16 == 0) */
    sub sp, sp, #272

    /* Save general purpose registers x0..x29 */
    stp x0, x1, [sp, #0]
    stp x2, x3, [sp, #16]
    stp x4, x5, [sp, #32]
    stp x6, x7, [sp, #48]
    stp x8, x9, [sp, #64]
    stp x10, x11, [sp, #80]
    stp x12, x13, [sp, #96]
    stp x14, x15, [sp, #112]
    stp x16, x17, [sp, #128]
    stp x18, x19, [sp, #144]
    stp x20, x21, [sp, #160]
    stp x22, x23, [sp, #176]
    stp x24, x25, [sp, #192]
    stp x26, x27, [sp, #208]
    stp x28, x29, [sp, #224]

    /* Save link register x30 and zero padding */
    stp x30, xzr, [sp, #240]

    /* Save exception system registers: elr_el1 (PC) and spsr_el1 (PSTATE) */
    mrs x0, elr_el1
    mrs x1, spsr_el1
    stp x0, x1, [sp, #256]

    /* Call Rust IRQ dispatcher */
    bl handle_irq_dispatch

    /* Restore exception system registers */
    ldp x0, x1, [sp, #256]
    msr elr_el1, x0
    msr spsr_el1, x1

    /* Restore general purpose registers */
    ldr x30, [sp, #240]
    ldp x28, x29, [sp, #224]
    ldp x26, x27, [sp, #208]
    ldp x24, x25, [sp, #192]
    ldp x22, x23, [sp, #176]
    ldp x20, x21, [sp, #160]
    ldp x18, x19, [sp, #144]
    ldp x16, x17, [sp, #128]
    ldp x14, x15, [sp, #112]
    ldp x12, x13, [sp, #96]
    ldp x10, x11, [sp, #80]
    ldp x8, x9, [sp, #64]
    ldp x6, x7, [sp, #48]
    ldp x4, x5, [sp, #32]
    ldp x2, x3, [sp, #16]
    ldp x0, x1, [sp, #0]

    /* Reclaim stack frame */
    add sp, sp, #272

    /* Return from exception */
    eret
