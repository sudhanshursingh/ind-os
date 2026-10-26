//! Reference User-Mode Drivers for IndOS
//! Compliant with Bible §8.10 (SP-27), INDOS-SPEC-REFDRV-01, Rule N-01, Rule N-04, and REQ-KRN-107.

pub mod framebuffer;
pub mod input;
pub mod virtio_blk;
pub mod virtio_net;

pub use framebuffer::FramebufferDriver;
pub use input::{InputDriver, InputEvent, InputEventType, MAX_INPUT_EVENTS};
pub use virtio_blk::{VirtioBlkDriver, SECTOR_SIZE};
pub use virtio_net::{VirtioNetDriver, DEFAULT_MTU};
