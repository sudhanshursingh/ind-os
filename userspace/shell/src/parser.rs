//! Ind OS Shell Command-Line Parser
//!
//! Standards:
//! - INV-SH-001: Bounded panic-free parsing with zero heap allocation

use crate::types::{CommandInvocation, ParseError, MAX_ARGS, MAX_PIPELINE_STAGES};

pub struct ShellParser;

impl ShellParser {
    pub fn parse_command<'a>(input: &'a str) -> Result<CommandInvocation<'a>, ParseError> {
        let trimmed = input.trim();
        if trimmed.is_empty() {
            return Err(ParseError::EmptyLine);
        }

        let mut invocation = CommandInvocation::empty();
        let bytes = trimmed.as_bytes();
        let len = bytes.len();
        let mut idx = 0;
        let mut in_name = true;

        while idx < len {
            // Skip leading whitespace
            while idx < len && (bytes[idx] == b' ' || bytes[idx] == b'\t') {
                idx += 1;
            }
            if idx >= len {
                break;
            }

            let start: usize;
            let end: usize;

            if bytes[idx] == b'"' || bytes[idx] == b'\'' {
                let quote = bytes[idx];
                idx += 1;
                start = idx;
                while idx < len && bytes[idx] != quote {
                    idx += 1;
                }
                if idx >= len {
                    return Err(ParseError::UnclosedQuote);
                }
                end = idx;
                idx += 1; // skip closing quote
            } else {
                start = idx;
                while idx < len && bytes[idx] != b' ' && bytes[idx] != b'\t' && bytes[idx] != b'|' {
                    idx += 1;
                }
                end = idx;
            }

            let token = &trimmed[start..end];
            if in_name {
                invocation.name = token;
                in_name = false;
            } else {
                if invocation.arg_count >= MAX_ARGS {
                    return Err(ParseError::TooManyArguments);
                }
                invocation.args[invocation.arg_count] = token;
                invocation.arg_count += 1;
            }
        }

        if invocation.name.is_empty() {
            return Err(ParseError::EmptyLine);
        }

        Ok(invocation)
    }

    pub fn parse_pipeline<'a>(
        input: &'a str,
        stages: &mut [CommandInvocation<'a>; MAX_PIPELINE_STAGES],
    ) -> Result<usize, ParseError> {
        let trimmed = input.trim();
        if trimmed.is_empty() {
            return Err(ParseError::EmptyLine);
        }

        let mut count = 0;
        let mut start = 0;
        let bytes = trimmed.as_bytes();
        let len = bytes.len();

        for i in 0..=len {
            if i == len || bytes[i] == b'|' {
                let part = trimmed[start..i].trim();
                if part.is_empty() {
                    return Err(ParseError::InvalidPipelineSyntax);
                }
                if count >= MAX_PIPELINE_STAGES {
                    return Err(ParseError::TooManyArguments);
                }
                stages[count] = Self::parse_command(part)?;
                count += 1;
                start = i + 1;
            }
        }

        Ok(count)
    }
}
