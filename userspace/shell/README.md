# Ind OS Userspace Shell v1 (`indus-shell`)
**Sprint:** SP-35  
**Requirements Traceability:** `REQ-USE-201`, `REQ-USE-202`, `REQ-USE-203`  
**Invariants:** `INV-SH-001` (Bounded Panic-Free Parsing), `INV-SH-002` (Pipelines as Capability-Mediated IPC Chains)  
**Standard:** `#![no_std]` Rust, Zero external crates (Rule N-01), Zero unsafe code (Rule N-04)

## Architecture Overview
The interactive shell v1 daemon (`shell`) provides:
1. Command-line parsing with zero heap allocation and bounded argument limits.
2. Built-in developer tools (`help`, `version`, `ps`, `ls`, `cat`, `echo`, `caps`, `clear`).
3. Capability-mediated stream pipelines (`cmd1 | cmd2`) connecting stdout/stdin endpoints via IPC channels.
4. Dogfood interactive work sessions over serial/UART terminal channels.
