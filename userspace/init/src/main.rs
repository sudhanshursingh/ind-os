//! Ind OS Root Userspace Init Process (PID 1 Entry Point)
//!
//! Constitutional References:
//! - REQ-USE-201: Init process entry
//! - Rule N-01: Zero external crates
//! - Rule N-04: Zero unsafe code in new modules

#![no_std]
#![no_main]

use core::panic::PanicInfo;
use indus_init::{InitSubsystem, MAX_SERVICES};

#[no_mangle]
pub extern "C" fn _start() -> ! {
    let mut init = InitSubsystem::new();
    let _ = init.load_baseline();

    let mut order = [""; MAX_SERVICES];
    let _ = init.resolve_startup_order(&mut order);

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
