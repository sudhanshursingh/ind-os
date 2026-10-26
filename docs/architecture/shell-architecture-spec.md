# Ind OS Architectural Specification: Userspace Shell (`indus-shell`)
**Document Identifier:** `INDOS-SPEC-SHELL-01`  
**Sprint:** SP-35  
**Phase:** Phase 5 (Userspace & the Developer OS)  
**Status:** **APPROVED / ACTIVE**  
**Requirements Traceability:** `REQ-USE-201`, `REQ-USE-202`, `REQ-USE-203` (Interactive Shell v1)  
**Invariants Enforced:** `INV-SH-001` (Bounded Panic-Free Parsing), `INV-SH-002` (Pipelines as Capability-Mediated IPC Chains)  
**Author:** Sudhanshu Raj Singh (`sudhanshurajsingh@users.noreply.github.com`)

---

## 1. Scope & Objective

This specification details the architecture, command parser, pipeline model, and execution semantics for **Shell v1 (`indus-shell`)**.

As the primary human-interactive component of Phase 5, `indus-shell` achieves the programme's **First Dogfood Milestone**: enabling a developer to execute interactive diagnostic commands, inspect process trees, read system files, and chain tools through capability-mediated pipelines without ambient authority.

---

## 2. Invariants

### 2.1 Invariant `INV-SH-001`: Bounded Panic-Free Parsing
- The command parser operates over stack-allocated fixed-capacity string buffers (`StaticString<256>`).
- Tokenization uses zero heap allocations and deterministic bounding (maximum 16 arguments per command).
- Malformed inputs, unclosed quotes, or line overflows return structured errors (`ParseError::LineTooLong`, `ParseError::UnclosedQuote`) without causing a panic or termination of the shell daemon.

### 2.2 Invariant `INV-SH-002`: Pipelines as Capability-Mediated IPC Chains
- Shell pipelines (e.g. `cmd1 | cmd2`) do not use ambient OS file descriptors or global memory buffers.
- Instead, pipelines establish an ephemeral capability-mediated IPC channel (`IpcChannel` / `CapabilityToken` from `libindus`).
- Command 1 is granted write rights (`CAP_RIGHTS_WRITE`) to the channel endpoint.
- Command 2 is granted read rights (`CAP_RIGHTS_READ`) to the matching channel endpoint.
- Channel closure or termination automatically revokes endpoint tokens.

---

## 3. Built-In Commands & Work Session

Shell v1 provides the following essential built-in commands:
- `help`: Lists available commands and syntax.
- `version`: Displays kernel version, architecture (ARM64), and OS build identity.
- `ps`: Inspects active userspace processes managed by `init` (`auditd`, `vfs-indfs`, `devmgr`, `shell`).
- `ls [path]`: Lists files and directories in mounted IndFS / tmpfs hierarchies.
- `cat [path]`: Streams file contents through standard output or pipeline IPC.
- `echo [args...]`: Prints formatted text.
- `caps`: Dumps active capability tokens held by the current shell context.
- `clear`: Emits ANSI escape sequence to reset terminal display.

---

## 4. Developer Dogfood Session Scenario

To satisfy the DoD for SP-35, the shell must successfully execute a scripted developer work session:
1. `version`: verify running kernel version.
2. `ps`: verify system services (`auditd`, `vfs-indfs`, `devmgr`).
3. `ls /sys`: inspect device hierarchy.
4. `cat /sys/manifest`: inspect declarative service manifest.
5. `echo "dogfood session active" | cat`: verify pipeline IPC chain data forwarding.
