//! Interactive Touch-Card UI Shell & App Launcher (SP-114, REQ-UX-301..305, REQ-HW-106)
//!
//! Standards: Bible §8.16; INV-TOUCH-SHELL-001..003
//! Rule N-01: Zero external crates
//! Rule N-04: Pure safe Rust

pub const CAP_INTERACTIVE_SHELL: u64 = 0x0000_0000_0000_0800;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveCard {
    Telemetry,    // Bento Home
    AppDrawer,    // Core Apps
    AisAssistant, // AIS Intent
    FilesApp,
    SettingsApp,
    WhyExplorer,
    NotesApp,
    TerminalApp,
    ConsentModal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppAction {
    LaunchTerminal,
    LaunchSettings,
    LaunchFilesystem,
    LaunchAis,
    LaunchEditor,
    OpenWhyExplorer,
    OpenConsentModal,
    ReturnHome,
    ApplyEco60Hz,
    DismissProposal,
    SwitchLanguage(u8),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InteractiveShellError {
    PermissionDenied,
    InvalidCoordinates,
    NoActionForTouch,
}

pub struct InteractiveShellEngine {
    active_card: ActiveCard,
    uptime_seconds: u32,
    heartbeat_ticks: u64,
    launched_apps_count: u32,
    touch_switches: u32,
}

impl InteractiveShellEngine {
    pub const fn new() -> Self {
        Self {
            active_card: ActiveCard::Telemetry,
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

    pub fn handle_tap(&mut self, x: u32, y: u32, caps: u64) -> Result<Option<AppAction>, InteractiveShellError> {
        if (caps & CAP_INTERACTIVE_SHELL) == 0 {
            return Err(InteractiveShellError::PermissionDenied);
        }
        if x >= 1080 || y >= 2460 {
            return Err(InteractiveShellError::InvalidCoordinates);
        }

        // Bottom Dock Hit Testing (y: 2240..2360)
        if y >= 2200 && y <= 2400 {
            if x < 360 {
                self.active_card = ActiveCard::Telemetry;
                self.touch_switches += 1;
                return Ok(None);
            } else if x < 720 {
                self.active_card = ActiveCard::AppDrawer;
                self.touch_switches += 1;
                return Ok(None);
            } else {
                self.active_card = ActiveCard::AisAssistant;
                self.touch_switches += 1;
                return Ok(None);
            }
        }

        // App Drawer Hit Testing (when ActiveCard == AppDrawer)
        if self.active_card == ActiveCard::AppDrawer && y >= 600 && y <= 1800 {
            self.launched_apps_count += 1;
            if y < 800 {
                return Ok(Some(AppAction::LaunchTerminal));
            } else if y < 1000 {
                return Ok(Some(AppAction::LaunchSettings));
            } else if y < 1200 {
                return Ok(Some(AppAction::LaunchFilesystem));
            } else if y < 1400 {
                return Ok(Some(AppAction::LaunchAis));
            } else {
                return Ok(Some(AppAction::LaunchEditor));
            }
        }

        Ok(None)
    }

    pub fn active_card(&self) -> ActiveCard {
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
