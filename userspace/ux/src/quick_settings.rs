//! Quick Settings Surface (SP-130, REQ-APP-COMMON-014)
//!
//! Bible §8.18.2, §8.20, §9.9.1 REQ-APP-COMMON-014
//! Invariants:
//! - INV-QS-001: Secondary top pull-down model (Collapsed -> HalfExpanded -> FullyExpanded).
//! - INV-QS-002: Airplane mode overrides and disables all radio toggles (Wi-Fi, Bluetooth, Cellular).
//! - INV-QS-003: Privacy Feed shortcut actively displays active sensor indicators (Camera, Mic, GPS)
//!               that cannot be masked or suppressed by apps.
//! - INV-QS-004: Hardware toggle modifications require CAP_QUICK_SETTINGS_WRITE; unprivileged apps
//!               cannot manipulate radios or hardware power states.
//! Rules: N-01 (zero external crates), N-04 (no unsafe), N-05 (user control over privacy and radios)

/// Capability required to read Quick Settings state.
pub const CAP_QUICK_SETTINGS_READ: u64 = 0x0004_0000_0000;
/// Capability required to toggle Quick Settings hardware controls.
pub const CAP_QUICK_SETTINGS_WRITE: u64 = 0x0008_0000_0000;

/// Sensor active bitflags for Privacy Feed shortcut (INV-QS-003).
pub const SENSOR_MIC_ACTIVE: u8 = 0x01;
pub const SENSOR_CAM_ACTIVE: u8 = 0x02;
pub const SENSOR_LOC_ACTIVE: u8 = 0x04;

/// Expansion state for the Quick Settings pull-down drawer (INV-QS-001).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QsExpansionState {
    /// Hidden above the status bar / screen.
    Collapsed,
    /// First pull-down / peek: displays primary 4 toggles + brightness slider.
    HalfExpanded,
    /// Full secondary pull-down: displays all hardware toggles, sliders, and privacy feed.
    FullyExpanded,
}

/// Power modes supported by the battery fuel governor in Quick Settings.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QsPowerMode {
    Eco,
    Balanced,
    Performance,
}

/// AIS operational mode selected via Quick Settings toggle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QsAisMode {
    Active,
    Restricted,
    Killed,
}

/// Distinct hardware toggle identifiers on the Quick Settings grid.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QsToggleId {
    Airplane,
    Wifi,
    Bluetooth,
    Cellular,
    Torch,
    BatterySaver,
    AisMode,
    DoNotDisturb,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QsError {
    MissingReadCapability,
    MissingWriteCapability,
    InvalidExpansionTransition,
    RadioLockedByAirplaneMode,
    ValueOutOfRange,
}

/// State of all hardware toggles and telemetry in Quick Settings.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct QuickSettingsState {
    pub airplane_mode: bool,
    pub wifi_enabled: bool,
    pub bluetooth_enabled: bool,
    pub cellular_enabled: bool,
    pub torch_enabled: bool,
    pub torch_brightness: u8,
    pub power_mode: QsPowerMode,
    pub ais_mode: QsAisMode,
    pub dnd_enabled: bool,
    pub active_sensors: u8,
    pub brightness_percent: u8,
    pub volume_percent: u8,
}

impl QuickSettingsState {
    pub const fn new() -> Self {
        Self {
            airplane_mode: false,
            wifi_enabled: true,
            bluetooth_enabled: true,
            cellular_enabled: true,
            torch_enabled: false,
            torch_brightness: 100,
            power_mode: QsPowerMode::Balanced,
            ais_mode: QsAisMode::Active,
            dnd_enabled: false,
            active_sensors: 0,
            brightness_percent: 75,
            volume_percent: 60,
        }
    }
}

/// The Quick Settings System Surface engine (INV-QS-001..004).
pub struct QuickSettingsEngine {
    state: QuickSettingsState,
    expansion: QsExpansionState,
}

impl QuickSettingsEngine {
    pub const fn new() -> Self {
        Self {
            state: QuickSettingsState::new(),
            expansion: QsExpansionState::Collapsed,
        }
    }

    pub fn expansion(&self) -> QsExpansionState {
        self.expansion
    }

    pub fn state(&self) -> &QuickSettingsState {
        &self.state
    }

    // ── Expansion State Machine (INV-QS-001) ──────────────────────────────────

    /// Handles top pull-down drag gesture in pixels.
    /// Thresholds: drag < 100 -> Collapsed, 100..300 -> HalfExpanded, >= 300 -> FullyExpanded.
    pub fn handle_drag_gesture(&mut self, drag_px: u32) -> QsExpansionState {
        if drag_px < 100 {
            self.expansion = QsExpansionState::Collapsed;
        } else if drag_px < 300 {
            self.expansion = QsExpansionState::HalfExpanded;
        } else {
            self.expansion = QsExpansionState::FullyExpanded;
        }
        self.expansion
    }

    /// Explicitly expand to half-screen drawer.
    pub fn expand_half(&mut self) {
        self.expansion = QsExpansionState::HalfExpanded;
    }

    /// Explicitly expand to full-screen drawer.
    pub fn expand_full(&mut self) {
        self.expansion = QsExpansionState::FullyExpanded;
    }

    /// Collapse drawer completely.
    pub fn collapse(&mut self) {
        self.expansion = QsExpansionState::Collapsed;
    }

    // ── Toggle Manipulation (INV-QS-002, INV-QS-004) ──────────────────────────

    /// Toggle a hardware control. Requires CAP_QUICK_SETTINGS_WRITE.
    pub fn toggle(&mut self, toggle_id: QsToggleId, caller_caps: u64) -> Result<bool, QsError> {
        if (caller_caps & CAP_QUICK_SETTINGS_WRITE) == 0 {
            return Err(QsError::MissingWriteCapability);
        }

        match toggle_id {
            QsToggleId::Airplane => {
                self.state.airplane_mode = !self.state.airplane_mode;
                if self.state.airplane_mode {
                    // INV-QS-002: Airplane mode overrides and disables all radios immediately
                    self.state.wifi_enabled = false;
                    self.state.bluetooth_enabled = false;
                    self.state.cellular_enabled = false;
                }
                Ok(self.state.airplane_mode)
            }
            QsToggleId::Wifi => {
                if self.state.airplane_mode {
                    return Err(QsError::RadioLockedByAirplaneMode);
                }
                self.state.wifi_enabled = !self.state.wifi_enabled;
                Ok(self.state.wifi_enabled)
            }
            QsToggleId::Bluetooth => {
                if self.state.airplane_mode {
                    return Err(QsError::RadioLockedByAirplaneMode);
                }
                self.state.bluetooth_enabled = !self.state.bluetooth_enabled;
                Ok(self.state.bluetooth_enabled)
            }
            QsToggleId::Cellular => {
                if self.state.airplane_mode {
                    return Err(QsError::RadioLockedByAirplaneMode);
                }
                self.state.cellular_enabled = !self.state.cellular_enabled;
                Ok(self.state.cellular_enabled)
            }
            QsToggleId::Torch => {
                self.state.torch_enabled = !self.state.torch_enabled;
                Ok(self.state.torch_enabled)
            }
            QsToggleId::BatterySaver => {
                self.state.power_mode = match self.state.power_mode {
                    QsPowerMode::Eco => QsPowerMode::Balanced,
                    QsPowerMode::Balanced | QsPowerMode::Performance => QsPowerMode::Eco,
                };
                Ok(self.state.power_mode == QsPowerMode::Eco)
            }
            QsToggleId::AisMode => {
                self.state.ais_mode = match self.state.ais_mode {
                    QsAisMode::Active => QsAisMode::Restricted,
                    QsAisMode::Restricted => QsAisMode::Killed,
                    QsAisMode::Killed => QsAisMode::Active,
                };
                Ok(self.state.ais_mode == QsAisMode::Active)
            }
            QsToggleId::DoNotDisturb => {
                self.state.dnd_enabled = !self.state.dnd_enabled;
                Ok(self.state.dnd_enabled)
            }
        }
    }

    // ── Sliders & Controls ───────────────────────────────────────────────────

    pub fn set_brightness(&mut self, percent: u8, caller_caps: u64) -> Result<(), QsError> {
        if (caller_caps & CAP_QUICK_SETTINGS_WRITE) == 0 {
            return Err(QsError::MissingWriteCapability);
        }
        if percent > 100 {
            return Err(QsError::ValueOutOfRange);
        }
        self.state.brightness_percent = percent;
        Ok(())
    }

    pub fn set_volume(&mut self, percent: u8, caller_caps: u64) -> Result<(), QsError> {
        if (caller_caps & CAP_QUICK_SETTINGS_WRITE) == 0 {
            return Err(QsError::MissingWriteCapability);
        }
        if percent > 100 {
            return Err(QsError::ValueOutOfRange);
        }
        self.state.volume_percent = percent;
        Ok(())
    }

    // ── Privacy Feed Telemetry (INV-QS-003) ───────────────────────────────────

    /// Sets active sensor indicators (Camera, Microphone, Location).
    /// Called by the enforcement layer to ensure apps cannot hide sensor usage.
    pub fn update_active_sensors(&mut self, sensor_flags: u8, caller_caps: u64) -> Result<(), QsError> {
        if (caller_caps & CAP_QUICK_SETTINGS_WRITE) == 0 {
            return Err(QsError::MissingWriteCapability);
        }
        self.state.active_sensors = sensor_flags;
        Ok(())
    }

    /// Returns true if any privacy-sensitive sensor is active (INV-QS-003).
    pub fn is_privacy_feed_active(&self) -> bool {
        self.state.active_sensors != 0
    }

    pub fn is_microphone_active(&self) -> bool {
        (self.state.active_sensors & SENSOR_MIC_ACTIVE) != 0
    }

    pub fn is_camera_active(&self) -> bool {
        (self.state.active_sensors & SENSOR_CAM_ACTIVE) != 0
    }

    pub fn is_location_active(&self) -> bool {
        (self.state.active_sensors & SENSOR_LOC_ACTIVE) != 0
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// Tests
// ──────────────────────────────────────────────────────────────────────────────
#[cfg(test)]
mod tests {
    use super::*;

    fn make_qs() -> QuickSettingsEngine {
        QuickSettingsEngine::new()
    }

    #[test]
    // INV-QS-001: drag gesture expansion transitions
    fn test_drag_expansion_states() {
        let mut qs = make_qs();
        assert_eq!(qs.expansion(), QsExpansionState::Collapsed);

        assert_eq!(qs.handle_drag_gesture(50), QsExpansionState::Collapsed);
        assert_eq!(qs.handle_drag_gesture(150), QsExpansionState::HalfExpanded);
        assert_eq!(qs.handle_drag_gesture(350), QsExpansionState::FullyExpanded);

        qs.collapse();
        assert_eq!(qs.expansion(), QsExpansionState::Collapsed);
    }

    #[test]
    // INV-QS-004: toggle requires CAP_QUICK_SETTINGS_WRITE
    fn test_toggle_requires_write_cap() {
        let mut qs = make_qs();
        let err = qs.toggle(QsToggleId::Wifi, 0);
        assert_eq!(err, Err(QsError::MissingWriteCapability));

        let ok = qs.toggle(QsToggleId::Wifi, CAP_QUICK_SETTINGS_WRITE);
        assert!(ok.is_ok());
    }

    #[test]
    // INV-QS-002: Airplane mode overrides and disables all radios
    fn test_airplane_mode_disables_radios() {
        let mut qs = make_qs();
        assert!(qs.state().wifi_enabled);
        assert!(qs.state().bluetooth_enabled);
        assert!(qs.state().cellular_enabled);

        // Turn on Airplane mode
        qs.toggle(QsToggleId::Airplane, CAP_QUICK_SETTINGS_WRITE).unwrap();
        assert!(qs.state().airplane_mode);
        assert!(!qs.state().wifi_enabled);
        assert!(!qs.state().bluetooth_enabled);
        assert!(!qs.state().cellular_enabled);

        // While airplane mode is active, toggling individual radios is rejected
        let err = qs.toggle(QsToggleId::Wifi, CAP_QUICK_SETTINGS_WRITE);
        assert_eq!(err, Err(QsError::RadioLockedByAirplaneMode));
        let err_bt = qs.toggle(QsToggleId::Bluetooth, CAP_QUICK_SETTINGS_WRITE);
        assert_eq!(err_bt, Err(QsError::RadioLockedByAirplaneMode));

        // Disable Airplane mode
        qs.toggle(QsToggleId::Airplane, CAP_QUICK_SETTINGS_WRITE).unwrap();
        assert!(!qs.state().airplane_mode);

        // Now radios can be re-enabled
        assert!(qs.toggle(QsToggleId::Wifi, CAP_QUICK_SETTINGS_WRITE).is_ok());
        assert!(qs.state().wifi_enabled);
    }

    #[test]
    // Torch and DND toggles
    fn test_torch_and_dnd_toggles() {
        let mut qs = make_qs();
        assert!(!qs.state().torch_enabled);
        assert!(!qs.state().dnd_enabled);

        qs.toggle(QsToggleId::Torch, CAP_QUICK_SETTINGS_WRITE).unwrap();
        assert!(qs.state().torch_enabled);

        qs.toggle(QsToggleId::DoNotDisturb, CAP_QUICK_SETTINGS_WRITE).unwrap();
        assert!(qs.state().dnd_enabled);
    }

    #[test]
    // Battery saver toggles power mode between Balanced and Eco
    fn test_battery_saver_toggle() {
        let mut qs = make_qs();
        assert_eq!(qs.state().power_mode, QsPowerMode::Balanced);

        qs.toggle(QsToggleId::BatterySaver, CAP_QUICK_SETTINGS_WRITE).unwrap();
        assert_eq!(qs.state().power_mode, QsPowerMode::Eco);

        qs.toggle(QsToggleId::BatterySaver, CAP_QUICK_SETTINGS_WRITE).unwrap();
        assert_eq!(qs.state().power_mode, QsPowerMode::Balanced);
    }

    #[test]
    // AIS mode cycle
    fn test_ais_mode_cycle() {
        let mut qs = make_qs();
        assert_eq!(qs.state().ais_mode, QsAisMode::Active);

        qs.toggle(QsToggleId::AisMode, CAP_QUICK_SETTINGS_WRITE).unwrap();
        assert_eq!(qs.state().ais_mode, QsAisMode::Restricted);

        qs.toggle(QsToggleId::AisMode, CAP_QUICK_SETTINGS_WRITE).unwrap();
        assert_eq!(qs.state().ais_mode, QsAisMode::Killed);

        qs.toggle(QsToggleId::AisMode, CAP_QUICK_SETTINGS_WRITE).unwrap();
        assert_eq!(qs.state().ais_mode, QsAisMode::Active);
    }

    #[test]
    // INV-QS-003: Privacy feed sensor indicators
    fn test_privacy_feed_sensor_indicators() {
        let mut qs = make_qs();
        assert!(!qs.is_privacy_feed_active());

        // Activate Mic & Cam
        qs.update_active_sensors(SENSOR_MIC_ACTIVE | SENSOR_CAM_ACTIVE, CAP_QUICK_SETTINGS_WRITE).unwrap();
        assert!(qs.is_privacy_feed_active());
        assert!(qs.is_microphone_active());
        assert!(qs.is_camera_active());
        assert!(!qs.is_location_active());

        // Add GPS
        qs.update_active_sensors(SENSOR_MIC_ACTIVE | SENSOR_CAM_ACTIVE | SENSOR_LOC_ACTIVE, CAP_QUICK_SETTINGS_WRITE).unwrap();
        assert!(qs.is_location_active());

        // Deactivate all
        qs.update_active_sensors(0, CAP_QUICK_SETTINGS_WRITE).unwrap();
        assert!(!qs.is_privacy_feed_active());
    }

    #[test]
    // Sliders validate 0..100 range and cap requirement
    fn test_sliders_range_and_cap() {
        let mut qs = make_qs();
        let err_cap = qs.set_brightness(50, 0);
        assert_eq!(err_cap, Err(QsError::MissingWriteCapability));

        let err_range = qs.set_brightness(101, CAP_QUICK_SETTINGS_WRITE);
        assert_eq!(err_range, Err(QsError::ValueOutOfRange));

        assert!(qs.set_brightness(80, CAP_QUICK_SETTINGS_WRITE).is_ok());
        assert_eq!(qs.state().brightness_percent, 80);

        assert!(qs.set_volume(45, CAP_QUICK_SETTINGS_WRITE).is_ok());
        assert_eq!(qs.state().volume_percent, 45);
    }
}

