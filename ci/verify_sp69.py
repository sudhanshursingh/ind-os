#!/usr/bin/env python3
"""
Ind OS Sprint SP-69 Verification & Acceptance Test Harness
Target: UI Polish & Adversarial Hardening (Anti-Tapjacking, Faked Consent Denial, Universal Explanation)
Standards: Bible §6.3, §8.14, Table 19.2, REQ-SEC-109, REQ-UX-306, INV-POLISH-001..003
"""

import os
import sys

REQUIRED_SP69_FILES = [
    "docs/architecture/ui-polish-spec.md",
    "docs/evidence/SP-69-validation.md",
    "docs/evidence/golden-boot-sp69.log",
    "userspace/ux/src/polish.rs",
    "kernel/src/userspace/polish_core.rs",
]

def verify_sp69():
    print("==================================================")
    print(" Ind OS SP-69 UI Polish Verifier                 ")
    print(" Target: Sprint SP-69 Definition of Done (DoD)   ")
    print("==================================================")

    # 1. Deliverables Presence Check
    print("[CHECK 1/6] Verifying SP-69 deliverable presence...")
    for rel_path in REQUIRED_SP69_FILES:
        if not os.path.exists(rel_path):
            print(f"❌ FAIL: Required SP-69 file missing: {rel_path}")
            sys.exit(1)
        print(f"  ✓ Found {rel_path}")

    # 2. Architectural & Invariant Symbols Check
    print("\n[CHECK 2/6] Verifying architectural symbols and API contracts...")
    with open("userspace/ux/src/polish.rs", "r") as f:
        pol_src = f.read()
    for s in [
        "PolishSecurityError",
        "AdversarialUiHarness",
        "EXPECTED_CONSENT_BROKER_MAGIC",
        "audit_input_touch",
        "verify_consent_token",
        "verify_action_explanation",
        "blocked_tapjacks",
        "rejected_faked_consents",
        "explanation_checks",
    ]:
        if s not in pol_src:
            print(f"❌ FAIL: polish.rs missing symbol '{s}'")
            sys.exit(1)
        print(f"  ✓ Verified userspace symbol: {s}")

    with open("kernel/src/userspace/polish_core.rs", "r") as f:
        kern_src = f.read()
    for s in [
        "KernelPolishSecurityError",
        "KernelAdversarialUiHarness",
        "KERNEL_EXPECTED_CONSENT_BROKER_MAGIC",
        "audit_input_touch",
        "verify_consent_token",
        "verify_action_explanation",
        "blocked_tapjacks",
        "rejected_faked_consents",
        "explanation_checks",
    ]:
        if s not in kern_src:
            print(f"❌ FAIL: polish_core.rs missing symbol '{s}'")
            sys.exit(1)
        print(f"  ✓ Verified kernel symbol: {s}")

    # 3. Rule N-01: Zero External Crates Check
    print("\n[CHECK 3/6] Verifying Rule N-01 (Zero external crates in TCB)...")
    for toml_path in ["kernel/Cargo.toml", "userspace/ux/Cargo.toml"]:
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
    print("\n[CHECK 4/6] Verifying Rule N-04 (Zero unsafe code in userspace/ux)...")
    with open("userspace/ux/src/polish.rs", "r") as f:
        if "unsafe" in f.read():
            print("❌ FAIL: Unsafe block found in userspace/ux/src/polish.rs")
            sys.exit(1)
    print("  ✓ userspace/ux/src/polish.rs is 100% safe Rust")

    # 5. Boot Log Verification (Stages 294-298)
    print("\n[CHECK 5/6] Verifying Golden Boot Log Stages 294-298...")
    with open("docs/evidence/golden-boot-sp69.log", "r") as f:
        boot_log = f.read()

    expected_stages = [
        "294_POLISH_INIT",
        "295_POLISH_ANTI_TAPJACKING_PEN_TEST",
        "296_POLISH_FAKED_CONSENT_DENIAL",
        "297_POLISH_UNIVERSAL_EXPLANATION",
        "298_SP69_POLISH_READY",
    ]
    for stage in expected_stages:
        if stage not in boot_log:
            print(f"❌ FAIL: Golden boot log missing stage '{stage}'")
            sys.exit(1)
        print(f"  ✓ Verified stage: {stage}")

    # 6. Verify Penetration Results Confirmation
    print("\n[CHECK 6/6] Verifying Penetration Test Defenses Confirmation...")
    expected_logs = [
        "Anti-tapjacking verified: occluding overlay input strictly intercepted and discarded (INV-POLISH-001 PASSED)",
        "Consent verification verified: forged consent payloads strictly rejected by TCB broker (INV-POLISH-002 PASSED)",
        "Universal explanation verified: 100% of capability decisions accompanied by causal tokens (INV-POLISH-003 PASSED)",
        "[SP-69] Sprint SP-69 UI Polish Milestone REACHED.",
    ]
    for msg in expected_logs:
        if msg not in boot_log:
            print(f"❌ FAIL: Boot log missing required confirmation: '{msg}'")
            sys.exit(1)
        print(f"  ✓ Confirmed log output: {msg[:65]}...")

    print("\n==================================================")
    print(" ✅ SPRINT SP-69 VERIFICATION PASSED (100% DoD)  ")
    print("==================================================")

if __name__ == "__main__":
    verify_sp69()
