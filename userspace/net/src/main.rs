#![no_std]
#![no_main]

use core::panic::PanicInfo;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    let mut table = net::SocketTable::new();
    let token = table
        .create_socket(net::SocketType::Stream, net::CAP_NET_SOCKET | net::CAP_NET_BIND)
        .expect("Create socket failed");

    table
        .bind(&token, net::Ipv4Address::loopback(), 8080)
        .expect("Bind failed");

    loop {
        core::hint::spin_loop();
    }
}

#[no_mangle]
pub extern "C" fn main() -> i32 {
    0
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {
        core::hint::spin_loop();
    }
}
