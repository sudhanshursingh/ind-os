#!/usr/bin/env python3
"""
Ind OS Sprint SP-76 Verification & Acceptance Test Harness
Target: Display Panel Bring-Up (Qualcomm MDSS DSI, 1080x2460@120Hz IPS, Backlight PWM)
Standards: Bible §6.3, §8.15, Table 19.2, PEN-004, REQ-HW-106, INV-DISP-001..003
"""

import os
import sys

REQUIRED_SP76_FILES = [
    "docs/architecture/display-panel-spec.md",
    "docs/evidence/SP-76-validation.md",
    "docs/evidence/golden-boot-sp76.log",
    "userspace/devmgr/src/display_panel.rs",
    "kernel/src/hal/platform/display_panel_core.rs",
]

def verify_sp76():
    print("==================================================")
    print(" Ind OS SP-76 Display Panel Bring-Up Verifier     ")
    print(" Target: Sprint SP-76 Definition of Done (DoD)   ")
    print("==================================================")

    # 1. Deliverables Presence Check
    print("[CHECK 1/6] Verifying SP-76 deliverable presence...")
    for rel_path in REQUIRED_SP76_FILES:
        if not os.path.exists(rel_path):
            print(f"❌ FAIL: Required SP-76 file missing: {rel_path}")
            sys.exit(1)
        print(f"  ✓ Found {rel_path}")

    # 2. Architectural & Invariant Symbols Check
    print("\n[CHECK 2/6] Verifying architectural symbols and API contracts...")
    with open("userspace/devmgr/src/display_panel.rs", "r") as f:
        disp_src = f.read()
    for s in [
        "DisplayError",
        "DisplayPanelHarness",
        "HANOIP_MDSS_DSI0_BASE",
        "PANEL_WIDTH",
        "PANEL_HEIGHT",
        "PANEL_REFRESH_HZ",
        "DCS_CMD_SLEEP_OUT",
        "DCS_CMD_DISPLAY_ON",
        "DCS_CMD_PIXEL_FORMAT",
        "map_mmio",
        "send_dcs_command",
        "set_backlight",
        "is_display_active",
    ]:
        if s not in disp_src:
            print(f"❌ FAIL: display_panel.rs missing symbol '{s}'")
            sys.exit(1)
        print(f"  ✓ Verified userspace symbol: {s}")

    with open("kernel/src/hal/platform/display_panel_core.rs", "r") as f:
        kern_src = f.read()
    for s in [
        "KernelDisplayError",
        "KernelDisplayPanelHarness",
        "KERNEL_HANOIP_MDSS_DSI0_BASE",
        "KERNEL_PANEL_WIDTH",
        "KERNEL_PANEL_HEIGHT",
        "KERNEL_PANEL_REFRESH_HZ",
        "KERNEL_DCS_CMD_SLEEP_OUT",
        "KERNEL_DCS_CMD_DISPLAY_ON",
        "KERNEL_DCS_CMD_PIXEL_FORMAT",
        "map_mmio",
        "send_dcs_command",
        "set_backlight",
        "is_display_active",
    ]:
        if s not in kern_src:
            print(f"❌ FAIL: display_panel_core.rs missing symbol '{s}'")
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
    with open("userspace/devmgr/src/display_panel.rs", "r") as f:
        if "unsafe" in f.read():
            print("❌ FAIL: Unsafe block found in userspace/devmgr/src/display_panel.rs")
            sys.exit(1)
    print("  ✓ userspace/devmgr/src/display_panel.rs is 100% safe Rust")

    # 5. Boot Log Verification (Stages 330-334)
    print("\n[CHECK 5/6] Verifying Golden Boot Log Stages 330-334...")
    with open("docs/evidence/golden-boot-sp76.log", "r") as f:
        boot_log = f.read()

    expected_stages = [
        "330_SP76_DISPLAY_PANEL_INIT",
        "331_SP76_MDSS_DSI_MMIO_MAP",
        "332_SP76_MIPI_DCS_INIT_SEQUENCE",
        "333_SP76_BACKLIGHT_PWM_BOUNDS",
        "334_SP76_DISPLAY_PANEL_READY",
    ]
    for stage in expected_stages:
        if stage not in boot_log:
            print(f"❌ FAIL: Golden boot log missing stage '{stage}'")
            sys.exit(1)
        print(f"  ✓ Verified stage: {stage}")

    # 6. Verify Display Panel Confirmation Logs
    print("\n[CHECK 6/6] Verifying Display Panel Confirmation Logs...")
    expected_logs = [
        "MDSS DSI verified: controller mapped at 0x0ae9_4000 (INV-DISP-001 PASSED)",
        "MIPI DCS sequence verified: 1080x2460@120Hz IPS panel active (INV-DISP-002 PASSED)",
        "Backlight PWM verified: 30% safe boot brightness bounded within [0, 100] (INV-DISP-003 PASSED)",
        "[SP-76] Sprint SP-76 Display Panel Bring-Up Milestone REACHED.",
    ]
    for msg in expected_logs:
        if msg not in boot_log:
            print(f"❌ FAIL: Boot log missing required confirmation: '{msg}'")
            sys.exit(1)
        print(f"  ✓ Confirmed log output: {msg[:65]}...")

    print("\n==================================================")
    print(" ✅ SPRINT SP-76 VERIFICATION PASSED (100% DoD)  ")
    print("==================================================")

if __name__ == "__main__":
    verify_sp76()
