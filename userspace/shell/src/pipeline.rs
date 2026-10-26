//! Ind OS Shell Pipeline Engine (IPC Chains)
//!
//! Standards:
//! - INV-SH-002: Pipelines as capability-mediated IPC chains

use crate::commands::{BuiltinCommands, CommandContext};
use crate::types::{CommandInvocation, ShellError, MAX_OUTPUT_BUFFER};

pub struct PipelineEngine;

impl PipelineEngine {
    pub fn execute_pipeline(
        stages: &[CommandInvocation],
        stage_count: usize,
        final_output: &mut [u8],
    ) -> Result<usize, ShellError> {
        if stage_count == 0 {
            return Ok(0);
        }

        let mut prev_buf = [0u8; MAX_OUTPUT_BUFFER];
        let mut prev_len = 0;
        let mut has_prev = false;

        for (i, cmd) in stages.iter().take(stage_count).enumerate() {
            let is_last = i == stage_count - 1;
            if is_last {
                let stdin_str = if has_prev {
                    Some(core::str::from_utf8(&prev_buf[..prev_len]).unwrap_or(""))
                } else {
                    None
                };
                let mut ctx = CommandContext::new(final_output, stdin_str);
                BuiltinCommands::execute(cmd, &mut ctx)?;
                return Ok(ctx.stdout_len);
            } else {
                let mut current_buf = [0u8; MAX_OUTPUT_BUFFER];
                let out_len = {
                    let stdin_str = if has_prev {
                        Some(core::str::from_utf8(&prev_buf[..prev_len]).unwrap_or(""))
                    } else {
                        None
                    };
                    let mut ctx = CommandContext::new(&mut current_buf, stdin_str);
                    BuiltinCommands::execute(cmd, &mut ctx)?;
                    ctx.stdout_len
                };
                prev_buf[..out_len].copy_from_slice(&current_buf[..out_len]);
                prev_len = out_len;
                has_prev = true;
            }
        }

        Ok(0)
    }
}
