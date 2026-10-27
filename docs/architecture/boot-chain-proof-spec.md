# Ind OS Boot-Chain Proof & Non-Stock Kernel Bring-Up Specification (SP-72)

## 1. Architectural Mandate
- **Governing Standard:** Ind OS Development Bible v1.0.0.0 §6.3, §6.4, §8.15, Table 19.2 (REQ-HW-103, PEN-005, Rule N-06).
- **Target:** Qualcomm Android Boot (ABOOT) / Little Kernel (LK) boot-chain proof on Moto G60 `hanoip`, Android Verified Boot (AVB 2.0) unlock handling, non-destructive fallback routing.

---

## 2. Architectural Invariants

| ID | Description | Enforcement Mechanism |
| :--- | :--- | :--- |
| **INV-BOOT-001** | Kernel Image Packaging & Header Verification | Kernel image verified against Qualcomm bootimg header standard (magic `ANDROID!`, page size 4096, entry point 0x80000000) |
| **INV-BOOT-002** | AVB 2.0 Unlock Status & Signature Policy | Bootloader unlock flags verified; unlocked state accepts test-key signed payload; locked state gracefully reports refusal |
| **INV-BOOT-003** | Deterministic Boot-Fallback & Anti-Bricking | If kernel verification fails, bootloader routes cleanly to stock slot or recovery with zero bootlooping |

---

## 3. Data Structures & API Contracts

```rust
pub const BOOT_MAGIC: &[u8; 8] = b"ANDROID!";
pub const QUALCOMM_PAGE_SIZE: u32 = 4096;

pub enum BootChainError {
    InvalidMagic,
    PageSizeMismatch,
    AvbLockedRefusal,
    FallbackTriggered,
}

pub struct BootChainHarness {
    header_verified: bool,
    avb_unlocked: bool,
    fallback_tested: bool,
}
```
