# Ind OS Architecture Specification: Touch Input Pipeline & Kernel IPC Optimization (SP-111)

**Sprint:** SP-111  
**Phase:** Phase 9.5 (Performance & Usability Pass)  
**Constitutional References:**
- `Ind_OS_Development_Bible_v1.0.0.0.md` §8.16, Table 19.2
- `REQ-HW-107`, `REQ-UX-102`, `REQ-PERF-104`
- Rule N-01: Zero external crates in TCB
- Rule N-04: Zero unsafe code in new userspace modules

---

## 1. Executive Summary

Sprint SP-111 eliminates the top two performance hotspots identified in SP-110:
1. **FocalTech FT3518 Touch Event Latency:** Qualcomm GENI I2C1 touch event polling is optimized with zero-allocation ring-buffer reads, reducing input-to-render latency to $< 8.0\text{ ms}$ ($\le 120\text{Hz}$ frame budget).
2. **Synchronous IPC Fast-Path:** Capability-checked rendezvous round-trip optimized to $< 0.85\ \mu\text{s}$ through register-staged message passing.

---

## 2. Architectural Invariants

| Invariant ID | Title | Description & Guarantee |
|---|---|---|
| **INV-TOUCH-OPT-001** | Sub-16ms Touch-to-Render Latency | From I2C interrupt delivery to compositor swap buffer completes in $< 16\text{ ms}$ (p95 $< 8.3\text{ ms}$). |
| **INV-TOUCH-OPT-002** | Zero-Copy Touch Ring Dispatch | Touch coordinates dispatched to UX Shell without heap allocations. |
| **INV-TOUCH-OPT-003** | Sub-Microsecond Synchronous IPC | IPC ping-pong latency bounded at $\le 1.0\ \mu\text{s}$ (budget $\le 2.0\ \mu\text{s}$). |

---

## 3. Data Structures & API Contracts

```rust
pub const CAP_TOUCH_OPT: u64 = 0x0000_0000_0000_0100;
pub const TOUCH_OPT_MAX_BATCH: usize = 16;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FastTouchPoint {
    pub x: u16,
    pub y: u16,
    pub pressure: u8,
    pub timestamp_us: u32,
}
```
