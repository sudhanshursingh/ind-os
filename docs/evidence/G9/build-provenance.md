# Gate G9 Build Provenance & Toolchain Manifest

- **Operating System:** Ind OS (MOTO G60 BRING-UP / HARDWARE V0.1)
- **Target Architecture:** `aarch64-unknown-none` (ARM64 EL1 Bare-Metal)
- **Host System:** Darwin ARM64 (macOS)
- **Rust Toolchain:** `rustc 1.85.0-nightly` (Zero external crates, `#![no_std]`, no-alloc in critical path)
- **Git Commit:** `c9233fb768b5a83a04f98101a91cf27a1b028812`
- **Clean Checkout Reproducibility:** Verified with clean checkout build script.
- **Binary Checksum CRC32:** `0x5C01D92E`
- **Compiler Flags:** `-C panic=abort -C opt-level=2 -C target-cpu=cortex-a76`
