# Ind OS Specification: App Framework, Plain-Language Manifests & Sandboxing (SP-63)

- **Sprint Identifier:** SP-63
- **Phase:** Phase 8 (UX Shell + App Framework)
- **Target Gate:** Gate G8 (USER-READY OS V0.1)
- **Standards:** Ind OS Development Bible v1.0.0.0 §6.3, §6.4, §8.14, Table 19.2, REQ-USE-207, Rule N-01, Rule N-04
- **Sign-off:** Sudhanshu Raj Singh <sudhanshurajsingh@users.noreply.github.com>

---

## 1. Architectural Objectives

1. **Plain-Language Capability Manifests (REQ-USE-207):**
   - Every app package must specify its requested capabilities alongside a plain-language user justification (e.g. "Storage: Save notes to local disk").
   - Undeclared capabilities cannot be requested at runtime (`INV-APP-001`).
2. **Deterministic Sandbox Spawning:**
   - Isolated execution domain, resource bounds (memory budget, execution slice), and capability token binding (`INV-APP-002`).
3. **User-Mediated Inter-App Sharing:**
   - Apps cannot directly exchange capabilities or resources without an explicit user grant (`INV-APP-003`).

---

## 2. Invariants & Security Guarantees

| Invariant ID | Requirement | Architectural Invariant |
| :--- | :--- | :--- |
| `INV-APP-001` | REQ-USE-207 | Apps must define valid plain-language manifests; undeclared capabilities are denied. |
| `INV-APP-002` | REQ-USE-207 | Sandboxed app spawn restricts process to declared capability mask. |
| `INV-APP-003` | REQ-USE-207 | Inter-app capability transfer requires an authenticated user mediation grant. |

---

## 3. Data Structures & API Contract

```rust
pub struct AppManifest {
    pub app_id: &'static str,
    pub display_name: &'static str,
    pub declared_caps: u64,
    pub justification: &'static str,
}

pub struct SandboxedApp {
    pub pid: u32,
    pub app_id: &'static str,
    pub active_caps: u64,
    pub memory_budget_kb: usize,
}

pub struct UserMediationGrant {
    pub grant_id: u32,
    pub source_pid: u32,
    pub target_pid: u32,
    pub granted_cap: u64,
    pub user_confirmed: bool,
}
```
