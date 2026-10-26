//! Asynchronous Notifications for IndK
//! Implements 64-bit atomic bitmask signaling and polling per Bible §8.9 and INV-IPC-003.

use crate::syscall::{EBADF, EBUSY, ENOMEM, ETIMEDOUT};
use crate::task::ProcessId;

pub type NotificationId = u32;
pub const MAX_NOTIFICATIONS: usize = 32;

/// Notification Object State
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NotificationState {
    Idle,
    Waiting { receiver_pid: ProcessId },
    Signaled { bitmask: u64 },
}

/// Asynchronous Notification Object
pub struct NotificationObject {
    pub id: NotificationId,
    pub state: NotificationState,
    pub is_allocated: bool,
    pub signal_count: u64,
}

impl NotificationObject {
    pub const fn empty(id: NotificationId) -> Self {
        Self {
            id,
            state: NotificationState::Idle,
            is_allocated: false,
            signal_count: 0,
        }
    }
}

/// Global registry for Asynchronous Notifications
pub struct NotificationRegistry {
    notifications: [NotificationObject; MAX_NOTIFICATIONS],
    total_signals: u64,
}

impl NotificationRegistry {
    pub const fn new() -> Self {
        const INIT_NOTIF: NotificationObject = NotificationObject::empty(0);
        let mut notifications = [INIT_NOTIF; MAX_NOTIFICATIONS];
        let mut i = 0;
        while i < MAX_NOTIFICATIONS {
            notifications[i].id = i as NotificationId;
            i += 1;
        }

        Self {
            notifications,
            total_signals: 0,
        }
    }

    /// Allocates a new notification object
    pub fn create_notification(&mut self) -> Result<NotificationId, i64> {
        for i in 0..MAX_NOTIFICATIONS {
            if !self.notifications[i].is_allocated {
                self.notifications[i].is_allocated = true;
                self.notifications[i].state = NotificationState::Idle;
                self.notifications[i].signal_count = 0;
                return Ok(i as NotificationId);
            }
        }
        Err(ENOMEM)
    }

    /// Destroys an existing notification object
    pub fn destroy_notification(&mut self, notif_id: NotificationId) -> Result<(), i64> {
        let idx = notif_id as usize;
        if idx >= MAX_NOTIFICATIONS || !self.notifications[idx].is_allocated {
            return Err(EBADF);
        }
        self.notifications[idx].is_allocated = false;
        self.notifications[idx].state = NotificationState::Idle;
        Ok(())
    }

    /// Signals a notification object by setting bits in its 64-bit mask (INV-IPC-003)
    pub fn signal(&mut self, notif_id: NotificationId, bits: u64) -> Result<(), i64> {
        let idx = notif_id as usize;
        if idx >= MAX_NOTIFICATIONS || !self.notifications[idx].is_allocated {
            return Err(EBADF);
        }

        self.total_signals += 1;
        self.notifications[idx].signal_count += 1;

        match self.notifications[idx].state {
            NotificationState::Idle => {
                self.notifications[idx].state = NotificationState::Signaled { bitmask: bits };
                Ok(())
            }
            NotificationState::Signaled { bitmask } => {
                // Bitwise OR accumulating signals atomically
                self.notifications[idx].state = NotificationState::Signaled {
                    bitmask: bitmask | bits,
                };
                Ok(())
            }
            NotificationState::Waiting { .. } => {
                // Transition waiting receiver to signaled state
                self.notifications[idx].state = NotificationState::Signaled { bitmask: bits };
                Ok(())
            }
        }
    }

    /// Polls a notification non-blockingly, returning and clearing any pending bitmask
    pub fn poll(&mut self, notif_id: NotificationId) -> Result<u64, i64> {
        let idx = notif_id as usize;
        if idx >= MAX_NOTIFICATIONS || !self.notifications[idx].is_allocated {
            return Err(EBADF);
        }

        match self.notifications[idx].state {
            NotificationState::Signaled { bitmask } => {
                self.notifications[idx].state = NotificationState::Idle;
                Ok(bitmask)
            }
            NotificationState::Idle | NotificationState::Waiting { .. } => Ok(0),
        }
    }

    /// Waits on a notification with timeout and clear bitmask (INV-IPC-003, INV-IPC-004)
    pub fn wait(
        &mut self,
        _receiver_pid: ProcessId,
        notif_id: NotificationId,
        clear_mask: u64,
        timeout_ticks: u64,
    ) -> Result<u64, i64> {
        let idx = notif_id as usize;
        if idx >= MAX_NOTIFICATIONS || !self.notifications[idx].is_allocated {
            return Err(EBADF);
        }

        match self.notifications[idx].state {
            NotificationState::Signaled { bitmask } => {
                let delivered = bitmask & clear_mask;
                let remaining = bitmask & !clear_mask;
                if remaining == 0 {
                    self.notifications[idx].state = NotificationState::Idle;
                } else {
                    self.notifications[idx].state = NotificationState::Signaled { bitmask: remaining };
                }
                Ok(delivered)
            }
            NotificationState::Idle => {
                if timeout_ticks == 0 {
                    // Non-blocking poll returned zero bits
                    Ok(0)
                } else {
                    // Timed wait expires if not already signaled (INV-IPC-004)
                    Err(ETIMEDOUT)
                }
            }
            NotificationState::Waiting { .. } => Err(EBUSY),
        }
    }

    pub fn total_signals(&self) -> u64 {
        self.total_signals
    }
}
