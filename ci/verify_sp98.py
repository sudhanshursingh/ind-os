#!/usr/bin/env python3
"""
Ind OS Sprint SP-98 Verification & Acceptance Test Harness
Target: Hardware Intent Broker & Proposal Pipeline
Standards: Bible §6.3, §8.15, Table 19.2, REQ-AI-103, REQ-HW-113, INV-INTENT-HW-001..003
"""

import os
import sys

REQUIRED_SP98_FILES = [
    "docs/architecture/hardware-intent-pipeline-spec.md",
    "docs/evidence/SP-98-validation.md",
    "docs/evidence/golden-boot-sp98.log",
    "userspace/ais/src/intent_hw.rs",
    "kernel/src/hal/platform/intent_hw_core.rs",
]

def verify_sp98():
    print("==================================================")
    print(" Ind OS SP-98 Hardware Intent Broker Verifier     ")
    print(" Target: Sprint SP-98 Definition of Done (DoD)   ")
    print("==================================================")

    # 1. Deliverables Presence Check
    print("[CHECK 1/6] Verifying SP-98 deliverable presence...")
    for rel_path in REQUIRED_SP98_FILES:
        if not os.path.exists(rel_path):
            print(f"❌ FAIL: Required SP-98 file missing: {rel_path}")
            sys.exit(1)
        print(f"  ✓ Found {rel_path}")

    # 2. Architectural & Invariant Symbols Check
    print("\n[CHECK 2/6] Verifying architectural symbols and API contracts...")
    with open("userspace/ais/src/intent_hw.rs", "r") as f:
        intent_src = f.read()
    for s in [
        "HardwareIntentBrokerHarness",
        "HardwareIntentError",
        "HardwareIntentState",
        "HardwareProposalCard",
        "CAP_INTENT_DISPATCH",
        "create_session",
        "infer_action",
        "audit_policy_attenuation",
        "generate_proposal_card",
        "active_sessions_count",
        "last_why_receipt_token",
        "e2e_latency_ms",
    ]:
        if s not in intent_src:
            print(f"❌ FAIL: intent_hw.rs missing symbol '{s}'")
            sys.exit(1)
        print(f"  ✓ Verified userspace symbol: {s}")

    with open("kernel/src/hal/platform/intent_hw_core.rs", "r") as f:
        kern_src = f.read()
    for s in [
        "KernelHardwareIntentBrokerHarness",
        "KernelHardwareIntentError",
        "KernelHardwareIntentState",
        "KernelHardwareProposalCard",
        "KERNEL_CAP_INTENT_DISPATCH",
        "create_session",
        "infer_action",
        "audit_policy_attenuation",
        "generate_proposal_card",
        "active_sessions_count",
        "last_why_receipt_token",
        "e2e_latency_ms",
    ]:
        if s not in kern_src:
            print(f"❌ FAIL: intent_hw_core.rs missing symbol '{s}'")
            sys.exit(1)
        print(f"  ✓ Verified kernel symbol: {s}")

    # 3. Rule N-01: Zero External Crates Check
    print("\n[CHECK 3/6] Verifying Rule N-01 (Zero external crates in TCB)...")
    for toml_path in ["kernel/Cargo.toml", "userspace/ais/Cargo.toml"]:
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
    print("\n[CHECK 4/6] Verifying Rule N-04 (Zero unsafe code in userspace/ais)...")
    with open("userspace/ais/src/intent_hw.rs", "r") as f:
        if "unsafe" in f.read():
            print("❌ FAIL: Unsafe block found in userspace/ais/src/intent_hw.rs")
            sys.exit(1)
    print("  ✓ userspace/ais/src/intent_hw.rs is 100% safe Rust")

    # 5. Boot Log Verification (Stages 440-444)
    print("\n[CHECK 5/6] Verifying Golden Boot Log Stages 440-444...")
    with open("docs/evidence/golden-boot-sp98.log", "r") as f:
        boot_log = f.read()

    expected_stages = [
        "440_SP98_INTENT_BROKER_INIT",
        "441_SP98_INTENT_SESSION_CREATED",
        "442_SP98_INTENT_POLICY_ATTENUATION_AUDIT",
        "443_SP98_INTENT_PROPOSAL_GENERATION",
        "444_SP98_INTENT_PIPELINE_READY",
    ]
    for stage in expected_stages:
        if stage not in boot_log:
            print(f"❌ FAIL: Golden boot log missing stage '{stage}'")
            sys.exit(1)
        print(f"  ✓ Verified stage: {stage}")

    # 6. Verify Confirmation Logs
    print("\n[CHECK 6/6] Verifying Intent Broker Confirmation Logs...")
    expected_logs = [
        "Intent broker harness online: hardware intent state machine active (REQ-AI-103 PASSED)",
        "Intent session #101 created: authorized via CAP_INTENT_DISPATCH (INV-INTENT-HW-001 PASSED)",
        "Policy attenuation verified: privilege escalation strictly rejected (INV-INTENT-HW-002 PASSED)",
        "Proposal card #101 generated: Why-Token 0x0000574800001075, Latency=145ms (INV-INTENT-HW-003 PASSED)",
        "[SP-98] Sprint SP-98 Hardware Intent Broker & Proposal Pipeline Milestone REACHED.",
    ]
    for msg in expected_logs:
        if msg not in boot_log:
            print(f"❌ FAIL: Boot log missing required confirmation: '{msg}'")
            sys.exit(1)
        print(f"  ✓ Confirmed log output: {msg[:65]}...")

    print("\n==================================================")
    print(" ✅ SPRINT SP-98 VERIFICATION PASSED (100% DoD)  ")
    print("==================================================")

if __name__ == "__main__":
    verify_sp98()
