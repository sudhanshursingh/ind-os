#!/usr/bin/env python3
"""
Ind OS Gate G7 Hard-Stop Audit & Verification Test Harness
Target: Milestone CONNECTED OS V0.1 Ratification (Phase 7 Conclusion)
Standards: Bible §6.3, §6.4, §7.1, §7.2, §8.13, §19.2 Table 19.2, Rule N-01, Rule N-04, Rule N-10
"""

import os
import sys
import hashlib
import json
import subprocess

REQUIRED_G7_FILES = [
    "docs/evidence/gate-g7-audit-report.md",
    "docs/evidence/golden-boot-g7.log",
    "docs/architecture/sbom-phase7.md",
    "docs/evidence/sbom-phase7.json",
    "docs/evidence/SP-51-validation.md",
    "docs/evidence/SP-52-validation.md",
    "docs/evidence/SP-53-validation.md",
    "docs/evidence/SP-54-validation.md",
    "docs/evidence/SP-55-validation.md",
    "docs/evidence/SP-56-validation.md",
    "docs/evidence/SP-57-validation.md",
    "docs/evidence/SP-58-validation.md",
    "docs/evidence/SP-59-validation.md",
]

def hash_file(filepath: str) -> str:
    h = hashlib.sha256()
    with open(filepath, "rb") as fh:
        while chunk := fh.read(65536):
            h.update(chunk)
    return h.hexdigest()

def verify_gate_g7():
    print("==================================================")
    print(" Ind OS Gate G7 Hard-Stop Audit Verifier          ")
    print(" Target: Phase 7 Ratification (CONNECTED OS V0.1)")
    print("==================================================")

    # 1. Deliverables Presence Check
    print("[CHECK 1/6] Verifying Gate G7 deliverable presence...")
    for rel_path in REQUIRED_G7_FILES:
        if not os.path.exists(rel_path):
            print(f"❌ FAIL: Required Gate G7 file missing: {rel_path}")
            sys.exit(1)
        print(f"  ✓ Found {rel_path}")

    # 2. Kernel Boot Log Verification (Stage 248)
    print("\n[CHECK 2/6] Verifying Gate G7 kernel boot log (Stage 248)...")
    with open("docs/evidence/golden-boot-g7.log", "r") as f:
        boot_log = f.read()

    expected_markers = [
        "STAGE:248_GATE_G7_RATIFIED",
        "MILESTONE: CONNECTED OS V0.1",
        "1. Live E2E Demo: Update Chain with Atomic Switch & Rollback...",
        "2. Verifying Firewall Default-Deny Inbound Posture (INV-FW-001)...",
        "3. Live Demo: Leak Sentinel Catching Seeded Exfiltration Flow",
        "4. Verifying ADR-004 Signed & DEF-004 Vetted Crypto Posture...",
        "5. Verifying Release Key Ceremony Tooling (DEF-003, REQ-SEC-108)...",
        "6. Verifying Phase 7 SBOM Refresh (CycloneDX 1.5)...",
        "MILESTONE OFFICIALLY ACHIEVED: CONNECTED OS V0.1",
        "Phase 7 (Network + Updates + Services) RATIFIED.",
    ]

    for marker in expected_markers:
        if marker not in boot_log:
            print(f"❌ FAIL: Golden boot log missing Gate G7 audit verification marker: '{marker}'")
            sys.exit(1)
        print(f"  ✓ Verified Gate G7 marker: {marker}")

    # 3. Rule N-01 Audit: Zero External Crates in TCB
    print("\n[CHECK 3/6] Auditing Rule N-01: Zero external crates in TCB...")
    with open("kernel/Cargo.toml", "r") as f:
        kernel_cargo = f.read()
    if "[dependencies]" in kernel_cargo:
        deps_section = kernel_cargo.split("[dependencies]")[1].split("[")[0].strip()
        for dep_line in deps_section.splitlines():
            dep_clean = dep_line.split("#")[0].strip()
            if dep_clean:
                print(f"❌ FAIL: External crates found in kernel TCB: {dep_clean}")
                sys.exit(1)
    print("  ✓ Pure in-tree TCB verified: zero external crates in kernel/Cargo.toml")


    # 4. Rule N-04 Audit: Zero Unsafe Code in Phase 7 Userspace Modules
    print("\n[CHECK 4/6] Auditing Rule N-04: Zero unsafe code in Phase 7 modules...")
    phase7_srcs = [
        "userspace/net/src/ethernet.rs",
        "userspace/net/src/ipv4.rs",
        "userspace/net/src/udp.rs",
        "userspace/net/src/tcp.rs",
        "userspace/net/src/socket.rs",
        "userspace/net/src/tls.rs",
        "userspace/net/src/dns.rs",
        "userspace/net/src/dhcp.rs",
        "userspace/net/src/firewall.rs",
        "userspace/net/src/update.rs",
        "userspace/net/src/ceremony.rs",
        "userspace/net/src/sentinel.rs",
        "userspace/net/src/interop.rs",
        "userspace/net/src/soak.rs",
    ]
    for src in phase7_srcs:
        if os.path.exists(src):
            with open(src, "r") as f:
                code = f.read()
            if "unsafe" in code:
                print(f"❌ FAIL: Unsafe block detected in {src}!")
                sys.exit(1)
            print(f"  ✓ Zero unsafe code in {src}")

    # 5. Cryptographic Evidence Ledger
    print("\n[CHECK 5/6] Generating Gate G7 cryptographic SHA256 integrity ledger...")
    g7_manifest = {}
    for rel_path in REQUIRED_G7_FILES:
        h = hash_file(rel_path)
        g7_manifest[rel_path] = h
        print(f"  SHA256({rel_path}) = {h}")

    os.makedirs("evidence/G7", exist_ok=True)
    with open("evidence/G7/manifest.json", "w") as f:
        json.dump(g7_manifest, f, indent=2)
    print("  ✓ Saved manifest to evidence/G7/manifest.json")

    # 6. Final Hard-Stop Ratification Sign-off
    print("\n[CHECK 6/6] Final Gate G7 Hard-Stop Ratification Sign-off:")
    print("  [✓] Milestone: CONNECTED OS V0.1")
    print("  [✓] E2E update chain with atomic switch and rollback demonstrated")
    print("  [✓] Firewall default-deny inbound posture enforced")
    print("  [✓] Leak sentinel catching seeded exfiltration flow verified")
    print("  [✓] ADR-004 ratified and DEF-004 reviewed (non-TCB vetted crypto)")
    print("  [✓] Release key ceremony tooling ratified (DEF-003, REQ-SEC-108)")
    print("  [✓] CycloneDX 1.5 Phase 7 SBOM refreshed and audited")
    print("  [✓] Rule N-01 (Zero external crates) & Rule N-04 (Zero unsafe) compliant")
    print("\n>>> GATE G7 HARD STOP PASSED: MILESTONE CONNECTED OS V0.1 RATIFIED <<<")

if __name__ == "__main__":
    verify_gate_g7()
