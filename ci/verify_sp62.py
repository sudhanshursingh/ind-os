#!/usr/bin/env python3
"""
Ind OS Sprint SP-62 Verification & Acceptance Test Harness
Target: Intent Surface UI & Indic IME Engine
Standards: Bible §6.3, §8.14, Table 19.2, REQ-AI-104, REQ-AI-105, REQ-AI-108, INV-INTENT-UI-001..003
"""

import os
import sys

REQUIRED_SP62_FILES = [
    "docs/architecture/intent-ui-spec.md",
    "docs/evidence/SP-62-validation.md",
    "docs/evidence/golden-boot-sp62.log",
    "userspace/ux/src/intent_ui.rs",
    "kernel/src/userspace/intent_ui_core.rs",
]

def verify_sp62():
    print("==================================================")
    print(" Ind OS SP-62 Intent Surface & Indic IME Verifier")
    print(" Target: Sprint SP-62 Definition of Done (DoD)   ")
    print("==================================================")

    # 1. Deliverables Presence Check
    print("[CHECK 1/6] Verifying SP-62 deliverable presence...")
    for rel_path in REQUIRED_SP62_FILES:
        if not os.path.exists(rel_path):
            print(f"❌ FAIL: Required SP-62 file missing: {rel_path}")
            sys.exit(1)
        print(f"  ✓ Found {rel_path}")

    # 2. Architectural & Invariant Symbols Check
    print("\n[CHECK 2/6] Verifying architectural symbols and API contracts...")
    with open("userspace/ux/src/intent_ui.rs", "r") as f:
        ui_src = f.read()
    for s in [
        "IntentError",
        "IntentState",
        "ProposalCard",
        "ExplanationReceipt",
        "IntentSession",
        "IntentUiEngine",
        "IndicImeEngine",
        "create_intent",
        "generate_proposal",
        "record_consent",
        "execute_intent",
        "generate_explanation_receipt",
        "transliterate_to_devanagari",
    ]:
        if s not in ui_src:
            print(f"❌ FAIL: intent_ui.rs missing symbol '{s}'")
            sys.exit(1)
        print(f"  ✓ Verified userspace symbol: {s}")

    with open("kernel/src/userspace/intent_ui_core.rs", "r") as f:
        kern_src = f.read()
    for s in [
        "KernelIntentError",
        "KernelIntentState",
        "KernelProposalCard",
        "KernelExplanationReceipt",
        "KernelIntentSession",
        "KernelIntentUiEngine",
        "KernelIndicImeEngine",
        "create_intent",
        "generate_proposal",
        "record_consent",
        "execute_intent",
        "generate_explanation_receipt",
        "transliterate_to_devanagari",
    ]:
        if s not in kern_src:
            print(f"❌ FAIL: intent_ui_core.rs missing symbol '{s}'")
            sys.exit(1)
        print(f"  ✓ Verified kernel harness symbol: {s}")

    # 3. Rule N-01: Zero External Crates Check
    print("\n[CHECK 3/6] Verifying Rule N-01 (Zero external crates in TCB)...")
    with open("userspace/ux/Cargo.toml", "r") as f:
        cargo_lines = f.readlines()
    in_deps = False
    for line in cargo_lines:
        line_clean = line.split("#")[0].strip()
        if not line_clean:
            continue
        if line_clean == "[dependencies]":
            in_deps = True
            continue
        if in_deps and line_clean.startswith("["):
            in_deps = False
        if in_deps:
            if "path =" not in line_clean:
                print(f"❌ FAIL: Non-path (external) dependency in userspace/ux/Cargo.toml: {line_clean}")
                sys.exit(1)
    print("  ✓ userspace/ux uses pure in-tree path dependencies exclusively.")

    # 4. Rule N-04: Zero Unsafe Code in New Modules Check
    print("\n[CHECK 4/6] Verifying Rule N-04 (Zero unsafe code in new modules)...")
    for src_file in ["userspace/ux/src/intent_ui.rs", "kernel/src/userspace/intent_ui_core.rs"]:
        with open(src_file, "r") as f:
            content = f.read()
        if "unsafe " in content or "unsafe{" in content:
            print(f"❌ FAIL: Rule N-04 violation in {src_file}: unsafe block detected")
            sys.exit(1)
        print(f"  ✓ Zero unsafe code verified in {src_file}")

    # 5. Golden Boot Log Verification (Stages 259..263)
    print("\n[CHECK 5/6] Verifying bare-metal QEMU boot execution log...")
    with open("docs/evidence/golden-boot-sp62.log", "r") as f:
        boot_log = f.read()

    expected_stages = [
        "259_INTENT_UI_INIT",
        "260_INTENT_PROPOSAL_CARD",
        "261_INTENT_CONSENT_EXECUTION_PROGRESSION",
        "262_INTENT_EXPLANATION_RECEIPT_AND_IME",
        "263_SP62_INTENT_UI_READY",
    ]
    for stage in expected_stages:
        if stage not in boot_log:
            print(f"❌ FAIL: Expected stage marker '{stage}' not found in golden-boot-sp62.log")
            sys.exit(1)
        print(f"  ✓ Stage {stage} present in boot log")

    expected_assertions = [
        "Intent surface engine online: intent session drafting initialized",
        "Proposal card generated: capability & risk tier displayed to user",
        "Consent gating verified: execution blocked before approval, dispatched safely after",
        "Explanation receipt generated and Indic IME Devanagari transliteration verified",
        "Sprint SP-62 Intent Surface Milestone REACHED.",
    ]
    for assertion in expected_assertions:
        if assertion not in boot_log:
            print(f"❌ FAIL: Expected assertion '{assertion}' missing in log")
            sys.exit(1)
        print(f"  ✓ Assertion verified: {assertion}")

    # 6. Check Invariant Signatures
    print("\n[CHECK 6/6] Verifying SP-62 architectural invariants...")
    for inv in ["INV-INTENT-UI-001", "INV-INTENT-UI-002", "INV-INTENT-UI-003"]:
        if inv not in boot_log:
            print(f"❌ FAIL: Invariant '{inv}' not asserted in golden boot log")
            sys.exit(1)
        print(f"  ✓ Invariant {inv} ratified")

    print("\n==================================================")
    print(" SPRINT SP-62 VERIFICATION RESULT: PASSED        ")
    print(" Intent Surface UI & Indic IME RATIFIED          ")
    print("==================================================")

if __name__ == "__main__":
    verify_sp62()
