# Ind OS Architecture Specification: Display Compositor 120Hz Refresh Pipelining (SP-113)

**Sprint:** SP-113  
**Phase:** Phase 9.5 (Performance & Usability Pass)  
**Constitutional References:**
- `Ind_OS_Development_Bible_v1.0.0.0.md` §8.16, Table 19.2; `REQ-HW-106`, `REQ-PERF-104`
- Rule N-01: Zero external crates in TCB
- Rule N-04: Zero unsafe code in new userspace modules

---

## 1. Executive Summary

Sprint SP-113 targets the display scanout and compositing pipeline on Motorola Moto G60 (Tianma ILI7807S FHD panel, 1080x2460 @ 120Hz). It implements double-buffered damage bounding with sub-8.33 ms frame times and zero jank.

---

## 2. Architectural Invariants

| Invariant ID | Title | Description & Guarantee |
|---|---|---|
| **INV-COMP-OPT-001** | 120Hz Frame Budget ($\le 8.33\text{ ms}$) | 95th percentile compositor frame time bounded at $\le 8.33\text{ ms}$ under 4 concurrent surfaces. |
| **INV-COMP-OPT-002** | Zero Jank Jitter ($\le 1.0\text{ ms}$) | Inter-frame swap jitter bounded below 1.0 ms across 1,000 continuous frames. |
| **INV-COMP-OPT-003** | Damage Bounding Efficiency | Pixel rasterization restricted to damaged bounding boxes, saving $> 60\%$ DRAM bandwidth. |

---

## 3. Data Structures & API Contracts

```rust
pub const CAP_COMPOSITOR_OPT: u64 = 0x0000_0000_0000_0400;
pub const TARGET_FRAME_TIME_US: u32 = 8333; // 120Hz = 8.333ms

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FrameTimingSample {
    pub frame_idx: u32,
    pub render_time_us: u32,
    pub swap_time_us: u32,
    pub dirty_pixels: u32,
}
```
