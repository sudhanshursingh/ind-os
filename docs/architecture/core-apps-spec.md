# Ind OS Specification: Core Apps Suite (SP-66)

- **Sprint Identifier:** SP-66
- **Phase:** Phase 8 (UX Shell + App Framework)
- **Target Gate:** Gate G8 (USER-READY OS V0.1)
- **Standards:** Ind OS Development Bible v1.0.0.0 §6.3, §6.4, §8.14, Table 19.2, REQ-USE-207, Rule N-01, Rule N-04
- **Sign-off:** Sudhanshu Raj Singh <sudhanshurajsingh@users.noreply.github.com>

---

## 1. Architectural Objectives

1. **Least-Privilege Core Apps (REQ-USE-207):**
   - Five dogfood applications running with restricted capability masks:
     1. `FilesApp`: File manager browsing flash files (`CORE_CAP_READ`).
     2. `SettingsApp`: Consent and system controls surface (`KERNEL_CAP_STORE_ADMIN`).
     3. `WhyExplorerApp`: Auditable explanation and causal timeline explorer (`CORE_CAP_READ`).
     4. `TerminalApp`: Interactive command-line environment (`CORE_CAP_READ | CORE_CAP_WRITE`).
     5. `NotesApp`: Secure plain-text notepad (`CORE_CAP_READ | CORE_CAP_WRITE`).
2. **Settings Consent Surface:**
   - Provides live review and revocation of user-mediated capability grants (`INV-COREAPPS-002`).
3. **Why-Explorer Causal Navigation:**
   - Interrogates `KernelWhyEngine` to present auditable cause-and-effect explanations (`INV-COREAPPS-003`).

---

## 2. Invariants & Security Guarantees

| Invariant ID | Requirement | Architectural Invariant |
| :--- | :--- | :--- |
| `INV-COREAPPS-001` | REQ-USE-207 | All 5 core applications initialize with verified least-privilege capability manifests. |
| `INV-COREAPPS-002` | REQ-USE-207 | Settings app consent surface successfully audits and revokes active capability grants. |
| `INV-COREAPPS-003` | REQ-AI-105 | Why-Explorer retrieves causal explanation records directly from the Why Engine. |

---

## 3. Data Structures & API Contract

```rust
pub struct CoreAppDescriptor {
    pub name: &'static str,
    pub app_id: &'static str,
    pub required_caps: u64,
    pub is_running: bool,
}

pub struct CoreAppsSuite {
    pub files: CoreAppDescriptor,
    pub settings: CoreAppDescriptor,
    pub why_explorer: CoreAppDescriptor,
    pub terminal: CoreAppDescriptor,
    pub notes: CoreAppDescriptor,
}
```
