# Network Interop Specification: File Sharing & Remote Shell (SP-57)

**Standard:** Ind OS Development Bible §6.3, §8.13, Table 19.2  
**Status:** **RATIFIED**  
**Phase:** Phase 7 (SP-57)

---

## 1. Architectural Purpose & Scope

Sprint SP-57 integrates high-level application network protocols on top of the Ind OS core network stack (`indus-net`):
1. **Capability-Gated Network File Transfer (NFTP):** Secure file reading and writing across network streams with explicit storage capability gating.
2. **Network Remote Shell (NetShell):** Authenticated, capability-isolated remote shell sessions over TCP stream sockets.
3. **Loopback Interop & Ping/Pong:** Local loopback framing enabling reliable communication and diagnostic health probes across services.

---

## 2. Architectural Invariants

- **INV-NET-INTEROP-001 (Storage & Network Capability Binding):** All network file transfer requests must present both valid network stream tokens (`CAP_NET_SEND` / `CAP_NET_RECV`) and explicit storage capability tokens (`CAP_STORE_READ` or `CAP_STORE_ADMIN`). Requests without capabilities are rejected with `AccessDenied`.
- **INV-NET-INTEROP-002 (Authenticated Remote Shell Isolation):** Remote shell execution is bound to an authenticated session token. Shell processes inherit strictly bounded capability tokens with zero ambient root privileges.
- **INV-NET-INTEROP-003 (Loopback Diagnostic Verification):** High-speed, zero-allocation loopback engine for inter-service communication and deterministic diagnostic ping/pong testing.

---

## 3. Protocol Framing & Data Structures

```text
+-------------------------------------------------------------+
| NFTP Header: Opcode (1B) | CapToken (4B) | PathHash (4B)   |
+-------------------------------------------------------------+
| Offset (4B) | Length (2B) | Payload (0-256B)               |
+-------------------------------------------------------------+
```

```text
+-------------------------------------------------------------+
| NetShell Header: SessionId (4B) | AuthToken (4B) | CmdLen(2B)|
+-------------------------------------------------------------+
| Command Payload (0-128B)                                    |
+-------------------------------------------------------------+
```
