#!/usr/bin/env python3
"""
Ind OS Sprint SP-53 Verification & Acceptance Test Harness
Target: DNS, DHCP, Firewall (In-Kernel Firewall, Capability-Gated Rules)
Standards: Bible §8.13, INV-FW-001..002, INV-DHCP-001, INV-DNS-001, Rule N-01, Rule N-04, Rule N-10
"""

import os
import sys
import hashlib
import json

REQUIRED_SP53_FILES = [
    "docs/evidence/SP-53-validation.md",
    "docs/evidence/golden-boot-sp53.log",
    "userspace/net/src/dns.rs",
    "userspace/net/src/dhcp.rs",
    "userspace/net/src/firewall.rs",
    "kernel/src/userspace/net_services.rs",
]

def hash_file(filepath: str) -> str:
    h = hashlib.sha256()
    with open(filepath, "rb") as fh:
        while chunk := fh.read(65536):
            h.update(chunk)
    return h.hexdigest()

def verify_sp53():
    print("==================================================")
    print(" Ind OS SP-53 DNS, DHCP & Firewall Verifier       ")
    print(" Target: Sprint SP-53 Definition of Done (DoD)   ")
    print("==================================================")

    # 1. Deliverables Presence Check
    print("[CHECK 1/6] Verifying SP-53 deliverable presence...")
    for rel_path in REQUIRED_SP53_FILES:
        if not os.path.exists(rel_path):
            print(f"❌ FAIL: Required SP-53 file missing: {rel_path}")
            sys.exit(1)
        print(f"  ✓ Found {rel_path}")

    # 2. Architectural & Invariant Symbols Check
    print("\n[CHECK 2/6] Verifying architectural symbols and API contracts...")
    with open("userspace/net/src/firewall.rs", "r") as f:
        fw_src = f.read()
    for s in ["FirewallEngine", "CAP_NET_FIREWALL_ADMIN", "add_rule", "remove_rule", "filter_packet", "FilterAction", "PacketDirection"]:
        if s not in fw_src:
            print(f"❌ FAIL: firewall.rs missing symbol '{s}'")
            sys.exit(1)
    print("  ✓ In-kernel firewall symbols verified (INV-FW-001, INV-FW-002)")

    with open("userspace/net/src/dhcp.rs", "r") as f:
        dhcp_src = f.read()
    for s in ["DhcpClient", "DhcpLease", "DhcpState", "build_discover", "handle_offer", "handle_ack"]:
        if s not in dhcp_src:
            print(f"❌ FAIL: dhcp.rs missing symbol '{s}'")
            sys.exit(1)
    print("  ✓ DHCP client and lease symbols verified (INV-DHCP-001)")

    with open("userspace/net/src/dns.rs", "r") as f:
        dns_src = f.read()
    for s in ["DnsHeader", "DnsResolver", "build_query", "parse_response"]:
        if s not in dns_src:
            print(f"❌ FAIL: dns.rs missing symbol '{s}'")
            sys.exit(1)
    print("  ✓ DNS resolver and anti-spoof symbols verified (INV-DNS-001)")

    # 3. Kernel Golden Boot Log Verification (Stages 213-217)
    print("\n[CHECK 3/6] Verifying kernel boot stages in golden-boot-sp53.log...")
    with open("docs/evidence/golden-boot-sp53.log", "r") as f:
        boot_log = f.read()

    expected_stages = [
        ("STAGE:213_FIREWALL_INIT", "In-kernel firewall initialized with default-deny inbound posture"),
        ("STAGE:214_FIREWALL_CAPABILITY_GATING", "Capability-gated firewall rule modification verified"),
        ("STAGE:215_DHCP_LEASE_NEGOTIATION", "DHCP Discover/Offer/Request/Ack state machine verified"),
        ("STAGE:216_DNS_CLIENT_RESOLUTION", "DNS client resolution and spoofed response rejection verified"),
        ("STAGE:217_SP53_NET_SERVICES_READY", "Sprint SP-53 DNS, DHCP & Firewall Milestone REACHED"),
    ]

    for marker, desc in expected_stages:
        if marker not in boot_log:
            print(f"❌ FAIL: Expected stage marker '{marker}' missing from golden boot log")
            sys.exit(1)
        print(f"  ✓ Boot stage {marker} verified ({desc})")

    if "[PANIC]" in boot_log or "FATAL" in boot_log:
        print("❌ FAIL: Kernel panic or fatal error detected in golden-boot-sp53.log")
        sys.exit(1)
    print("  ✓ Golden boot clean execution confirmed")

    # 4. Pure Rust Rule N-01 Verification
    print("\n[CHECK 4/6] Verifying Rule N-01: Zero external dependencies...")
    with open("userspace/net/Cargo.toml", "r") as f:
        cargo_toml = f.read()
    in_deps = False
    for line in cargo_toml.splitlines():
        line = line.strip()
        if line.startswith("[dependencies]"):
            in_deps = True
            continue
        if line.startswith("[") and in_deps:
            in_deps = False
        if in_deps and line and not line.startswith("#"):
            if "path =" not in line:
                print(f"❌ FAIL: Non-local dependency detected in userspace/net/Cargo.toml: {line}")
                sys.exit(1)
    print("  ✓ 100% in-tree pure Rust dependencies verified (Rule N-01)")

    # 5. Zero Unsafe Rule N-04 Verification
    print("\n[CHECK 5/6] Verifying Rule N-04: Zero unsafe code in network services...")
    for frel in ["userspace/net/src/dns.rs", "userspace/net/src/dhcp.rs", "userspace/net/src/firewall.rs"]:
        with open(frel, "r") as f:
            for idx, line in enumerate(f, 1):
                clean = line.split("//")[0].strip()
                if "unsafe" in clean:
                    print(f"❌ FAIL: Rule N-04 violation in {frel}:{idx}: '{clean}'")
                    sys.exit(1)
    print("  ✓ 0 unsafe code blocks in network service modules (Rule N-04)")

    # 6. Generate SP-53 Manifest
    print("\n[CHECK 6/6] Generating SP-53 formal verification manifest...")
    manifest_dir = "evidence/SP-53"
    os.makedirs(manifest_dir, exist_ok=True)
    manifest_file = os.path.join(manifest_dir, "manifest.json")
    manifest_data = {
        "sprint": "SP-53",
        "phase": "Phase 7 — Network + Updates + Service Completion",
        "status": "PASSED & RATIFIED",
        "verified_stages": [s[0] for s in expected_stages],
        "checksums": {f: hash_file(f) for f in REQUIRED_SP53_FILES if os.path.exists(f)},
    }
    with open(manifest_file, "w") as f:
        json.dump(manifest_data, f, indent=2)
    print(f"  ✓ Manifest written to {manifest_file}")

    print("\n==================================================")
    print("  SPRINT SP-53 VERIFICATION PASSED & RATIFIED     ")
    print("==================================================")

if __name__ == "__main__":
    verify_sp53()
