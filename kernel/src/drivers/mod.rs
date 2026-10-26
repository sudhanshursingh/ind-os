//! Device drivers for IndK Microkernel

pub mod gicv2;
pub mod pl011;
pub mod timer;

pub use gicv2::{GicV2, GIC, SPURIOUS_IRQ};
pub use pl011::Pl011Uart;
pub use timer::{TIMER_IRQ, DEFAULT_TICK_HZ};
