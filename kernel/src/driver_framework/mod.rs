//! User-Mode Driver Framework Subsystem for IndOS
//! Compliant with Bible §8.10 (SP-26), INDOS-SPEC-DRV-01, Rule N-01, Rule N-04, and REQ-KRN-107.

pub mod device_manager;
pub mod dma_broker;
pub mod irq_broker;
pub mod protocol;
pub mod reference;

pub use device_manager::{DeviceDescriptor, DeviceManager, DeviceState, DeviceType, MAX_DEVICES};
pub use dma_broker::{DmaBroker, DmaDirection, DmaWindow, MAX_DMA_WINDOWS, MAX_DMA_WINDOW_SIZE};
pub use irq_broker::{IrqBroker, IrqState, IrqSubscription, MAX_IRQS};
pub use protocol::{DriverMessage, DriverOp, DriverStatus};
pub use reference::{FramebufferDriver, InputDriver, InputEvent, InputEventType, VirtioBlkDriver, VirtioNetDriver};

use crate::sync::Spinlock;

/// Global Device Manager singleton
pub static DEVICE_MANAGER: Spinlock<DeviceManager> = Spinlock::new(DeviceManager::new());

/// Global IRQ Broker singleton
pub static IRQ_BROKER: Spinlock<IrqBroker> = Spinlock::new(IrqBroker::new());

/// Global DMA Broker singleton
pub static DMA_BROKER: Spinlock<DmaBroker> = Spinlock::new(DmaBroker::new());
