# Ind OS Stage A Capability Broker & LSM Architecture
**Specification ID:** `IND-SEC-LSM-01`  
**Status:** DRAFT / APPROVED FOR STAGE A  
**Implementation Target:** Linux LSM (`ind_cap_lsm`) + Userspace Capability Broker (`ind-cap-brokerd`)

---

## 1. Overview & Objective

In Stage A (Sprints 19–82), mainline Linux serves as the driver host before transition to the seL4 microkernel in Stage B. 
To satisfy **FS-D1** and **FS-D8**, the capability model must be strictly non-bypassable even in the Linux userspace/kernel boundary.

The architecture comprises two synchronized enforcement tiers:
1. **`ind_cap_lsm` (Kernel-space Linux Security Module):**
   - Intercepts file descriptors, sockets, and memory mappings.
   - Enforces the strict air-gap on the AI daemon (`ind_ai_daemon`).
   - Rejects unmediated IPC or device access.
2. **`ind-cap-brokerd` (Userspace Capability Broker in TCB):**
   - Maintains the active capability table and derivation tree.
   - Maps legacy v1.0 SELinux domains to capability tokens.
   - Executes atomic cascade revocation.

```
+--------------------------------------------------------------------------+
|  Layer 7 Shell / Core Apps (Capability Token Consumers)                 |
+--------------------------------------------------------------------------+
                                     | (Intent ABI v1.0)
+--------------------------------------------------------------------------+
|  Layer 6: Semantic Gateway & Policy Engine                               |
|   +--------------------------------------------------------------------+ |
|   | ind-cap-brokerd (Userspace Capability Broker & Audit Logger)       | |
|   +--------------------------------------------------------------------+ |
+--------------------------------------------------------------------------+
                                     | (ioctl / Netlink)
+--------------------------------------------------------------------------+
|  Layer 3: Linux Kernel Core                                              |
|   +--------------------------------------------------------------------+ |
|   | ind_cap_lsm (Kernel LSM: File, Socket & Memory-Map Gatekeeper)     | |
|   +--------------------------------------------------------------------+ |
+--------------------------------------------------------------------------+
```

---

## 2. Mapping MVP SELinux Domains to Capability Tokens

During Stage A, legacy domains from Phase 6 v1.0 are mapped to formal capability sets:

| Legacy MVP Domain | Equivalent Stage A Capability Token URI | Permitted Rights | Air-Gap Restriction |
| :--- | :--- | :--- | :--- |
| `ind_ai_daemon` | `cap:sys.ai.runtime` | `EXECUTE`, `READ` (task-scoped MMOs) | **Strict 0 network capabilities.** AF_INET/AF_INET6 blocked at `socket_create`. |
| `ind_action_verifier` | `cap:sys.gateway.policy` | `DERIVE`, `REVOKE`, `INVOKE` | Local capability broker domain only. |
| `ind_shell` | `cap:ui.shell.launcher` | `INVOKE` (`GREEN` ambient, `AMBER` on voice) | Sandboxed Wayland surface client. |
| `ind_banking_guard` | `cap:financial.shield` | `READ`, `EXECUTE` (Biometric required) | Screen-blackout and OTP isolation capabilities. |

---

## 3. Kernel LSM Enforcement Points

`ind_cap_lsm` hooks the following LSM entry points:
- `socket_create`: Denied unconditionally for any process tagged with `cap:sys.ai.runtime`.
- `bprm_check_security`: Enforces that no binary without a valid SHA-256 signature in the transparency log can execute.
- `task_prctl`: Prevents privilege escalation or capability token tampering.
