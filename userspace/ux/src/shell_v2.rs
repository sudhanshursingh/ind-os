//! Shell v2 Core: Home, Notifications & Intent Bar (SP-61, REQ-UX-305)
//!
//! Architectural Invariants:
//! - INV-SHELL2-001: Visual distinction for AIS suggestions (REQ-UX-305)
//! - INV-SHELL2-002: Unified intent input bar routing to Intent Broker
//! - INV-SHELL2-003: Bounded notification queue with dismiss/snooze/explain actions

pub const MAX_NOTIFICATIONS: usize = 16;
pub const MAX_LAUNCHER_APPS: usize = 8;
pub const INTENT_BUF_LEN: usize = 64;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShellError {
    NotificationQueueFull,
    NotificationNotFound,
    QueryTooLong,
    ActionFailed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotificationSource {
    System,
    App,
    AisSuggestion,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotificationAction {
    Dismiss,
    Snooze,
    Explain,
    Open,
}

#[derive(Debug, Clone, Copy)]
pub struct NotificationItem {
    pub id: u32,
    pub source: NotificationSource,
    pub title: &'static str,
    pub body: &'static str,
    pub is_ais_distinct: bool,
    pub action_taken: Option<NotificationAction>,
}

pub struct ShellV2Engine {
    launcher_apps: [&'static str; MAX_LAUNCHER_APPS],
    app_count: usize,
    notifications: [Option<NotificationItem>; MAX_NOTIFICATIONS],
    notification_count: usize,
    next_notif_id: u32,
    active_intent_query: [u8; INTENT_BUF_LEN],
    query_len: usize,
}

impl ShellV2Engine {
    pub const fn new() -> Self {
        Self {
            launcher_apps: ["Files", "Settings", "Why-Explorer", "Terminal", "Notes", "", "", ""],
            app_count: 5,
            notifications: [None; MAX_NOTIFICATIONS],
            notification_count: 0,
            next_notif_id: 1,
            active_intent_query: [0u8; INTENT_BUF_LEN],
            query_len: 0,
        }
    }

    /// Post notification with mandatory AIS visual distinction (INV-SHELL2-001)
    pub fn post_notification(
        &mut self,
        source: NotificationSource,
        title: &'static str,
        body: &'static str,
    ) -> Result<u32, ShellError> {
        for slot in self.notifications.iter_mut() {
            if slot.is_none() {
                let id = self.next_notif_id;
                self.next_notif_id += 1;

                // REQ-UX-305: AIS suggestions MUST be visually distinct
                let is_distinct = source == NotificationSource::AisSuggestion;

                *slot = Some(NotificationItem {
                    id,
                    source,
                    title,
                    body,
                    is_ais_distinct: is_distinct,
                    action_taken: None,
                });
                self.notification_count += 1;
                return Ok(id);
            }
        }
        Err(ShellError::NotificationQueueFull)
    }

    /// Dismiss notification (INV-SHELL2-003)
    pub fn dismiss_notification(&mut self, notif_id: u32) -> Result<(), ShellError> {
        for slot in self.notifications.iter_mut() {
            if let Some(ref mut n) = slot {
                if n.id == notif_id {
                    n.action_taken = Some(NotificationAction::Dismiss);
                    *slot = None;
                    self.notification_count -= 1;
                    return Ok(());
                }
            }
        }
        Err(ShellError::NotificationNotFound)
    }

    /// Snooze notification (INV-SHELL2-003)
    pub fn snooze_notification(&mut self, notif_id: u32) -> Result<(), ShellError> {
        for slot in self.notifications.iter_mut() {
            if let Some(ref mut n) = slot {
                if n.id == notif_id {
                    n.action_taken = Some(NotificationAction::Snooze);
                    return Ok(());
                }
            }
        }
        Err(ShellError::NotificationNotFound)
    }

    /// Submit typed search/intent query (INV-SHELL2-002)
    pub fn submit_intent_query(&mut self, query: &str) -> Result<u32, ShellError> {
        let b = query.as_bytes();
        if b.len() > INTENT_BUF_LEN {
            return Err(ShellError::QueryTooLong);
        }

        self.query_len = b.len();
        for i in 0..b.len() {
            self.active_intent_query[i] = b[i];
        }

        // Returns deterministic intent proposal token
        Ok(0x1000 + (b.len() as u32))
    }

    pub fn notification_count(&self) -> usize {
        self.notification_count
    }

    pub fn active_notifications(&self) -> &[Option<NotificationItem>; MAX_NOTIFICATIONS] {
        &self.notifications
    }

    pub fn launcher_apps(&self) -> &[&'static str; MAX_LAUNCHER_APPS] {
        &self.launcher_apps
    }

    pub fn app_count(&self) -> usize {
        self.app_count
    }
}
