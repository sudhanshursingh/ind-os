//! Ind OS Unified Log Bus Daemon Entry Point
//!
//! Constitutional References:
//! - REQ-USE-204: Unified log bus, redaction filters, `why` command
//! - Rule N-01: Zero external crates
//! - Rule N-04: Zero unsafe code in new modules

#![no_std]
#![no_main]

use core::panic::PanicInfo;
use indus_logd::{DiagnosticCause, LogBus, LogLevel, WhyExplanationEngine};

#[no_mangle]
pub extern "C" fn _start() -> ! {
    let mut bus = LogBus::new();
    let mut why = WhyExplanationEngine::new();

    // Log daemon startup
    bus.ingest(100, LogLevel::Info, 1, 1001, "Unified log bus service online");

    // Register baseline diagnostic cause for devmgr restart test
    why.record_failure(
        5001,
        "devmgr",
        DiagnosticCause::ProcessCrash,
        "devmgr crashed during hotplug stress fuzzing",
        "Restarted with exponential backoff (20ms -> 40ms)",
    );

    loop {
        core::hint::spin_loop();
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {
        core::hint::spin_loop();
    }
}
