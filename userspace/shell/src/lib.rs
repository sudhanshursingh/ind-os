//! Ind OS Userspace Shell Library (`indus-shell`)
//!
//! Standards:
//! - REQ-USE-201, REQ-USE-202, REQ-USE-203
//! - INV-SH-001: Bounded panic-free parsing
//! - INV-SH-002: Pipelines as capability-mediated IPC chains

#![no_std]

pub mod commands;
pub mod parser;
pub mod pipeline;
pub mod types;

pub use commands::{BuiltinCommands, CommandContext};
pub use parser::ShellParser;
pub use pipeline::PipelineEngine;
pub use types::{CommandInvocation, ParseError, ShellError, MAX_ARGS, MAX_OUTPUT_BUFFER, MAX_PIPELINE_STAGES};

pub struct ShellSession;

impl ShellSession {
    pub fn execute_line<'a>(line: &'a str, output: &mut [u8]) -> Result<usize, ShellError> {
        let mut stages = [
            CommandInvocation::empty(),
            CommandInvocation::empty(),
            CommandInvocation::empty(),
            CommandInvocation::empty(),
        ];
        let count = ShellParser::parse_pipeline(line, &mut stages)
            .map_err(|_| ShellError::PipelineError)?;

        PipelineEngine::execute_pipeline(&stages, count, output)
    }
}
