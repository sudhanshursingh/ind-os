//! Platform Backends for Ind OS Hardware Abstraction Layer (HAL)
//!
//! Constitutional References:
//! - REQ-HAL-102: At least two backends: QEMU virt (first) and hanoip (second)
//! - REQ-HAL-103: Backend-specific code isolated behind compile-time modules

pub mod qemu_virt;
pub mod hanoip;
