# Architectural Decision Record (ADR-004)
## Cryptography Posture: Vetted External TLS Library (DEF-004 / DEC-004)

**Status:** APPROVED  
**Date:** 2026-10-27  
**Deciders:** Technical Founder (Sudhanshu Raj Singh), Core Security Team  
**Governing Document:** `Ind_OS_Development_Bible_v1.0.0.0.md` (§7.1 DEC-004, §7.2 DEF-004, §8.13 Phase 7: Sprint SP-52)  
**Requirements Addressed:** `REQ-SEC-102`, `DEF-004`, `DEC-004`, `R-05`, `INV-TLS-001..003`  

---

### 1. Context & Problem Statement

In Sprint SP-52 (TLS Integration), Ind OS implements its transport layer security for encrypted network communication and authenticated system updates.
Under Bible Rule N-01 ("from scratch means from scratch"), there is an apparent tension with cryptographic security:
1. **Heartbleed-Class Risk (R-05):** Implementing TLS 1.3, X.509 certificate parsers, and ASN.1 decoders from scratch in an unreviewed system introduces catastrophic risk of side-channel timing leaks, parser memory flaws, and cryptographic oracle vulnerabilities.
2. **Rule N-01 Scope Note (§5.1):** Rule N-01 governs *OS code* (kernel and core supervision). It explicitly permits vetted external libraries in clearly-separated, SBOM-tracked, non-TCB positions where a DEF item explicitly permits them (`DEF-004` precedent).
3. **DEC-004 Ratification:** DEC-004 formally establishes that vetted, audited external TLS libraries (e.g. rustls or equivalent) are the default for the TLS layer, compiled from source by our toolchain, and sandboxed in userspace.

---

### 2. Decision

We adopt **DEC-004 / DEF-004**:
1. **Non-TCB Userspace Isolation (`INV-TLS-001`):**
   - The TLS engine operates exclusively in userspace within `userspace/net` (`indus-net`).
   - Zero cryptographic cipher code or certificate parsing logic resides within the microkernel TCB (`kernel/`).
   - All network socket interactions are governed by capability tokens (`SocketCapToken`, `CAP_NET_CONNECT`, `CAP_NET_SEND`, `CAP_NET_RECV`).
2. **TLS 1.3 Protocol & Pinned Ciphers (`INV-TLS-002`):**
   - The TLS engine strictly mandates TLS 1.3 (`0x0304`), deprecating all legacy versions (TLS 1.0, 1.1, 1.2) to prevent downgrade attacks.
   - Pinned AEAD cipher suites: `TLS_AES_128_GCM_SHA256` (`0x1301`) and `TLS_CHACHA20_POLY1305_SHA256` (`0x1303`).
3. **Trust Anchor & Certificate Pinning (`INV-TLS-003`):**
   - Strict root trust anchor validation and hostname/SAN matching.
   - Immediate rejection of expired, untrusted, self-signed, or tampered certificates.
4. **Toolchain Compilation & SBOM Tracking:**
   - 100% of TLS components are compiled from verified source code by our in-tree toolchain.
   - All dependencies and components are tracked with SHA-256 hashes in `evidence/SP-52/manifest.json`.

---

### 3. Consequences

- **Positive:** Eliminates custom crypto vulnerabilities; prevents Heartbleed-class protocol flaws; satisfies strict security criteria for update verification (`SP-54`).
- **Negative:** Requires rigorous build-time integration and SBOM tracking.
- **TCB Invariant:** No kernel compromise is possible through malformed TLS frames because TLS parsing is completely isolated in userspace.
