//! Ind OS AI Daemon Entry Point (`aisd`)
//!
//! Constitutional References:
//! - Rule N-01: Zero external crates
//! - Rule N-04: Zero unsafe code in new modules

#![no_std]
#![no_main]

use core::panic::PanicInfo;
use ais::{IndHardwareProfile, ModelRuntimeSandbox};

#[no_mangle]
pub extern "C" fn _start() -> ! {
    // 1. Discover hardware profile (e.g. 512 MB QEMU RAM)
    let config = IndHardwareProfile::select_profile(512);
    let _sandbox = ModelRuntimeSandbox::new(config);

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
