//! Kernel Integration Test Suite Verification Harness (SP-39)
//!
//! Constitutional References:
//! - REQ-QA-201: Automated integration test suite
//! - INV-QA-001: Deterministic whole-system user journeys
//! - INV-QA-002: Zero-leak soak cycling
//! - Rule N-01: Zero external crates
//! - Rule N-04: Zero unsafe code in new modules

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KernelJourneyStatus {
    Passed,
    Failed,
}

#[derive(Debug, Clone, Copy)]
pub struct KernelJourneyReport {
    pub journey_id: u32,
    pub name: &'static str,
    pub status: KernelJourneyStatus,
}

#[derive(Debug, Clone, Copy)]
pub struct KernelSoakReport {
    pub completed_cycles: u32,
    pub target_cycles: u32,
    pub leak_count: u32,
    pub passed: bool,
}

pub struct KernelUserJourneyRunner;

impl KernelUserJourneyRunner {
    pub fn run_all() -> [KernelJourneyReport; 5] {
        [
            Self::journey_1_init_supervision(),
            Self::journey_2_hotplug_capabilities(),
            Self::journey_3_shell_session(),
            Self::journey_4_logging_diagnostics(),
            Self::journey_5_store_consent(),
        ]
    }

    pub fn journey_1_init_supervision() -> KernelJourneyReport {
        let mut init = crate::userspace::KernelInitSubsystem::new();
        let _ = init.load_baseline();
        let mut order = [""; crate::userspace::MAX_INIT_SERVICES];
        let res = init.resolve_order(&mut order);
        let status = if res.is_ok() && order[0] == "auditd" {
            KernelJourneyStatus::Passed
        } else {
            KernelJourneyStatus::Failed
        };
        KernelJourneyReport {
            journey_id: 1,
            name: "Init & Supervision",
            status,
        }
    }

    pub fn journey_2_hotplug_capabilities() -> KernelJourneyReport {
        let mut dm = crate::userspace::KernelDeviceManager::new();
        let desc = crate::userspace::KernelHardwareDescriptor::new(0x0a000000, 512, 48, 0);
        let plug = dm.plug_virtio_device(0, crate::userspace::KernelDeviceType::Console, desc);
        let unplug = dm.unplug_virtio_device(0);
        let status = if plug.is_ok() && unplug.is_ok() {
            KernelJourneyStatus::Passed
        } else {
            KernelJourneyStatus::Failed
        };
        KernelJourneyReport {
            journey_id: 2,
            name: "Hotplug & Capabilities",
            status,
        }
    }

    pub fn journey_3_shell_session() -> KernelJourneyReport {
        let p = crate::userspace::KernelShellParser::parse_command("echo hello");
        let mut tbuf = crate::userspace::KernelTextBuffer::new();
        let edit_res = tbuf.insert_line("test.txt");

        let status = if p.is_ok() && edit_res.is_ok() {
            KernelJourneyStatus::Passed
        } else {
            KernelJourneyStatus::Failed
        };
        KernelJourneyReport {
            journey_id: 3,
            name: "Interactive Shell & Editor",
            status,
        }
    }

    pub fn journey_4_logging_diagnostics() -> KernelJourneyReport {
        let mut bus = crate::userspace::KernelLogBus::new();
        bus.ingest(100, crate::userspace::KernelLogLevel::Info, 1, 1001, "login token:0x1234");
        let mut why = crate::userspace::KernelWhyEngine::new();
        why.record(2001, "devmgr", crate::userspace::KernelDiagnosticCause::ProcessCrash, "crash", "restart");
        let explained = why.explain(2001);

        let status = if explained.is_some() && bus.count() == 1 {
            KernelJourneyStatus::Passed
        } else {
            KernelJourneyStatus::Failed
        };
        KernelJourneyReport {
            journey_id: 4,
            name: "Logging & Diagnostics",
            status,
        }
    }

    pub fn journey_5_store_consent() -> KernelJourneyReport {
        let mut store = crate::userspace::KernelSettingsAndConsentStore::new();
        let s_set = store.set_setting("sec.enforce", 1, crate::userspace::KERNEL_CAP_STORE_ADMIN);
        let s_get = store.get_setting("sec.enforce", crate::userspace::KERNEL_CAP_STORE_READ);
        let c_set = store.set_consent(1, 10, crate::userspace::KernelConsentState::Granted, 200, crate::userspace::KERNEL_CAP_STORE_ADMIN);

        let status = if s_set.is_ok() && s_get == Ok(1) && c_set.is_ok() {
            KernelJourneyStatus::Passed
        } else {
            KernelJourneyStatus::Failed
        };
        KernelJourneyReport {
            journey_id: 5,
            name: "Store & Consent",
            status,
        }
    }
}

pub struct KernelSoakHarness;

impl KernelSoakHarness {
    pub fn run_cycles(target_cycles: u32) -> KernelSoakReport {
        let mut completed = 0;
        let mut leaks = 0;

        for _ in 0..target_cycles {
            let mut heap = crate::userspace::KernelUserHeap::new();
            let p1 = heap.allocate(128, 8);
            if p1.is_err() {
                leaks += 1;
                break;
            }

            let cap = crate::userspace::KernelCapToken::new(101, crate::userspace::CORE_CAP_READ | crate::userspace::CORE_CAP_WRITE);
            let ro_cap = cap.attenuate(crate::userspace::CORE_CAP_READ);
            if ro_cap.can_write() {
                leaks += 1;
                break;
            }

            heap.reset();
            if heap.allocated_bytes() != 0 {
                leaks += 1;
                break;
            }

            completed += 1;
        }

        KernelSoakReport {
            completed_cycles: completed,
            target_cycles,
            leak_count: leaks,
            passed: leaks == 0 && completed == target_cycles,
        }
    }
}
