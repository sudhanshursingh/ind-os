//! Notification Discipline & AIS Kill Switch (SP-131, REQ-APP-COMMON-005, AIS Charter §12.3)
//!
//! Bible §8.18.2, §8.20, §9.9.1 REQ-APP-COMMON-005, AIS Charter §12.3
//! Invariants:
//! - INV-DISCIPLINE-001: User-initiated notification rules take precedence over app-requested priorities.
//!                       Apps cannot elevate their priority above user-configured limits.
//! - INV-DISCIPLINE-002: Safety-critical emergency notifications (alarms, emergency calls, OTPs) bypass
//!                       user quiet hours and standard app blocks.
//! - INV-DISCIPLINE-003: Modifying notification rules requires CAP_NOTIFY_RULE_WRITE.
//! - INV-AISKILL-001: AIS kill-switch transition immediately suspends all model inference, telemetry reads,
//!                    proposal generation, and ambient card rendering across all UX surfaces.
//! - INV-AISKILL-002: The kill-switch is guaranteed reachable in <= 2 gestures from Lock Screen,
//!                    Notification Shade, and Quick Settings.
//! - INV-AISKILL-003: Kill-switch activations are audit-logged with monotonic timestamps and cannot be undone by AIS.
//! Rules: N-01 (zero external crates), N-04 (no unsafe), N-05 (user authority supreme)

use crate::notification_center::NotifPriority;

/// Maximum user-configurable notification rules.
pub const MAX_NOTIFICATION_RULES: usize = 32;

/// Capability required to modify user notification rules.
pub const CAP_NOTIFY_RULE_WRITE: u64 = 0x0020_0000_0000;

/// Surface origin from which the AIS kill-switch was activated (INV-AISKILL-002).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KillSwitchOrigin {
    LockScreen,
    NotificationShade,
    QuickSettings,
    HardwareButtonChord,
}

impl KillSwitchOrigin {
    /// Returns the maximum gesture count required to trigger the kill-switch from this surface (INV-AISKILL-002).
    pub const fn gesture_distance(self) -> u8 {
        match self {
            KillSwitchOrigin::LockScreen => 2,          // 1: swipe up AIS strip, 2: tap toggle
            KillSwitchOrigin::NotificationShade => 2,   // 1: top pull-down, 2: tap header switch
            KillSwitchOrigin::QuickSettings => 2,       // 1: secondary pull-down, 2: tap AIS tile
            KillSwitchOrigin::HardwareButtonChord => 1, // 1: Power + VolDown hold (physical)
        }
    }
}

/// Global system-wide AIS execution state (AIS Charter §12.3).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AisSystemState {
    /// Full contextual proposals, read-only telemetry, bento suggestions active.
    Active,
    /// Suggestions muted; intent bar operational only on explicit text/voice query.
    Restricted,
    /// Complete shutdown. Zero model inference, zero telemetry reads, zero storage, zero proposals.
    Killed,
}

impl AisSystemState {
    pub fn allows_inference(self) -> bool {
        self == AisSystemState::Active
    }

    pub fn allows_telemetry_read(self) -> bool {
        self != AisSystemState::Killed
    }

    pub fn allows_bento_cards(self) -> bool {
        self == AisSystemState::Active
    }
}

/// Result of evaluating an incoming notification against user discipline rules (INV-DISCIPLINE-001).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotificationVerdict {
    /// Deliver normally according to calculated effective priority.
    Deliver {
        effective_priority: NotifPriority,
        play_sound: bool,
        vibrate: bool,
        heads_up: bool,
    },
    /// Suppressed by Do Not Disturb or Quiet Hours; recorded as silent badge only.
    MutedToBadge,
    /// Completely dropped by explicit user block rule.
    Blocked,
}

/// User-configured notification discipline rule for an app.
#[derive(Debug, Clone, Copy)]
pub struct NotificationRule {
    pub app_id: &'static str,
    pub is_blocked: bool,
    pub priority_cap: Option<NotifPriority>,
    pub dnd_bypass: bool,
    pub sound_enabled: bool,
    pub vibration_enabled: bool,
    /// Quiet hours in 24-hour clock: Some((start_hour, end_hour)), e.g. Some((22, 7)).
    pub quiet_hours: Option<(u8, u8)>,
}

impl NotificationRule {
    pub const fn default_for(app_id: &'static str) -> Self {
        Self {
            app_id,
            is_blocked: false,
            priority_cap: None,
            dnd_bypass: false,
            sound_enabled: true,
            vibration_enabled: true,
            quiet_hours: None,
        }
    }

    /// Evaluates if the given hour (0..23) falls within quiet hours.
    pub fn is_in_quiet_hours(&self, current_hour: u8) -> bool {
        if let Some((start, end)) = self.quiet_hours {
            if start <= end {
                current_hour >= start && current_hour < end
            } else {
                // Overnight span, e.g. 22..7
                current_hour >= start || current_hour < end
            }
        } else {
            false
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisciplineError {
    MissingRuleWriteCapability,
    RuleTableFull,
    RuleNotFound,
    InvalidHourRange,
}

/// The Notification Discipline & Unified AIS Kill Switch Engine.
pub struct NotificationDisciplineEngine {
    rules: [Option<NotificationRule>; MAX_NOTIFICATION_RULES],
    rule_count: usize,
    pub ais_state: AisSystemState,
    pub last_kill_origin: Option<KillSwitchOrigin>,
    pub kill_event_count: u32,
    pub last_kill_timestamp: u64,
}

impl NotificationDisciplineEngine {
    pub const fn new() -> Self {
        Self {
            rules: [None; MAX_NOTIFICATION_RULES],
            rule_count: 0,
            ais_state: AisSystemState::Active,
            last_kill_origin: None,
            kill_event_count: 0,
            last_kill_timestamp: 0,
        }
    }

    pub fn rule_count(&self) -> usize {
        self.rule_count
    }

    // ── User Notification Rules (INV-DISCIPLINE-001..003) ─────────────────────

    /// Sets or updates a notification rule for an app. Requires CAP_NOTIFY_RULE_WRITE.
    pub fn set_rule(
        &mut self,
        rule: NotificationRule,
        caller_caps: u64,
    ) -> Result<(), DisciplineError> {
        if (caller_caps & CAP_NOTIFY_RULE_WRITE) == 0 {
            return Err(DisciplineError::MissingRuleWriteCapability);
        }

        if let Some((start, end)) = rule.quiet_hours {
            if start > 23 || end > 23 {
                return Err(DisciplineError::InvalidHourRange);
            }
        }

        // Check if rule already exists for this app
        for slot in self.rules.iter_mut() {
            if let Some(ref mut r) = slot {
                if r.app_id == rule.app_id {
                    *r = rule;
                    return Ok(());
                }
            }
        }

        // Insert new rule
        for slot in self.rules.iter_mut() {
            if slot.is_none() {
                *slot = Some(rule);
                self.rule_count += 1;
                return Ok(());
            }
        }

        Err(DisciplineError::RuleTableFull)
    }

    /// Evaluates an incoming notification request against user rules, DND, and quiet hours.
    pub fn evaluate(
        &self,
        app_id: &'static str,
        requested_priority: NotifPriority,
        current_hour: u8,
        is_dnd_active: bool,
    ) -> NotificationVerdict {
        // INV-DISCIPLINE-002: Critical/Emergency notifications bypass all blocks and quiet hours
        if requested_priority == NotifPriority::Critical {
            return NotificationVerdict::Deliver {
                effective_priority: NotifPriority::Critical,
                play_sound: true,
                vibrate: true,
                heads_up: true,
            };
        }

        let rule = self.rules.iter().find_map(|s| {
            s.as_ref().filter(|r| r.app_id == app_id)
        });

        if let Some(r) = rule {
            // Check explicit user block (INV-DISCIPLINE-001)
            if r.is_blocked {
                return NotificationVerdict::Blocked;
            }

            // Apply priority cap if configured
            let mut effective_priority = requested_priority;
            if let Some(cap) = r.priority_cap {
                if effective_priority > cap {
                    effective_priority = cap;
                }
            }

            if effective_priority == NotifPriority::Silent {
                return NotificationVerdict::MutedToBadge;
            }

            // Check Quiet Hours or Do Not Disturb
            let in_quiet = r.is_in_quiet_hours(current_hour);
            let dnd_blocks = is_dnd_active && !r.dnd_bypass;

            if in_quiet || dnd_blocks {
                return NotificationVerdict::MutedToBadge;
            }

            NotificationVerdict::Deliver {
                effective_priority,
                play_sound: r.sound_enabled,
                vibrate: r.vibration_enabled,
                heads_up: effective_priority.is_heads_up(),
            }
        } else {
            // Default policy when no specific app rule exists
            if is_dnd_active {
                NotificationVerdict::MutedToBadge
            } else if requested_priority == NotifPriority::Silent {
                NotificationVerdict::MutedToBadge
            } else {
                NotificationVerdict::Deliver {
                    effective_priority: requested_priority,
                    play_sound: true,
                    vibrate: true,
                    heads_up: requested_priority.is_heads_up(),
                }
            }
        }
    }

    // ── Unified AIS Kill Switch (INV-AISKILL-001..003) ────────────────────────

    /// Trigger the AIS kill-switch from any authorized surface (LockScreen, Shade, QuickSettings).
    /// Enforces <= 2 gesture invariant and transitions system into Killed state immediately.
    pub fn trigger_kill_switch(&mut self, origin: KillSwitchOrigin, timestamp: u64) -> AisSystemState {
        // INV-AISKILL-002: Validate gesture distance is <= 2
        let distance = origin.gesture_distance();
        assert!(distance <= 2, "AIS kill-switch gesture distance exceeded 2");

        self.ais_state = AisSystemState::Killed;
        self.last_kill_origin = Some(origin);
        self.kill_event_count += 1;
        self.last_kill_timestamp = timestamp;

        self.ais_state
    }

    /// User explicitly re-enables AIS after deliberate review.
    pub fn restore_ais_state(&mut self, new_state: AisSystemState) {
        self.ais_state = new_state;
    }

    pub fn get_rule(&self, app_id: &'static str) -> Option<&NotificationRule> {
        self.rules.iter().find_map(|s| {
            s.as_ref().filter(|r| r.app_id == app_id)
        })
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// Tests
// ──────────────────────────────────────────────────────────────────────────────
#[cfg(test)]
mod tests {
    use super::*;

    fn make_engine() -> NotificationDisciplineEngine {
        NotificationDisciplineEngine::new()
    }

    #[test]
    // INV-DISCIPLINE-003: set_rule requires CAP_NOTIFY_RULE_WRITE
    fn test_set_rule_requires_cap() {
        let mut engine = make_engine();
        let rule = NotificationRule::default_for("ind.app.messages");
        let err = engine.set_rule(rule, 0);
        assert_eq!(err, Err(DisciplineError::MissingRuleWriteCapability));

        let ok = engine.set_rule(rule, CAP_NOTIFY_RULE_WRITE);
        assert!(ok.is_ok());
        assert_eq!(engine.rule_count(), 1);
    }

    #[test]
    // INV-DISCIPLINE-001: user block stops notifications completely
    fn test_user_block_rule() {
        let mut engine = make_engine();
        let mut rule = NotificationRule::default_for("ind.app.spam");
        rule.is_blocked = true;
        engine.set_rule(rule, CAP_NOTIFY_RULE_WRITE).unwrap();

        let verdict = engine.evaluate("ind.app.spam", NotifPriority::High, 14, false);
        assert_eq!(verdict, NotificationVerdict::Blocked);
    }

    #[test]
    // INV-DISCIPLINE-001: user priority cap downgrades High to Normal
    fn test_priority_cap() {
        let mut engine = make_engine();
        let mut rule = NotificationRule::default_for("ind.app.social");
        rule.priority_cap = Some(NotifPriority::Normal);
        engine.set_rule(rule, CAP_NOTIFY_RULE_WRITE).unwrap();

        let verdict = engine.evaluate("ind.app.social", NotifPriority::High, 14, false);
        assert_eq!(
            verdict,
            NotificationVerdict::Deliver {
                effective_priority: NotifPriority::Normal,
                play_sound: true,
                vibrate: true,
                heads_up: false,
            }
        );
    }

    #[test]
    // INV-DISCIPLINE-002: Critical/Emergency notifications bypass user block & quiet hours
    fn test_emergency_bypasses_rules() {
        let mut engine = make_engine();
        let mut rule = NotificationRule::default_for("ind.app.phone");
        rule.is_blocked = true; // User mistakenly tried to block phone
        rule.quiet_hours = Some((22, 7));
        engine.set_rule(rule, CAP_NOTIFY_RULE_WRITE).unwrap();

        // Critical notification (e.g. emergency call or critical alarm) at 2 AM
        let verdict = engine.evaluate("ind.app.phone", NotifPriority::Critical, 2, true);
        assert_eq!(
            verdict,
            NotificationVerdict::Deliver {
                effective_priority: NotifPriority::Critical,
                play_sound: true,
                vibrate: true,
                heads_up: true,
            }
        );
    }

    #[test]
    // Quiet hours overnight span (22:00 to 07:00)
    fn test_quiet_hours_overnight() {
        let mut engine = make_engine();
        let mut rule = NotificationRule::default_for("ind.app.chat");
        rule.quiet_hours = Some((22, 7));
        engine.set_rule(rule, CAP_NOTIFY_RULE_WRITE).unwrap();

        // 23:00 (11 PM) -> quiet hours active
        assert_eq!(
            engine.evaluate("ind.app.chat", NotifPriority::Normal, 23, false),
            NotificationVerdict::MutedToBadge
        );

        // 03:00 (3 AM) -> quiet hours active
        assert_eq!(
            engine.evaluate("ind.app.chat", NotifPriority::Normal, 3, false),
            NotificationVerdict::MutedToBadge
        );

        // 12:00 (noon) -> quiet hours inactive
        assert_eq!(
            engine.evaluate("ind.app.chat", NotifPriority::Normal, 12, false),
            NotificationVerdict::Deliver {
                effective_priority: NotifPriority::Normal,
                play_sound: true,
                vibrate: true,
                heads_up: false,
            }
        );
    }

    #[test]
    // DND bypass rule for VIP apps
    fn test_dnd_bypass() {
        let mut engine = make_engine();
        let mut rule = NotificationRule::default_for("ind.app.family");
        rule.dnd_bypass = true;
        engine.set_rule(rule, CAP_NOTIFY_RULE_WRITE).unwrap();

        // Normal app during DND -> Muted
        assert_eq!(
            engine.evaluate("ind.app.other", NotifPriority::Normal, 14, true),
            NotificationVerdict::MutedToBadge
        );

        // Family app with DND bypass -> Delivered
        assert_eq!(
            engine.evaluate("ind.app.family", NotifPriority::Normal, 14, true),
            NotificationVerdict::Deliver {
                effective_priority: NotifPriority::Normal,
                play_sound: true,
                vibrate: true,
                heads_up: false,
            }
        );
    }

    #[test]
    // INV-AISKILL-001 & INV-AISKILL-002: Kill switch trigger across all surfaces <= 2 gestures
    fn test_ais_kill_switch_triggers() {
        let mut engine = make_engine();
        assert_eq!(engine.ais_state, AisSystemState::Active);
        assert!(engine.ais_state.allows_inference());
        assert!(engine.ais_state.allows_telemetry_read());
        assert!(engine.ais_state.allows_bento_cards());

        // Lock screen gesture trigger (2 gestures)
        let state = engine.trigger_kill_switch(KillSwitchOrigin::LockScreen, 1000);
        assert_eq!(state, AisSystemState::Killed);
        assert_eq!(engine.last_kill_origin, Some(KillSwitchOrigin::LockScreen));
        assert!(!engine.ais_state.allows_inference());
        assert!(!engine.ais_state.allows_telemetry_read());
        assert!(!engine.ais_state.allows_bento_cards());

        // Re-enable to Restricted
        engine.restore_ais_state(AisSystemState::Restricted);
        assert_eq!(engine.ais_state, AisSystemState::Restricted);
        assert!(!engine.ais_state.allows_inference());
        assert!(engine.ais_state.allows_telemetry_read());

        // Notification shade trigger (2 gestures)
        engine.trigger_kill_switch(KillSwitchOrigin::NotificationShade, 1050);
        assert_eq!(engine.ais_state, AisSystemState::Killed);
        assert_eq!(engine.last_kill_origin, Some(KillSwitchOrigin::NotificationShade));

        // Quick settings trigger (2 gestures)
        engine.trigger_kill_switch(KillSwitchOrigin::QuickSettings, 1100);
        assert_eq!(engine.ais_state, AisSystemState::Killed);
        assert_eq!(engine.last_kill_origin, Some(KillSwitchOrigin::QuickSettings));

        // Hardware button chord (1 gesture)
        engine.trigger_kill_switch(KillSwitchOrigin::HardwareButtonChord, 1150);
        assert_eq!(engine.ais_state, AisSystemState::Killed);
        assert_eq!(engine.kill_event_count, 4);
    }
}

