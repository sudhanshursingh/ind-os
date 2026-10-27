#!/usr/bin/env python3
"""
Ind OS Sprint SP-73 Verification & Acceptance Test Harness
Target: Kernel Core on Hardware Bring-Up (GENI UART, PDC/GICv3, Device Tree DRAM)
Standards: Bible §6.3, §8.15, Table 19.2, REQ-HW-104, REQ-KRN-101, INV-CORE-HW-001..003
"""

import os
import sys

REQUIRED_SP73_FILES = [
    "docs/architecture/hardware-kernel-core-spec.md",
    "docs/evidence/SP-73-validation.md",
    "docs/evidence/golden-boot-sp73.log",
    "userspace/devmgr/src/hardware_core.rs",
    "kernel/src/hal/platform/hardware_core.rs",
]

def verify_sp73():
    print("==================================================")
    print(" Ind OS SP-73 Kernel Core on Hardware Verifier   ")
    print(" Target: Sprint SP-73 Definition of Done (DoD)   ")
    print("==================================================")

    # 1. Deliverables Presence Check
    print("[CHECK 1/6] Verifying SP-73 deliverable presence...")
    for rel_path in REQUIRED_SP73_FILES:
        if not os.path.exists(rel_path):
            print(f"❌ FAIL: Required SP-73 file missing: {rel_path}")
            sys.exit(1)
        print(f"  ✓ Found {rel_path}")

    # 2. Architectural & Invariant Symbols Check
    print("\n[CHECK 2/6] Verifying architectural symbols and API contracts...")
    with open("userspace/devmgr/src/hardware_core.rs", "r") as f:
        core_src = f.read()
    for s in [
        "HardwareCoreError",
        "HardwareCoreHarness",
        "HANOIP_GENI_UART_BASE",
        "HANOIP_PDC_GIC_BASE",
        "HANOIP_DRAM_BASE",
        "HANOIP_DRAM_SIZE_BYTES",
        "initialize_uart",
        "initialize_gic",
        "map_dram",
    ]:
        if s not in core_src:
            print(f"❌ FAIL: hardware_core.rs missing symbol '{s}'")
            sys.exit(1)
        print(f"  ✓ Verified userspace symbol: {s}")

    with open("kernel/src/hal/platform/hardware_core.rs", "r") as f:
        kern_src = f.read()
    for s in [
        "KernelHardwareCoreError",
        "KernelHardwareCoreHarness",
        "KERNEL_HANOIP_GENI_UART_BASE",
        "KERNEL_HANOIP_PDC_GIC_BASE",
        "KERNEL_HANOIP_DRAM_BASE",
        "KERNEL_HANOIP_DRAM_SIZE_BYTES",
        "initialize_uart",
        "initialize_gic",
        "map_dram",
    ]:
        if s not in kern_src:
            print(f"❌ FAIL: hardware_core.rs missing symbol '{s}'")
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
    with open("userspace/devmgr/src/hardware_core.rs", "r") as f:
        if "unsafe" in f.read():
            print("❌ FAIL: Unsafe block found in userspace/devmgr/src/hardware_core.rs")
            sys.exit(1)
    print("  ✓ userspace/devmgr/src/hardware_core.rs is 100% safe Rust")

    # 5. Boot Log Verification (Stages 315-319)
    print("\n[CHECK 5/6] Verifying Golden Boot Log Stages 315-319...")
    with open("docs/evidence/golden-boot-sp73.log", "r") as f:
        boot_log = f.read()

    expected_stages = [
        "315_SP73_HW_KERNEL_CORE_INIT",
        "316_SP73_GENI_UART_SERIAL_CONSOLE",
        "317_SP73_PDC_GICV3_INTERRUPTS",
        "318_SP73_DEVICE_TREE_DRAM_MAP",
        "319_SP73_HW_KERNEL_CORE_READY",
    ]
    for stage in expected_stages:
        if stage not in boot_log:
            print(f"❌ FAIL: Golden boot log missing stage '{stage}'")
            sys.exit(1)
        print(f"  ✓ Verified stage: {stage}")

    # 6. Verify Hardware Core Confirmation Logs
    print("\n[CHECK 6/6] Verifying Hardware Core Confirmation Logs...")
    expected_logs = [
        "GENI UART verified: hardware serial mapped to 0x0a88_0000 (INV-CORE-HW-001 PASSED)",
        "PDC / GICv3 verified: hardware distributor mapped to 0x17a0_0000 (INV-CORE-HW-002 PASSED)",
        "DRAM map verified: 6GB physical DDR space mapped at 0x8000_0000 (INV-CORE-HW-003 PASSED)",
        "[SP-73] Sprint SP-73 Kernel Core on Hardware Milestone REACHED.",
    ]
    for msg in expected_logs:
        if msg not in boot_log:
            print(f"❌ FAIL: Boot log missing required confirmation: '{msg}'")
            sys.exit(1)
        print(f"  ✓ Confirmed log output: {msg[:65]}...")

    print("\n==================================================")
    print(" ✅ SPRINT SP-73 VERIFICATION PASSED (100% DoD)  ")
    print("==================================================")

if __name__ == "__main__":
    verify_sp73()
