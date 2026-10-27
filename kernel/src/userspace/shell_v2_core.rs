//! Kernel Shell v2 Verification Harness (SP-61, REQ-UX-305)
//!
//! Architectural Invariants:
//! - INV-SHELL2-001: Visual distinction for AIS suggestions (REQ-UX-305)
//! - INV-SHELL2-002: Unified intent input bar routing to Intent Broker
//! - INV-SHELL2-003: Bounded notification queue with dismiss/snooze/explain actions

pub const KERNEL_MAX_NOTIFICATIONS: usize = 16;
pub const KERNEL_MAX_LAUNCHER_APPS: usize = 8;
pub const KERNEL_INTENT_BUF_LEN: usize = 64;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KernelShellError {
    NotificationQueueFull,
    NotificationNotFound,
    QueryTooLong,
    ActionFailed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KernelNotificationSource {
    System,
    App,
    AisSuggestion,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KernelNotificationAction {
    Dismiss,
    Snooze,
    Explain,
    Open,
}

#[derive(Debug, Clone, Copy)]
pub struct KernelNotificationItem {
    pub id: u32,
    pub source: KernelNotificationSource,
    pub title: &'static str,
    pub body: &'static str,
    pub is_ais_distinct: bool,
    pub action_taken: Option<KernelNotificationAction>,
}

pub struct KernelShellV2Engine {
    launcher_apps: [&'static str; KERNEL_MAX_LAUNCHER_APPS],
    app_count: usize,
    notifications: [Option<KernelNotificationItem>; KERNEL_MAX_NOTIFICATIONS],
    notification_count: usize,
    next_notif_id: u32,
    active_intent_query: [u8; KERNEL_INTENT_BUF_LEN],
    query_len: usize,
}

impl KernelShellV2Engine {
    pub const fn new() -> Self {
        Self {
            launcher_apps: ["Files", "Settings", "Why-Explorer", "Terminal", "Notes", "", "", ""],
            app_count: 5,
            notifications: [None; KERNEL_MAX_NOTIFICATIONS],
            notification_count: 0,
            next_notif_id: 1,
            active_intent_query: [0u8; KERNEL_INTENT_BUF_LEN],
            query_len: 0,
        }
    }

    pub fn post_notification(
        &mut self,
        source: KernelNotificationSource,
        title: &'static str,
        body: &'static str,
    ) -> Result<u32, KernelShellError> {
        for slot in self.notifications.iter_mut() {
            if slot.is_none() {
                let id = self.next_notif_id;
                self.next_notif_id += 1;

                let is_distinct = source == KernelNotificationSource::AisSuggestion;

                *slot = Some(KernelNotificationItem {
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
        Err(KernelShellError::NotificationQueueFull)
    }

    pub fn dismiss_notification(&mut self, notif_id: u32) -> Result<(), KernelShellError> {
        for slot in self.notifications.iter_mut() {
            if let Some(ref mut n) = slot {
                if n.id == notif_id {
                    n.action_taken = Some(KernelNotificationAction::Dismiss);
                    *slot = None;
                    self.notification_count -= 1;
                    return Ok(());
                }
            }
        }
        Err(KernelShellError::NotificationNotFound)
    }

    pub fn snooze_notification(&mut self, notif_id: u32) -> Result<(), KernelShellError> {
        for slot in self.notifications.iter_mut() {
            if let Some(ref mut n) = slot {
                if n.id == notif_id {
                    n.action_taken = Some(KernelNotificationAction::Snooze);
                    return Ok(());
                }
            }
        }
        Err(KernelShellError::NotificationNotFound)
    }

    pub fn submit_intent_query(&mut self, query: &str) -> Result<u32, KernelShellError> {
        let b = query.as_bytes();
        if b.len() > KERNEL_INTENT_BUF_LEN {
            return Err(KernelShellError::QueryTooLong);
        }

        self.query_len = b.len();
        for i in 0..b.len() {
            self.active_intent_query[i] = b[i];
        }

        Ok(0x1000 + (b.len() as u32))
    }

    pub fn notification_count(&self) -> usize {
        self.notification_count
    }

    pub fn active_notifications(&self) -> &[Option<KernelNotificationItem>; KERNEL_MAX_NOTIFICATIONS] {
        &self.notifications
    }

    pub fn launcher_apps(&self) -> &[&'static str; KERNEL_MAX_LAUNCHER_APPS] {
        &self.launcher_apps
    }

    pub fn app_count(&self) -> usize {
        self.app_count
    }
}
