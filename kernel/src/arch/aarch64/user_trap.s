/*
 * Low-level User Mode Entry, Exit, and Lower EL Trap Trampolines (ARM64)
 * Ratified in docs/architecture/boot-spec.md & INDOS-BIBLE-001 §8.8 (SP-11).
 */

.section .text, "ax"

/*
 * common_lower_el_sync_entry
 * Entered when an EL0 user process triggers a synchronous exception (e.g. SVC #0)
 */
.global common_lower_el_sync_entry
.type common_lower_el_sync_entry, @function
common_lower_el_sync_entry:
    /* Allocate 288 bytes for UserContext on current EL1 stack */
    sub sp, sp, #288

    /* Save general-purpose registers x0..x29 */
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
    str x30, [sp, #240]

    /* Save system registers */
    mrs x1, sp_el0
    mrs x2, elr_el1
    stp x1, x2, [sp, #248]

    mrs x3, spsr_el1
    mrs x4, esr_el1
    stp x3, x4, [sp, #264]
    str xzr, [sp, #280]

    /* Call Rust syscall / exception dispatcher: handle_lower_el_sync(&mut UserContext) */
    mov x0, sp
    bl handle_lower_el_sync

    /* Restore system registers */
    ldp x1, x2, [sp, #248]
    msr sp_el0, x1
    msr elr_el1, x2

    ldp x3, x4, [sp, #264]
    msr spsr_el1, x3

    /* Restore general-purpose registers */
    ldp x2, x3, [sp, #16]
    ldp x4, x5, [sp, #32]
    ldp x6, x7, [sp, #48]
    ldp x8, x9, [sp, #64]
    ldp x10, x11, [sp, #80]
    ldp x12, x13, [sp, #96]
    ldp x14, x15, [sp, #112]
    ldp x16, x17, [sp, #128]
    ldp x18, x19, [sp, #144]
    ldp x20, x21, [sp, #160]
    ldp x22, x23, [sp, #176]
    ldp x24, x25, [sp, #192]
    ldp x26, x27, [sp, #208]
    ldp x28, x29, [sp, #224]
    ldr x30, [sp, #240]

    ldp x0, x1, [sp, #0]
    add sp, sp, #288

    /* Return to EL0 */
    eret

/*
 * kernel_call_user(user_ctx: *const UserContext, kctx: *mut KernelContext) -> i32
 * Saves kernel callee-saved context, then jumps to user mode in EL0.
 */
.global kernel_call_user
.type kernel_call_user, @function
kernel_call_user:
    /* Save kernel callee-saved registers into kctx (x1) */
    stp x19, x20, [x1, #0]
    stp x21, x22, [x1, #16]
    stp x23, x24, [x1, #32]
    stp x25, x26, [x1, #48]
    stp x27, x28, [x1, #64]
    stp x29, x30, [x1, #80]
    mov x2, sp
    str x2, [x1, #96]

    /* Load user system registers from user_ctx (x0) */
    ldr x1, [x0, #248]       /* sp_el0 */
    msr sp_el0, x1
    ldr x2, [x0, #256]       /* elr_el1 */
    msr elr_el1, x2
    ldr x3, [x0, #264]       /* spsr_el1 */
    msr spsr_el1, x3

    /* Load user general-purpose registers */
    ldp x2, x3, [x0, #16]
    ldp x4, x5, [x0, #32]
    ldp x6, x7, [x0, #48]
    ldp x8, x9, [x0, #64]
    ldp x10, x11, [x0, #80]
    ldp x12, x13, [x0, #96]
    ldp x14, x15, [x0, #112]
    ldp x16, x17, [x0, #128]
    ldp x18, x19, [x0, #144]
    ldp x20, x21, [x0, #160]
    ldp x22, x23, [x0, #176]
    ldp x24, x25, [x0, #192]
    ldp x26, x27, [x0, #208]
    ldp x28, x29, [x0, #224]
    ldr x30, [x0, #240]

    ldp x0, x1, [x0, #0]

    /* Execute eret: drops privilege from EL1 to EL0! */
    eret

/*
 * kernel_return_from_user(kctx: *const KernelContext, exit_code: i32) -> !
 * Restores kernel callee-saved state and returns from kernel_call_user.
 */
.global kernel_return_from_user
.type kernel_return_from_user, @function
kernel_return_from_user:
    /* Restore kernel stack pointer */
    ldr x2, [x0, #96]
    mov sp, x2

    /* Restore callee-saved registers */
    ldp x19, x20, [x0, #0]
    ldp x21, x22, [x0, #16]
    ldp x23, x24, [x0, #32]
    ldp x25, x26, [x0, #48]
    ldp x27, x28, [x0, #64]
    ldp x29, x30, [x0, #80]

    /* Place exit_code in w0 (return value) */
    mov w0, w1
    ret

/*
 * common_lower_el_irq_entry
 * Entered when a hardware IRQ arrives while running in EL0
 */
.global common_lower_el_irq_entry
.type common_lower_el_irq_entry, @function
common_lower_el_irq_entry:
    /* Allocate 288 bytes for UserContext on current EL1 stack */
    sub sp, sp, #288

    /* Save general-purpose registers x0..x29 */
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
    str x30, [sp, #240]

    /* Save system registers */
    mrs x1, sp_el0
    mrs x2, elr_el1
    stp x1, x2, [sp, #248]

    mrs x3, spsr_el1
    mrs x4, esr_el1
    stp x3, x4, [sp, #264]
    str xzr, [sp, #280]

    /* Call Rust lower EL IRQ handler: handle_lower_el_irq(&mut UserContext) */
    mov x0, sp
    bl handle_lower_el_irq

    /* Restore system registers */
    ldp x1, x2, [sp, #248]
    msr sp_el0, x1
    msr elr_el1, x2

    ldp x3, x4, [sp, #264]
    msr spsr_el1, x3

    /* Restore general-purpose registers */
    ldp x2, x3, [sp, #16]
    ldp x4, x5, [sp, #32]
    ldp x6, x7, [sp, #48]
    ldp x8, x9, [sp, #64]
    ldp x10, x11, [sp, #80]
    ldp x12, x13, [sp, #96]
    ldp x14, x15, [sp, #112]
    ldp x16, x17, [sp, #128]
    ldp x18, x19, [sp, #144]
    ldp x20, x21, [sp, #160]
    ldp x22, x23, [sp, #176]
    ldp x24, x25, [sp, #192]
    ldp x26, x27, [sp, #208]
    ldp x28, x29, [sp, #224]
    ldr x30, [sp, #240]

    ldp x0, x1, [sp, #0]
    add sp, sp, #288

    /* Return to EL0 */
    eret

/* ----------------------------------------------------------------
 * Embedded Test User Processes for Sprint SP-12 Multitasking
 * ---------------------------------------------------------------- */
.section .rodata
.align 4
.global user_task_alpha_start
.global user_task_alpha_end
user_task_alpha_start:
    /* 1. SYS_WRITE(1, msg_alpha_1, len) */
    mov x8, #1
    mov x0, #1
    adr x1, 10f
    mov x2, #(11f - 10f)
    svc #0

    /* 2. SYS_SCHED_HINT(hint=1 (LatencySensitive)) */
    mov x8, #4
    mov x0, #1
    svc #0

    /* 3. SYS_YIELD() */
    mov x8, #0
    svc #0

    /* 4. SYS_WRITE(1, msg_alpha_2, len) */
    mov x8, #1
    mov x0, #1
    adr x1, 12f
    mov x2, #(13f - 12f)
    svc #0

    /* 5. Busy work loop to allow timer ticks to arrive and preempt */
    movz x19, #0x200, lsl #16
14: subs x19, x19, #1
    b.ne 14b

    /* 6. SYS_WRITE(1, msg_alpha_3, len) */
    mov x8, #1
    mov x0, #1
    adr x1, 15f
    mov x2, #(16f - 15f)
    svc #0

    /* 7. SYS_EXIT(0) */
    mov x8, #2
    mov x0, #0
    svc #0
    b .

10: .ascii "[ALPHA] Process 1 executing in EL0 (iter 1)\n"
11:
12: .ascii "[ALPHA] Process 1 resumed after yield/preemption (iter 2)\n"
13:
15: .ascii "[ALPHA] Process 1 completed successfully, exiting.\n"
16:
.align 4
user_task_alpha_end:

.align 4
.global user_task_beta_start
.global user_task_beta_end
user_task_beta_start:
    /* 1. SYS_WRITE(1, msg_beta_1, len) */
    mov x8, #1
    mov x0, #1
    adr x1, 20f
    mov x2, #(21f - 20f)
    svc #0

    /* 2. Busy work loop to allow timer ticks to arrive and preempt */
    movz x19, #0x200, lsl #16
24: subs x19, x19, #1
    b.ne 24b

    /* 3. SYS_YIELD() */
    mov x8, #0
    svc #0

    /* 4. SYS_WRITE(1, msg_beta_2, len) */
    mov x8, #1
    mov x0, #1
    adr x1, 22f
    mov x2, #(23f - 22f)
    svc #0

    /* 5. SYS_WRITE(1, msg_beta_3, len) */
    mov x8, #1
    mov x0, #1
    adr x1, 25f
    mov x2, #(26f - 25f)
    svc #0

    /* 6. SYS_EXIT(0) */
    mov x8, #2
    mov x0, #0
    svc #0
    b .

20: .ascii "[BETA]  Process 2 executing in EL0 (iter 1)\n"
21:
22: .ascii "[BETA]  Process 2 resumed after yield/preemption (iter 2)\n"
23:
25: .ascii "[BETA]  Process 2 completed successfully, exiting.\n"
26:
.align 4
user_task_beta_end:

