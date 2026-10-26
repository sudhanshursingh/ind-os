#!/usr/bin/env python3
"""
Sprint SP-19 Verification Harness (Policy Engine v1 / ENF: Declarative Rule Language,
Default-Deny Evaluation, Deterministic Precedence, and Dual-Check Capability Enforcement).
Ratified under INDOS-BIBLE-001 / Phase 3 The Security Spine (§6.3, §8.9, §10, §14) and INDOS-SPEC-ENF-01.

Verifies:
1. Deliverable presence:
   - docs/architecture/policy-engine-spec.md
   - docs/evidence/SP-19-validation.md
   - docs/evidence/golden-boot-sp19.log
   - kernel/src/policy/rule.rs
   - kernel/src/policy/engine.rs
   - kernel/src/policy/mod.rs
2. Static architectural & invariant checks:
   - Invariant INV-ENF-001: Default-Deny Security Model.
   - Invariant INV-ENF-002: Deterministic Precedence & Versioned Rule Resolution.
   - Requirement REQ-SEC-001: Dual-Check Capability Enforcement (Token + Policy).
3. Unsafe-code budget audit: 100% pure safe Rust across all kernel/src/policy/ modules.
4. Clean kernel compilation with zero warnings/errors.
5. Golden boot log verification against SP-19 boot stages (Stages 45, 46, 47, 48).
6. Evidence manifest generation at evidence/SP-19/manifest.json.
"""

import sys
import os
import subprocess
import hashlib
import json
import time

# Ensure ~/.cargo/bin is in PATH for cargo invocations
cargo_bin = os.path.expanduser("~/.cargo/bin")
if cargo_bin not in os.environ.get("PATH", ""):
    os.environ["PATH"] = f"{cargo_bin}:{os.environ.get('PATH', '')}"

REQUIRED_SP19_FILES = [
    "docs/architecture/policy-engine-spec.md",
    "docs/evidence/SP-19-validation.md",
    "docs/evidence/golden-boot-sp19.log",
    "kernel/src/policy/rule.rs",
    "kernel/src/policy/engine.rs",
    "kernel/src/policy/mod.rs",
]

def verify_sp19():
    print("==================================================")
    print(" Ind OS SP-19 Policy Engine v1 Verifier          ")
    print(" Target: Sprint SP-19 Definition of Done (DoD)   ")
    print("==================================================")

    # 1. Deliverables Presence Check
    print("[CHECK 1/6] Verifying SP-19 deliverable presence...")
    for rel_path in REQUIRED_SP19_FILES:
        if not os.path.exists(rel_path):
            print(f"❌ FAIL: Required SP-19 file missing: {rel_path}")
            sys.exit(1)
        print(f"  ✓ Found {rel_path}")

    # 2. Static Architectural & Invariant Checks
    print("\n[CHECK 2/6] Verifying Policy Engine architecture & invariants...")
    with open("docs/architecture/policy-engine-spec.md", "r") as f:
        spec_src = f.read()
    for inv in ["INV-ENF-001", "INV-ENF-002", "REQ-SEC-001"]:
        if inv not in spec_src:
            print(f"❌ FAIL: Policy spec missing invariant '{inv}'")
            sys.exit(1)
    print("  ✓ Policy Engine invariants INV-ENF-001, INV-ENF-002, and REQ-SEC-001 documented in INDOS-SPEC-ENF-01")

    with open("kernel/src/policy/rule.rs", "r") as f:
        rule_src = f.read()
    for symbol in ["PolicyRule", "PolicyAction", "PolicyDecision", "parse_declaration", "Subject", "Resource"]:
        if symbol not in rule_src:
            print(f"❌ FAIL: rule.rs missing symbol '{symbol}'")
            sys.exit(1)
    print("  ✓ Declarative rule language grammar, parse_declaration, and rule types present in rule.rs")

    with open("kernel/src/policy/engine.rs", "r") as f:
        engine_src = f.read()
    for symbol in ["PolicyEngine", "evaluate", "check_capability_access", "load_declarations", "MAX_POLICY_RULES"]:
        if symbol not in engine_src:
            print(f"❌ FAIL: engine.rs missing symbol '{symbol}'")
            sys.exit(1)
    print("  ✓ PolicyEngine evaluation, dual-check capability enforcement, and ruleset loading present in engine.rs")

    with open("kernel/src/policy/mod.rs", "r") as f:
        mod_src = f.read()
    if "POLICY_ENGINE" not in mod_src:
        print("❌ FAIL: mod.rs missing global POLICY_ENGINE singleton")
        sys.exit(1)
    print("  ✓ Global POLICY_ENGINE singleton declared and re-exported in policy/mod.rs")

    # 3. Unsafe-Code Budget Scanner Check
    print("\n[CHECK 3/6] Auditing unsafe code in policy subsystem...")
    for path in ["kernel/src/policy/mod.rs", "kernel/src/policy/rule.rs", "kernel/src/policy/engine.rs"]:
        with open(path, "r") as f:
            content = f.read()
            if "unsafe " in content:
                print(f"❌ FAIL: Forbidden unsafe code detected in {path}")
                sys.exit(1)
    print("  ✓ 100% pure safe Rust in kernel/src/policy/ (Zero unsafe blocks)")

    # 4. Clean Kernel Compilation Check
    print("\n[CHECK 4/6] Verifying clean kernel compilation...")
    cmd = ["cargo", "check", "--target", "aarch64-unknown-none"]
    res = subprocess.run(cmd, cwd="kernel", capture_output=True, text=True)
    if res.returncode != 0:
        print(f"❌ FAIL: Kernel compilation failed:\n{res.stderr}")
        sys.exit(1)
    if "warning:" in res.stderr:
        print(f"❌ FAIL: Compilation generated warnings:\n{res.stderr}")
        sys.exit(1)
    print("  ✓ Kernel compiles cleanly with zero warnings/errors")

    # 5. Golden Boot Log Verification
    print("\n[CHECK 5/6] Verifying golden boot log against SP-19 boot stages...")
    with open("docs/evidence/golden-boot-sp19.log", "r") as f:
        log_content = f.read()

    required_stages = [
        "[STAGE:45_POLICY_ENGINE_INIT]",
        "Loaded 4 declarative policy rules (v1 ruleset active, INV-ENF-002)",
        "[STAGE:46_POLICY_EVALUATION]",
        "Default-deny verified for unknown subject (INV-ENF-001 PASSED)",
        "Explicit allow matched rule #1 (PID 1 -> EP 101)",
        "Deterministic precedence verified: Deny rule #3 overrides allow (INV-ENF-002 PASSED)",
        "RequireConsent action verified for AIS Role 3",
        "[STAGE:47_POLICY_CAP_INTEGRATION]",
        "Dual-check: PID 2 authorized for READ (token valid + policy allowed)",
        "Dual-check: PID 2 WRITE blocked by standing policy despite possessing token (REQ-SEC-001 PASSED)",
        "[STAGE:48_POLICY_ENGINE_READY]",
        "[ENF] Sprint SP-19 Policy Engine v1 Milestone REACHED.",
    ]

    for stage_str in required_stages:
        if stage_str not in log_content:
            print(f"❌ FAIL: Golden boot log missing expected output: '{stage_str}'")
            sys.exit(1)
        print(f"  ✓ Found boot verification signature: {stage_str}")

    # 6. Evidence Manifest Generation
    print("\n[CHECK 6/6] Generating evidence manifest...")
    manifest_dir = "evidence/SP-19"
    os.makedirs(manifest_dir, exist_ok=True)
    manifest_path = os.path.join(manifest_dir, "manifest.json")

    manifest_data = {
        "sprint": "SP-19",
        "title": "Policy Engine v1 (ENF: Declarative Rule Language, Default-Deny, Deterministic Precedence, Dual-Check Capability Access)",
        "timestamp": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        "invariants": {
            "INV-ENF-001": "Default-Deny Security Model (absence of allow yields deterministic denial)",
            "INV-ENF-002": "Deterministic Precedence & Versioned Rule Resolution (deny overrides allow, priority ordering)",
            "REQ-SEC-001": "Dual-Check Capability Enforcement (capability token valid AND policy allows access)"
        },
        "files": {}
    }

    for path in REQUIRED_SP19_FILES:
        with open(path, "rb") as f:
            h = hashlib.sha256(f.read()).hexdigest()
        manifest_data["files"][path] = {
            "sha256": h,
            "bytes": os.path.getsize(path)
        }

    with open(manifest_path, "w") as f:
        json.dump(manifest_data, f, indent=2)

    print(f"  ✓ Manifest written to {manifest_path}")

    print("\n==================================================")
    print(" ✅ ALL SP-19 VERIFICATION CHECKS PASSED (6/6)    ")
    print(" Sprint SP-19 Definition of Done 100% SATISFIED!  ")
    print("==================================================")

if __name__ == "__main__":
    verify_sp19()
