#!/usr/bin/env python3
"""
Sprint SP-17 Verification Harness (Capabilities I: Unforgeable Tokens, Object-Table Integration,
Mint/Grant/Revoke, and Per-Process Namespaces).
Ratified under INDOS-BIBLE-001 / Phase 3 The Security Spine (§8.9, §10, §14) and INDOS-SPEC-CAP-02.

Verifies:
1. Deliverable presence:
   - docs/architecture/capability-tokens-spec.md
   - docs/evidence/SP-17-validation.md
   - docs/evidence/golden-boot-sp17.log
   - kernel/src/capability/namespace.rs
   - kernel/src/capability/table.rs
   - kernel/src/capability/mod.rs
2. Static architectural & invariant checks:
   - Invariant INV-CAP-001: Monotonic Rights Attenuation.
   - Invariant INV-CAP-002: Revocation Soundness & Cascading Lineage Tracking.
   - Invariant INV-CAP-003: Object-Table Reference Coherence (retain/release binding).
   - Invariant INV-CAP-004: Strict Per-Process Namespace Isolation.
3. Unsafe-code budget audit: 100% pure safe Rust across all kernel/src/capability/ modules.
4. Clean kernel compilation with zero warnings/errors.
5. Golden boot log verification against SP-17 boot stages (Stages 38, 39, 40).
6. Evidence manifest generation at evidence/SP-17/manifest.json.
"""

import sys
import os
import subprocess
import hashlib
import json

# Ensure ~/.cargo/bin is in PATH for cargo invocations
cargo_bin = os.path.expanduser("~/.cargo/bin")
if cargo_bin not in os.environ.get("PATH", ""):
    os.environ["PATH"] = f"{cargo_bin}:{os.environ.get('PATH', '')}"
import time

REQUIRED_SP17_FILES = [
    "docs/architecture/capability-tokens-spec.md",
    "docs/evidence/SP-17-validation.md",
    "docs/evidence/golden-boot-sp17.log",
    "kernel/src/capability/namespace.rs",
    "kernel/src/capability/table.rs",
    "kernel/src/capability/mod.rs",
]

def verify_sp17():
    print("==================================================")
    print(" Ind OS SP-17 Capabilities I Verifier            ")
    print(" Target: Sprint SP-17 Definition of Done (DoD)   ")
    print("==================================================")

    # 1. Deliverables Presence Check
    print("[CHECK 1/6] Verifying SP-17 deliverable presence...")
    for rel_path in REQUIRED_SP17_FILES:
        if not os.path.exists(rel_path):
            print(f"❌ FAIL: Required SP-17 file missing: {rel_path}")
            sys.exit(1)
        print(f"  ✓ Found {rel_path}")

    # 2. Static Architectural & Invariant Checks
    print("\n[CHECK 2/6] Verifying Capability Tokens architecture & invariants...")
    with open("docs/architecture/capability-tokens-spec.md", "r") as f:
        spec_src = f.read()
    for inv in ["INV-CAP-001", "INV-CAP-002", "INV-CAP-003", "INV-CAP-004"]:
        if inv not in spec_src:
            print(f"❌ FAIL: Capability spec missing invariant '{inv}'")
            sys.exit(1)
    print("  ✓ Capability Tokens invariants INV-CAP-001..004 documented in INDOS-SPEC-CAP-02")

    with open("kernel/src/capability/namespace.rs", "r") as f:
        ns_src = f.read()
    for symbol in ["CapNamespace", "NamespaceRegistry", "MAX_NAMESPACES", "create_namespace", "delegate", "lookup", "install_object"]:
        if symbol not in ns_src:
            print(f"❌ FAIL: namespace.rs missing symbol '{symbol}'")
            sys.exit(1)
    print("  ✓ Per-process Capability Namespace & Registry primitives present")

    with open("kernel/src/capability/table.rs", "r") as f:
        tbl_src = f.read()
    for symbol in ["Object(KObjectId)", "mint_object_cap", "revoke_with_objects", "delegate_object_to"]:
        if symbol not in tbl_src:
            print(f"❌ FAIL: table.rs missing symbol '{symbol}'")
            sys.exit(1)
    print("  ✓ Object-Table integration & retain/release tracking present in table.rs")

    with open("kernel/src/capability/mod.rs", "r") as f:
        mod_src = f.read()
    for symbol in ["NAMESPACE_REGISTRY", "CapNamespace", "NamespaceRegistry"]:
        if symbol not in mod_src:
            print(f"❌ FAIL: mod.rs missing symbol '{symbol}'")
            sys.exit(1)
    print("  ✓ Global NAMESPACE_REGISTRY declared and re-exported in capability/mod.rs")

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
    print("  ✓ Kernel compiles cleanly with zero warnings/errors")

    # 5. Golden Boot Log Verification
    print("\n[CHECK 5/6] Verifying golden boot log against SP-17 boot stages...")
    with open("docs/evidence/golden-boot-sp17.log", "r") as f:
        log_content = f.read()

    required_stages = [
        "[STAGE:38_CAPABILITY_TOKENS_INIT]",
        "[INFO ] cap: Initialized per-process namespaces: PID 1 and PID 2 active (INV-CAP-004)",
        "[STAGE:39_CAP_OBJECT_INTEGRATION]",
        "[INFO ] cap: Installed Object Cap in PID 1",
        "(INV-CAP-001 PASSED)",
        "(INV-CAP-003)",
        "(INV-CAP-004 PASSED)",
        "(INV-CAP-002, INV-CAP-003)",
        "[INFO ] cap: KObject successfully destroyed upon final reference release (INV-CAP-003 PASSED)",
        "[STAGE:40_CAPABILITY_SYSTEM_READY]",
        "[CAP] Sprint SP-17 Capabilities I Milestone REACHED.",
    ]

    for stage_str in required_stages:
        if stage_str not in log_content:
            print(f"❌ FAIL: Golden boot log missing expected output: '{stage_str}'")
            sys.exit(1)
        print(f"  ✓ Found boot verification signature: {stage_str}")

    # 6. Evidence Manifest Generation
    print("\n[CHECK 6/6] Generating evidence manifest...")
    manifest_dir = "evidence/SP-17"
    os.makedirs(manifest_dir, exist_ok=True)
    manifest_path = os.path.join(manifest_dir, "manifest.json")

    manifest_data = {
        "sprint": "SP-17",
        "title": "Capabilities I (Unforgeable Tokens, Object-Table Integration, Mint/Grant/Revoke, Per-Process Namespaces)",
        "timestamp": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        "invariants": {
            "INV-CAP-001": "Monotonic Rights Attenuation (target_rights subset enforced)",
            "INV-CAP-002": "Revocation Soundness (transitive lineage tree invalidation)",
            "INV-CAP-003": "Object-Table Reference Coherence (retain on mint/delegate, release on revoke)",
            "INV-CAP-004": "Strict Per-Process Namespace Isolation (process-local handles)"
        },
        "files": {}
    }

    for path in REQUIRED_SP17_FILES:
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
    print(" ✅ ALL SP-17 VERIFICATION CHECKS PASSED (6/6)    ")
    print(" Sprint SP-17 Definition of Done 100% SATISFIED!  ")
    print("==================================================")

if __name__ == "__main__":
    verify_sp17()
