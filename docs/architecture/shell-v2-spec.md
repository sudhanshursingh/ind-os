# Shell v2 Specification (The Ind Surface) (SP-61)

**Standard:** Ind OS Development Bible §6.3, §8.14, REQ-UX-305, Table 19.2  
**Status:** **RATIFIED**  
**Phase:** Phase 8 (SP-61)

---

## 1. Architectural Purpose & Scope

**Shell v2** represents the core user-facing environment ("The Ind Surface"), providing:
1. **Home Screen & App Grid:** Organized, capability-governed launcher surface.
2. **Notification Manager & AIS Distinction (REQ-UX-305):** Central notification feed where AIS proactive suggestions are visually distinct, non-coercive, and explicitly marked as advisory.
3. **Unified Search & Intent Bar:** System-wide input surface that accepts typed and spoken intents and seamlessly routes proposals to the Intent Broker.

---

## 2. Architectural Invariants

- **INV-SHELL2-001 (Visual Distinction of AIS Suggestions, REQ-UX-305):** Proactive AIS suggestions must carry mandatory non-coercive metadata and visual styling tags. They can never mimic critical system security alerts or bypass dismiss/snooze actions.
- **INV-SHELL2-002 (Unified Intent Input Dispatch):** Search/intent queries are parsed into structured intent proposals and dispatched directly to the Intent Broker with provenance tracking.
- **INV-SHELL2-003 (Bounded Notification Queue):** Static circular buffer of active notifications with zero heap allocation and instant dismiss/snooze lifecycle control.
