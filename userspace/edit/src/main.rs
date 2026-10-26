//! Ind OS Modal Text Editor Entry Point
//!
//! Constitutional References:
//! - Rule N-01: Zero external crates
//! - Rule N-04: Zero unsafe code in new modules

#![no_std]
#![no_main]

use core::panic::PanicInfo;
use indus_edit::{EditorMode, EditorSession};

#[no_mangle]
pub extern "C" fn _start() -> ! {
    let mut ed = EditorSession::new("test.txt");

    // Insert text
    ed.mode = EditorMode::Insert;
    let _ = ed.handle_key(b'H');
    let _ = ed.handle_key(b'i');

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
