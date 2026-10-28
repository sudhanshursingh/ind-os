//! Kernel Hardware Interactive Touch-Card UI Shell Engine (SP-114, REQ-UX-301..305, REQ-HW-106)
//!
//! Standards: Bible §8.16; INV-TOUCH-SHELL-001..003
//! Rule N-01: Zero external crates

pub const KERNEL_CAP_INTERACTIVE_SHELL: u64 = 0x0000_0000_0000_0800;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KernelActiveCard {
    Telemetry,    // Maps to Bento Home / System Dashboard
    AppDrawer,    // Core Apps Drawer (Files, Settings, Why, Notes, Terminal)
    AisAssistant, // AIS Intent & Proactive Suggestion Card
    FilesApp,     // IndFS Files Explorer
    SettingsApp,  // Privacy & System Settings
    WhyExplorer,  // Causal Timeline & Provenance Receipts
    NotesApp,     // Indic IME Notes
    TerminalApp,  // Interactive Shell Terminal
    ConsentModal, // Hardware-Isolated Consent Surface (PlaneId::HardwareIsolated)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KernelAppAction {
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

        self.touch_switches += 1;

        // 1. Navigation Bar (y >= 2320)
        if y >= 2320 {
            if x < 360 {
                self.active_card = KernelActiveCard::Telemetry;
                return Ok(Some(KernelAppAction::ReturnHome));
            } else if x < 720 {
                self.active_card = KernelActiveCard::Telemetry;
                return Ok(Some(KernelAppAction::ReturnHome));
            } else {
                self.active_card = KernelActiveCard::AppDrawer;
                return Ok(Some(KernelAppAction::LaunchTerminal));
            }
        }

        // 2. If in an App View or Modal, check Back / Close buttons
        if matches!(self.active_card, KernelActiveCard::FilesApp | KernelActiveCard::SettingsApp | KernelActiveCard::WhyExplorer | KernelActiveCard::NotesApp | KernelActiveCard::TerminalApp | KernelActiveCard::ConsentModal) {
            // Top back button: x: 60..320, y: 70..160
            if y >= 70 && y <= 160 && x <= 320 {
                self.active_card = KernelActiveCard::Telemetry;
                return Ok(Some(KernelAppAction::ReturnHome));
            }
            // Consent Modal action buttons (y: 1400..1650)
            if self.active_card == KernelActiveCard::ConsentModal {
                if y >= 1400 && y <= 1520 {
                    self.active_card = KernelActiveCard::Telemetry;
                    return Ok(Some(KernelAppAction::ReturnHome));
                }
                if y >= 1530 && y <= 1620 && x >= 400 && x <= 680 {
                    self.active_card = KernelActiveCard::WhyExplorer;
                    return Ok(Some(KernelAppAction::OpenWhyExplorer));
                }
            }
            return Ok(None);
        }

        // 3. Top Status Bar (y < 64) -> Tapping Gaja or Status Bar opens Why Explorer (PRD §7.2)
        if y < 64 {
            self.active_card = KernelActiveCard::WhyExplorer;
            return Ok(Some(KernelAppAction::OpenWhyExplorer));
        }

        // 4. Intent Bar (y: 84..184)
        if y >= 84 && y <= 184 {
            if x >= 740 && x < 810 {
                return Ok(Some(KernelAppAction::SwitchLanguage(0)));
            } else if x >= 810 && x < 880 {
                return Ok(Some(KernelAppAction::SwitchLanguage(1)));
            } else if x >= 880 && x < 960 {
                return Ok(Some(KernelAppAction::SwitchLanguage(2)));
            } else {
                self.active_card = KernelActiveCard::AisAssistant;
                return Ok(Some(KernelAppAction::LaunchAis));
            }
        }

        // 5. Bento Grid Hit Testing (y: 204..2200)
        // Tile 1: Ambient Greeting (y: 204..454) -> Opens Why Explorer
        if y >= 204 && y <= 454 {
            self.active_card = KernelActiveCard::WhyExplorer;
            return Ok(Some(KernelAppAction::OpenWhyExplorer));
        }

        // Tile 2: Battery Gauge (x: 60..520, y: 474..734) -> Opens Settings
        if y >= 474 && y <= 734 && x <= 530 {
            self.active_card = KernelActiveCard::SettingsApp;
            return Ok(Some(KernelAppAction::LaunchSettings));
        }

        // Tile 3: Sovereign Security (x: 550..1020, y: 474..734) -> Opens Consent Modal
        if y >= 474 && y <= 734 && x > 530 {
            self.active_card = KernelActiveCard::ConsentModal;
            return Ok(Some(KernelAppAction::OpenConsentModal));
        }

        // Tile 4: Core Native Apps (y: 754..1054)
        if y >= 754 && y <= 1054 {
            self.launched_apps_count += 1;
            if x < 250 {
                self.active_card = KernelActiveCard::FilesApp;
                return Ok(Some(KernelAppAction::LaunchFilesystem));
            } else if x < 440 {
                self.active_card = KernelActiveCard::SettingsApp;
                return Ok(Some(KernelAppAction::LaunchSettings));
            } else if x < 630 {
                self.active_card = KernelActiveCard::WhyExplorer;
                return Ok(Some(KernelAppAction::OpenWhyExplorer));
            } else if x < 820 {
                self.active_card = KernelActiveCard::NotesApp;
                return Ok(Some(KernelAppAction::LaunchEditor));
            } else {
                self.active_card = KernelActiveCard::TerminalApp;
                return Ok(Some(KernelAppAction::LaunchTerminal));
            }
        }

        // Tile 5: Proactive AIS Proposal Card (y: 1074..1624)
        if y >= 1074 && y <= 1624 {
            if y >= 1470 && y <= 1600 {
                if x < 410 {
                    return Ok(Some(KernelAppAction::ApplyEco60Hz));
                } else if x < 710 {
                    return Ok(Some(KernelAppAction::DismissProposal));
                } else {
                    self.active_card = KernelActiveCard::WhyExplorer;
                    return Ok(Some(KernelAppAction::OpenWhyExplorer));
                }
            }
        }

        Ok(None)
    }

    pub fn set_active_card(&mut self, card: KernelActiveCard) {
        self.active_card = card;
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
