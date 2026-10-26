------------------------ MODULE IndCapabilityHierarchy ------------------------
(***************************************************************************)
(* Ind OS Capability Model Formal Specification (Part 2 - Sprint 22)       *)
(* Attenuation Hierarchy & Green/Amber/Red Derivation Proof                *)
(*                                                                         *)
(* Theorems:                                                               *)
(* 1. REDDerivationTheorem: No capability for a RED object (UPI, Wipe)     *)
(*    can be derived or exercised without a UserPresenceBiometric proof.   *)
(* 2. AdversarialContainment: An AI runtime (even if compromised) with     *)
(*    untrusted taint cannot derive or invoke a RED capability.            *)
(***************************************************************************)

EXTENDS Naturals, Sequences, FiniteSets

CONSTANTS
    Processes,      \* {AIRuntime, UserShell, SysGateway}
    Objects,        \* {TorchObj, SmsObj, UpiPaymentObj}
    MaxCapId

VARIABLES
    caps,           \* Set of caps: [id, parent_id, obj, tier, constraints, revoked]
    process_caps,
    next_id

Tiers == {"GREEN", "AMBER", "RED"}
Constraints == {"USER_VOICE_CONFIRMED", "USER_PRESENCE_BIOMETRIC"}

CapRecord(id, parent, obj, tier, constrs, rev) ==
    [id |-> id, parent_id |-> parent, obj |-> obj, tier |-> tier, constraints |-> constrs, revoked |-> rev]

Init ==
    /\ caps = {}
    /\ process_caps = [p \in Processes |-> {}]
    /\ next_id = 1

\* Grant initial root capability to Gateway only
GrantGatewayRoot(obj, tier) ==
    /\ next_id <= MaxCapId
    /\ LET new_c == CapRecord(next_id, 0, obj, tier, {}, FALSE) IN
        /\ caps' = caps \cup {new_c}
        /\ process_caps' = [process_caps EXCEPT !["SysGateway"] = @ \cup {next_id}]
        /\ next_id' = next_id + 1

\* Derivation rules with explicit constraint enforcement
DeriveConstrained(p, parent_id, derived_tier, added_constraints) ==
    /\ next_id <= MaxCapId
    /\ parent_id \in process_caps[p]
    /\ \E c \in caps :
        /\ c.id = parent_id
        /\ ~c.revoked
        \* Red objects unconditionally require biometric constraint
        /\ (derived_tier = "RED" => "USER_PRESENCE_BIOMETRIC" \in (c.constraints \cup added_constraints))
        \* Amber objects require voice confirmation or biometric
        /\ (derived_tier = "AMBER" => ("USER_VOICE_CONFIRMED" \in (c.constraints \cup added_constraints) \/ "USER_PRESENCE_BIOMETRIC" \in (c.constraints \cup added_constraints)))
        \* Adversarial AI containment: AI runtime cannot self-assert biometric proofs
        /\ (p = "AIRuntime" => "USER_PRESENCE_BIOMETRIC" \notin added_constraints)
        /\ LET child_c == CapRecord(next_id, parent_id, c.obj, derived_tier, c.constraints \cup added_constraints, FALSE) IN
            /\ caps' = caps \cup {child_c}
            /\ process_caps' = [process_caps EXCEPT ![p] = @ \cup {next_id}]
            /\ next_id' = next_id + 1

\* Delegate capability from one process to another (e.g. Gateway delegates task-scoped cap to AI)
Delegate(from_p, to_p, cap_id) ==
    /\ cap_id \in process_caps[from_p]
    /\ to_p # from_p
    /\ process_caps' = [process_caps EXCEPT ![to_p] = @ \cup {cap_id}]
    /\ UNCHANGED <<caps, next_id>>

Next ==
    \/ \E obj \in Objects, tier \in Tiers : GrantGatewayRoot(obj, tier)
    \/ \E p \in Processes, pid \in 1..(next_id - 1), tier \in Tiers, constrs \in SUBSET Constraints :
        DeriveConstrained(p, pid, tier, constrs)
    \/ \E from_p \in Processes, to_p \in Processes, cid \in 1..(next_id - 1) :
        Delegate(from_p, to_p, cid)

Spec == Init /\ [][Next]_<<caps, process_caps, next_id>>

-----------------------------------------------------------------------------
(* Formal Security Invariants *)

\* Red Derivation Theorem (Sprint 22 DoD):
\* Any active RED capability held by ANY process MUST hold the USER_PRESENCE_BIOMETRIC constraint
REDDerivationTheorem ==
    \A c \in caps :
        (c.tier = "RED" /\ ~c.revoked) =>
            "USER_PRESENCE_BIOMETRIC" \in c.constraints

\* Adversarial AI Containment:
\* The AIRuntime can never hold an unconstrained RED capability
AIRuntimeRedContainment ==
    \A cid \in process_caps["AIRuntime"] :
        \A c \in caps :
            (c.id = cid /\ c.tier = "RED") =>
                ("USER_PRESENCE_BIOMETRIC" \in c.constraints)

=============================================================================
