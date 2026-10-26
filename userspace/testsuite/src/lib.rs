//! Ind OS Integration Test Suite Library (`indus-testsuite`)
//!
//! Standards:
//! - REQ-QA-201 Groundwork
//! - INV-QA-001: Deterministic user journey execution
//! - INV-QA-002: Zero-leak soak cycling

#![no_std]

pub mod journeys;
pub mod soak;
pub mod types;

pub use journeys::UserJourneyRunner;
pub use soak::SoakHarness;
pub use types::{JourneyReport, JourneyStatus, SoakSummary};
