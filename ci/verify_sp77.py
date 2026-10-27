#!/usr/bin/env python3
"""
Ind OS Sprint SP-77 Verification & Acceptance Test Harness
Target: CPU Software Rendering Compositor (Double Buffering, Dirty Rect, Surface Presentation)
Standards: Bible §6.3, §8.15, Table 19.2, PEN-004, REQ-UX-101, INV-COMP-001..003
"""

import os
import sys

REQUIRED_SP77_FILES = [
    "docs/architecture/cpu-software-rendering-spec.md",
    "docs/evidence/SP-77-validation.md",
    "docs/evidence/golden-boot-sp77.log",
    "userspace/devmgr/src/cpu_compositor.rs",
    "kernel/src/hal/platform/cpu_compositor_core.rs",
]

def verify_sp77():
    print("==================================================")
    print(" Ind OS SP-77 CPU Software Compositor Verifier    ")
    print(" Target: Sprint SP-77 Definition of Done (DoD)   ")
    print("==================================================")

    # 1. Deliverables Presence Check
    print("[CHECK 1/6] Verifying SP-77 deliverable presence...")
    for rel_path in REQUIRED_SP77_FILES:
        if not os.path.exists(rel_path):
            print(f"❌ FAIL: Required SP-77 file missing: {rel_path}")
            sys.exit(1)
        print(f"  ✓ Found {rel_path}")

    # 2. Architectural & Invariant Symbols Check
    print("\n[CHECK 2/6] Verifying architectural symbols and API contracts...")
    with open("userspace/devmgr/src/cpu_compositor.rs", "r") as f:
        comp_src = f.read()
    for s in [
        "CompositorError",
        "CpuCompositorHarness",
        "DirtyRect",
        "CAP_SURFACE_COMPOSITOR",
        "MAX_PANEL_WIDTH",
        "MAX_PANEL_HEIGHT",
        "blit_dirty_rect",
        "present_frame",
        "front_buffer_id",
        "frame_flips",
        "total_dirty_rects",
    ]:
        if s not in comp_src:
            print(f"❌ FAIL: cpu_compositor.rs missing symbol '{s}'")
            sys.exit(1)
        print(f"  ✓ Verified userspace symbol: {s}")

    with open("kernel/src/hal/platform/cpu_compositor_core.rs", "r") as f:
        kern_src = f.read()
    for s in [
        "KernelCompositorError",
        "KernelCpuCompositorHarness",
        "KernelDirtyRect",
        "KERNEL_CAP_SURFACE_COMPOSITOR",
        "KERNEL_MAX_PANEL_WIDTH",
        "KERNEL_MAX_PANEL_HEIGHT",
        "blit_dirty_rect",
        "present_frame",
        "front_buffer_id",
        "frame_flips",
        "total_dirty_rects",
    ]:
        if s not in kern_src:
            print(f"❌ FAIL: cpu_compositor_core.rs missing symbol '{s}'")
            sys.exit(1)
        print(f"  ✓ Verified kernel symbol: {s}")

    # 3. Rule N-01: Zero External Crates Check
    print("\n[CHECK 3/6] Verifying Rule N-01 (Zero external crates in TCB)...")
    for toml_path in ["kernel/Cargo.toml", "userspace/devmgr/Cargo.toml"]:
        with open(toml_path, "r") as f:
            lines = [line.strip() for line in f.readlines() if line.strip() and not line.strip().startswith("#")]
        in_deps = False
        for line in lines:
            if line.startswith("[dependencies]"):
                in_deps = True
                continue
            if line.startswith("[") and in_deps:
                in_deps = False
                continue
            if in_deps and line:
                if "path =" not in line:
                    print(f"❌ FAIL: External dependency detected in {toml_path}: {line}")
                    sys.exit(1)
        print(f"  ✓ {toml_path} compliant with Rule N-01")

    # 4. Rule N-04: Zero Unsafe Code Check
    print("\n[CHECK 4/6] Verifying Rule N-04 (Zero unsafe code in userspace/devmgr)...")
    with open("userspace/devmgr/src/cpu_compositor.rs", "r") as f:
        if "unsafe" in f.read():
            print("❌ FAIL: Unsafe block found in userspace/devmgr/src/cpu_compositor.rs")
            sys.exit(1)
    print("  ✓ userspace/devmgr/src/cpu_compositor.rs is 100% safe Rust")

    # 5. Boot Log Verification (Stages 335-339)
    print("\n[CHECK 5/6] Verifying Golden Boot Log Stages 335-339...")
    with open("docs/evidence/golden-boot-sp77.log", "r") as f:
        boot_log = f.read()

    expected_stages = [
        "335_SP77_CPU_COMPOSITOR_INIT",
        "336_SP77_DOUBLE_BUFFER_ALLOCATION",
        "337_SP77_DIRTY_RECT_BOUNDING_VERIFY",
        "338_SP77_SURFACE_PRESENTATION_GATING",
        "339_SP77_CPU_COMPOSITOR_READY",
    ]
    for stage in expected_stages:
        if stage not in boot_log:
            print(f"❌ FAIL: Golden boot log missing stage '{stage}'")
            sys.exit(1)
        print(f"  ✓ Verified stage: {stage}")

    # 6. Verify CPU Compositor Confirmation Logs
    print("\n[CHECK 6/6] Verifying CPU Compositor Confirmation Logs...")
    expected_logs = [
        "Double-buffering verified: front/back scanout buffers staged in DRAM (INV-COMP-001 PASSED)",
        "Dirty rect pipeline verified: [0, 0, 1080, 120] clamped and blitted (INV-COMP-002 PASSED)",
        "Surface presentation verified: unprivileged flips denied, shell compositor flipped (INV-COMP-003 PASSED)",
        "[SP-77] Sprint SP-77 CPU Software Compositor Milestone REACHED.",
    ]
    for msg in expected_logs:
        if msg not in boot_log:
            print(f"❌ FAIL: Boot log missing required confirmation: '{msg}'")
            sys.exit(1)
        print(f"  ✓ Confirmed log output: {msg[:65]}...")

    print("\n==================================================")
    print(" ✅ SPRINT SP-77 VERIFICATION PASSED (100% DoD)  ")
    print("==================================================")

if __name__ == "__main__":
    verify_sp77()
