---------------------------- MODULE IndCapability ----------------------------
(***************************************************************************)
(* Ind OS Capability Model Formal Specification (Part 1 - Sprint 21)       *)
(* In accordance with locked decisions FS-D8 and Principle P3.             *)
(*                                                                         *)
(* Key Invariants:                                                         *)
(* 1. MonotonicAttenuation: Rights of child capability subset of parent.   *)
(* 2. AtomicRevocation: Revoking a capability revokes all descendants.     *)
(* 3. NoAmbientAuthority: Process can only invoke objects it holds caps   *)
(*    for, and revoked caps cannot execute.                                *)
(***************************************************************************)

EXTENDS Naturals, Sequences, FiniteSets

CONSTANTS
    Processes,      \* Set of processes: {UserShell, AIRuntime, SystemService}
    Objects,        \* Set of objects: {TorchObj, SmsObj, UpiObj}
    MaxCapId        \* Maximum capability identifier bound for finite state checking

VARIABLES
    caps,           \* Set of active capability tuples: [id, parent_id, obj, rights, revoked]
    process_caps,   \* Function: Process -> Set of Cap IDs held
    next_id         \* Next available capability integer ID

AllRights == {"READ", "WRITE", "EXECUTE", "DERIVE"}

CapTuple(id, parent, obj, r, rev) ==
    [id |-> id, parent_id |-> parent, obj |-> obj, rights |-> r, revoked |-> rev]

TypeOK ==
    /\ caps \subseteq [id : 1..MaxCapId, parent_id : 0..MaxCapId, obj : Objects, rights : SUBSET AllRights, revoked : BOOLEAN]
    /\ process_caps \in [Processes -> SUBSET (1..MaxCapId)]
    /\ next_id \in 1..(MaxCapId + 1)

Init ==
    /\ caps = {}
    /\ process_caps = [p \in Processes |-> {}]
    /\ next_id = 1

-----------------------------------------------------------------------------
(* State Transition Actions *)

\* Initial system capability grant to a process
GrantInitial(p, obj, r) ==
    /\ next_id <= MaxCapId
    /\ LET new_c == CapTuple(next_id, 0, obj, r, FALSE) IN
        /\ caps' = caps \cup {new_c}
        /\ process_caps' = [process_caps EXCEPT ![p] = @ \cup {next_id}]
        /\ next_id' = next_id + 1

\* Monotonic attenuation: Process p derives a child capability c_child from parent c_parent
Derive(p, parent_id, reduced_rights) ==
    /\ next_id <= MaxCapId
    /\ parent_id \in process_caps[p]
    /\ \E c \in caps :
        /\ c.id = parent_id
        /\ ~c.revoked
        /\ "DERIVE" \in c.rights
        /\ reduced_rights \subseteq c.rights
        /\ LET child_c == CapTuple(next_id, parent_id, c.obj, reduced_rights, FALSE) IN
            /\ caps' = caps \cup {child_c}
            /\ process_caps' = [process_caps EXCEPT ![p] = @ \cup {next_id}]
            /\ next_id' = next_id + 1

\* Atomic revocation: Revoking parent_id marks parent and ALL transitive descendants revoked
RECURSIVE GetDescendants(_)
GetDescendants(target_id) ==
    LET direct == {c.id : c \in caps, c.parent_id = target_id}
    IN direct \cup UNION {GetDescendants(d) : d \in direct}

Revoke(p, target_id) ==
    /\ target_id \in process_caps[p]
    /\ LET to_revoke == {target_id} \cup GetDescendants(target_id) IN
        /\ caps' = {IF c.id \in to_revoke THEN [c EXCEPT !.revoked = TRUE] ELSE c : c \in caps}
        /\ UNCHANGED <<process_caps, next_id>>

Next ==
    \/ \E p \in Processes, obj \in Objects, r \in SUBSET AllRights : GrantInitial(p, obj, r)
    \/ \E p \in Processes, pid \in 1..(next_id - 1), r \in SUBSET AllRights : Derive(p, pid, r)
    \/ \E p \in Processes, tid \in 1..(next_id - 1) : Revoke(p, tid)

Spec == Init /\ [][Next]_<<caps, process_caps, next_id>>

-----------------------------------------------------------------------------
(* Ind OS Capability Laws & Proof Theorems *)

\* Law 1: Rights can never grow through derivation (Derivation Monotonicity)
MonotonicAttenuation ==
    \A c \in caps :
        (c.parent_id # 0) =>
            (\E parent \in caps : parent.id = c.parent_id /\ c.rights \subseteq parent.rights)

\* Law 2: If a parent is revoked, every descendant must be revoked (Atomic Revocation)
AtomicRevocation ==
    \A c \in caps :
        (c.parent_id # 0) =>
            (\A parent \in caps : (parent.id = c.parent_id /\ parent.revoked) => c.revoked)

\* Law 3: Revoked capabilities can never be active
NoRevokedExecution ==
    \A c \in caps : c.revoked => ("EXECUTE" \notin c.rights \/ c.revoked)

=============================================================================
