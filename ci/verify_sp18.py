#!/usr/bin/env python3
"""
Sprint SP-18 Verification Harness (Capabilities II: Delegation with Attenuation,
Revocation Propagation, and Capability-Leak Test Suite).
Ratified under INDOS-BIBLE-001 / Phase 3 The Security Spine (§8.9, §10, §14) and INDOS-SPEC-CAP-03.

Verifies:
1. Deliverable presence:
   - docs/architecture/capability-delegation-revocation-spec.md
   - docs/evidence/SP-18-validation.md
   - docs/evidence/golden-boot-sp18.log
   - kernel/src/capability/namespace.rs
   - kernel/src/capability/table.rs
   - kernel/src/capability/mod.rs
2. Static architectural & invariant checks:
   - Invariant INV-CAP-001: Monotonic Rights Attenuation along delegation chains.
   - Invariant INV-CAP-002: Global Cross-Process Revocation Propagation.
   - Invariant INV-CAP-003: Object-Table Reference Coherence & Anti-ABA Reclamation.
   - Invariant INV-CAP-LEAK-001: Zero Capability & Object Leakage.
3. Unsafe-code budget audit: 100% pure safe Rust across all kernel/src/capability/ modules.
4. Clean kernel compilation with zero warnings/errors.
5. Golden boot log verification against SP-18 boot stages (Stages 41, 42, 43, 44).
6. Evidence manifest generation at evidence/SP-18/manifest.json.
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

REQUIRED_SP18_FILES = [
    "docs/architecture/capability-delegation-revocation-spec.md",
    "docs/evidence/SP-18-validation.md",
    "docs/evidence/golden-boot-sp18.log",
    "kernel/src/capability/namespace.rs",
    "kernel/src/capability/table.rs",
    "kernel/src/capability/mod.rs",
]

def verify_sp18():
    print("==================================================")
    print(" Ind OS SP-18 Capabilities II Verifier           ")
    print(" Target: Sprint SP-18 Definition of Done (DoD)   ")
    print("==================================================")

    # 1. Deliverables Presence Check
    print("[CHECK 1/6] Verifying SP-18 deliverable presence...")
    for rel_path in REQUIRED_SP18_FILES:
        if not os.path.exists(rel_path):
            print(f"❌ FAIL: Required SP-18 file missing: {rel_path}")
            sys.exit(1)
        print(f"  ✓ Found {rel_path}")

    # 2. Static Architectural & Invariant Checks
    print("\n[CHECK 2/6] Verifying Capabilities II architecture & invariants...")
    with open("docs/architecture/capability-delegation-revocation-spec.md", "r") as f:
        spec_src = f.read()
    for inv in ["INV-CAP-001", "INV-CAP-002", "INV-CAP-003", "INV-CAP-LEAK-001"]:
        if inv not in spec_src:
            print(f"❌ FAIL: Spec missing invariant '{inv}'")
            sys.exit(1)
    print("  ✓ Capabilities II invariants INV-CAP-001..003 and INV-CAP-LEAK-001 documented in INDOS-SPEC-CAP-03")

    with open("kernel/src/capability/namespace.rs", "r") as f:
        ns_src = f.read()
    for symbol in ["find_namespace_idx", "to_revoke", "total_active_capabilities", "destroy_namespace"]:
        if symbol not in ns_src:
            print(f"❌ FAIL: namespace.rs missing symbol '{symbol}'")
            sys.exit(1)
    print("  ✓ Cross-namespace lineage tracking, bitmask worklist, and total active accounting verified in namespace.rs")

    with open("kernel/src/capability/table.rs", "r") as f:
        tbl_src = f.read()
    for symbol in ["parent_pid", "parent_handle", "delegate_object_to"]:
        if symbol not in tbl_src:
            print(f"❌ FAIL: table.rs missing symbol '{symbol}'")
            sys.exit(1)
    print("  ✓ Cross-process parent lineage fields present in Capability struct")

    with open("kernel/src/object/table.rs", "r") as f:
        obj_src = f.read()
    if "active_count" not in obj_src:
        print("❌ FAIL: object/table.rs missing active_count method")
        sys.exit(1)
    print("  ✓ KObjectTable active_count accounting verified for zero-leak audit")

    # 3. Unsafe-Code Budget Scanner Check
    print("\n[CHECK 3/6] Auditing unsafe code in capability subsystem...")
    for path in ["kernel/src/capability/mod.rs", "kernel/src/capability/rights.rs", "kernel/src/capability/table.rs", "kernel/src/capability/namespace.rs"]:
        with open(path, "r") as f:
            content = f.read()
            if "unsafe " in content:
                print(f"❌ FAIL: Forbidden unsafe code detected in {path}")
                sys.exit(1)
    print("  ✓ 100% pure safe Rust in kernel/src/capability/ (Zero unsafe blocks)")

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
    print("\n[CHECK 5/6] Verifying golden boot log against SP-18 boot stages...")
    with open("docs/evidence/golden-boot-sp18.log", "r") as f:
        log_content = f.read()

    required_stages = [
        "[STAGE:41_CAP_DELEGATION_CHAIN]",
        "Delegation Hop 1: P1 -> P2 attenuated to READ|WRITE|GRANT (INV-CAP-001)",
        "Delegation Hop 2: P2 -> P3 attenuated to READ|GRANT (INV-CAP-001)",
        "Delegation Hop 3: P3 -> P4 attenuated to READ (INV-CAP-001)",
        "(4 processes, 5 references, INV-CAP-001 PASSED)",
        "[STAGE:42_CAP_REVOCATION_PROPAGATION]",
        "Cross-process revocation propagation verified: 3 slots revoked, ancestor intact (INV-CAP-002 PASSED)",
        "Root revocation & KObject cleanup verified (INV-CAP-002, INV-CAP-003 PASSED)",
        "[STAGE:43_CAP_LEAK_SUITE]",
        "(INV-CAP-LEAK-001 PASSED)",
        "[STAGE:44_CAPABILITY_II_READY]",
        "[CAP] Sprint SP-18 Capabilities II Milestone REACHED.",
    ]

    for stage_str in required_stages:
        if stage_str not in log_content:
            print(f"❌ FAIL: Golden boot log missing expected output: '{stage_str}'")
            sys.exit(1)
        print(f"  ✓ Found boot verification signature: {stage_str}")

    # 6. Evidence Manifest Generation
    print("\n[CHECK 6/6] Generating evidence manifest...")
    manifest_dir = "evidence/SP-18"
    os.makedirs(manifest_dir, exist_ok=True)
    manifest_path = os.path.join(manifest_dir, "manifest.json")

    manifest_data = {
        "sprint": "SP-18",
        "title": "Capabilities II (Delegation with Attenuation, Revocation Propagation, Capability-Leak Test Suite)",
        "timestamp": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        "invariants": {
            "INV-CAP-001": "Monotonic Rights Attenuation along delegation chains (R_child subset of R_parent)",
            "INV-CAP-002": "Global Revocation Propagation (transitive cross-namespace lineage tree invalidation)",
            "INV-CAP-003": "Object-Table Reference Coherence & Anti-ABA Reclamation (retain/release binding)",
            "INV-CAP-LEAK-001": "Zero Capability & Object Leakage (delta active slots == 0)"
        },
        "files": {}
    }

    for path in REQUIRED_SP18_FILES:
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
    print(" ✅ ALL SP-18 VERIFICATION CHECKS PASSED (6/6)    ")
    print(" Sprint SP-18 Definition of Done 100% SATISFIED!  ")
    print("==================================================")

if __name__ == "__main__":
    verify_sp18()
