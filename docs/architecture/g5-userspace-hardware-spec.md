# Ind OS SP-103 Architecture Specification
## Full G5-Standard Userspace Running on Hardware

**Sprint:** SP-103 (Stage 465–469)  
**Standard:** Ind OS Development Bible v1.0.0.0 §6.3, §8.15, Table 19.2; REQ-USR-101, REQ-SEC-113  
**Target Hardware:** Motorola Moto G60 (Qualcomm Snapdragon 732G SM7150-AC, Kryo 470 CPU, 6GB LPDDR4X RAM)  
**Status:** APPROVED FOR IMPLEMENTATION

---

### 1. Purpose and Scope

Sprint SP-103 verifies that the complete Phase 5 (G5-standard) userspace ecosystem executes natively on physical ARM64 hardware HAL. This encompasses the five core processes:
- `PID 1: indus-init` (Root supervisor & capability arbitrator)
- `PID 2: indus-logd` (Structured tamper-evident logging daemon)
- `PID 3: indus-store` (Settings, consent, and IndFS storage manager)
- `PID 4: indus-shell` (UX Shell v2 compositor & intent surface)
- `PID 5: indus-edit` (Text editor & productivity tool)

All five processes execute concurrently with strict hardware memory isolation (ASID/TTBR0), capability domain attenuation, and cross-process IPC ring validation.

---

### 2. Architectural Invariants

- **INV-G5-HW-001 (TCB Core Services Hardware Spawn):**  
  `Init` (PID 1), `Logd` (PID 2), and `Store` (PID 3) spawn with minimal capability allocations:
  - `Init`: `CAP_SYS_ADMIN` (`0x0001_0000_0000_0000`)
  - `Logd`: `CAP_LOG_RECORD` (`0x0000_0000_0000_0100`)
  - `Store`: `CAP_STORAGE_MOUNT` (`0x0000_0000_0001_0000`)

- **INV-G5-HW-002 (UX Surface Applications Hardware Spawn):**  
  `Shell` (PID 4) and `Edit` (PID 5) spawn in isolated userspace address spaces with display, touch, and client IPC capabilities:
  - `Shell`: `CAP_DISPLAY_RENDER` + `CAP_TOUCH_INPUT`
  - `Edit`: `CAP_DISPLAY_RENDER` + `CAP_IPC_SYNC`

- **INV-G5-HW-003 (Deterministic Multi-Process IPC Concurrency):**  
  A 500-round cross-process synchronous IPC ring (`Init -> Logd -> Store -> Shell -> Edit -> Init`) must complete with zero dropped packets and latency bounded under 5ms per transaction round.

---

### 3. Constitutional Conformance

- **Rule N-01:** Zero external crates in TCB.
- **Rule N-04:** Pure memory-safe Rust in userspace implementation (`userspace/init/src/g5_hw.rs`).
- **Rule N-06 / PRD-001 §7.5:** Zero dynamic heap churn during IPC ring transit; static buffer rings ensure deterministic bounds.
