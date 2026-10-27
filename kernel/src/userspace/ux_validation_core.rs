//! Kernel UX Validation Verification Harness (SP-68, REQ-UX-305)
//!
//! Architectural Invariants:
//! - INV-UXVAL-001: 100% completion rate across multi-lingual scripted persona tasks
//! - INV-UXVAL-002: Error recovery and prompt clarity across Hindi, Tamil, English personas
//! - INV-UXVAL-003: Zero unaddressed P0/P1 blockers in Top-10 friction list

pub const KERNEL_MAX_PERSONA_TASKS: usize = 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KernelUxValidationError {
    TaskFailed,
    PersonaNotFound,
    TableFull,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KernelPersonaTaskResult {
    pub persona_name: &'static str,
    pub language: &'static str,
    pub task_name: &'static str,
    pub completed: bool,
    pub latency_ms: u32,
}

pub struct KernelUxValidationHarness {
    results: [Option<KernelPersonaTaskResult>; KERNEL_MAX_PERSONA_TASKS],
    task_count: usize,
}

impl KernelUxValidationHarness {
    pub const fn new() -> Self {
        Self {
            results: [None; KERNEL_MAX_PERSONA_TASKS],
            task_count: 0,
        }
    }

    pub fn record_task_result(
        &mut self,
        persona_name: &'static str,
        language: &'static str,
        task_name: &'static str,
        completed: bool,
        latency_ms: u32,
    ) -> Result<(), KernelUxValidationError> {
        for slot in self.results.iter_mut() {
            if slot.is_none() {
                *slot = Some(KernelPersonaTaskResult {
                    persona_name,
                    language,
                    task_name,
                    completed,
                    latency_ms,
                });
                self.task_count += 1;
                return Ok(());
            }
        }
        Err(KernelUxValidationError::TableFull)
    }

    pub fn success_rate_percent(&self) -> u32 {
        if self.task_count == 0 {
            return 0;
        }
        let mut success = 0;
        for slot in self.results.iter() {
            if let Some(ref r) = slot {
                if r.completed {
                    success += 1;
                }
            }
        }
        (success * 100) / (self.task_count as u32)
    }

    pub fn task_count(&self) -> usize {
        self.task_count
    }
}
