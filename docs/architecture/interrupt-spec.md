# IndK Interrupt & Exception Subsystem Architecture Specification
**Document ID:** `INDOS-SPEC-IRQ-01`  
**Phase Target:** Phase 1 (SP-08) $\rightarrow$ Gate G1  
**Requirement Mapping:** `REQ-KRN-101`, `REQ-KRN-104`  
**Platform:** ARM Generic Interrupt Controller (GICv2 / GICv3) & Generic Timers

---

## 1. Scope & Objectives

This specification defines interrupt controller discovery, initialization of the GIC Distributor (`GICD`) and CPU Interfaces (`GICC` / `GICR`), ARM Generic Physical Timer programming, and the critical-section synchronization discipline.

---

## 2. GIC Subsystem Architecture

```text
                  +---------------------------+
                  |  Peripherals / Devices    |
                  |  (PL011 UART, Timers, IRQ)|
                  +---------------------------+
                                |
                                v
                  +---------------------------+
                  |      GIC Distributor      |
                  | (Routes IRQs, Priorities) |
                  +---------------------------+
                                |
                +---------------+---------------+
                |                               |
                v                               v
    +-----------------------+       +-----------------------+
    |   GIC CPU Interface   |       |   GIC CPU Interface   |
    |        Core 0         |       |        Core 1         |
    +-----------------------+       +-----------------------+
                |                               |
                v                               v
         CPU Core 0 IRQ                  CPU Core 1 IRQ
```

### 2.1 Interrupt Categories
1. **Software Generated Interrupts (SGIs, 0–15):** Used for Inter-Processor Interrupts (IPIs) in SMP.
2. **Private Peripheral Interrupts (PPIs, 16–31):** Dedicated per-core timers (e.g., Virtual Timer IRQ 27, Non-Secure Physical Timer IRQ 30).
3. **Shared Peripheral Interrupts (SPIs, 32–1019):** Peripheral device interrupts (PL011 UART IRQ 33, virtio devices).

---

## 3. Critical Section Discipline & Synchronization

In accordance with memory safety and deterministic execution:
1. **Interrupts-Disabled Spinlock (`IrqSafeSpinLock<T>`):**
   - Disables IRQs locally on the current core before taking the lock (`msr daifset, #2`).
   - Restores the previous interrupt state on drop.
   - Prevents deadlocks where an IRQ handler attempts to acquire a spinlock already held by the interrupted thread.
2. **Top-Half vs. Bottom-Half:**
   - Hardware IRQ handlers (top-half) acknowledge the interrupt, clear device status, and enqueue a work item.
   - Prolonged computation or model-related processing is forbidden inside top-half handlers.

---

## 4. Invariants (Machine-Verifiable)

- **INV-IRQ-001:** IRQ dispatch latency from exception entry to handler execution is $\le 10$ µs on QEMU `virt`.
- **INV-IRQ-002:** Nesting depth of interrupt disable calls is tracked per CPU core; enabling IRQs prematurely while holding a spinlock triggers a kernel panic.
- **INV-IRQ-003:** Spurious interrupts (Interrupt ID 1023) are filtered and logged to the diagnostic ring buffer without panic.
- **INV-IRQ-004:** The ARM Generic Physical Timer (`CNTP_TVAL_EL0`) is configured for a deterministic 100 Hz (10 ms) scheduler tick.
