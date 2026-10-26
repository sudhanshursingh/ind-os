//! Serial Console Subsystem for IndK
//! Provides thread-safe formatted printing via PL011 UART and Spinlock.

use core::fmt::{self, Write};
use crate::drivers::pl011::{Pl011Uart, DEFAULT_PL011_BASE};
use crate::sync::Spinlock;

/// Global Console singleton holding the PL011 UART driver instance
pub static CONSOLE: Spinlock<Pl011Uart> = Spinlock::new(unsafe {
    Pl011Uart::new(DEFAULT_PL011_BASE)
});

/// Initialize the early serial console hardware
pub fn init_console() {
    CONSOLE.lock().init();
}

/// Low-level formatted print dispatcher
#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    let mut console = CONSOLE.lock();
    let _ = console.write_fmt(args);
}

/// Print formatted text to the serial console
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        $crate::console::_print(format_args!($($arg)*));
    };
}

/// Print formatted text followed by a newline to the serial console
#[macro_export]
macro_rules! println {
    () => {
        $crate::print!("\n");
    };
    ($($arg:tt)*) => {
        $crate::print!("{}\n", format_args!($($arg)*));
    };
}
