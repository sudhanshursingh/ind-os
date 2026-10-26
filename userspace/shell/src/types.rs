//! Ind OS Shell Core Types
//!
//! Standards:
//! - INV-SH-001: Bounded panic-free parsing
//! - INV-SH-002: Pipelines as capability-mediated IPC chains

pub const MAX_ARGS: usize = 16;
pub const MAX_PIPELINE_STAGES: usize = 4;
pub const MAX_OUTPUT_BUFFER: usize = 512;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseError {
    EmptyLine,
    LineTooLong,
    TooManyArguments,
    UnclosedQuote,
    InvalidPipelineSyntax,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShellError {
    CommandNotFound,
    ExecutionFailed,
    PipelineError,
    PermissionDenied,
    BufferOverflow,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CommandInvocation<'a> {
    pub name: &'a str,
    pub args: [&'a str; MAX_ARGS],
    pub arg_count: usize,
}

impl<'a> CommandInvocation<'a> {
    pub const fn empty() -> Self {
        Self {
            name: "",
            args: [""; MAX_ARGS],
            arg_count: 0,
        }
    }
}
