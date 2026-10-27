# Ind OS Architecture Specification: Interactive Touch-Card UI Shell & App Launcher (SP-114)

**Sprint:** SP-114  
**Phase:** Phase 9.5 (Performance & Usability Pass)  
**Constitutional References:**
- `Ind_OS_Development_Bible_v1.0.0.0.md` §8.16, Table 19.2; `REQ-UX-301..305`, `REQ-HW-106`
- Rule N-01: Zero external crates in TCB
- Rule N-04: Zero unsafe code in new userspace modules

---

## 1. Executive Summary

Sprint SP-114 bridges the hardware display and FocalTech FT3518 touch digitizer with the Ind OS Shell v2 UX state machine on physical hardware:
1. **Interactive Touch Cards:** Transitions from the static hardware splash into dynamic interactive cards:
   - **Card 0 (Telemetry & Health):** System hardware telemetry, SoC Kryo core status, LPDDR4X pool, UFS 2.1 IndFS state.
   - **Card 1 (App Drawer & Launcher):** Fast launching of `Terminal`, `Settings`, `Filesystem`, `AIS Assistant`, and `Editor`.
   - **Card 2 (AIS Intent Proposal Engine):** Dynamic intent cards powered by on-device Qwen-2.5-1.5B sandbox.
2. **Bottom Navigation Dock:** 3 interactive touch-sensitive dock buttons (`DASHBOARD`, `APPS`, `AIS`) with hit-testing and active visual feedback.
3. **Live Hardware Heartbeat:** Real-time ticking indicator driven by ARM generic timer (`CNTPCT_EL0`).

---

## 2. Architectural Invariants

| Invariant ID | Title | Description & Guarantee |
|---|---|---|
| **INV-TOUCH-SHELL-001** | Zero-Latency Card Switch | Tapping navigation dock switches active surface within 1 frame ($< 8.33\text{ ms}$). |
| **INV-TOUCH-SHELL-002** | Accurate Touch Hit-Testing | Touch tap coordinates mapped to bounding boxes with zero ambiguity. |
| **INV-TOUCH-SHELL-003** | Continuous Live Heartbeat | Clock and heartbeat indicator tick at $\ge 1\text{Hz}$ without system hangs. |

---

## 3. Data Structures & API Contracts

```rust
pub const CAP_INTERACTIVE_SHELL: u64 = 0x0000_0000_0000_0800;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveCard {
    Telemetry,
    AppDrawer,
    AisAssistant,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppAction {
    LaunchTerminal,
    LaunchSettings,
    LaunchFilesystem,
    LaunchAis,
    LaunchEditor,
}
```
