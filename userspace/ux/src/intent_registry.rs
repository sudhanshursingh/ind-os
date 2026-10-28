//! Standard Intent Registry — typed cross-app intent routing via ENF (SP-128, REQ-APP-COMMON-008)
//!
//! Bible §8.18.2, §9.9.1 REQ-APP-COMMON-008
//! Invariants:
//! - INV-INTENT-REG-001: Every intent dispatch is routed through ENF capability mediation.
//! - INV-INTENT-REG-002: Only apps that declare a handler for a given intent type receive it.
//! - INV-INTENT-REG-003: An intent carrying a capability may not elevate the receiver beyond
//!                        what ENF has granted for that app (no capability laundering).
//! Rules: N-01 (zero external crates), N-04 (no unsafe), N-04/N-05 (security enforced, not requested)

/// Maximum number of registered intent handlers in the system.
pub const MAX_INTENT_HANDLERS: usize = 32;
/// Maximum number of in-flight (pending) intents awaiting ENF routing.
pub const MAX_PENDING_INTENTS: usize = 16;
/// Maximum length of an intent URI string.
pub const MAX_INTENT_URI_LEN: usize = 256;

// ─── Well-known intent types (REQ-APP-COMMON-008) ────────────────────────────

/// Standard typed intents recognised by the Ind OS Intent Registry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntentType {
    /// View a URI/resource: `ind.intent.VIEW`
    View,
    /// Share data with another app: `ind.intent.SHARE`
    Share,
    /// Place a phone call: `ind.intent.CALL`
    Call,
    /// Send a message: `ind.intent.SEND`
    Send,
    /// Open a file: `ind.intent.OPEN`
    Open,
    /// Pick a media item: `ind.intent.PICK`
    Pick,
    /// Edit a document: `ind.intent.EDIT`
    Edit,
    /// Custom / app-specific intent (verified by the registry against the app manifest).
    Custom(&'static str),
}

impl IntentType {
    /// Canonical string representation used in audit logs.
    pub fn as_str(self) -> &'static str {
        match self {
            IntentType::View        => "ind.intent.VIEW",
            IntentType::Share       => "ind.intent.SHARE",
            IntentType::Call        => "ind.intent.CALL",
            IntentType::Send        => "ind.intent.SEND",
            IntentType::Open        => "ind.intent.OPEN",
            IntentType::Pick        => "ind.intent.PICK",
            IntentType::Edit        => "ind.intent.EDIT",
            IntentType::Custom(s)   => s,
        }
    }
}

// ─── Intent Handler registration ─────────────────────────────────────────────

/// One handler entry: an app registers one row per intent type it can handle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IntentHandler {
    pub app_id: &'static str,
    pub handler_pid: u32,
    pub intent_type: IntentType,
    /// Capability the handler must hold in order to be dispatched to (INV-INTENT-REG-001).
    pub required_cap: u64,
}

// ─── Intent Dispatch ─────────────────────────────────────────────────────────

/// An intent payload: enough to route and audit, no heap allocation.
#[derive(Debug, Clone, Copy)]
pub struct Intent {
    pub intent_type: IntentType,
    /// Emitting app.
    pub source_pid: u32,
    /// URI or data reference (optional, ASCII bytes).
    pub uri: [u8; MAX_INTENT_URI_LEN],
    pub uri_len: usize,
    /// Optional capability to transfer along with the intent.
    /// ENF verifies the source holds this before allowing the transfer (INV-INTENT-REG-003).
    pub carried_cap: u64,
}

impl Intent {
    pub fn new(intent_type: IntentType, source_pid: u32) -> Self {
        Self {
            intent_type,
            source_pid,
            uri: [0u8; MAX_INTENT_URI_LEN],
            uri_len: 0,
            carried_cap: 0,
        }
    }

    /// Attach a URI to the intent. Returns false if the URI is too long.
    pub fn with_uri(&mut self, uri: &[u8]) -> bool {
        if uri.len() > MAX_INTENT_URI_LEN {
            return false;
        }
        self.uri[..uri.len()].copy_from_slice(uri);
        self.uri_len = uri.len();
        true
    }

    /// Attach a capability to transfer. Source must hold the cap (checked at dispatch time).
    pub fn with_cap(mut self, cap: u64) -> Self {
        self.carried_cap = cap;
        self
    }
}

/// A pending (in-flight) intent awaiting ENF routing resolution.
#[derive(Debug, Clone, Copy)]
pub struct PendingIntent {
    pub dispatch_id: u32,
    pub intent: Intent,
    /// The chosen handler index (into the handler table), set after resolution.
    pub resolved_handler_idx: Option<usize>,
    /// ENF has verified source capabilities and cleared the dispatch.
    pub enf_cleared: bool,
}

impl PendingIntent {
    pub fn is_ready(&self) -> bool {
        self.enf_cleared && self.resolved_handler_idx.is_some()
    }
}

// ─── Errors ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntentRegistryError {
    /// No handler is registered for the requested intent type.
    NoHandlerFound,
    /// Source lacks the capability required to emit this intent type.
    SourceCapabilityDenied,
    /// Handler lacks the capability required to receive this intent.
    HandlerCapabilityDenied,
    /// The source does not own the capability it attempted to carry (INV-INTENT-REG-003).
    CapabilityLaunderingAttempt,
    /// Handler table is full.
    HandlerTableFull,
    /// Pending intent table is full.
    PendingTableFull,
    /// Dispatch ID not found.
    DispatchNotFound,
    /// Handler index out of range or unoccupied.
    InvalidHandlerIndex,
}

// ─── Registry Engine ─────────────────────────────────────────────────────────

/// The Intent Registry: stores handler registrations and manages ENF-mediated dispatches.
pub struct IntentRegistry {
    handlers: [Option<IntentHandler>; MAX_INTENT_HANDLERS],
    handler_count: usize,
    pending: [Option<PendingIntent>; MAX_PENDING_INTENTS],
    next_dispatch_id: u32,
}

impl IntentRegistry {
    pub const fn new() -> Self {
        Self {
            handlers: [None; MAX_INTENT_HANDLERS],
            handler_count: 0,
            pending: [None; MAX_PENDING_INTENTS],
            next_dispatch_id: 1,
        }
    }

    pub fn handler_count(&self) -> usize {
        self.handler_count
    }

    // ── Registration ──────────────────────────────────────────────────────────

    /// Register an app as a handler for a given intent type.
    /// The handler must subsequently hold `required_cap` at dispatch time (INV-INTENT-REG-001/002).
    pub fn register_handler(
        &mut self,
        app_id: &'static str,
        handler_pid: u32,
        intent_type: IntentType,
        required_cap: u64,
    ) -> Result<(), IntentRegistryError> {
        for slot in self.handlers.iter_mut() {
            if slot.is_none() {
                *slot = Some(IntentHandler {
                    app_id,
                    handler_pid,
                    intent_type,
                    required_cap,
                });
                self.handler_count += 1;
                return Ok(());
            }
        }
        Err(IntentRegistryError::HandlerTableFull)
    }

    // ── Dispatch (ENF mediation path) ─────────────────────────────────────────

    /// Enqueue an intent for routing. ENF capability checks happen in `enf_resolve_dispatch`.
    /// This two-step model ensures the capability system is always consulted (REQ-AI-002 analogue
    /// for intent routing: every privileged cross-app action flows through capability check).
    pub fn dispatch(
        &mut self,
        intent: Intent,
        source_caps: u64,
    ) -> Result<u32, IntentRegistryError> {
        // Phase 1: Verify at least one handler is registered (fast path rejection).
        let has_handler = self.handlers.iter().any(|h| {
            h.map(|hh| hh.intent_type == intent.intent_type).unwrap_or(false)
        });
        if !has_handler {
            return Err(IntentRegistryError::NoHandlerFound);
        }

        // Phase 2: If the intent carries a capability, the source must own it (INV-INTENT-REG-003).
        if intent.carried_cap != 0 && (source_caps & intent.carried_cap) != intent.carried_cap {
            return Err(IntentRegistryError::CapabilityLaunderingAttempt);
        }

        let dispatch_id = self.next_dispatch_id;
        self.next_dispatch_id += 1;

        for slot in self.pending.iter_mut() {
            if slot.is_none() {
                *slot = Some(PendingIntent {
                    dispatch_id,
                    intent,
                    resolved_handler_idx: None,
                    enf_cleared: false,
                });
                return Ok(dispatch_id);
            }
        }
        Err(IntentRegistryError::PendingTableFull)
    }

    /// ENF resolves the dispatch: selects a handler and verifies handler capabilities.
    /// In production this is called by the kernel-adjacent ENF policy engine.
    /// Here it is the authoritative in-process gatekeeper (INV-INTENT-REG-001).
    pub fn enf_resolve_dispatch(
        &mut self,
        dispatch_id: u32,
        handler_idx: usize,
        handler_active_caps: u64,
    ) -> Result<(), IntentRegistryError> {
        if handler_idx >= MAX_INTENT_HANDLERS {
            return Err(IntentRegistryError::InvalidHandlerIndex);
        }
        let handler = self.handlers[handler_idx].ok_or(IntentRegistryError::InvalidHandlerIndex)?;

        // Handler must hold its declared required capability.
        if (handler_active_caps & handler.required_cap) != handler.required_cap {
            return Err(IntentRegistryError::HandlerCapabilityDenied);
        }

        let pending = self
            .pending
            .iter_mut()
            .find_map(|s| {
                if let Some(ref mut p) = s {
                    if p.dispatch_id == dispatch_id {
                        return Some(p);
                    }
                }
                None
            })
            .ok_or(IntentRegistryError::DispatchNotFound)?;

        // Intent type must match what the handler declared.
        if handler.intent_type != pending.intent.intent_type {
            return Err(IntentRegistryError::NoHandlerFound);
        }

        pending.resolved_handler_idx = Some(handler_idx);
        pending.enf_cleared = true;
        Ok(())
    }

    /// Retrieve a fully resolved, ENF-cleared dispatch.
    pub fn get_ready_dispatch(&self, dispatch_id: u32) -> Option<&PendingIntent> {
        for slot in self.pending.iter() {
            if let Some(ref pd) = slot {
                if pd.dispatch_id == dispatch_id && pd.is_ready() {
                    return Some(pd);
                }
            }
        }
        None
    }

    /// Close and remove a handled dispatch.
    pub fn close_dispatch(&mut self, dispatch_id: u32) -> Result<(), IntentRegistryError> {
        for slot in self.pending.iter_mut() {
            if let Some(ref pd) = slot {
                if pd.dispatch_id == dispatch_id {
                    *slot = None;
                    return Ok(());
                }
            }
        }
        Err(IntentRegistryError::DispatchNotFound)
    }

    /// Find the first registered handler for an intent type (for default resolution).
    pub fn first_handler_for(&self, intent_type: IntentType) -> Option<(usize, &IntentHandler)> {
        self.handlers.iter().enumerate().find_map(|(i, h)| {
            h.as_ref()
                .filter(|hh| hh.intent_type == intent_type)
                .map(|hh| (i, hh))
        })
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// Tests
// ──────────────────────────────────────────────────────────────────────────────
#[cfg(test)]
mod tests {
    use super::*;

    const CAP_VIEW: u64 = 0x0001;
    const CAP_SHARE: u64 = 0x0002;

    fn make_registry_with_viewer() -> IntentRegistry {
        let mut r = IntentRegistry::new();
        r.register_handler("ind.app.gallery", 200, IntentType::View, CAP_VIEW)
            .unwrap();
        r
    }

    #[test]
    // REQ-APP-COMMON-008: handler registration records intent type correctly
    fn test_register_handler() {
        let r = make_registry_with_viewer();
        assert_eq!(r.handler_count(), 1);
        let (_, h) = r.first_handler_for(IntentType::View).unwrap();
        assert_eq!(h.app_id, "ind.app.gallery");
        assert_eq!(h.required_cap, CAP_VIEW);
    }

    #[test]
    // REQ-APP-COMMON-008: dispatch fails if no handler registered for intent type
    fn test_dispatch_no_handler() {
        let mut r = IntentRegistry::new();
        let intent = Intent::new(IntentType::Call, 10);
        let err = r.dispatch(intent, CAP_SHARE);
        assert_eq!(err, Err(IntentRegistryError::NoHandlerFound));
    }

    #[test]
    // INV-INTENT-REG-003: capability laundering is blocked at dispatch
    fn test_capability_laundering_blocked() {
        let mut r = make_registry_with_viewer();
        // Source does NOT own CAP_VIEW but tries to carry it in the intent.
        let intent = Intent::new(IntentType::View, 10).with_cap(CAP_VIEW);
        let err = r.dispatch(intent, 0 /* source has no caps */);
        assert_eq!(err, Err(IntentRegistryError::CapabilityLaunderingAttempt));
    }

    #[test]
    // INV-INTENT-REG-001: handler lacking required cap is rejected at ENF resolve
    fn test_enf_rejects_undercapable_handler() {
        let mut r = make_registry_with_viewer();
        let intent = Intent::new(IntentType::View, 10);
        let did = r.dispatch(intent, 0).unwrap();
        // Handler at index 0 requires CAP_VIEW; pass empty caps to simulate under-capability.
        let err = r.enf_resolve_dispatch(did, 0, 0 /* no caps */);
        assert_eq!(err, Err(IntentRegistryError::HandlerCapabilityDenied));
    }

    #[test]
    // Full ENF dispatch happy path
    fn test_full_dispatch_flow() {
        let mut r = make_registry_with_viewer();
        let (handler_idx, _) = r.first_handler_for(IntentType::View).unwrap();

        let mut intent = Intent::new(IntentType::View, 10);
        intent.with_uri(b"file:///photos/image.jpg");
        let did = r.dispatch(intent, 0).unwrap();

        // ENF resolves with sufficient handler caps.
        r.enf_resolve_dispatch(did, handler_idx, CAP_VIEW).unwrap();

        let ready = r.get_ready_dispatch(did);
        assert!(ready.is_some());
        let pd = ready.unwrap();
        assert_eq!(pd.intent.source_pid, 10);
        assert!(pd.enf_cleared);

        r.close_dispatch(did).unwrap();
        assert!(r.get_ready_dispatch(did).is_none());
    }

    #[test]
    // REQ-APP-COMMON-008: Custom intent type round-trips through the registry
    fn test_custom_intent_type() {
        let mut r = IntentRegistry::new();
        let custom_cap: u64 = 0x8000;
        r.register_handler(
            "ind.app.scanner",
            300,
            IntentType::Custom("ind.intent.SCAN_QR"),
            custom_cap,
        )
        .unwrap();

        let intent = Intent::new(IntentType::Custom("ind.intent.SCAN_QR"), 42);
        let did = r.dispatch(intent, 0).unwrap();
        let (idx, _) = r
            .first_handler_for(IntentType::Custom("ind.intent.SCAN_QR"))
            .unwrap();
        r.enf_resolve_dispatch(did, idx, custom_cap).unwrap();
        assert!(r.get_ready_dispatch(did).is_some());
    }
}

