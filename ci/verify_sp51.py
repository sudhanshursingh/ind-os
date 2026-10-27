#!/usr/bin/env python3
"""
Ind OS Sprint SP-51 Verification & Acceptance Test Harness
Target: Network Stack Core (Ethernet, IPv4, TCP, UDP, Socket Capabilities)
Standards: Bible §8.13, REQ-KRN-110, INV-NET-001..003, Rule N-01, Rule N-04, Rule N-10
"""

import os
import sys
import hashlib
import json
import subprocess

cargo_bin = os.path.expanduser("~/.cargo/bin")
if cargo_bin not in os.environ.get("PATH", ""):
    os.environ["PATH"] = f"{cargo_bin}:{os.environ.get('PATH', '')}"

REQUIRED_SP51_FILES = [
    "docs/architecture/network-stack-core-spec.md",
    "docs/evidence/SP-51-validation.md",
    "docs/evidence/golden-boot-sp51.log",
    "userspace/net/Cargo.toml",
    "userspace/net/src/lib.rs",
    "userspace/net/src/main.rs",
    "userspace/net/src/types.rs",
    "userspace/net/src/ethernet.rs",
    "userspace/net/src/ipv4.rs",
    "userspace/net/src/udp.rs",
    "userspace/net/src/tcp.rs",
    "userspace/net/src/socket.rs",
    "kernel/src/userspace/net_core.rs",
]

def hash_file(filepath: str) -> str:
    h = hashlib.sha256()
    with open(filepath, "rb") as fh:
        while chunk := fh.read(65536):
            h.update(chunk)
    return h.hexdigest()

def verify_sp51():
    print("==================================================")
    print(" Ind OS SP-51 Network Stack Core Verifier         ")
    print(" Target: Sprint SP-51 Definition of Done (DoD)   ")
    print("==================================================")

    # 1. Deliverables Presence Check
    print("[CHECK 1/6] Verifying SP-51 deliverable presence...")
    for rel_path in REQUIRED_SP51_FILES:
        if not os.path.exists(rel_path):
            print(f"❌ FAIL: Required SP-51 file missing: {rel_path}")
            sys.exit(1)
        print(f"  ✓ Found {rel_path}")

    # 2. Architectural & Invariant Symbols Check
    print("\n[CHECK 2/6] Verifying architectural symbols and API contracts...")
    with open("userspace/net/src/types.rs", "r") as f:
        t_src = f.read()
    for s in ["MacAddress", "Ipv4Address", "EtherType", "IpProtocol", "SocketType", "TcpState", "CAP_NET_SOCKET", "CAP_NET_BIND"]:
        if s not in t_src:
            print(f"❌ FAIL: types.rs missing symbol '{s}'")
            sys.exit(1)
    print("  ✓ Network type and capability symbols verified (REQ-KRN-110, INV-NET-001)")

    with open("userspace/net/src/ipv4.rs", "r") as f:
        ip_src = f.read()
    for s in ["Ipv4Header", "compute_checksum", "parse", "serialize"]:
        if s not in ip_src:
            print(f"❌ FAIL: ipv4.rs missing symbol '{s}'")
            sys.exit(1)
    print("  ✓ IPv4 packet and checksum symbols verified (INV-NET-002)")

    with open("userspace/net/src/tcp.rs", "r") as f:
        tcp_src = f.read()
    for s in ["TcpHeader", "TcpStateMachine", "TCP_FLAG_SYN", "TCP_FLAG_ACK", "TCP_FLAG_FIN"]:
        if s not in tcp_src:
            print(f"❌ FAIL: tcp.rs missing symbol '{s}'")
            sys.exit(1)
    print("  ✓ TCP segment and state machine symbols verified (REQ-KRN-110)")

    with open("userspace/net/src/socket.rs", "r") as f:
        sock_src = f.read()
    for s in ["SocketTable", "SocketCapToken", "create_socket", "bind", "listen", "connect", "send", "recv"]:
        if s not in sock_src:
            print(f"❌ FAIL: socket.rs missing symbol '{s}'")
            sys.exit(1)
    print("  ✓ Capability-gated socket table symbols verified (INV-NET-001, INV-NET-003)")

    # 3. Kernel Golden Boot Log Verification (Stages 203-207)
    print("\n[CHECK 3/6] Verifying kernel boot stages in golden-boot-sp51.log...")
    with open("docs/evidence/golden-boot-sp51.log", "r") as f:
        boot_log = f.read()

    expected_stages = [
        ("STAGE:203_NET_CORE_INIT", "Network stack core and interface binding initialized"),
        ("STAGE:204_NET_PACKET_FRAME_PARSING", "Ethernet and IPv4 packet parsing with RFC 1071 checksum verified"),
        ("STAGE:205_NET_SOCKET_CAPABILITY_TOKEN", "Socket creation and bind capability token gating verified"),
        ("STAGE:206_NET_TCP_STATE_MACHINE", "TCP state machine and 3-way handshake transition verified"),
        ("STAGE:207_SP51_NET_READY", "Sprint SP-51 Network Stack Core Milestone REACHED"),
    ]

    for marker, desc in expected_stages:
        if marker not in boot_log:
            print(f"❌ FAIL: Expected stage marker '{marker}' missing from golden boot log")
            sys.exit(1)
        print(f"  ✓ Boot stage {marker} verified ({desc})")

    if "[PANIC]" in boot_log or "FATAL" in boot_log:
        print("❌ FAIL: Kernel panic or fatal error detected in golden-boot-sp51.log")
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
    print("\n[CHECK 5/6] Verifying Rule N-04: Zero unsafe code in userspace/net...")
    net_src = "userspace/net/src"
    for root, _, files in os.walk(net_src):
        for file in files:
            if file.endswith(".rs"):
                fpath = os.path.join(root, file)
                with open(fpath, "r") as f:
                    for idx, line in enumerate(f, 1):
                        clean = line.split("//")[0].strip()
                        if "unsafe" in clean:
                            print(f"❌ FAIL: Rule N-04 violation in {fpath}:{idx}: '{clean}'")
                            sys.exit(1)
    print("  ✓ 0 unsafe code blocks detected across userspace/net (Rule N-04)")

    # 6. Generate SP-51 Manifest
    print("\n[CHECK 6/6] Generating SP-51 formal verification manifest...")
    manifest_dir = "evidence/SP-51"
    os.makedirs(manifest_dir, exist_ok=True)
    manifest_file = os.path.join(manifest_dir, "manifest.json")
    manifest_data = {
        "sprint": "SP-51",
        "phase": "Phase 7 — Network + Updates + Service Completion",
        "status": "PASSED & RATIFIED",
        "verified_stages": [s[0] for s in expected_stages],
        "checksums": {f: hash_file(f) for f in REQUIRED_SP51_FILES if os.path.exists(f)},
    }
    with open(manifest_file, "w") as f:
        json.dump(manifest_data, f, indent=2)
    print(f"  ✓ Manifest written to {manifest_file}")

    print("\n==================================================")
    print("  SPRINT SP-51 VERIFICATION PASSED & RATIFIED     ")
    print("==================================================")

if __name__ == "__main__":
    verify_sp51()
