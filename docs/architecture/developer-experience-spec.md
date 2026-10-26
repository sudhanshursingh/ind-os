# Ind OS Architectural Specification: Developer Experience (`indus-edit`, Remote Shell, Source Docs)
**Document Identifier:** `INDOS-SPEC-DEVX-01`  
**Sprint:** SP-38  
**Phase:** Phase 5 (Userspace & the Developer OS)  
**Status:** **APPROVED / ACTIVE**  
**Requirements Traceability:** Bible §8.11 (Sprint SP-38)  
**Invariants Enforced:** `INV-DEVX-001` (Bounded Text Buffer Safety), `INV-DEVX-002` (Capability-Mediated Remote Stream)  
**Author:** Sudhanshu Raj Singh (`sudhanshurajsingh@users.noreply.github.com`)

---

## 1. Scope & Objective

This specification details the developer tooling foundation for the **Developer OS v0.1** milestone, specifically:
1. **In-OS Text Editor (`edit` / `indus-edit`):** A lightweight, modal, `#![no_std]` text editor providing deterministic buffer manipulation, line insertion, deletion, and file I/O within Ind OS.
2. **Remote Shell over Serial / VirtIO:** Capability-authenticated interactive character bridge enabling headless development sessions from a host terminal.
3. **Docs Generated from Source:** In-tree verification of architecture specifications, requirement traceability, and docstrings.

---

## 2. Invariants

### 2.1 Invariant `INV-DEVX-001`: Bounded Text Buffer Safety
- The text editor buffer operates over static, fixed-capacity 2D memory arrays (`MAX_LINES = 64`, `MAX_COLS = 80`).
- Text insertion, deletion, and split operations are strictly bounds-checked and panic-free.
- Exceeding buffer capacity returns `EditError::BufferFull` without compromising memory safety.

### 2.2 Invariant `INV-DEVX-002`: Capability-Mediated Remote Stream
- Terminal input/output multiplexing over serial UART and VirtIO console requires an explicit `TerminalCapToken` granted by `devmgr`.
- Remote connections cannot spawn arbitrary processes or bypass the capability namespace.

---

## 3. Text Editor Architecture

```rust
pub struct TextBuffer {
    pub lines: [[u8; MAX_COLS]; MAX_LINES],
    pub line_lens: [usize; MAX_LINES],
    pub line_count: usize,
    pub cursor_row: usize,
    pub cursor_col: usize,
}
```
Supported editor actions:
- `insert_char(c)`
- `delete_char()`
- `newline()`
- `load_from_str(s)`
- `export_to_str(buf)`
