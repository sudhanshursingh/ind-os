# TLS Integration Specification (SP-52, DEF-004, DEC-004)
**Phase:** Phase 7 — Network + Updates + Service Completion (SP-51 – SP-59)  
**Sprint:** SP-52 — TLS Integration  
**Document ID:** `INDOS-SPEC-TLS-001`  
**Status:** **APPROVED & IMPLEMENTED**  
**Author:** Ind OS Security & Networking Architecture Group  

---

## 1. Architectural Invariants

- **INV-TLS-001 (Non-TCB Userspace Isolation):** The TLS engine runs strictly outside the kernel TCB. Sockets are accessed only via capability tokens (`CAP_NET_CONNECT`, `CAP_NET_SEND`, `CAP_NET_RECV`).
- **INV-TLS-002 (TLS 1.3 Record Layer & Pinned Ciphers):** Strict enforcement of TLS 1.3 record format (`0x0304`). Negotiated ciphers restricted to AEAD suites:
  - `TLS_AES_128_GCM_SHA256` (`0x1301`)
  - `TLS_CHACHA20_POLY1305_SHA256` (`0x1303`)
- **INV-TLS-003 (Trust Anchor Validation & Anti-Tamper):** Trust anchor root certificates, hostname/SAN validation, and strict rejection of unauthorized or forged certificates.

---

## 2. Record Layer & Handshake Flow

```text
 Client                                               Server
   |                                                    |
   | ------------ ClientHello (TLS 1.3, SNI) ---------> |
   |                                                    |
   | <----------- ServerHello (TLS 1.3, Cipher) ------- |
   | <----------- EncryptedExtensions ----------------- |
   | <----------- Certificate & CertVerify ------------ |
   | <----------- Finished ---------------------------- |
   |                                                    |
   | ------------ Finished ---------------------------> |
   |                                                    |
   | <======== Authenticated & Encrypted Stream =======> |
```

---

## 3. Capability-Gated Stream Interface

```rust
pub struct TlsStream {
    socket_token: SocketCapToken,
    state: TlsSessionState,
    cipher: CipherSuite,
    server_name: &'static str,
}
```
All read/write operations on `TlsStream` require capability rights on the underlying `SocketCapToken`. Unprivileged callers cannot inject or eavesdrop on encrypted session data.
