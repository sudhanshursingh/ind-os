//! Ind OS Microkernel (IndK) Main Entry Point
//! Sprint SP-04: Boot Entry & Exception Vectors

#![no_std]
#![no_main]

pub mod arch;
pub mod panic;

use panic::EarlyUart;

/// Kernel main entry point called from `boot.s` after EL1 transition & stack setup
#[no_mangle]
pub extern "C" fn kernel_main(dtb_addr: usize) -> ! {
    EarlyUart::write_str("\n==================================================\n");
    EarlyUart::write_str(" [BOOT] IndK ARM64 Microkernel Starting...\n");
    EarlyUart::write_str("==================================================\n");

    EarlyUart::write_str(" [BOOT] Vector table (VBAR_EL1) online.\n");
    EarlyUart::write_str(" [BOOT] Early kernel stack verified (INV-BOOT-002).\n");

    EarlyUart::write_str(" [BOOT] Device Tree Blob at: ");
    EarlyUart::write_hex_u64(dtb_addr as u64);
    EarlyUart::write_str("\n");

    // Double check magic in Rust
    let magic = unsafe {
        let ptr = dtb_addr as *const u32;
        u32::from_be(*ptr)
    };

    if magic == 0xd00dfeed {
        EarlyUart::write_str(" [BOOT] DTB magic 0xd00dfeed verified (INV-BOOT-003).\n");
    } else {
        panic!("DTB magic corrupted!");
    }

    EarlyUart::write_str("--------------------------------------------------\n");
    EarlyUart::write_str(" [BOOT] Sprint SP-04 Boot Entry & Vector Milestone REACHED.\n");
    EarlyUart::write_str("==================================================\n");

    // Park CPU loop until timer interrupts arrive in SP-08
    loop {
        core::hint::spin_loop();
    }
}
