# Ind OS Intent ABI v1.0 — Public Specification
**Document ID:** `IND-ABI-1.0`  
**Status:** BIBLE-LOCKED / PUBLIC  
**Applies to:** Stage A (Mainline Linux) & Stage B (seL4 Microkernel)  
**Standard Port:** `unix:/run/ind/intent_gateway.sock`

---

## 1. Overview & Architectural Role

The **Intent ABI v1.0** defines the typed, non-bypassable Application Binary Interface across Layer 5 (AI Runtime in TEE/CCA), Layer 6 (Semantic Gateway & Policy Engine), and Layer 7 (Intent & Experience Shell).

In accordance with **Principle P3 (Security Below AI)** and **FS-D5 (Intent ABI v1.0)**:
1. **Never trust AI self-reported tier:** The AI runtime declares a `declared_tier`, but Layer 6 dynamically calculates the true `computed_tier` using hardcoded action type mappings, held capabilities, and data taint labels.
2. **Deterministic Taint Propagation:** Any action receiving `UNTRUSTED_SCREEN_TEXT`, `UNTRUSTED_WEB_CONTENT`, or `UNTRUSTED_SMS` cannot trigger an `AMBER` or `RED` capability without explicit escalation. Any attempt to route untrusted text into a money-moving or credential-altering capability immediately produces `DENY_AND_AUDIT`.
3. **No Ambient Authority:** Actions must explicitly reference capability URIs.

---

## 2. Intent Envelope Schema

```json
{
  "$schema": "https://indos.dev/schemas/intent-v1.json",
  "intent_id": "urn:uuid:f47ac10b-58cc-4372-a567-0e02b2c3d479",
  "timestamp_ms": 1793000000000,
  "session_id": "urn:uuid:6ba7b810-9dad-11d1-80b4-00c04fd430c8",
  "declared_tier": "GREEN",
  "actions": [
    {
      "action_type": "SYS_TOGGLE",
      "target": "cap:sys.device.torch",
      "params": {
        "state": "on"
      },
      "data_taint_labels": ["USER_VOICE"],
      "required_capabilities": ["cap:sys.device.torch"]
    }
  ],
  "user_feedback": {
    "tts_text": "Turning on torch",
    "visual_state": "STATUS_INDICATOR"
  }
}
```

---

## 3. Action Types & Base Tiers

| Action Type | Description | Base Tier | Permitted Taint Sources |
| :--- | :--- | :--- | :--- |
| `SYS_TOGGLE` | Low-risk hardware switch (torch, rotation, brightness) | `GREEN` | `USER_VOICE`, `SYSTEM_STATE` |
| `READ_SCREEN` | Read visible semantic layout text | `GREEN` | `USER_VOICE` |
| `OPEN_APP` | Launch capability-native application | `AMBER` | `USER_VOICE` |
| `INPUT_TEXT` | Enter text into input field | `AMBER` | `USER_VOICE` |
| `CLICK_NODE` | Tap interactive surface node | `AMBER` | `USER_VOICE` |
| `SCROLL` | Viewport scroll navigation | `GREEN` | `USER_VOICE` |
| `PLACE_CALL` | Initiate telephony voice call | `AMBER` | `USER_VOICE` |
| `SEND_MESSAGE` | Transmit SMS / instant message | `AMBER` | `USER_VOICE` |
| `PAYMENT_INTENT` | High-risk financial transaction / UPI | `RED` | `USER_VOICE` (Must have `user_presence_biometric`) |

---

## 4. Gateway Decision Codes

The Semantic Gateway responds to every submitted `IntentEnvelope` with one of four canonical decisions:

1. `ALLOW`:
   - Computed tier is `GREEN` or held capability satisfies requirements.
   - All taint constraints are met.
   - Action dispatched immediately.
2. `ALLOW_WITH_CONFIRMATION`:
   - Computed tier is `AMBER`.
   - Requires explicit voice confirmation (`"Confirm send SMS to..."`) before capability token attenuation.
3. `DENY`:
   - Required capability is absent, schema validation fails, or action is syntactically invalid.
   - Logged normally; no security incident flagged.
4. `DENY_AND_AUDIT`:
   - Critical invariant violated or hostile behavior detected:
     - Prompt injection attempt detected.
     - `UNTRUSTED_SCREEN_TEXT` directed toward `PAYMENT_INTENT` or `SEND_MESSAGE`.
     - Attempt to bypass biometric proof on a `RED` capability.
   - Session terminated; hash-chained incident logged to hardware-backed audit log.

---

## 5. Formal Schema Definition

### 5.1 JSON-Schema (Draft 2020-12)

The authoritative schema is maintained at `kernel/docs/intent-v1.schema.json`.
Validation rules enforce:
- Valid UUID format for `intent_id` and `session_id`.
- Integer epoch millisecond timestamp > 0.
- Enumerated `declared_tier` in `["GREEN", "AMBER", "RED"]`.
- `actions` array containing 1 to 16 action items.
- Unique URI format for `required_capabilities`.
