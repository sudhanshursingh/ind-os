//! Ind OS Settings & Consent Store Process Entry Point
//!
//! Constitutional References:
//! - REQ-USE-206: Settings & consent store
//! - Rule N-01: Zero external crates
//! - Rule N-04: Zero unsafe code in new modules

#![no_std]
#![no_main]

use core::panic::PanicInfo;
use indus_store::{ConsentState, SettingsAndConsentStore, CAP_STORE_ADMIN};

#[no_mangle]
pub extern "C" fn _start() -> ! {
    let mut store = SettingsAndConsentStore::new();

    // Baseline security setting: telemetry read-only enforced
    let _ = store.set_setting("telemetry.read_only", 1, CAP_STORE_ADMIN);

    // Baseline consent: devmgr granted hardware capability management
    let _ = store.set_consent(3, 101, ConsentState::Granted, 100, CAP_STORE_ADMIN);

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
