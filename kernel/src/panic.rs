//! Structured Panic & Exception Reporting for IndK
//! Implements INV-BOOT-004: Unexpected early exceptions trigger formatted serial panic.

use core::panic::PanicInfo;
use crate::arch::ExceptionContext;

const UART0_BASE: usize = 0x09000000;

/// Direct raw UART byte writer for early panic reporting before console driver
pub struct EarlyUart;

impl EarlyUart {
    #[inline(always)]
    pub fn write_byte(b: u8) {
        unsafe {
            core::ptr::write_volatile(UART0_BASE as *mut u8, b);
        }
    }

    pub fn write_str(s: &str) {
        for b in s.bytes() {
            if b == b'\n' {
                Self::write_byte(b'\r');
            }
            Self::write_byte(b);
        }
    }

    pub fn write_hex_u64(mut val: u64) {
        Self::write_str("0x");
        if val == 0 {
            Self::write_byte(b'0');
            return;
        }
        let hex_chars = b"0123456789abcdef";
        let mut buf = [0u8; 16];
        let mut i = 0;
        while val > 0 && i < 16 {
            buf[i] = hex_chars[(val & 0xf) as usize];
            val >>= 4;
            i += 1;
        }
        while i > 0 {
            i -= 1;
            Self::write_byte(buf[i]);
        }
    }
}

/// Structured panic called when an AArch64 hardware exception fires unexpectedly
pub fn panic_exception(ctx: &ExceptionContext) -> ! {
    EarlyUart::write_str("\n==================================================\n");
    EarlyUart::write_str(" [PANIC] IndK Hardware Exception Trap! (INV-BOOT-004)\n");
    EarlyUart::write_str("==================================================\n");

    EarlyUart::write_str(" Vector ID:  ");
    EarlyUart::write_hex_u64(ctx.exception_id);
    EarlyUart::write_str("\n Fault PC:   ");
    EarlyUart::write_hex_u64(ctx.elr_el1);
    EarlyUart::write_str("\n Fault Addr: ");
    EarlyUart::write_hex_u64(ctx.far_el1);
    EarlyUart::write_str("\n Syndrome:   ");
    EarlyUart::write_hex_u64(ctx.esr_el1);
    EarlyUart::write_str("\n SPSR_EL1:   ");
    EarlyUart::write_hex_u64(ctx.spsr_el1);
    EarlyUart::write_str("\n Stack SP:   ");
    EarlyUart::write_hex_u64(ctx.sp);
    EarlyUart::write_str("\n Link Reg:   ");
    EarlyUart::write_hex_u64(ctx.lr);
    EarlyUart::write_str("\n--------------------------------------------------\n");
    EarlyUart::write_str(" Halting CPU core.\n");
    EarlyUart::write_str("==================================================\n");

    loop {
        core::hint::spin_loop();
    }
}

/// Standard Rust panic handler
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    EarlyUart::write_str("\n==================================================\n");
    EarlyUart::write_str(" [PANIC] IndK Kernel Panic!\n");
    EarlyUart::write_str("==================================================\n");

    if let Some(loc) = info.location() {
        EarlyUart::write_str(" Location: ");
        EarlyUart::write_str(loc.file());
        EarlyUart::write_str(":");
        EarlyUart::write_hex_u64(loc.line() as u64);
        EarlyUart::write_str("\n");
    }

    EarlyUart::write_str(" Message:  ");
    // Format message if available
    let msg = info.message();
    EarlyUart::write_str(msg.as_str().unwrap_or("structured panic payload"));
    EarlyUart::write_str("\n--------------------------------------------------\n");
    EarlyUart::write_str(" System halted.\n");
    EarlyUart::write_str("==================================================\n");

    // Visual hardware panic alert on physical screen (0x9c000000 / 0x9d000000)
    unsafe {
        for &fb_addr in &[0x9c00_0000usize, 0x9d00_0000usize] {
            let fb = fb_addr as *mut u32;
            for i in 0..(1080 * 120) {
                core::ptr::write_volatile(fb.add(i), 0xFFFF_0055); // Crimson panic banner
            }
        }
    }

    loop {
        core::hint::spin_loop();
    }
}
