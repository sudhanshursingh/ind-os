#!/usr/bin/env python3
"""
Test runner for the 200 machine-readable Intent ABI v1.0 test vectors (Sprint 20 DoD).
Verifies:
1. All 200 vectors pass.
2. 100% of malformed corpus rejected with DENY.
3. 100% of hostile taint/injection rejected with DENY_AND_AUDIT.
4. Autonomous computed tier correctly derived regardless of AI's declared tier.
"""

import sys
import os
import json
import re

VECTORS_FILE = "kernel/docs/test-vectors/intent_vectors_200.json"
UUID_REGEX = re.compile(r"^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$", re.IGNORECASE)
CAP_REGEX = re.compile(r"^cap:[a-z0-9_.-]+$")

VALID_ACTION_TIERS = {
    "SYS_TOGGLE": "GREEN",
    "SCROLL": "GREEN",
    "READ_SCREEN": "GREEN",
    "OPEN_APP": "AMBER",
    "CLICK_NODE": "AMBER",
    "INPUT_TEXT": "AMBER",
    "PLACE_CALL": "AMBER",
    "SEND_MESSAGE": "AMBER",
    "PAYMENT_INTENT": "RED",
}

TIER_ORDER = {"GREEN": 0, "AMBER": 1, "RED": 2}

def validate_vector(envelope):
    # Structural checks
    if not isinstance(envelope, dict):
        return "DENY", None, "Root must be object"

    # Reject unexpected root fields
    allowed_root = {"intent_id", "timestamp_ms", "session_id", "declared_tier", "actions", "user_feedback"}
    if any(k not in allowed_root for k in envelope.keys()):
        return "DENY", None, "Extra forbidden property in root"

    intent_id = envelope.get("intent_id")
    if not isinstance(intent_id, str) or not UUID_REGEX.match(intent_id):
        return "DENY", None, "Invalid intent_id UUID"

    session_id = envelope.get("session_id")
    if not isinstance(session_id, str) or not UUID_REGEX.match(session_id):
        return "DENY", None, "Invalid session_id UUID"

    timestamp_ms = envelope.get("timestamp_ms")
    if not isinstance(timestamp_ms, int) or timestamp_ms <= 0:
        return "DENY", None, "timestamp_ms must be positive integer"

    declared_tier = envelope.get("declared_tier")
    if declared_tier not in ["GREEN", "AMBER", "RED"]:
        return "DENY", None, "Invalid declared_tier"

    actions = envelope.get("actions")
    if not isinstance(actions, list) or len(actions) == 0 or len(actions) > 16:
        return "DENY", None, "actions length must be 1..16"

    max_tier_val = 0
    max_tier_name = "GREEN"

    for act in actions:
        if not isinstance(act, dict):
            return "DENY", None, "Action item must be object"
            
        allowed_act_keys = {"action_type", "target", "params", "data_taint_labels", "required_capabilities"}
        if any(k not in allowed_act_keys for k in act.keys()):
            return "DENY", None, "Extra property in action item"

        action_type = act.get("action_type")
        if action_type not in VALID_ACTION_TIERS:
            return "DENY", None, f"Unknown action_type: {action_type}"

        target = act.get("target")
        if not isinstance(target, str) or len(target) == 0 or len(target) > 256:
            return "DENY", None, "Invalid target"

        params = act.get("params")
        if not isinstance(params, dict):
            return "DENY", None, "params must be object"

        taint_labels = act.get("data_taint_labels")
        if not isinstance(taint_labels, list):
            return "DENY", None, "data_taint_labels must be list"

        caps = act.get("required_capabilities")
        if not isinstance(caps, list):
            return "DENY", None, "required_capabilities must be list"

        for cap in caps:
            if not isinstance(cap, str) or not CAP_REGEX.match(cap):
                return "DENY", None, f"Malformed capability URI: {cap}"

        tier_name = VALID_ACTION_TIERS[action_type]
        tier_val = TIER_ORDER[tier_name]
        if tier_val > max_tier_val:
            max_tier_val = tier_val
            max_tier_name = tier_name

        # Taint check
        is_untrusted = any(l in ["UNTRUSTED_SCREEN_TEXT", "UNTRUSTED_WEB_CONTENT", "UNTRUSTED_SMS"] for l in taint_labels)
        if is_untrusted:
            if tier_name == "RED" or action_type == "SEND_MESSAGE":
                return "DENY_AND_AUDIT", tier_name, "Hostile taint violation"

    if max_tier_name == "GREEN":
        decision = "ALLOW"
    else:
        decision = "ALLOW_WITH_CONFIRMATION"

    return decision, max_tier_name, "Valid"

def main():
    print("==================================================")
    print(" Ind OS Intent ABI v1.0 — 200 Vector Test Suite")
    print(" Target: Sprint 20 Definition of Done (DoD)")
    print("==================================================")

    if not os.path.exists(VECTORS_FILE):
        print(f"❌ Test vectors file not found: {VECTORS_FILE}")
        sys.exit(1)

    with open(VECTORS_FILE, "r", encoding="utf-8") as f:
        vectors = json.load(f)

    print(f"Loaded {len(vectors)} test vectors from: {VECTORS_FILE}\n")

    passed = 0
    failed = 0
    malformed_rejected = 0
    hostile_blocked = 0

    for v in vectors:
        v_id = v["id"]
        cat = v["category"]
        exp_dec = v["expected_decision"]
        exp_tier = v.get("expected_computed_tier")
        envelope = v["envelope"]

        dec, tier, detail = validate_vector(envelope)

        if cat == "MALFORMED" and dec == "DENY":
            malformed_rejected += 1
        if cat == "HOSTILE_TAINT" and dec == "DENY_AND_AUDIT":
            hostile_blocked += 1

        match = (dec == exp_dec)
        if exp_tier is not None and tier != exp_tier:
            match = False

        if match:
            passed += 1
        else:
            failed += 1
            print(f"❌ FAILED [{v_id}] {cat}: expected ({exp_dec}, {exp_tier}) got ({dec}, {tier}) - {detail}")

    print("--------------------------------------------------")
    print(f"Total Vectors Evaluated      : {len(vectors)}")
    print(f"Passed                      : {passed}")
    print(f"Failed                      : {failed}")
    print(f"Malformed Rejected (100%)   : {malformed_rejected}/60")
    print(f"Hostile Blocked & Audited   : {hostile_blocked}/30")
    print("--------------------------------------------------")

    if failed > 0:
        print("❌ Sprint 20 DoD NOT met: one or more test vectors failed.")
        sys.exit(1)

    print("✅ Sprint 20 DoD MET: All 200 vectors pass with 100% precision.")
    print("==================================================")

if __name__ == "__main__":
    main()
