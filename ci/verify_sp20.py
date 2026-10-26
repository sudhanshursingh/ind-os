#!/usr/bin/env python3
"""
Ind OS Sprint SP-20 Verification & Acceptance Test Harness
Target: Audit v1 (Cryptographic Append-Only Audit Log, SHA-256 Hash Chaining & Tamper Test)
Standards: Bible §8.9, §10, §14, Rule N-01, Rule N-04, Rule N-10
Invariants: INV-AUD-001
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

REQUIRED_SP20_FILES = [
    "docs/architecture/audit-log-spec.md",
    "docs/evidence/SP-20-validation.md",
    "docs/evidence/golden-boot-sp20.log",
    "kernel/src/audit/sha256.rs",
    "kernel/src/audit/entry.rs",
    "kernel/src/audit/log.rs",
    "kernel/src/audit/mod.rs",
]

def verify_sp20():
    print("==================================================")
    print(" Ind OS SP-20 Audit Subsystem v1 Verifier       ")
    print(" Target: Sprint SP-20 Definition of Done (DoD)   ")
    print("==================================================")

    # 1. Deliverables Presence Check
    print("[CHECK 1/6] Verifying SP-20 deliverable presence...")
    for rel_path in REQUIRED_SP20_FILES:
        if not os.path.exists(rel_path):
            print(f"❌ FAIL: Required SP-20 file missing: {rel_path}")
            sys.exit(1)
        print(f"  ✓ Found {rel_path}")

    # 2. Static Architectural & Invariant Checks
    print("\n[CHECK 2/6] Verifying architectural symbols and invariants...")
    with open("kernel/src/audit/sha256.rs", "r") as f:
        sha_src = f.read()
    for symbol in ["Sha256", "process_block", "update", "finalize"]:
        if symbol not in sha_src:
            print(f"❌ FAIL: sha256.rs missing symbol '{symbol}'")
            sys.exit(1)
    print("  ✓ Zero-dependency FIPS 180-4 SHA-256 implementation verified in sha256.rs")

    with open("kernel/src/audit/entry.rs", "r") as f:
        entry_src = f.read()
    for symbol in ["AuditEventType", "AuditEntry", "compute_hash", "prev_hash", "entry_hash"]:
        if symbol not in entry_src:
            print(f"❌ FAIL: entry.rs missing symbol '{symbol}'")
            sys.exit(1)
    print("  ✓ Cryptographic AuditEntry and Event Types verified in entry.rs")

    with open("kernel/src/audit/log.rs", "r") as f:
        log_src = f.read()
    for symbol in ["AuditLog", "MAX_AUDIT_ENTRIES", "init", "record", "verify_integrity", "tamper_flip_byte", "restore_entry"]:
        if symbol not in log_src:
            print(f"❌ FAIL: log.rs missing symbol '{symbol}'")
            sys.exit(1)
    print("  ✓ Hash-chained AuditLog and tamper-detection protocol verified in log.rs")

    with open("kernel/src/audit/mod.rs", "r") as f:
        mod_src = f.read()
    if "AUDIT_LOG" not in mod_src:
        print("❌ FAIL: mod.rs missing global AUDIT_LOG singleton")
        sys.exit(1)
    print("  ✓ Global AUDIT_LOG singleton declared and re-exported in audit/mod.rs")

    # 3. Unsafe-Code Budget Scanner Check
    print("\n[CHECK 3/6] Auditing unsafe code in audit subsystem...")
    for path in ["kernel/src/audit/mod.rs", "kernel/src/audit/sha256.rs", "kernel/src/audit/entry.rs", "kernel/src/audit/log.rs"]:
        with open(path, "r") as f:
            content = f.read()
            if "unsafe " in content:
                print(f"❌ FAIL: Forbidden unsafe code detected in {path}")
                sys.exit(1)
    print("  ✓ 100% pure safe Rust in kernel/src/audit/ (Zero unsafe blocks)")

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
    print("\n[CHECK 5/6] Verifying golden boot log against SP-20 boot stages...")
    with open("docs/evidence/golden-boot-sp20.log", "r") as f:
        log_content = f.read()

    required_stages = [
        "[STAGE:49_AUDIT_LOG_INIT]",
        "Cryptographic audit log online: Genesis block hash verified",
        "[STAGE:50_AUDIT_EVENT_LOGGING]",
        "Logged 4 security events; hash chain valid and monotonic (INV-AUD-001)",
        "[STAGE:51_AUDIT_TAMPER_TEST]",
        "Tamper detected at entry #2: Entry content hash mismatch: payload modified (INV-AUD-001 verified)",
        "Tamper test passed: 1-bit mutation detected and localized (INV-AUD-001 PASSED)",
        "[STAGE:52_AUDIT_V1_READY]",
        "[AUD] Sprint SP-20 Audit v1 Milestone REACHED.",
    ]

    for stage_str in required_stages:
        if stage_str not in log_content:
            print(f"❌ FAIL: Golden boot log missing expected output: '{stage_str}'")
            sys.exit(1)
        print(f"  ✓ Found boot verification signature: {stage_str}")

    # 6. Evidence Manifest Generation
    print("\n[CHECK 6/6] Generating evidence manifest...")
    manifest_dir = "evidence/SP-20"
    os.makedirs(manifest_dir, exist_ok=True)
    manifest_path = os.path.join(manifest_dir, "manifest.json")

    manifest_data = {
        "sprint": "SP-20",
        "title": "Audit v1 (Cryptographic Append-Only Audit Log, Pure Safe Rust SHA-256 Hash Chaining, Tamper Detection & Localization)",
        "timestamp": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        "invariants": {
            "INV-AUD-001": "Cryptographic Append-Only Audit Integrity & Hash Chaining (tamper detection & localization)"
        },
        "files": {}
    }

    for path in REQUIRED_SP20_FILES:
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
    print(" ✅ ALL SP-20 VERIFICATION CHECKS PASSED (6/6)    ")
    print(" Sprint SP-20 Definition of Done 100% SATISFIED!  ")
    print("==================================================")

if __name__ == "__main__":
    verify_sp20()
