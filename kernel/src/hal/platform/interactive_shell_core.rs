//! Kernel Hardware Interactive Touch-Card UI Shell Engine (SP-114, REQ-UX-301..305, REQ-HW-106)
//!
//! Standards: Bible §8.16; INV-TOUCH-SHELL-001..003
//! Rule N-01: Zero external crates

pub const KERNEL_CAP_INTERACTIVE_SHELL: u64 = 0x0000_0000_0000_0800;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KernelActiveCard {
    Telemetry,
    AppDrawer,
    AisAssistant,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KernelAppAction {
    LaunchTerminal,
    LaunchSettings,
    LaunchFilesystem,
    LaunchAis,
    LaunchEditor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KernelInteractiveShellError {
    PermissionDenied,
    InvalidCoordinates,
    NoActionForTouch,
}

pub struct KernelInteractiveShellEngine {
    active_card: KernelActiveCard,
    uptime_seconds: u32,
    heartbeat_ticks: u64,
    launched_apps_count: u32,
    touch_switches: u32,
}

impl KernelInteractiveShellEngine {
    pub const fn new() -> Self {
        Self {
            active_card: KernelActiveCard::Telemetry,
            uptime_seconds: 0,
            heartbeat_ticks: 0,
            launched_apps_count: 0,
            touch_switches: 0,
        }
    }

    pub fn tick_heartbeat(&mut self, elapsed_sec: u32) {
        self.uptime_seconds = elapsed_sec;
        self.heartbeat_ticks += 1;
    }

    pub fn handle_tap(&mut self, x: u32, y: u32, caps: u64) -> Result<Option<KernelAppAction>, KernelInteractiveShellError> {
        if (caps & KERNEL_CAP_INTERACTIVE_SHELL) == 0 {
            return Err(KernelInteractiveShellError::PermissionDenied);
        }
        if x >= 1080 || y >= 2460 {
            return Err(KernelInteractiveShellError::InvalidCoordinates);
        }

        // Bottom Dock Hit Testing (y: 2200..2400)
        if y >= 2200 && y <= 2400 {
            if x < 360 {
                self.active_card = KernelActiveCard::Telemetry;
                self.touch_switches += 1;
                return Ok(None);
            } else if x < 720 {
                self.active_card = KernelActiveCard::AppDrawer;
                self.touch_switches += 1;
                return Ok(None);
            } else {
                self.active_card = KernelActiveCard::AisAssistant;
                self.touch_switches += 1;
                return Ok(None);
            }
        }

        // App Drawer Hit Testing (when active_card == AppDrawer)
        if self.active_card == KernelActiveCard::AppDrawer && y >= 600 && y <= 1800 {
            self.launched_apps_count += 1;
            if y < 800 {
                return Ok(Some(KernelAppAction::LaunchTerminal));
            } else if y < 1000 {
                return Ok(Some(KernelAppAction::LaunchSettings));
            } else if y < 1200 {
                return Ok(Some(KernelAppAction::LaunchFilesystem));
            } else if y < 1400 {
                return Ok(Some(KernelAppAction::LaunchAis));
            } else {
                return Ok(Some(KernelAppAction::LaunchEditor));
            }
        }

        Ok(None)
    }

    pub fn active_card(&self) -> KernelActiveCard {
        self.active_card
    }

    pub fn uptime_seconds(&self) -> u32 {
        self.uptime_seconds
    }

    pub fn touch_switches(&self) -> u32 {
        self.touch_switches
    }

    pub fn launched_apps_count(&self) -> u32 {
        self.launched_apps_count
    }
}
