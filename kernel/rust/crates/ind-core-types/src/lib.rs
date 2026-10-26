//! # Ind OS Core Types
//!
//! Core primitives for process isolation, security tiers, and cryptographic session IDs.
//! Designed for `#![no_std]` compatibility to support bare-metal microkernel execution.

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum ActionTier {
    /// Safe, read-only actions (Torch, Clock, Weather). Ambiently executable.
    Green = 0,
    /// Moderately sensitive actions (SMS, Launch App). Requires user voice confirmation.
    Amber = 1,
    /// High-risk / financial actions (UPI, Settings Wipe, System Modify). Requires physical biometric presence.
    Red = 2,
}

impl ActionTier {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Green => "GREEN",
            Self::Amber => "AMBER",
            Self::Red => "RED",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct SessionId(pub [u8; 32]);

impl SessionId {
    pub const fn zero() -> Self {
        Self([0u8; 32])
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DenialCode {
    InvalidSchema,
    TaintedInput,
    MissingVoiceConfirmation,
    MissingBiometricProof,
    AirGapViolation,
    UnauthorizedCapability,
}
