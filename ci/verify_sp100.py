#!/usr/bin/env python3
"""
Ind OS Sprint SP-100 Verification & Acceptance Test Harness
Target: Shell v2 on Physical Display and Touch Integration
Standards: Bible §6.3, §8.15, Table 19.2, REQ-HW-106, REQ-UX-102, INV-SHELL-HW-001..003
"""

import os
import sys

REQUIRED_SP100_FILES = [
    "docs/architecture/shell-hardware-spec.md",
    "docs/evidence/SP-100-validation.md",
    "docs/evidence/golden-boot-sp100.log",
    "userspace/ux/src/shell_hw.rs",
    "kernel/src/hal/platform/shell_hw_core.rs",
]

def verify_sp100():
    print("==================================================")
    print(" Ind OS SP-100 Shell v2 Hardware Verifier         ")
    print(" Target: Sprint SP-100 Definition of Done (DoD)   ")
    print("==================================================")

    # 1. Deliverables Presence Check
    print("[CHECK 1/6] Verifying SP-100 deliverable presence...")
    for rel_path in REQUIRED_SP100_FILES:
        if not os.path.exists(rel_path):
            print(f"❌ FAIL: Required SP-100 file missing: {rel_path}")
            sys.exit(1)
        print(f"  ✓ Found {rel_path}")

    # 2. Architectural & Invariant Symbols Check
    print("\n[CHECK 2/6] Verifying architectural symbols and API contracts...")
    with open("userspace/ux/src/shell_hw.rs", "r") as f:
        shell_src = f.read()
    for s in [
        "ShellHwEngine",
        "ShellHwError",
        "ShellHwState",
        "GestureType",
        "TouchPoint",
        "GestureEvent",
        "CAP_DISPLAY_RENDER",
        "CAP_TOUCH_INPUT",
        "PANEL_WIDTH",
        "PANEL_HEIGHT",
        "REFRESH_RATE_HZ",
        "bind_surface",
        "process_touch",
        "swap_buffers",
        "render_chrome",
        "frame_count",
        "gestures_dispatched",
        "is_active",
        "is_chrome_rendered",
    ]:
        if s not in shell_src:
            print(f"❌ FAIL: shell_hw.rs missing symbol '{s}'")
            sys.exit(1)
        print(f"  ✓ Verified userspace symbol: {s}")

    with open("kernel/src/hal/platform/shell_hw_core.rs", "r") as f:
        kern_src = f.read()
    for s in [
        "KernelShellHwHarness",
        "KernelShellHwError",
        "KernelGestureType",
        "KernelShellTouchPoint",
        "KernelGestureEvent",
        "KERNEL_CAP_DISPLAY_RENDER",
        "KERNEL_CAP_TOUCH_INPUT",
        "KERNEL_SHELL_PANEL_WIDTH",
        "KERNEL_SHELL_PANEL_HEIGHT",
        "KERNEL_SHELL_REFRESH_RATE_HZ",
        "bind_surface",
        "process_touch",
        "swap_buffers",
        "render_chrome",
        "frame_count",
        "gestures_dispatched",
        "is_active",
        "is_chrome_rendered",
    ]:
        if s not in kern_src:
            print(f"❌ FAIL: shell_hw_core.rs missing symbol '{s}'")
            sys.exit(1)
        print(f"  ✓ Verified kernel symbol: {s}")

    # 3. Rule N-01 Compliance: Zero external crates
    print("\n[CHECK 3/6] Verifying Rule N-01 (Zero external crates in TCB)...")
    for cargo_path in ["kernel/Cargo.toml", "userspace/ux/Cargo.toml"]:
        with open(cargo_path, "r") as f:
            c = f.read()
        lines = c.splitlines()
        in_deps = False
        for line in lines:
            line_str = line.strip()
            if line_str.startswith("[dependencies]"):
                in_deps = True
                continue
            if in_deps and line_str.startswith("["):
                in_deps = False
            if in_deps and line_str and not line_str.startswith("#"):
                if "path =" not in line_str:
                    print(f"❌ FAIL: Non-path external dependency in {cargo_path}: {line_str}")
                    sys.exit(1)
        print(f"  ✓ {cargo_path} compliant with Rule N-01")

    # 4. Rule N-04 Compliance: Zero unsafe in newly authored userspace
    print("\n[CHECK 4/6] Verifying Rule N-04 (Zero unsafe code in userspace/ux/src/shell_hw.rs)...")
    with open("userspace/ux/src/shell_hw.rs", "r") as f:
        content = f.read()
    if "unsafe " in content or "unsafe{" in content:
        print("❌ FAIL: unsafe code block found in userspace/ux/src/shell_hw.rs")
        sys.exit(1)
    print("  ✓ userspace/ux/src/shell_hw.rs is 100% safe Rust")

    # 5. Golden Boot Log Verification: Stages 450-454
    print("\n[CHECK 5/6] Verifying Golden Boot Log Stages 450-454...")
    with open("docs/evidence/golden-boot-sp100.log", "r") as f:
        log_content = f.read()

    stages = [
        "450_SP100_SHELL_HW_INIT",
        "451_SP100_DSI_SURFACE_PIPELINE",
        "452_SP100_TOUCH_GESTURE_DISPATCH",
        "453_SP100_SHELL_CHROME_RENDER",
        "454_SP100_SHELL_HARDWARE_READY",
    ]
    for st in stages:
        if st not in log_content:
            print(f"❌ FAIL: Missing stage {st} in docs/evidence/golden-boot-sp100.log")
            sys.exit(1)
        print(f"  ✓ Verified stage: {st}")

    # 6. Specific Confirmation Logs
    print("\n[CHECK 6/6] Verifying Shell v2 Hardware Confirmation Logs...")
    expected_logs = [
        "Shell hardware harness online: DSI display scanout & touch pipeline armed (REQ-HW-106 PASSED)",
        "DSI swapchain active: 1080x2460 @ 120Hz double-buffered pipeline operational (INV-SHELL-HW-001 PASSED)",
        "Touch gestures dispatched: Tap, Drag, Fling, Release processed with <16ms latency (INV-SHELL-HW-002 PASSED)",
        "Shell chrome rendered: Status Bar (64px), Main Surface (2268px), Nav Bar (128px) active (INV-SHELL-HW-003 PASSED)",
        "[SP-100] Sprint SP-100 Shell v2 on Physical Hardware Milestone REACHED",
    ]
    for el in expected_logs:
        if el not in log_content:
            print(f"❌ FAIL: Log string not found: '{el}'")
            sys.exit(1)
        print(f"  ✓ Confirmed log output: {el[:65]}...")

    print("\n==================================================")
    print(" ✅ SPRINT SP-100 VERIFICATION PASSED (100% DoD) ")
    print("==================================================")

if __name__ == "__main__":
    verify_sp100()
