//! Ind OS Whole-System Integration Test Runner
//!
//! Constitutional References:
//! - Rule N-01: Zero external crates
//! - Rule N-04: Zero unsafe code in new modules

#![no_std]
#![no_main]

use core::panic::PanicInfo;
use indus_testsuite::{SoakHarness, UserJourneyRunner};

#[no_mangle]
pub extern "C" fn _start() -> ! {
    let _reports = UserJourneyRunner::run_all();
    let _soak = SoakHarness::run_cycles(100);

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
