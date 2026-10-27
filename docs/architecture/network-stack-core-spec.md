# Network Stack Core Specification (SP-51, REQ-KRN-110)
**Phase:** Phase 7 — Network + Updates + Service Completion (SP-51 – SP-59)  
**Sprint:** SP-51 — Network Stack Core  
**Document ID:** `INDOS-SPEC-NET-001`  
**Status:** **APPROVED & IMPLEMENTED**  
**Author:** Ind OS Core Networking Group  

---

## 1. Constitutional Scope & Governing Requirements

Under Ind OS Development Bible v1.0.0.0 §8.13, Sprint SP-51 establishes the foundation of Ind OS network communication:
- **REQ-KRN-110:** In-tree network stack implementing an Ethernet/IPv4/TCP/UDP subset with zero ambient authority.
- **INV-NET-001 (Sockets as Capability Tokens):** No process can open, bind, connect, listen, or transmit over a network interface without explicitly granted capability tokens (`CAP_NET_SOCKET`, `CAP_NET_BIND`, `CAP_NET_LISTEN`, `CAP_NET_CONNECT`, `CAP_NET_SEND`, `CAP_NET_RECV`, `CAP_NET_RAW`).
- **INV-NET-002 (Bounded Packet Processing & Zero-Copy Goals):** Fixed-size frame buffers (`DEFAULT_MTU = 1500`), RFC 1071 internet checksum enforcement, and zero heap allocation in packet parsing and state machine paths.
- **INV-NET-003 (Non-Bypassable Socket Table & Namespace Isolation):** Socket handles are managed strictly within a capability-protected table with deterministic port conflict resolution (`PortInUse`).

---

## 2. Protocol Frame Structures

### 2.1 Ethernet Header (14 bytes)
```text
 0                   1                   2                   3
 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                  Destination MAC (Bytes 0-3)                  |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|  Destination MAC (Bytes 4-5)  |    Source MAC (Bytes 0-1)     |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                   Source MAC (Bytes 2-5)                      |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|          EtherType            |                               |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+                               |
```

### 2.2 IPv4 Header (20 bytes)
- Version = 4, IHL = 5
- Total Length, ID, Flags, TTL, Protocol (TCP=6, UDP=17, ICMP=1)
- RFC 1071 Checksum: 16-bit one's complement sum over 16-bit words.

### 2.3 TCP Segment Header (20 bytes)
- Flags: SYN, ACK, FIN, RST, PSH, URG.
- FSM: Closed $\to$ Listen $\to$ SynReceived $\to$ Established $\to$ FinWait $\to$ Closed.

---

## 3. Capability Enforcement Matrix

| Capability Flag | Value | Description |
|---|---|---|
| `CAP_NET_SOCKET` | `0x0001` | Permission to allocate a socket slot |
| `CAP_NET_BIND` | `0x0002` | Permission to bind socket to local port |
| `CAP_NET_LISTEN` | `0x0004` | Permission to accept incoming TCP connections |
| `CAP_NET_CONNECT` | `0x0008` | Permission to initiate outbound TCP/UDP connections |
| `CAP_NET_SEND` | `0x0010` | Permission to transmit payload bytes |
| `CAP_NET_RECV` | `0x0020` | Permission to read received payload bytes |
| `CAP_NET_RAW` | `0x0040` | Highly privileged raw frame generation |
