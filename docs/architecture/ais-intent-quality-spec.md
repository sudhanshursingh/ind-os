# Ind OS Architecture Specification: AIS Intent Quality Sprint (SP-49)

## Document Information
- **Standard**: Ind OS Development Bible v1.0.0.0 §6.3, §8.12
- **Requirement**: REQ-AI-110, Bible §8.12 (AIS intent quality sprint: 200-intent evaluation corpus; task-completion rate measurement >= 95%)
- **Invariants**:
  - `INV-QUALITY-001`: Frozen 200-Intent Evaluation Corpus — A deterministic benchmark suite representing real-world user intent formulations across all five canonical system action domains.
  - `INV-QUALITY-002`: Task Completion Rate Threshold — The intent extraction engine must achieve a minimum task completion rate of 95% (>= 190 / 200 successful schema-valid extractions) under zero ambient authority.
- **Rule Set**: Rule N-01 (100% pure Rust), Rule N-04 (Zero unsafe).

---

## 1. Evaluation Methodology
The evaluation suite runs deterministically against the intent broker and schema registry. Each test case consists of:
- `raw_input`: Simulated user input or system prompt.
- `expected_category`: Expected ActionCategory.
- `expected_action_code`: Action code.
- `required_confidence`: Minimum confidence score (0..100).

---

## 2. Benchmark Corpus Categories
The 200 intents are distributed across:
1. FileOperations (50 intents): Open, write, search, copy, delete.
2. DeviceControl (40 intents): Volume, brightness, bluetooth, wifi toggle.
3. SystemSetting (40 intents): Dark mode, locale, power profile, timeouts.
4. Notification (35 intents): Reminders, status banners, alerts.
5. NetworkTransfer (35 intents): Fetch updates, sync documents.

---

## 3. Kernel Verification Stages (192–196)
- `192_QUALITY_V1_INIT`: AIS intent quality evaluator initialized.
- `193_QUALITY_V1_CORPUS_LOAD`: 200-intent frozen evaluation corpus loaded.
- `194_QUALITY_V1_PARSER_BENCHMARK`: Intent parser evaluation executed across all corpus items.
- `195_QUALITY_V1_COMPLETION_RATE`: >= 95% task completion rate verified (REQ-AI-110).
- `196_SP49_QUALITY_READY`: Sprint SP-49 Milestone reached.
