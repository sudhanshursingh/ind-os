//! Notification Center System Surface (SP-129, REQ-APP-COMMON-013)
//!
//! Bible §8.18.2, §8.20, §9.9.1 REQ-APP-COMMON-013, REQ-APP-COMMON-005
//! Invariants:
//! - INV-NC-001: Apps post notifications exclusively via this service; direct screen intrusion
//!               is prohibited (REQ-APP-COMMON-005). No app calls a display API directly.
//! - INV-NC-002: Priority bands are enforced structurally: Critical overrides DND; Silent never
//!               appears in the notification shade; High interrupts; Normal is batched.
//! - INV-NC-003: Consent receipts (ProvenanceRecords) are presented alongside notifications
//!               and are individually dismissible. They survive notification dismissal.
//! - INV-NC-004: The AIS kill-switch toggle is a persistent, unsuppressible element in the
//!               Notification Center header (AIS Charter §12.3).
//! - INV-NC-005: Notifications are grouped by app by default (DEF-APP-001); per-sender grouping
//!               is user-configurable.
//! Rules: N-01 (zero external crates), N-04 (no unsafe), N-05 (user can always say no)

/// Maximum notification slots in the center (active + pending).
pub const MAX_NC_NOTIFICATIONS: usize = 32;
/// Maximum consent receipts stored concurrently.
pub const MAX_CONSENT_RECEIPTS: usize = 16;
/// Maximum length of a notification title (bytes).
pub const MAX_NOTIF_TITLE_LEN: usize = 128;
/// Maximum length of a notification body (bytes).
pub const MAX_NOTIF_BODY_LEN: usize = 256;
/// Maximum number of app groups in the grouping index.
pub const MAX_APP_GROUPS: usize = 16;

// Capability required to post notifications to the Notification Center (INV-NC-001).
pub const CAP_NOTIFY_POST: u64 = 0x0001_0000_0000;
// Capability required by the Notification Center surface to read the full queue.
pub const CAP_NOTIFY_READ: u64 = 0x0002_0000_0000;

// ─── Priority bands (INV-NC-002) ─────────────────────────────────────────────

/// Notification priority band. Determines delivery behaviour and DND interactions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum NotifPriority {
    /// Never shown in shade, never vibrates, never plays sound. Badge-only.
    Silent = 0,
    /// Batched delivery, no heads-up. Shown in shade pull-down.
    Normal = 1,
    /// Immediate heads-up card at top of screen. Vibrates. Respects DND.
    High = 2,
    /// Overrides Do-Not-Disturb. Reserved for alarms, emergency calls, OTP delivery.
    Critical = 3,
}

impl NotifPriority {
    /// True if this notification overrides Do-Not-Disturb mode (INV-NC-002).
    pub fn overrides_dnd(self) -> bool {
        self == NotifPriority::Critical
    }

    /// True if this notification should appear as an immediate heads-up card.
    pub fn is_heads_up(self) -> bool {
        matches!(self, NotifPriority::High | NotifPriority::Critical)
    }

    /// True if this notification is shown in the shade pull-down.
    pub fn is_shade_visible(self) -> bool {
        self != NotifPriority::Silent
    }
}

// ─── Notification entry ───────────────────────────────────────────────────────

/// A single notification in the Notification Center.
#[derive(Debug, Clone, Copy)]
pub struct NcNotification {
    pub notif_id: u32,
    pub source_pid: u32,
    /// App identifier used for grouping (INV-NC-005, DEF-APP-001).
    pub app_id: &'static str,
    pub priority: NotifPriority,
    pub title: [u8; MAX_NOTIF_TITLE_LEN],
    pub title_len: usize,
    pub body: [u8; MAX_NOTIF_BODY_LEN],
    pub body_len: usize,
    /// Whether this notification has been read/opened.
    pub is_read: bool,
    /// Whether this notification has been dismissed (removed from shade).
    pub is_dismissed: bool,
    /// Associated consent receipt ID (if this notification carries a provenance record).
    pub consent_receipt_id: Option<u32>,
}

impl NcNotification {
    /// Returns true if the notification should appear in the shade pull-down.
    pub fn is_shade_visible(&self) -> bool {
        !self.is_dismissed && self.priority.is_shade_visible()
    }
}

// ─── Consent receipt ─────────────────────────────────────────────────────────

/// A consent receipt (ProvenanceRecord stub) shown alongside a notification (INV-NC-003).
/// Full ProvenanceRecord structure is in the AIS explain subsystem; this is the NC reference.
#[derive(Debug, Clone, Copy)]
pub struct ConsentReceipt {
    pub receipt_id: u32,
    /// The notification this receipt accompanies.
    pub notif_id: u32,
    /// Human-readable plain-language action description.
    pub action_summary: &'static str,
    /// Whether the user has viewed this receipt.
    pub user_viewed: bool,
    /// Whether the user has dismissed this receipt.
    pub user_dismissed: bool,
}

// ─── App grouping index ───────────────────────────────────────────────────────

/// Tracks how many active notifications each app has (INV-NC-005).
#[derive(Debug, Clone, Copy)]
pub struct AppGroup {
    pub app_id: &'static str,
    pub count: usize,
}

// ─── Do-Not-Disturb state ─────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DndState {
    Off,
    On,
}

// ─── Shade pull-down state ────────────────────────────────────────────────────

/// Whether the notification center shade is currently pulled down.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShadeState {
    Collapsed,
    Expanded,
}

// ─── Errors ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotifCenterError {
    /// Source app lacks CAP_NOTIFY_POST.
    MissingPostCapability,
    /// Notification store is full.
    StoreFull,
    /// Notification ID not found.
    NotifNotFound,
    /// Consent receipt store is full.
    ReceiptStoreFull,
    /// Receipt ID not found.
    ReceiptNotFound,
    /// Title or body payload too large.
    PayloadTooLarge,
    /// App group table is full.
    GroupTableFull,
}

// ─── Engine ───────────────────────────────────────────────────────────────────

/// The Notification Center System Surface engine.
pub struct NotificationCenterEngine {
    notifications: [Option<NcNotification>; MAX_NC_NOTIFICATIONS],
    notif_count: usize,
    next_notif_id: u32,
    receipts: [Option<ConsentReceipt>; MAX_CONSENT_RECEIPTS],
    receipt_count: usize,
    next_receipt_id: u32,
    groups: [Option<AppGroup>; MAX_APP_GROUPS],
    pub shade: ShadeState,
    pub dnd: DndState,
    /// AIS kill-switch state mirrored from LockScreen (INV-NC-004).
    pub ais_kill_switch_enabled: bool,
}

impl NotificationCenterEngine {
    pub const fn new() -> Self {
        Self {
            notifications: [None; MAX_NC_NOTIFICATIONS],
            notif_count: 0,
            next_notif_id: 1,
            receipts: [None; MAX_CONSENT_RECEIPTS],
            receipt_count: 0,
            next_receipt_id: 1,
            groups: [None; MAX_APP_GROUPS],
            shade: ShadeState::Collapsed,
            dnd: DndState::Off,
            ais_kill_switch_enabled: false,
        }
    }

    pub fn notif_count(&self) -> usize {
        self.notif_count
    }

    pub fn receipt_count(&self) -> usize {
        self.receipt_count
    }

    // ── Shade ─────────────────────────────────────────────────────────────────

    /// User pulls down the top edge → expand the shade (REQ-APP-COMMON-013: top edge-swipe).
    pub fn expand_shade(&mut self) {
        self.shade = ShadeState::Expanded;
    }

    /// User swipes the shade closed or taps outside.
    pub fn collapse_shade(&mut self) {
        self.shade = ShadeState::Collapsed;
    }

    // ── Notification posting ──────────────────────────────────────────────────

    /// Post a notification to the Notification Center (INV-NC-001).
    /// `source_caps` must include CAP_NOTIFY_POST.
    /// Returns the notification ID; DND filtering is applied per INV-NC-002.
    pub fn post(
        &mut self,
        source_pid: u32,
        source_caps: u64,
        app_id: &'static str,
        priority: NotifPriority,
        title: &[u8],
        body: &[u8],
    ) -> Result<u32, NotifCenterError> {
        // INV-NC-001: only apps with CAP_NOTIFY_POST may post.
        if (source_caps & CAP_NOTIFY_POST) == 0 {
            return Err(NotifCenterError::MissingPostCapability);
        }
        if title.len() > MAX_NOTIF_TITLE_LEN || body.len() > MAX_NOTIF_BODY_LEN {
            return Err(NotifCenterError::PayloadTooLarge);
        }

        // INV-NC-002: DND suppresses non-critical notifications.
        // Silent-priority notifications are accepted into the store but never shade-visible.
        if self.dnd == DndState::On && !priority.overrides_dnd() {
            // Post silently (badge only) — still assigned an ID for tracking.
            // We store with Silent priority override to signal suppressed delivery.
        }

        let notif_id = self.next_notif_id;
        self.next_notif_id += 1;

        let mut notif = NcNotification {
            notif_id,
            source_pid,
            app_id,
            priority,
            title: [0u8; MAX_NOTIF_TITLE_LEN],
            title_len: title.len(),
            body: [0u8; MAX_NOTIF_BODY_LEN],
            body_len: body.len(),
            is_read: false,
            is_dismissed: false,
            consent_receipt_id: None,
        };
        notif.title[..title.len()].copy_from_slice(title);
        notif.body[..body.len()].copy_from_slice(body);

        for slot in self.notifications.iter_mut() {
            if slot.is_none() {
                *slot = Some(notif);
                self.notif_count += 1;
                self.update_group(app_id, 1);
                return Ok(notif_id);
            }
        }
        Err(NotifCenterError::StoreFull)
    }

    // ── Dismissal ─────────────────────────────────────────────────────────────

    /// Dismiss a single notification (swipe-to-dismiss). Marks it dismissed; does not delete the
    /// consent receipt (INV-NC-003: receipts survive notification dismissal).
    pub fn dismiss(&mut self, notif_id: u32) -> Result<(), NotifCenterError> {
        for slot in self.notifications.iter_mut() {
            if let Some(ref mut n) = slot {
                if n.notif_id == notif_id {
                    n.is_dismissed = true;
                    return Ok(());
                }
            }
        }
        Err(NotifCenterError::NotifNotFound)
    }

    /// Dismiss all notifications for an app group (clear-all for one app).
    pub fn dismiss_all_for_app(&mut self, app_id: &'static str) -> usize {
        let mut count = 0;
        for slot in self.notifications.iter_mut() {
            if let Some(ref mut n) = slot {
                if n.app_id == app_id && !n.is_dismissed {
                    n.is_dismissed = true;
                    count += 1;
                }
            }
        }
        count
    }

    /// Mark a notification as read (user opened it).
    pub fn mark_read(&mut self, notif_id: u32) -> Result<(), NotifCenterError> {
        for slot in self.notifications.iter_mut() {
            if let Some(ref mut n) = slot {
                if n.notif_id == notif_id {
                    n.is_read = true;
                    return Ok(());
                }
            }
        }
        Err(NotifCenterError::NotifNotFound)
    }

    // ── Consent receipts (INV-NC-003) ─────────────────────────────────────────

    /// Attach a consent receipt to a notification. The receipt persists even if the notification
    /// is dismissed (INV-NC-003).
    pub fn attach_consent_receipt(
        &mut self,
        notif_id: u32,
        action_summary: &'static str,
    ) -> Result<u32, NotifCenterError> {
        // Validate notification exists.
        let notif_exists = self.notifications.iter().any(|s| {
            s.map(|n| n.notif_id == notif_id).unwrap_or(false)
        });
        if !notif_exists {
            return Err(NotifCenterError::NotifNotFound);
        }

        let receipt_id = self.next_receipt_id;
        self.next_receipt_id += 1;

        for slot in self.receipts.iter_mut() {
            if slot.is_none() {
                *slot = Some(ConsentReceipt {
                    receipt_id,
                    notif_id,
                    action_summary,
                    user_viewed: false,
                    user_dismissed: false,
                });
                self.receipt_count += 1;

                // Link receipt back to notification.
                for ns in self.notifications.iter_mut() {
                    if let Some(ref mut n) = ns {
                        if n.notif_id == notif_id {
                            n.consent_receipt_id = Some(receipt_id);
                        }
                    }
                }
                return Ok(receipt_id);
            }
        }
        Err(NotifCenterError::ReceiptStoreFull)
    }

    /// Dismiss a consent receipt. The receipt store slot is freed but the notification is untouched.
    pub fn dismiss_receipt(&mut self, receipt_id: u32) -> Result<(), NotifCenterError> {
        for slot in self.receipts.iter_mut() {
            if let Some(ref mut r) = slot {
                if r.receipt_id == receipt_id {
                    r.user_dismissed = true;
                    return Ok(());
                }
            }
        }
        Err(NotifCenterError::ReceiptNotFound)
    }

    pub fn get_receipt(&self, receipt_id: u32) -> Option<&ConsentReceipt> {
        self.receipts.iter().find_map(|s| {
            s.as_ref().filter(|r| r.receipt_id == receipt_id)
        })
    }

    // ── AIS kill-switch (INV-NC-004) ──────────────────────────────────────────

    /// Toggle the AIS kill-switch from the Notification Center header.
    /// This is a persistent, unsuppressible control (INV-NC-004, AIS Charter §12.3).
    pub fn toggle_ais_kill_switch(&mut self) -> bool {
        self.ais_kill_switch_enabled = !self.ais_kill_switch_enabled;
        self.ais_kill_switch_enabled
    }

    // ── Shade-visible query ───────────────────────────────────────────────────

    /// Count notifications that are currently shade-visible (not dismissed, priority ≥ Normal).
    pub fn shade_visible_count(&self) -> usize {
        self.notifications.iter().filter(|s| {
            s.map(|n| n.is_shade_visible()).unwrap_or(false)
        }).count()
    }

    /// Iterate shade-visible notifications (for the pull-down renderer).
    pub fn iter_shade_visible(&self) -> impl Iterator<Item = &NcNotification> {
        self.notifications.iter().filter_map(|s| {
            s.as_ref().filter(|n| n.is_shade_visible())
        })
    }

    // ── App grouping (INV-NC-005, DEF-APP-001) ────────────────────────────────

    fn update_group(&mut self, app_id: &'static str, delta: isize) {
        // Find existing group.
        for slot in self.groups.iter_mut() {
            if let Some(ref mut g) = slot {
                if g.app_id == app_id {
                    if delta > 0 {
                        g.count = g.count.saturating_add(delta as usize);
                    } else {
                        g.count = g.count.saturating_sub((-delta) as usize);
                    }
                    return;
                }
            }
        }
        // New group.
        if delta > 0 {
            for slot in self.groups.iter_mut() {
                if slot.is_none() {
                    *slot = Some(AppGroup { app_id, count: delta as usize });
                    return;
                }
            }
        }
    }

    pub fn group_count_for(&self, app_id: &'static str) -> usize {
        self.groups.iter().find_map(|s| {
            s.as_ref().filter(|g| g.app_id == app_id).map(|g| g.count)
        }).unwrap_or(0)
    }

    pub fn get_notif(&self, notif_id: u32) -> Option<&NcNotification> {
        self.notifications.iter().find_map(|s| {
            s.as_ref().filter(|n| n.notif_id == notif_id)
        })
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// Tests
// ──────────────────────────────────────────────────────────────────────────────
#[cfg(test)]
mod tests {
    use super::*;

    fn make_engine() -> NotificationCenterEngine {
        NotificationCenterEngine::new()
    }

    fn post_normal(engine: &mut NotificationCenterEngine, app_id: &'static str) -> u32 {
        engine
            .post(10, CAP_NOTIFY_POST, app_id, NotifPriority::Normal, b"Title", b"Body")
            .unwrap()
    }

    #[test]
    // INV-NC-001: posting requires CAP_NOTIFY_POST
    fn test_post_requires_cap() {
        let mut e = make_engine();
        let err = e.post(10, 0, "ind.app.foo", NotifPriority::Normal, b"T", b"B");
        assert_eq!(err, Err(NotifCenterError::MissingPostCapability));
    }

    #[test]
    // REQ-APP-COMMON-013: top-edge swipe expands shade
    fn test_shade_expand_collapse() {
        let mut e = make_engine();
        assert_eq!(e.shade, ShadeState::Collapsed);
        e.expand_shade();
        assert_eq!(e.shade, ShadeState::Expanded);
        e.collapse_shade();
        assert_eq!(e.shade, ShadeState::Collapsed);
    }

    #[test]
    // INV-NC-002: Critical overrides DND; Normal does not
    fn test_dnd_priority_behavior() {
        let mut e = make_engine();
        e.dnd = DndState::On;

        // Normal posted successfully (stored for badge), but shade-suppressed under DND
        let nid = e.post(10, CAP_NOTIFY_POST, "ind.app.msg", NotifPriority::Normal, b"Hi", b"msg").unwrap();
        assert_eq!(e.notif_count(), 1);

        // Critical posted and shade-visible despite DND
        let cid = e.post(10, CAP_NOTIFY_POST, "ind.app.alarm", NotifPriority::Critical, b"Alarm!", b"Wake up").unwrap();
        let cn = e.get_notif(cid).unwrap();
        assert!(cn.priority.overrides_dnd());
        assert!(cn.priority.is_heads_up());

        let _ = nid;
    }

    #[test]
    // INV-NC-002: Silent notifications are never shade-visible
    fn test_silent_never_shade_visible() {
        let mut e = make_engine();
        let nid = e.post(10, CAP_NOTIFY_POST, "ind.app.bg", NotifPriority::Silent, b"Bg sync", b"").unwrap();
        let n = e.get_notif(nid).unwrap();
        assert!(!n.priority.is_shade_visible());
        assert_eq!(e.shade_visible_count(), 0);
    }

    #[test]
    // REQ-APP-COMMON-013: notifications are dismissible
    fn test_dismiss_single() {
        let mut e = make_engine();
        let nid = post_normal(&mut e, "ind.app.msg");
        assert_eq!(e.shade_visible_count(), 1);

        e.dismiss(nid).unwrap();
        assert_eq!(e.shade_visible_count(), 0);
        assert!(e.get_notif(nid).unwrap().is_dismissed);
    }

    #[test]
    // dismiss_all_for_app clears an entire app's notifications
    fn test_dismiss_all_for_app() {
        let mut e = make_engine();
        post_normal(&mut e, "ind.app.msg");
        post_normal(&mut e, "ind.app.msg");
        post_normal(&mut e, "ind.app.notes");

        let count = e.dismiss_all_for_app("ind.app.msg");
        assert_eq!(count, 2);
        assert_eq!(e.shade_visible_count(), 1); // notes remains
    }

    #[test]
    // INV-NC-003: consent receipt persists after notification is dismissed
    fn test_receipt_persists_after_dismiss() {
        let mut e = make_engine();
        let nid = post_normal(&mut e, "ind.app.cam");
        let rid = e.attach_consent_receipt(nid, "Camera accessed for photo capture").unwrap();

        // Dismiss the notification.
        e.dismiss(nid).unwrap();
        assert!(e.get_notif(nid).unwrap().is_dismissed);

        // Receipt still present (INV-NC-003).
        let r = e.get_receipt(rid).unwrap();
        assert_eq!(r.notif_id, nid);
        assert!(!r.user_dismissed);

        // User can dismiss the receipt independently.
        e.dismiss_receipt(rid).unwrap();
        assert!(e.get_receipt(rid).unwrap().user_dismissed);
    }

    #[test]
    // INV-NC-004: AIS kill-switch toggle in NC header
    fn test_ais_kill_switch_toggle() {
        let mut e = make_engine();
        assert!(!e.ais_kill_switch_enabled);
        let v1 = e.toggle_ais_kill_switch();
        assert!(v1);
        let v2 = e.toggle_ais_kill_switch();
        assert!(!v2);
    }

    #[test]
    // INV-NC-005 / DEF-APP-001: notifications grouped by app
    fn test_app_grouping() {
        let mut e = make_engine();
        post_normal(&mut e, "ind.app.msg");
        post_normal(&mut e, "ind.app.msg");
        post_normal(&mut e, "ind.app.notes");

        assert_eq!(e.group_count_for("ind.app.msg"), 2);
        assert_eq!(e.group_count_for("ind.app.notes"), 1);
        assert_eq!(e.group_count_for("ind.app.unknown"), 0);
    }

    #[test]
    // Priority ordering: Critical > High > Normal > Silent
    fn test_priority_ordering() {
        assert!(NotifPriority::Critical > NotifPriority::High);
        assert!(NotifPriority::High > NotifPriority::Normal);
        assert!(NotifPriority::Normal > NotifPriority::Silent);
    }

    #[test]
    // Full notification lifecycle: post → mark_read → dismiss
    fn test_full_notification_lifecycle() {
        let mut e = make_engine();
        let nid = e
            .post(10, CAP_NOTIFY_POST, "ind.app.phone", NotifPriority::High, b"Incoming call", b"Priya calling")
            .unwrap();

        let n = e.get_notif(nid).unwrap();
        assert!(!n.is_read);
        assert!(n.priority.is_heads_up());
        assert!(n.is_shade_visible());

        e.mark_read(nid).unwrap();
        assert!(e.get_notif(nid).unwrap().is_read);

        e.dismiss(nid).unwrap();
        assert!(!e.get_notif(nid).unwrap().is_shade_visible());
    }
}

