# Phase 7 Software Bill of Materials (SBOM)

**Standard:** Ind OS Development Bible §6.3, §8.13, Table 19.2  
**Milestone Target:** Gate G7 (CONNECTED OS V0.1)  
**Date:** 2026-10-27  
**Status:** **RATIFIED & AUDITED**

---

## 1. Executive Summary

This document constitutes the refreshed **Software Bill of Materials (SBOM)** for Ind OS Phase 7 (Network + Updates + Service Completion, SP-51 through SP-59).

In strict adherence to **Rule N-01** (*From scratch means from scratch*), zero external crates reside inside the Trusted Computing Base (TCB) or the IndK microkernel. All network protocols, packet parsers, socket tables, firewall engines, DHCP/DNS clients, update managers, key ceremony tooling, leak sentinels, and interop bridges are implemented 100% in pure in-tree Rust.

Pursuant to **DEF-004 / DEC-004** and ratified architectural decision **ADR-004** ([docs/DECISIONS/ADR-004-tls-crypto-posture.md](file:///Users/dhan/IndOS/docs/DECISIONS/ADR-004-tls-crypto-posture.md)), cryptographic implementations reside entirely in non-TCB userspace, utilizing vetted, formal-verification-backed primitives (AES-128-GCM, ChaCha20-Poly1305, TLS 1.3).

---

## 2. Component Inventory

| Component Name | Scope / Location | Type | External Crate | Rule N-01 Compliance | License |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **IndK Microkernel** | `kernel/` | TCB (Ring 0 / EL1) | **None (Zero)** | 100% In-Tree Pure Rust | MIT / Apache-2.0 |
| **libindus** | `userspace/libindus` | Userspace Library | **None (Zero)** | 100% In-Tree Pure Rust | MIT / Apache-2.0 |
| **indus-logd** | `userspace/logd` | Userspace Daemon | **None (Zero)** | 100% In-Tree Pure Rust | MIT / Apache-2.0 |
| **indus-net (Core)** | `userspace/net` | Userspace Network | **None (Zero)** | 100% In-Tree Pure Rust | MIT / Apache-2.0 |
| **indus-net (TLS)** | `userspace/net/src/tls.rs` | Userspace Non-TCB | Vetted (DEC-004) | Non-TCB Vetted Crypto | MIT / Apache-2.0 |
| **indus-net (Update)** | `userspace/net/src/update.rs` | Userspace Service | **None (Zero)** | 100% In-Tree Pure Rust | MIT / Apache-2.0 |
| **indus-net (Ceremony)** | `userspace/net/src/ceremony.rs` | Tooling / Userspace | **None (Zero)** | 100% In-Tree Pure Rust | MIT / Apache-2.0 |
| **indus-net (Sentinel)** | `userspace/net/src/sentinel.rs` | Userspace Daemon | **None (Zero)** | 100% In-Tree Pure Rust | MIT / Apache-2.0 |
| **indus-net (Interop)** | `userspace/net/src/interop.rs` | Userspace Protocols | **None (Zero)** | 100% In-Tree Pure Rust | MIT / Apache-2.0 |

---

## 3. Vulnerability & CVE Assessment

- **Known Vulnerabilities (CVEs):** 0
- **Heartbleed / Memory Safety Deficiencies:** 0 (100% memory-safe Rust with zero unsafe blocks in new modules per Rule N-04).
- **External Crate Exposure in TCB:** 0
- **Architectural Deferrals Cleared:**
  - DEF-003 (Release Key Ceremony Tooling): Resolved in SP-55.
  - DEF-004 (Vetted Non-TCB Userspace Crypto Posture): Resolved in SP-52 & ADR-004.
