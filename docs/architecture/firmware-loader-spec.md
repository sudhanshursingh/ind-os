# Ind OS Architecture Specification: Firmware Blob Loader (§14.3, Sprint SP-86)

**Document ID:** SPEC-ARCH-SP86-FIRMWARE-LOADER  
**Status:** Approved  
**Target:** Isolated Firmware Blob Loading & SBOM Integrity Management  
**Standards:** Ind OS Development Bible v1.0.0.0 §6.3, §8.15, §14.3, Table 19.2; REQ-SEC-105, REQ-HW-109  
**Hardware:** Motorola Moto G60 (`hanoip` / Qualcomm Snapdragon 732G SM7150-AC)

---

### 1. Architectural Invariants

- **INV-FW-001 (Isolated Outside TCB):** All firmware parsing, unpacking, and verification occur in userspace (`devmgr`), completely outside the kernel Trusted Computing Base (TCB). The kernel merely receives pre-verified contiguous DMA buffer descriptors.
- **INV-FW-002 (Cryptographic Integrity & SBOM):** Each firmware image must match an in-tree SBOM record with expected payload size and SHA-256 digest before hardware coprocessor handoff.
- **INV-FW-003 (Capability-Gated Dispatch):** Coprocessor firmware staging and reset release require explicit possession of `CAP_FIRMWARE_LOAD` (`0x0100_0000`).

---

### 2. Supported Subsystem Payloads

| Target Subsystem | Firmware Blob Name | Memory Destination |
|---|---|---|
| WLAN / Wi-Fi | `wlanmdsp.mbn` | `0x8900_0000` |
| Bluetooth | `nvm47_3.bin` | `0x8920_0000` |
| Audio DSP / ADSP | `adsp.mbn` | `0x8940_0000` |
| Modem / MPSS | `modem.mbn` | `0x8980_0000` |

---

### 3. Safety & Isolation Contracts

1. **Rule N-01:** Pure in-tree Rust, zero external crates in TCB.
2. **Rule N-04:** Userspace loader `userspace/devmgr/src/firmware_loader.rs` is 100% safe Rust.
3. **Rule N-06 / PRD-001 §7.5:** Corrupt or non-matching firmware payloads trigger safe rejection without affecting kernel stability.
