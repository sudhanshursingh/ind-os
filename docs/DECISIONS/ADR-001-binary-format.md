# Architectural Decision Record (ADR-001)
## Native Micro-Executable Format (INDBIN) & ELF64 Subset Interoperability

**Status:** APPROVED  
**Date:** 2026-10-26  
**Deciders:** Core Architecture Team  
**Governing Document:** `Ind_OS_Development_Bible_v1.0.0.0.md` (§8.8, Phase 2 Core Kernel: Sprint SP-11)  
**Requirements Addressed:** `REQ-KRN-101`, `REQ-KRN-102`, `REQ-KRN-103`, `REQ-SEC-101`

---

### Context & Problem Statement

In Sprint SP-11 (Processes & Loading), Ind OS must introduce its process abstraction and binary executable loading pipeline.
Under Rule N-01 (from-scratch kernel and userspace, zero external crates) and the microkernel security architecture, binary loading has direct implications on:
1. **TCB Attack Surface:** Parsing complex formats like full ELF64 (with section headers, dynamic link tables, relocation types, string tables, and GNU extensions) in kernel space introduces substantial parser complexity and memory safety risks.
2. **Deterministic Startup Latency:** Microkernel services, early security monitors, and system processes require minimal, deterministic load times without relocation overhead or page-alignment padding waste.
3. **Capability & Security Envelope Manifests:** Future capability management (Phase 3: SP-16..SP-23) requires verifiable, signed integrity digests and declarative capability privilege grants embedded in the binary header.

We must decide the executable binary format strategy for Ind OS:
- **Option A:** Full ELF64 reader only.
- **Option B:** Own documented native binary format (`INDBIN`) only.
- **Option C (Adopted):** Hybrid approach — A native, hardened micro-binary format (`INDBIN`) designed for fast, zero-relocation direct loading, atomic integrity checksumming, and capability manifests, paired with an architectural path for a clean-room AArch64 static ELF64 subset reader.

---

### Decision

We adopt **Option C**:
1. **Primary Kernel-Native Format (`INDBIN`):**
   - Ind OS defines and implements its own native executable binary format (`INDBIN`, magic `0x42444E49` / `b"INDB"`).
   - The format is minimal, 64-byte fixed-header, page-aligned, relocation-free, and directly mappable into per-process `AddressSpace` page tables.
   - It incorporates fixed code (`.text`) and data (`.data`) segment descriptors, stack size requirements, entry point virtual address, and a 32-byte cryptographic SHA-256 integrity hash field.
   - It enforces memory invariants at load time: segments must not overlap the NULL trap zone (`0x0000_0000..0x0040_0000`, `INV-MEM-001`), and permissions are strictly enforced (`PXN` on user code/data, `UXN` on data per `INV-MEM-002`).

2. **Toolchain & ELF64 Subset Reader Strategy:**
   - Standard host toolchains (e.g. `rustc`, `clang`, `ld.lld`) produce standard AArch64 ELF64 binaries.
   - A companion tool (`indbin-pack` / host utility) strips ELF metadata and converts static AArch64 ELF loadable segments (`PT_LOAD`) directly into verified `INDBIN` images.
   - In later userspace sprints (Phase 4 / Phase 5), a clean-room static ELF64 subset loader can be hosted in userspace (outside the TCB) to run unmodified static ELF binaries without compromising microkernel simplicity.

---

### Consequences

#### Positive:
- **Zero Kernel Parser Bloat:** Kernel loader is fewer than 150 lines of pure, verifiable Rust code.
- **Deterministic Loading:** Zero relocations, zero section iterating, single-pass segment mapping.
- **Strict Invariant Enforcement:** The loader unconditionally validates user-space bounds and applies `PTE_PXN` and `PTE_UXN` bit protections before any execution.
- **Security-First Architecture:** Integrates seamlessly with Phase 3 capability grants and integrity hashing.

#### Negative / Trade-offs:
- Requires a conversion step for binaries built with standard ELF toolchains, automated in `Makefile` and build scripts.
