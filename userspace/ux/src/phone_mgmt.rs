//! Phone App II: Call History, Suraksha Scam Shield, Dual-SIM & Speed Dial (SP-133)
//!
//! Bible §8.18.2, §8.20, §9.9.2 REQ-APP-PHONE-003, REQ-APP-PHONE-006..010
//! Invariants:
//! - INV-PHONE-HIST-001: Call history is stored strictly on-device; zero cloud sync or external exfiltration.
//! - INV-PHONE-SIM-001: Per-call SIM selection seamlessly falls back to active SIM on single-SIM devices.
//! - INV-PHONE-SCAM-001: Suraksha Scam Shield operates 100% offline using deterministic pattern heuristics.
//! - INV-PHONE-AIS-001: Voice intent "Call X" produces a proposal card requiring confirmed user tap;
//!                       silent dialing by AI is structurally prohibited.
//! - INV-PHONE-OFFLINE-001: All phone management operations function without cellular data/internet.
//! Rules: N-01 (zero external crates), N-04 (no unsafe), N-05 (user authority supreme)

pub const MAX_CALL_LOG_ENTRIES: usize = 64;
pub const MAX_SPEED_DIAL_SLOTS: usize = 10;
pub const MAX_BLOCKLIST_ENTRIES: usize = 32;

/// Capabilities required for phone management operations.
pub const CAP_PHONE_HISTORY_READ: u64 = 0x0080_0000_0000;
pub const CAP_PHONE_HISTORY_WRITE: u64 = 0x0100_0000_0000;

/// SIM slot enumeration for dual-SIM devices (REQ-APP-PHONE-006).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SimSlot {
    Sim1,
    Sim2,
}

/// Call direction in local call history (REQ-APP-PHONE-003).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CallDirection {
    Incoming,
    Outgoing,
    Missed,
    Rejected,
    BlockedScam,
}

/// One call record in the local-only call history log (INV-PHONE-HIST-001).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CallLogRecord {
    pub record_id: u32,
    pub number: [u8; 32],
    pub number_len: usize,
    pub contact_name: Option<&'static str>,
    pub direction: CallDirection,
    pub sim_slot: SimSlot,
    pub duration_seconds: u32,
    pub timestamp_sec: u64,
    pub is_scam_flagged: bool,
}

impl CallLogRecord {
    pub fn number_as_str(&self) -> Result<&str, core::str::Utf8Error> {
        core::str::from_utf8(&self.number[..self.number_len])
    }
}

/// Dual-SIM configuration and status (REQ-APP-PHONE-006).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DualSimConfig {
    pub sim1_active: bool,
    pub sim1_carrier: &'static str,
    pub sim2_active: bool,
    pub sim2_carrier: &'static str,
    pub default_voice_sim: Option<SimSlot>,
}

impl DualSimConfig {
    pub const fn single_sim(carrier: &'static str) -> Self {
        Self {
            sim1_active: true,
            sim1_carrier: carrier,
            sim2_active: false,
            sim2_carrier: "",
            default_voice_sim: Some(SimSlot::Sim1),
        }
    }

    pub const fn dual_sim(carrier1: &'static str, carrier2: &'static str) -> Self {
        Self {
            sim1_active: true,
            sim1_carrier: carrier1,
            sim2_active: true,
            sim2_carrier: carrier2,
            default_voice_sim: None, // Ask user every time by default
        }
    }

    /// Resolves which SIM to use, with single-SIM fallback (INV-PHONE-SIM-001).
    pub fn resolve_sim(&self, requested: Option<SimSlot>) -> SimSlot {
        match (self.sim1_active, self.sim2_active) {
            (true, false) => SimSlot::Sim1, // Only SIM1 active
            (false, true) => SimSlot::Sim2, // Only SIM2 active
            (true, true) => requested.or(self.default_voice_sim).unwrap_or(SimSlot::Sim1),
            (false, false) => SimSlot::Sim1, // No SIM, fallback for emergency
        }
    }
}

/// Speed dial entry (REQ-APP-PHONE-007).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpeedDialEntry {
    pub key: u8, // 1..9, or 0
    pub contact_name: &'static str,
    pub number: [u8; 32],
    pub number_len: usize,
    pub preferred_sim: Option<SimSlot>,
}

/// Suraksha Scam Shield classification verdict (INV-PHONE-SCAM-001, REQ-APP-PHONE-008).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScamShieldVerdict {
    Safe,
    Suspicious { score: u8, reason: &'static str },
    KnownScam { reason: &'static str },
}

/// AIS voice proposal requiring explicit user confirmation (INV-PHONE-AIS-001, REQ-APP-PHONE-009).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CallProposalCard {
    pub proposal_id: u32,
    pub contact_name: &'static str,
    pub number: [u8; 32],
    pub number_len: usize,
    pub sim_slot: SimSlot,
    pub user_confirmed: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhoneMgmtError {
    MissingHistoryReadCapability,
    MissingHistoryWriteCapability,
    HistoryFull,
    RecordNotFound,
    SpeedDialSlotFull,
    InvalidSpeedDialKey,
    BlocklistFull,
    UnconfirmedAisCall,
}

/// Phone App Management Engine (History, Suraksha Shield, Dual-SIM, Speed Dial).
pub struct PhoneMgmtEngine {
    history: [Option<CallLogRecord>; MAX_CALL_LOG_ENTRIES],
    history_count: usize,
    next_record_id: u32,
    sim_config: DualSimConfig,
    speed_dial: [Option<SpeedDialEntry>; MAX_SPEED_DIAL_SLOTS],
    blocklist: [[u8; 32]; MAX_BLOCKLIST_ENTRIES],
    blocklist_lens: [usize; MAX_BLOCKLIST_ENTRIES],
    blocklist_count: usize,
    next_proposal_id: u32,
}

impl PhoneMgmtEngine {
    pub const fn new(sim_config: DualSimConfig) -> Self {
        Self {
            history: [None; MAX_CALL_LOG_ENTRIES],
            history_count: 0,
            next_record_id: 1,
            sim_config,
            speed_dial: [None; MAX_SPEED_DIAL_SLOTS],
            blocklist: [[0u8; 32]; MAX_BLOCKLIST_ENTRIES],
            blocklist_lens: [0usize; MAX_BLOCKLIST_ENTRIES],
            blocklist_count: 0,
            next_proposal_id: 1,
        }
    }

    pub fn history_count(&self) -> usize {
        self.history_count
    }

    pub fn sim_config(&self) -> &DualSimConfig {
        &self.sim_config
    }

    // ── Local Call History (INV-PHONE-HIST-001, REQ-APP-PHONE-003) ────────────

    /// Log a completed, missed, or blocked call locally. Requires CAP_PHONE_HISTORY_WRITE.
    pub fn log_call(
        &mut self,
        number: &[u8],
        contact_name: Option<&'static str>,
        direction: CallDirection,
        requested_sim: Option<SimSlot>,
        duration_seconds: u32,
        timestamp_sec: u64,
        is_scam: bool,
        caller_caps: u64,
    ) -> Result<u32, PhoneMgmtError> {
        if (caller_caps & CAP_PHONE_HISTORY_WRITE) == 0 {
            return Err(PhoneMgmtError::MissingHistoryWriteCapability);
        }

        let record_id = self.next_record_id;
        self.next_record_id += 1;

        let sim_slot = self.sim_config.resolve_sim(requested_sim);

        let mut num_buf = [0u8; 32];
        let len = number.len().min(32);
        num_buf[..len].copy_from_slice(&number[..len]);

        let record = CallLogRecord {
            record_id,
            number: num_buf,
            number_len: len,
            contact_name,
            direction,
            sim_slot,
            duration_seconds,
            timestamp_sec,
            is_scam_flagged: is_scam,
        };

        // Prepend to maintain reverse chronological order (newest at index 0)
        if self.history_count < MAX_CALL_LOG_ENTRIES {
            for i in (1..=self.history_count).rev() {
                self.history[i] = self.history[i - 1];
            }
            self.history[0] = Some(record);
            self.history_count += 1;
        } else {
            // Table full: evict oldest entry at end
            for i in (1..MAX_CALL_LOG_ENTRIES).rev() {
                self.history[i] = self.history[i - 1];
            }
            self.history[0] = Some(record);
        }

        Ok(record_id)
    }

    /// Read call history records. Requires CAP_PHONE_HISTORY_READ.
    pub fn get_history(&self, caller_caps: u64) -> Result<&[Option<CallLogRecord>; MAX_CALL_LOG_ENTRIES], PhoneMgmtError> {
        if (caller_caps & CAP_PHONE_HISTORY_READ) == 0 {
            return Err(PhoneMgmtError::MissingHistoryReadCapability);
        }
        Ok(&self.history)
    }

    // ── Speed Dial & Favorites (REQ-APP-PHONE-007) ────────────────────────────

    pub fn set_speed_dial(
        &mut self,
        key: u8,
        contact_name: &'static str,
        number: &[u8],
        preferred_sim: Option<SimSlot>,
        caller_caps: u64,
    ) -> Result<(), PhoneMgmtError> {
        if (caller_caps & CAP_PHONE_HISTORY_WRITE) == 0 {
            return Err(PhoneMgmtError::MissingHistoryWriteCapability);
        }
        if key > 9 {
            return Err(PhoneMgmtError::InvalidSpeedDialKey);
        }

        let mut num_buf = [0u8; 32];
        let len = number.len().min(32);
        num_buf[..len].copy_from_slice(&number[..len]);

        self.speed_dial[key as usize] = Some(SpeedDialEntry {
            key,
            contact_name,
            number: num_buf,
            number_len: len,
            preferred_sim,
        });

        Ok(())
    }

    pub fn lookup_speed_dial(&self, key: u8) -> Option<&SpeedDialEntry> {
        if key <= 9 {
            self.speed_dial[key as usize].as_ref()
        } else {
            None
        }
    }

    // ── Suraksha Scam Shield (INV-PHONE-SCAM-001, REQ-APP-PHONE-008) ──────────

    /// Evaluates an incoming or dialed phone number against offline threat heuristics.
    pub fn evaluate_scam_shield(&self, number: &[u8]) -> ScamShieldVerdict {
        let Ok(s) = core::str::from_utf8(number) else {
            return ScamShieldVerdict::Suspicious { score: 70, reason: "Invalid number encoding" };
        };

        // 1. Check local blocklist
        for i in 0..self.blocklist_count {
            let len = self.blocklist_lens[i];
            if &self.blocklist[i][..len] == number {
                return ScamShieldVerdict::KnownScam { reason: "User blocklisted number" };
            }
        }

        // 2. High-risk satellite / premium toll fraud prefixes (+881, +882, +883, 1900)
        if s.starts_with("+881") || s.starts_with("+882") || s.starts_with("+883") {
            return ScamShieldVerdict::KnownScam { reason: "High-risk satellite premium fraud prefix" };
        }
        if s.starts_with("1900") || s.starts_with("+1900") {
            return ScamShieldVerdict::KnownScam { reason: "Premium rate toll fraud" };
        }

        // 3. Digital Arrest / Enforcement impersonation pattern (+92, +93, or suspicious spoofed lengths)
        if s.starts_with("+92") || s.starts_with("+93") {
            return ScamShieldVerdict::Suspicious { score: 85, reason: "High-risk foreign jurisdiction" };
        }

        // 4. Robocall / invalid digit patterns (e.g. all repeating digits like "9999999999")
        if number.len() >= 10 && number.iter().all(|&b| b == number[0]) {
            return ScamShieldVerdict::Suspicious { score: 90, reason: "Suspicious repeating digit pattern" };
        }

        ScamShieldVerdict::Safe
    }

    /// Adds a number to the local blocklist.
    pub fn add_to_blocklist(&mut self, number: &[u8], caller_caps: u64) -> Result<(), PhoneMgmtError> {
        if (caller_caps & CAP_PHONE_HISTORY_WRITE) == 0 {
            return Err(PhoneMgmtError::MissingHistoryWriteCapability);
        }
        if self.blocklist_count >= MAX_BLOCKLIST_ENTRIES {
            return Err(PhoneMgmtError::BlocklistFull);
        }

        let len = number.len().min(32);
        self.blocklist[self.blocklist_count][..len].copy_from_slice(&number[..len]);
        self.blocklist_lens[self.blocklist_count] = len;
        self.blocklist_count += 1;

        Ok(())
    }

    // ── AIS Voice Intent Call Proposal (INV-PHONE-AIS-001, REQ-APP-PHONE-009) ─

    /// Creates a voice call proposal card from AIS intent. Silent dialing is strictly blocked.
    pub fn create_call_proposal(
        &mut self,
        contact_name: &'static str,
        number: &[u8],
        preferred_sim: Option<SimSlot>,
    ) -> CallProposalCard {
        let proposal_id = self.next_proposal_id;
        self.next_proposal_id += 1;

        let mut num_buf = [0u8; 32];
        let len = number.len().min(32);
        num_buf[..len].copy_from_slice(&number[..len]);

        let sim_slot = self.sim_config.resolve_sim(preferred_sim);

        CallProposalCard {
            proposal_id,
            contact_name,
            number: num_buf,
            number_len: len,
            sim_slot,
            user_confirmed: false, // Strict requirement: must be confirmed by user tap
        }
    }

    /// User explicitly taps the [Call] button on the proposal card.
    pub fn confirm_call_proposal(
        &self,
        proposal: &mut CallProposalCard,
    ) -> Result<(SimSlot, [u8; 32], usize), PhoneMgmtError> {
        proposal.user_confirmed = true;
        Ok((proposal.sim_slot, proposal.number, proposal.number_len))
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// Tests
// ──────────────────────────────────────────────────────────────────────────────
#[cfg(test)]
mod tests {
    use super::*;

    fn make_single_sim_engine() -> PhoneMgmtEngine {
        PhoneMgmtEngine::new(DualSimConfig::single_sim("Jio 4G"))
    }

    fn make_dual_sim_engine() -> PhoneMgmtEngine {
        PhoneMgmtEngine::new(DualSimConfig::dual_sim("Airtel", "Jio"))
    }

    #[test]
    // INV-PHONE-HIST-001: Call history logged in reverse chronological order
    fn test_call_history_logging() {
        let mut engine = make_single_sim_engine();
        let r1 = engine.log_call(
            b"+919876543210",
            Some("Priya"),
            CallDirection::Incoming,
            None,
            120,
            1000,
            false,
            CAP_PHONE_HISTORY_WRITE,
        ).unwrap();

        let r2 = engine.log_call(
            b"+911122334455",
            None,
            CallDirection::Outgoing,
            None,
            45,
            1050,
            false,
            CAP_PHONE_HISTORY_WRITE,
        ).unwrap();

        assert_eq!(engine.history_count(), 2);

        let history = engine.get_history(CAP_PHONE_HISTORY_READ).unwrap();
        // Newest call r2 at index 0
        assert_eq!(history[0].unwrap().record_id, r2);
        assert_eq!(history[0].unwrap().duration_seconds, 45);
        // Older call r1 at index 1
        assert_eq!(history[1].unwrap().record_id, r1);
        assert_eq!(history[1].unwrap().contact_name, Some("Priya"));
    }

    #[test]
    // Call history requires read/write capabilities
    fn test_history_capability_checks() {
        let mut engine = make_single_sim_engine();
        let err_write = engine.log_call(b"123", None, CallDirection::Missed, None, 0, 100, false, 0);
        assert_eq!(err_write, Err(PhoneMgmtError::MissingHistoryWriteCapability));

        let err_read = engine.get_history(0);
        assert_eq!(err_read.unwrap_err(), PhoneMgmtError::MissingHistoryReadCapability);
    }

    #[test]
    // INV-PHONE-SIM-001: Dual SIM resolution and single SIM fallback
    fn test_dual_sim_resolution() {
        let single = make_single_sim_engine();
        // Requested SIM2 on single-SIM hardware falls back to SIM1
        assert_eq!(single.sim_config().resolve_sim(Some(SimSlot::Sim2)), SimSlot::Sim1);

        let dual = make_dual_sim_engine();
        // Requested SIM2 honored on dual-SIM hardware
        assert_eq!(dual.sim_config().resolve_sim(Some(SimSlot::Sim2)), SimSlot::Sim2);
        assert_eq!(dual.sim_config().resolve_sim(Some(SimSlot::Sim1)), SimSlot::Sim1);
    }

    #[test]
    // REQ-APP-PHONE-007: Speed dial registration and lookup
    fn test_speed_dial() {
        let mut engine = make_single_sim_engine();
        engine.set_speed_dial(
            1,
            "Mom",
            b"+919988776655",
            Some(SimSlot::Sim1),
            CAP_PHONE_HISTORY_WRITE,
        ).unwrap();

        let entry = engine.lookup_speed_dial(1).unwrap();
        assert_eq!(entry.contact_name, "Mom");
        assert_eq!(&entry.number[..entry.number_len], b"+919988776655");

        // Invalid key > 9 rejected
        let err = engine.set_speed_dial(15, "Invalid", b"123", None, CAP_PHONE_HISTORY_WRITE);
        assert_eq!(err, Err(PhoneMgmtError::InvalidSpeedDialKey));
    }

    #[test]
    // INV-PHONE-SCAM-001: Suraksha Scam Shield offline pattern evaluations
    fn test_suraksha_scam_shield() {
        let mut engine = make_single_sim_engine();

        // Normal number is Safe
        assert_eq!(engine.evaluate_scam_shield(b"+919876543210"), ScamShieldVerdict::Safe);

        // Satellite toll fraud prefix
        assert!(matches!(
            engine.evaluate_scam_shield(b"+881631555555"),
            ScamShieldVerdict::KnownScam { .. }
        ));

        // Premium rate toll fraud
        assert!(matches!(
            engine.evaluate_scam_shield(b"1900112233"),
            ScamShieldVerdict::KnownScam { .. }
        ));

        // Suspicious repeating digits
        assert!(matches!(
            engine.evaluate_scam_shield(b"8888888888"),
            ScamShieldVerdict::Suspicious { .. }
        ));

        // User blocklist check
        engine.add_to_blocklist(b"+910000000000", CAP_PHONE_HISTORY_WRITE).unwrap();
        assert!(matches!(
            engine.evaluate_scam_shield(b"+910000000000"),
            ScamShieldVerdict::KnownScam { .. }
        ));
    }

    #[test]
    // INV-PHONE-AIS-001: Voice intent creates proposal card requiring explicit user confirmation
    fn test_ais_voice_call_proposal() {
        let mut engine = make_single_sim_engine();
        let mut proposal = engine.create_call_proposal("Doctor", b"+919876500000", None);

        assert_eq!(proposal.contact_name, "Doctor");
        assert!(!proposal.user_confirmed); // Must NOT dial silently

        // User confirms tap
        let (sim, number, len) = engine.confirm_call_proposal(&mut proposal).unwrap();
        assert!(proposal.user_confirmed);
        assert_eq!(sim, SimSlot::Sim1);
        assert_eq!(&number[..len], b"+919876500000");
    }
}
