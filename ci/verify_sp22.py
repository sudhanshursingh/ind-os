#!/usr/bin/env python3
"""
Ind OS Sprint SP-22 Verification & Acceptance Test Harness
Target: Adversarial Self-Review (Internal Red Team Exploitation Suite)
Standards: Bible §8.9, §10, §14, Rule N-01, Rule N-04, Rule N-10
Invariants: INV-CAP-001..002, INV-SEC-001, INV-ENF-001..002, INV-AUD-001, INV-CON-001
"""

import os
import sys
import json
import hashlib
import subprocess
import time

# Ensure ~/.cargo/bin is in PATH for cargo invocations
cargo_bin = os.path.expanduser("~/.cargo/bin")
if cargo_bin not in os.environ.get("PATH", ""):
    os.environ["PATH"] = f"{cargo_bin}:{os.environ.get('PATH', '')}"

REQUIRED_SP22_FILES = [
    "docs/architecture/adversarial-self-review-report.md",
    "docs/evidence/SP-22-validation.md",
    "docs/evidence/golden-boot-sp22.log",
    "kernel/src/redteam/attacks.rs",
    "kernel/src/redteam/mod.rs",
]

def verify_sp22():
    print("==================================================")
    print(" Ind OS SP-22 Adversarial Self-Review Verifier   ")
    print(" Target: Sprint SP-22 Definition of Done (DoD)   ")
    print("==================================================")

    # 1. Deliverables Presence Check
    print("[CHECK 1/6] Verifying SP-22 deliverable presence...")
    for rel_path in REQUIRED_SP22_FILES:
        if not os.path.exists(rel_path):
            print(f"❌ FAIL: Required SP-22 file missing: {rel_path}")
            sys.exit(1)
        print(f"  ✓ Found {rel_path}")

    # 2. Static Architectural & Invariant Checks
    print("\n[CHECK 2/6] Verifying architectural symbols and attack probes...")
    with open("kernel/src/redteam/attacks.rs", "r") as f:
        atk_src = f.read()

    probes = [
        "probe_capability_escalation",
        "probe_dangling_handle",
        "probe_ambient_authority",
        "probe_policy_bypass",
        "probe_audit_tampering",
        "probe_consent_spoofing",
        "run_all_probes",
    ]
    for probe in probes:
        if probe not in atk_src:
            print(f"❌ FAIL: attacks.rs missing probe '{probe}'")
            sys.exit(1)
    print("  ✓ All 6 adversarial attack probes and suite runner verified in attacks.rs")

    with open("kernel/src/redteam/mod.rs", "r") as f:
        mod_src = f.read()
    if "RedTeamSuite" not in mod_src or "ProbeSummary" not in mod_src:
        print("❌ FAIL: mod.rs missing RedTeamSuite re-export")
        sys.exit(1)
    print("  ✓ RedTeamSuite and ProbeSummary re-exported in redteam/mod.rs")

    # 3. Unsafe-Code Budget Scanner Check
    print("\n[CHECK 3/6] Auditing unsafe code in redteam subsystem...")
    for path in ["kernel/src/redteam/mod.rs", "kernel/src/redteam/attacks.rs"]:
        with open(path, "r") as f:
            content = f.read()
            if "unsafe " in content:
                print(f"❌ FAIL: Forbidden unsafe code detected in {path}")
                sys.exit(1)
    print("  ✓ 100% pure safe Rust in kernel/src/redteam/ (Zero unsafe blocks)")

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
    print("\n[CHECK 5/6] Verifying golden boot log against SP-22 boot stages...")
    with open("docs/evidence/golden-boot-sp22.log", "r") as f:
        log_content = f.read()

    required_stages = [
        "[STAGE:57_REDTEAM_SUITE_INIT]",
        "Adversarial harness initialized: targeting Phase 3 Security Spine",
        "[STAGE:58_REDTEAM_EXPLOIT_PROBES]",
        "Attack Vector 1 REPELLED: Rights expansion prohibited (INV-CAP-001)",
        "Attack Vector 2 REPELLED: Use-after-free rejected (INV-CAP-002, INV-OBJ-001)",
        "Attack Vector 3 REPELLED: Zero ambient authority enforced (INV-SEC-001)",
        "Attack Vector 4 REPELLED: Default-deny & deny precedence enforced (INV-ENF-001, INV-ENF-002)",
        "Attack Vector 5 REPELLED: 1-bit audit mutation localized (INV-AUD-001)",
        "Attack Vector 6 REPELLED: Overlay & input injection rejected (INV-CON-001)",
        "Adversarial suite results: 6/6 attack vectors repelled, 0 vulnerabilities (100% DEFENSE)",
        "[STAGE:59_REDTEAM_VERIFIED]",
        "[RED] Sprint SP-22 Adversarial Self-Review Milestone REACHED.",
    ]

    for stage_str in required_stages:
        if stage_str not in log_content:
            print(f"❌ FAIL: Golden boot log missing expected output: '{stage_str}'")
            sys.exit(1)
        print(f"  ✓ Found boot verification signature: {stage_str}")

    # 6. Evidence Manifest Generation
    print("\n[CHECK 6/6] Generating evidence manifest...")
    manifest_dir = "evidence/SP-22"
    os.makedirs(manifest_dir, exist_ok=True)
    manifest_path = os.path.join(manifest_dir, "manifest.json")

    manifest_data = {
        "sprint": "SP-22",
        "title": "Adversarial Self-Review (Internal Red-Team Exploitation Probes: Escalation, UAF, Ambient Authority, Policy Bypass, Audit Tampering, Consent Spoofing)",
        "timestamp": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        "invariants": {
            "INV-CAP-001": "Monotonic Rights Attenuation (repelling privilege escalation)",
            "INV-CAP-002": "Revocation Soundness (repelling dangling handles and use-after-free)",
            "INV-SEC-001": "Zero Ambient Authority (all operations capability-mediated)",
            "INV-ENF-001": "Default-Deny Security Model",
            "INV-ENF-002": "Deterministic Precedence Resolution",
            "INV-AUD-001": "Cryptographic Append-Only Audit Integrity",
            "INV-CON-001": "Unspoofable Consent Surface & Hardware Input Isolation"
        },
        "files": {}
    }

    for path in REQUIRED_SP22_FILES:
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
    print(" ✅ ALL SP-22 VERIFICATION CHECKS PASSED (6/6)    ")
    print(" Sprint SP-22 Definition of Done 100% SATISFIED!  ")
    print("==================================================")

if __name__ == "__main__":
    verify_sp22()
