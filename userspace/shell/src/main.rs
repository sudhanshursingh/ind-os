//! Ind OS Userspace Shell Entry Point
//!
//! Constitutional References:
//! - Rule N-01: Zero external crates
//! - Rule N-04: Zero unsafe code in new modules

#![no_std]
#![no_main]

use core::panic::PanicInfo;
use indus_shell::{ShellSession, MAX_OUTPUT_BUFFER};

#[no_mangle]
pub extern "C" fn _start() -> ! {
    let mut out = [0u8; MAX_OUTPUT_BUFFER];

    // Simulate startup commands
    let _ = ShellSession::execute_line("version", &mut out);
    let _ = ShellSession::execute_line("ps", &mut out);

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
