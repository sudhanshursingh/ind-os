# IndK — The Ind OS Microkernel

**Subsystem:** Core Kernel (`IndK`)  
**Architecture:** ARM64 (AArch64) bare-metal (`aarch64-unknown-none`)  
**Primary Target:** QEMU `virt` machine (Cortex-A53/A57) $\rightarrow$ Qualcomm Snapdragon 732G (`hanoip`)  
**Ownership:** `@sudhanshursingh` (per [CODEOWNERS](file:///Users/dhan/IndOS/CODEOWNERS))  
**Governing Document:** [Ind_OS_Development_Bible_v1.0.0.0.md](file:///Users/dhan/IndOS/Ind_OS_Development_Bible_v1.0.0.0.md) (INDOS-BIBLE-001)  
**License:** GPL-2.0-only (per Bible §15.2)

---

## 1. Scope & Philosophy

`IndK` is a memory-safe hybrid microkernel written in freestanding Rust (`no_std`).
- **Rule N-01 (From scratch):** Contains zero copied code from Linux, AOSP, seL4, Zircon, or BSD.
- **Rule N-04 (Enforced Security):** The kernel hosts the Enforcement Layer (`ENF`). Ambient authority is forbidden. The Adaptive Intelligence Subsystem (`AIS`) is strictly unprivileged userspace.

---

## 2. Directory Layout

- `src/main.rs`: Freestanding kernel entry point (`kernel_main`).
- `src/panic.rs`: Structured panic handler with early serial register dump.
- `src/arch/aarch64/boot.s`: Pure ARM64 assembly reset vector, EL2 $\rightarrow$ EL1 drop, stack alignment, BSS zeroing.
- `src/arch/aarch64/exceptions.s`: 2,048-byte aligned `VBAR_EL1` vector table with register context preservation.
- `linker.ld`: Linker script targeting ARM64 physical load address `0x40080000`.

---

## 3. Build & Execution

```bash
# Build bare-metal kernel ELF
cargo build --target aarch64-unknown-none

# Test inside QEMU headless runner
python3 ci/run_qemu_test.py --kernel target/aarch64-unknown-none/debug/indk --golden docs/evidence/golden-boot.log
```
