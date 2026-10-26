# Ind OS Userspace Device Manager (`devmgr`)
**Sprint:** SP-34  
**Requirements Traceability:** `REQ-USE-203`  
**Invariants:** `INV-DEV-001` (Zero Ambient Device Authority), `INV-DEV-002` (Hotplug Lifecycle & Atomic Revocation)  
**Standard:** `#![no_std]` Rust, Zero external crates (Rule N-01), Zero unsafe code (Rule N-04)

## Architecture Overview
The Device Manager daemon (`devmgr`) is responsible for:
1. Dynamic peripheral detection and hotplug event notifications across the system.
2. Capability minting and handouts (`REQ-USE-203`) granting scoped access tokens (MMIO ranges, IRQ lines, DMA buffers) to isolated user-mode driver daemons.
3. Managing device lifecycle, driver matching, and atomic capability revocation upon device departure.
4. VirtIO plug/unplug event cycle testing.
