//! Ind OS Userspace Device Manager Process Entry Point
//!
//! Constitutional References:
//! - REQ-USE-203: Device manager + hotplug events: capability handout
//! - Rule N-01: Zero external crates
//! - Rule N-04: Zero unsafe code in new modules

#![no_std]
#![no_main]

use core::panic::PanicInfo;
use devmgr::{DeviceType, HardwareDescriptor, VirtioDeviceManager};

#[no_mangle]
pub extern "C" fn _start() -> ! {
    let mut devmgr = VirtioDeviceManager::new();

    // Baseline VirtIO console & block capability handouts
    let console_desc = HardwareDescriptor::new(0x0a000000, 512, 48, 0);
    let _ = devmgr.plug_device(0, DeviceType::Console, console_desc);

    let blk_desc = HardwareDescriptor::new(0x0a000200, 512, 49, 4096);
    let _ = devmgr.plug_device(1, DeviceType::Block, blk_desc);

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
