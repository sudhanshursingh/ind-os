# Ind OS Software Bill of Materials (SBOM) & Licensing Inventory

**Document ID:** `INDOS-SBOM-001`  
**Governing Rule:** Rule N-09 (SBOM from day zero)  
**Rule N-01 Verification:** No Linux, Android, or BSD operating system source code in `IndK` or `IndUS`.

Every external tool, dependency, compiler, and firmware blob utilized in building or running Ind OS must be recorded here with its license, SHA-256 hash, and provenance.

---

## 1. Host Build Tools & Compilers (Non-TCB, Host Environment)

| Tool / Component | Version | License | Upstream Source / Provenance | Purpose |
| :--- | :--- | :--- | :--- | :--- |
| **Rust Toolchain** | `1.82.0` | MIT / Apache-2.0 | `https://github.com/rust-lang/rust` | Cross-compilation host compiler (`rustc`, `cargo`) |
| **LLVM / Clang / LLD** | `18.1.8` | Apache-2.0 with LLVM Exception | `https://github.com/llvm/llvm-project` | Linker and assembly toolchain |
| **QEMU System ARM64** | `9.1.0` | GPLv2 | `https://github.com/qemu/qemu` | Phase 0–6 primary virtual development target (`virt`) |
| **Python** | `3.11+` | PSF License | `https://python.org` | Evidence pack verification and build scripting |

---

## 2. In-Tree Dependencies (TCB & System Services)

*Currently zero in-tree external dependencies. All kernel core components (`IndK`) are authored from scratch under Apache-2.0.*

---

## 3. Firmware Blobs & Device Trees (Hardware Enablement - Phase 9)

*No firmware blobs included in Phase 0. Target is QEMU virt.*
