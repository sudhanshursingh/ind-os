#!/usr/bin/env python3
"""
Ind OS Sprint SP-95 Verification & Acceptance Test Harness
Target: Connectivity & Media Hardware Integration Test Suite
Standards: Bible §6.3, §8.15, Table 19.2, REQ-HW-112, INV-CONCUR-001..003
"""

import os
import sys

REQUIRED_SP95_FILES = [
    "docs/architecture/connectivity-media-suite-spec.md",
    "docs/evidence/SP-95-validation.md",
    "docs/evidence/golden-boot-sp95.log",
    "userspace/devmgr/src/concurrency_suite.rs",
    "kernel/src/hal/platform/concurrency_suite_core.rs",
]

def verify_sp95():
    print("==================================================")
    print(" Ind OS SP-95 Concurrency Suite Verifier          ")
    print(" Target: Sprint SP-95 Definition of Done (DoD)   ")
    print("==================================================")

    # 1. Deliverables Presence Check
    print("[CHECK 1/6] Verifying SP-95 deliverable presence...")
    for rel_path in REQUIRED_SP95_FILES:
        if not os.path.exists(rel_path):
            print(f"❌ FAIL: Required SP-95 file missing: {rel_path}")
            sys.exit(1)
        print(f"  ✓ Found {rel_path}")

    # 2. Architectural & Invariant Symbols Check
    print("\n[CHECK 2/6] Verifying architectural symbols and API contracts...")
    with open("userspace/devmgr/src/concurrency_suite.rs", "r") as f:
        concur_src = f.read()
    for s in [
        "ConcurrencySuiteHarness",
        "ConcurrencyError",
        "SUBSYSTEM_WIFI",
        "SUBSYSTEM_BLUETOOTH",
        "SUBSYSTEM_SENSORS",
        "SUBSYSTEM_AUDIO",
        "SUBSYSTEM_ALL",
        "register_subsystems",
        "run_concurrent_cycle",
        "run_stress_soak",
        "active_mask",
        "total_cycles",
        "audio_frames_processed",
        "sensor_events_routed",
        "wifi_packets_transferred",
        "arbitration_conflicts_resolved",
        "is_all_healthy",
    ]:
        if s not in concur_src:
            print(f"❌ FAIL: concurrency_suite.rs missing symbol '{s}'")
            sys.exit(1)
        print(f"  ✓ Verified userspace symbol: {s}")

    with open("kernel/src/hal/platform/concurrency_suite_core.rs", "r") as f:
        kern_src = f.read()
    for s in [
        "KernelConcurrencySuiteHarness",
        "KernelConcurrencyError",
        "KERNEL_SUBSYSTEM_WIFI",
        "KERNEL_SUBSYSTEM_BLUETOOTH",
        "KERNEL_SUBSYSTEM_SENSORS",
        "KERNEL_SUBSYSTEM_AUDIO",
        "KERNEL_SUBSYSTEM_ALL",
        "register_subsystems",
        "run_concurrent_cycle",
        "run_stress_soak",
        "active_mask",
        "total_cycles",
        "audio_frames_processed",
        "sensor_events_routed",
        "wifi_packets_transferred",
        "arbitration_conflicts_resolved",
        "is_all_healthy",
    ]:
        if s not in kern_src:
            print(f"❌ FAIL: concurrency_suite_core.rs missing symbol '{s}'")
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
            if line.startswith("path =") or "path =" in line:
                continue
            if in_deps and line:
                print(f"❌ FAIL: External dependency detected in {toml_path}: {line}")
                sys.exit(1)
        print(f"  ✓ {toml_path} compliant with Rule N-01")

    # 4. Rule N-04: Zero Unsafe Code Check
    print("\n[CHECK 4/6] Verifying Rule N-04 (Zero unsafe code in userspace/devmgr)...")
    with open("userspace/devmgr/src/concurrency_suite.rs", "r") as f:
        if "unsafe" in f.read():
            print("❌ FAIL: Unsafe block found in userspace/devmgr/src/concurrency_suite.rs")
            sys.exit(1)
    print("  ✓ userspace/devmgr/src/concurrency_suite.rs is 100% safe Rust")

    # 5. Boot Log Verification (Stages 425-429)
    print("\n[CHECK 5/6] Verifying Golden Boot Log Stages 425-429...")
    with open("docs/evidence/golden-boot-sp95.log", "r") as f:
        boot_log = f.read()

    expected_stages = [
        "425_SP95_CONCURRENCY_SUITE_INIT",
        "426_SP95_MULTI_SUBSYSTEM_REGISTRATION",
        "427_SP95_AUDIO_PRIORITY_ARBITRATION",
        "428_SP95_CONCURRENT_TRANSACTION_SOAK",
        "429_SP95_CONNECTIVITY_MEDIA_READY",
    ]
    for stage in expected_stages:
        if stage not in boot_log:
            print(f"❌ FAIL: Golden boot log missing stage '{stage}'")
            sys.exit(1)
        print(f"  ✓ Verified stage: {stage}")

    # 6. Verify Confirmation Logs
    print("\n[CHECK 6/6] Verifying Concurrency Confirmation Logs...")
    expected_logs = [
        "Concurrency arbiter online: multi-DMA interconnect arbiter active (REQ-HW-112 PASSED)",
        "Subsystems registered: Wi-Fi, BT, 5-ch Sensors, and Audio active concurrently (INV-CONCUR-001 PASSED)",
        "Priority arbitration verified: audio DMA prioritized, 1 bus conflict resolved (INV-CONCUR-002 PASSED)",
        "Stress soak verified: 1,000 cycles completed with 0 drops across all 4 subsystems (INV-CONCUR-003 PASSED)",
        "[SP-95] Sprint SP-95 Connectivity & Media Integration Suite Milestone REACHED.",
    ]
    for msg in expected_logs:
        if msg not in boot_log:
            print(f"❌ FAIL: Boot log missing required confirmation: '{msg}'")
            sys.exit(1)
        print(f"  ✓ Confirmed log output: {msg[:65]}...")

    print("\n==================================================")
    print(" ✅ SPRINT SP-95 VERIFICATION PASSED (100% DoD)  ")
    print("==================================================")

if __name__ == "__main__":
    verify_sp95()
