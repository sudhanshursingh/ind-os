//! Ind OS Integration Test Suite Types
//!
//! Standards:
//! - REQ-QA-201 Groundwork
//! - INV-QA-001: Deterministic user journey execution
//! - INV-QA-002: Zero-leak soak cycling

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JourneyStatus {
    Passed,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct JourneyReport {
    pub journey_id: u32,
    pub name: &'static str,
    pub status: JourneyStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SoakSummary {
    pub completed_cycles: u32,
    pub target_cycles: u32,
    pub leak_count: u32,
    pub passed: bool,
}
