# Ind OS Specification: Accessibility Services (P0 Core Subsystem, SP-64)

- **Sprint Identifier:** SP-64
- **Phase:** Phase 8 (UX Shell + App Framework)
- **Target Gate:** Gate G8 (USER-READY OS V0.1)
- **Standards:** Ind OS Development Bible v1.0.0.0 §6.3, §6.4, §8.14, Table 19.2, REQ-UX-304 (P0 Requirement), Rule N-01, Rule N-04
- **Sign-off:** Sudhanshu Raj Singh <sudhanshurajsingh@users.noreply.github.com>

---

## 1. Architectural Objectives

1. **Accessibility IPC Bus (REQ-UX-304):**
   - High-priority, deterministic communication bus connecting UI semantic nodes to assistive technology daemons.
2. **Screen Reader Engine:**
   - Navigates bounded accessibility tree (`Role`, `Label`, `Value`, `State`), synthesizing speech-ready textual tokens with zero heap allocations (`INV-A11Y-001`).
3. **Magnification Service:**
   - Dynamic zoom factor scaling from 1.0x (normal) to 4.0x (high magnification) with center-point clamping (`INV-A11Y-002`).
4. **Switch Access Engine:**
   - Single-switch and dual-switch scanning mechanism supporting step scanning and selection for motor-impaired users (`INV-A11Y-003`).

---

## 2. Invariants & Security Guarantees

| Invariant ID | Requirement | Architectural Invariant |
| :--- | :--- | :--- |
| `INV-A11Y-001` | REQ-UX-304 | Screen reader navigates semantic node tree over high-priority accessibility IPC bus. |
| `INV-A11Y-002` | REQ-UX-304 | Magnification zoom level is strictly bounded in $[1.0, 4.0]$. |
| `INV-A11Y-003` | REQ-UX-304 | Switch access advances focus sequentially with cycle wrap and debounce. |

---

## 3. Data Structures & API Contract

```rust
pub enum A11yRole {
    Button,
    TextBox,
    Heading,
    Checkbox,
    Dialog,
}

pub struct A11yNode {
    pub id: u32,
    pub role: A11yRole,
    pub label: &'static str,
    pub value: &'static str,
    pub is_focused: bool,
}

pub struct ScreenReaderEngine {
    // Semantic tree traversal
}

pub struct MagnifierService {
    pub zoom_level: u32, // fixed-point e.g. 100 = 1.0x, 400 = 4.0x
}

pub struct SwitchAccessEngine {
    // Scan step & selection
}
```
