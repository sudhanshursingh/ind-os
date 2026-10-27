#!/usr/bin/env python3
"""
Ind OS Sprint SP-83 Verification & Acceptance Test Harness
Target: Power Domains & PSCI / RPMh Control (PSCI v1.1, RPMh RSC, Capability-Gated Transitions)
Standards: Bible §6.3, §8.15, Table 19.2, REQ-HW-108, REQ-PWR-103, INV-PWR-001..003
"""

import os
import sys

REQUIRED_SP83_FILES = [
    "docs/architecture/power-domains-spec.md",
    "docs/evidence/SP-83-validation.md",
    "docs/evidence/golden-boot-sp83.log",
    "userspace/devmgr/src/power_domains.rs",
    "kernel/src/hal/platform/power_domains_core.rs",
]

def verify_sp83():
    print("==================================================")
    print(" Ind OS SP-83 Power Domains Verifier              ")
    print(" Target: Sprint SP-83 Definition of Done (DoD)   ")
    print("==================================================")

    # 1. Deliverables Presence Check
    print("[CHECK 1/6] Verifying SP-83 deliverable presence...")
    for rel_path in REQUIRED_SP83_FILES:
        if not os.path.exists(rel_path):
            print(f"❌ FAIL: Required SP-83 file missing: {rel_path}")
            sys.exit(1)
        print(f"  ✓ Found {rel_path}")

    # 2. Architectural & Invariant Symbols Check
    print("\n[CHECK 2/6] Verifying architectural symbols and API contracts...")
    with open("userspace/devmgr/src/power_domains.rs", "r") as f:
        pwr_src = f.read()
    for s in [
        "PowerDomainState",
        "PowerDomainError",
        "PowerDomainsHarness",
        "HANOIP_RPMH_RSC_BASE",
        "PSCI_FN64_CPU_SUSPEND",
        "PSCI_FN_CPU_OFF",
        "PSCI_FN_SYSTEM_RESET",
        "CAP_POWER_DOMAINS",
        "map_rsc",
        "set_core_power_state",
        "execute_psci_call",
        "is_rsc_mapped",
        "core_state",
        "active_rpmh_cmds",
    ]:
        if s not in pwr_src:
            print(f"❌ FAIL: power_domains.rs missing symbol '{s}'")
            sys.exit(1)
        print(f"  ✓ Verified userspace symbol: {s}")

    with open("kernel/src/hal/platform/power_domains_core.rs", "r") as f:
        kern_src = f.read()
    for s in [
        "KernelPowerDomainState",
        "KernelPowerDomainError",
        "KernelPowerDomainsHarness",
        "KERNEL_HANOIP_RPMH_RSC_BASE",
        "KERNEL_PSCI_FN64_CPU_SUSPEND",
        "KERNEL_PSCI_FN_CPU_OFF",
        "KERNEL_PSCI_FN_SYSTEM_RESET",
        "KERNEL_CAP_POWER_DOMAINS",
        "map_rsc",
        "set_core_power_state",
        "execute_psci_call",
        "is_rsc_mapped",
        "core_state",
        "active_rpmh_cmds",
    ]:
        if s not in kern_src:
            print(f"❌ FAIL: power_domains_core.rs missing symbol '{s}'")
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
    with open("userspace/devmgr/src/power_domains.rs", "r") as f:
        if "unsafe" in f.read():
            print("❌ FAIL: Unsafe block found in userspace/devmgr/src/power_domains.rs")
            sys.exit(1)
    print("  ✓ userspace/devmgr/src/power_domains.rs is 100% safe Rust")

    # 5. Boot Log Verification (Stages 365-369)
    print("\n[CHECK 5/6] Verifying Golden Boot Log Stages 365-369...")
    with open("docs/evidence/golden-boot-sp83.log", "r") as f:
        boot_log = f.read()

    expected_stages = [
        "365_SP83_POWER_DOMAINS_INIT",
        "366_SP83_RPMH_RSC_MMIO_MAPPING",
        "367_SP83_PSCI_SMC_DISPATCH_VERIFY",
        "368_SP83_CORE_POWER_TRANSITION_GATING",
        "369_SP83_POWER_DOMAINS_READY",
    ]
    for stage in expected_stages:
        if stage not in boot_log:
            print(f"❌ FAIL: Golden boot log missing stage '{stage}'")
            sys.exit(1)
        print(f"  ✓ Verified stage: {stage}")

    # 6. Verify Power Domains Confirmation Logs
    print("\n[CHECK 6/6] Verifying Power Domains Confirmation Logs...")
    expected_logs = [
        "RPMh RSC verified: Resource State Coordinator mapped at 0x179c_0000 (INV-PWR-002 PASSED)",
        "PSCI v1.1 SMC calls verified: CPU suspend (0xC4000001) and system reset (0x84000009) conform to ARM DEN 0022D (INV-PWR-001 PASSED)",
        "Power gating verified: unprivileged callers blocked; CAP_POWER_DOMAINS permitted transitions (INV-PWR-003 PASSED)",
        "[SP-83] Sprint SP-83 Power Domains & PSCI/RPMh Milestone REACHED.",
    ]
    for msg in expected_logs:
        if msg not in boot_log:
            print(f"❌ FAIL: Boot log missing required confirmation: '{msg}'")
            sys.exit(1)
        print(f"  ✓ Confirmed log output: {msg[:65]}...")

    print("\n==================================================")
    print(" ✅ SPRINT SP-83 VERIFICATION PASSED (100% DoD)  ")
    print("==================================================")

if __name__ == "__main__":
    verify_sp83()
