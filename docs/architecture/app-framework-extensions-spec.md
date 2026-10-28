# Ind OS Sprint SP-127: App Framework Extensions Specification

**Document ID:** `INDOS-SPEC-APP-FRAMEWORK-EXT-001`  
**Version:** `1.0.0.0`  
**Governing Requirement:** `REQ-APP-COMMON-006 (P0)`  
**Phase:** `Phase 11 (SP-127 – SP-170)`  
**Constitutional Rules:** `Rule N-01`, `Rule N-04`, `Rule N-06`, `Rule N-10`  
**Charter References:** Universal App Charter §8.20 (`INV-APP-001`, `INV-APP-002`, `INV-APP-003`, `INV-TOK-001`)  

---

## 1. Overview & Architectural Goals

Sprint SP-127 establishes the foundational **App Framework Extensions** required for all Phase 11 native applications and system surfaces. Prior to SP-127, user interaction occurred through isolated modal views or raw compositor surfaces. SP-127 provides a unified, capability-bounded, deterministic window and lifecycle management architecture:

1. **Formalized Lifecycle State Machine (`INV-APP-LIFE-001`):** Every application transitions deterministically through 6 explicit lifecycle states (`Unloaded`, `Created`, `Foreground`, `Background`, `Suspended`, `Terminated`).
2. **Deterministic Back-Stack Navigation (`INV-APP-STACK-001`):** LIFO window navigation with strict ownership of input focus and seamless fallback to the Bento Grid root.
3. **Standardized Glassmorphic Chrome Library (`INV-APP-CHROME-001`):** Universal window header affordance complying with Appendix A design tokens, providing standard back navigation, title, and action tray.
4. **Capability-Bounded Window Management (`INV-APP-WIN-001`):** Every window is strictly coupled to its owning process PID and declared capability mask (`CAP_DISPLAY_RENDER`), preventing ambient authority and unauthorized z-index tampering.

---

## 2. Normative Invariants

### INV-APP-LIFE-001: Strict Six-State Lifecycle Transition Machine

Applications must transition strictly through the following state machine:

```text
       [on_launch]            [on_foreground]
Unloaded ------> Created -------------------------> Foreground
                   |                                    |
                   |                                    | [on_background]
                   |                                    v
                   |          [on_resume]           Background
                   |    ------------------------        |
                   |    |                      |        | [on_suspend]
                   v    v                      |        v
               Terminated <----------------------- Suspended
                           [on_terminate]
```

- Any attempt to jump states illegally (e.g. `Suspended -> Foreground` without `Background`, or `Unloaded -> Foreground`) is rejected with `AppLifecycleError::InvalidTransition`.
- Backgrounded applications have their GPU frame generation suspended while preserving state in memory.
- Suspended applications have volatile heap pages marked for reclaim under memory pressure.

### INV-APP-STACK-001: Deterministic Back-Stack Navigation Manager

- The application framework maintains an OS-managed LIFO stack of active window contexts: `AppBackStack`.
- The window at the top of the stack (`depth - 1`) unconditionally holds the primary touch and keyboard focus.
- Triggering the Back affordance (via top chrome, bottom navigation gesture, or hardware back event) pops the active window.
- When `pop()` is executed on the final window in the stack, the back-stack transitions cleanly to `Depth = 0`, and focus returns to the Ind Surface v2 Bento Grid home canvas without application crash or orphaned compositor planes.
- Maximum back-stack depth is strictly bounded to `16` entries to prevent unbounded kernel/userspace memory growth.

### INV-APP-CHROME-001: Universal Glassmorphic App Chrome Library (`AppChrome`)

Every native application surface must render its title and primary navigation via the unified `AppChrome` component:
- **Left Affordance:** Back button with chevron (visible when back-stack depth > 1 or when modal child view is active; tap targets conform to minimum 48×48 px touch boundary).
- **Center Label:** Window title (`TOK-TYPE-H3` 22sp / `TOK-COLOR-TEXT-0` `#F2F3F7`) and optional status indicator/badge.
- **Right Action Tray:** Up to 3 standard icon actions (e.g., search, share, settings) and optional overflow menu.
- **Visual Styling:** Background fill `TOK-COLOR-GLASS-BG` (`rgba(255,255,255,0.06)`), bottom border `TOK-COLOR-GLASS-BORDER` (`rgba(255,255,255,0.12)`), corner radius `TOK-RADIUS-M` (16px).

### INV-APP-WIN-001: Capability-Bound Window Management (`AppWindowManager`)

- Window surfaces are created only by processes possessing `CAP_DISPLAY_RENDER`.
- Each window descriptor holds `(window_id, pid, app_id, title, active_caps, z_index)`.
- System surfaces (`PlaneId::HardwareIsolated`, Status Bar, Navigation Bar) reside in protected z-planes (`Z >= 200`) and cannot be covered, occluded, or spoofed by application windows (`Z <= 100`).

---

## 3. Data Structures & ABI Specifications

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppLifecycleState {
    Unloaded,
    Created,
    Foreground,
    Background,
    Suspended,
    Terminated,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppLifecycleEvent {
    Launch,
    Foreground,
    Background,
    Suspend,
    Resume,
    Terminate,
}

pub struct AppWindowDescriptor {
    pub window_id: u32,
    pub pid: u32,
    pub app_id: &'static str,
    pub title: &'static str,
    pub state: AppLifecycleState,
    pub active_caps: u64,
    pub z_index: u8,
}
```

---

## 4. Verification & Gate Compliance

Compliance with `REQ-APP-COMMON-006` is audited through `ci/verify_sp127.py`:
1. Verifies zero external crate dependencies in microkernel (`Rule N-01`).
2. Verifies 100% safe Rust across all app framework userspace implementations (`Rule N-04`).
3. Executes automated state-transition fuzzing to ensure `INV-APP-LIFE-001` cannot be bypassed.
4. Validates stack push, pop, overflow resistance, and home-fallback invariants (`INV-APP-STACK-001`).
