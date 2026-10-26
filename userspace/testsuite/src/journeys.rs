//! Ind OS Scripted User Journeys (Whole-System Behavior Tests)
//!
//! Standards:
//! - REQ-QA-201 Groundwork
//! - INV-QA-001: Deterministic user journey execution

use crate::types::{JourneyReport, JourneyStatus};
use devmgr::{DeviceType, HardwareDescriptor, VirtioDeviceManager};
use indus_edit::{EditorMode, EditorSession};
use indus_init::{InitSubsystem, MAX_SERVICES};
use indus_logd::{DiagnosticCause, LogBus, LogLevel, WhyExplanationEngine};
use indus_shell::ShellSession;
use indus_store::{ConsentState, SettingsAndConsentStore, CAP_STORE_ADMIN, CAP_STORE_READ};

pub struct UserJourneyRunner;

impl UserJourneyRunner {
    pub fn run_all() -> [JourneyReport; 5] {
        [
            Self::journey_1_init_supervision(),
            Self::journey_2_hotplug_capabilities(),
            Self::journey_3_shell_session(),
            Self::journey_4_logging_diagnostics(),
            Self::journey_5_store_consent(),
        ]
    }

    pub fn journey_1_init_supervision() -> JourneyReport {
        let mut init = InitSubsystem::new();
        if init.load_baseline().is_err() {
            return JourneyReport { journey_id: 1, name: "Init & Supervision", status: JourneyStatus::Failed };
        }
        let mut order = [""; MAX_SERVICES];
        if init.resolve_startup_order(&mut order).is_err() {
            return JourneyReport { journey_id: 1, name: "Init & Supervision", status: JourneyStatus::Failed };
        }
        JourneyReport { journey_id: 1, name: "Init & Supervision", status: JourneyStatus::Passed }
    }

    pub fn journey_2_hotplug_capabilities() -> JourneyReport {
        let mut dm = VirtioDeviceManager::new();
        let desc = HardwareDescriptor::new(0x0a000000, 512, 48, 0);
        let token_res = dm.plug_device(0, DeviceType::Console, desc);
        if token_res.is_err() {
            return JourneyReport { journey_id: 2, name: "Hotplug & Capabilities", status: JourneyStatus::Failed };
        }
        let unplug_res = dm.unplug_device(0);
        if unplug_res.is_err() {
            return JourneyReport { journey_id: 2, name: "Hotplug & Capabilities", status: JourneyStatus::Failed };
        }
        JourneyReport { journey_id: 2, name: "Hotplug & Capabilities", status: JourneyStatus::Passed }
    }

    pub fn journey_3_shell_session() -> JourneyReport {
        let mut out = [0u8; 256];
        let r1 = ShellSession::execute_line("version", &mut out);
        let r2 = ShellSession::execute_line("ps", &mut out);
        let r3 = ShellSession::execute_line("echo \"qa pipeline\" | cat", &mut out);

        // Also exercise text editor
        let mut ed = EditorSession::new("notes.txt");
        ed.mode = EditorMode::Insert;
        let r4 = ed.handle_key(b'o');
        let r5 = ed.handle_key(b'k');

        if r1.is_err() || r2.is_err() || r3.is_err() || r4.is_err() || r5.is_err() {
            return JourneyReport { journey_id: 3, name: "Interactive Shell & Editor", status: JourneyStatus::Failed };
        }
        JourneyReport { journey_id: 3, name: "Interactive Shell & Editor", status: JourneyStatus::Passed }
    }

    pub fn journey_4_logging_diagnostics() -> JourneyReport {
        let mut bus = LogBus::new();
        bus.ingest(100, LogLevel::Info, 1, 1001, "login token:0x1234");
        let mut why = WhyExplanationEngine::new();
        why.record_failure(2001, "devmgr", DiagnosticCause::ProcessCrash, "crash", "restart");
        if why.explain(2001).is_none() {
            return JourneyReport { journey_id: 4, name: "Logging & Diagnostics", status: JourneyStatus::Failed };
        }
        JourneyReport { journey_id: 4, name: "Logging & Diagnostics", status: JourneyStatus::Passed }
    }

    pub fn journey_5_store_consent() -> JourneyReport {
        let mut store = SettingsAndConsentStore::new();
        if store.set_setting("sec.enforce", 1, CAP_STORE_ADMIN).is_err() {
            return JourneyReport { journey_id: 5, name: "Store & Consent", status: JourneyStatus::Failed };
        }
        if store.get_setting("sec.enforce", CAP_STORE_READ).unwrap_or(0) != 1 {
            return JourneyReport { journey_id: 5, name: "Store & Consent", status: JourneyStatus::Failed };
        }
        if store.set_consent(1, 10, ConsentState::Granted, 200, CAP_STORE_ADMIN).is_err() {
            return JourneyReport { journey_id: 5, name: "Store & Consent", status: JourneyStatus::Failed };
        }
        JourneyReport { journey_id: 5, name: "Store & Consent", status: JourneyStatus::Passed }
    }
}
