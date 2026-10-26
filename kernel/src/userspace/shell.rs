//! Kernel Shell Verification Harness (SP-35)
//!
//! Architectural Invariants:
//! - REQ-USE-201, REQ-USE-202, REQ-USE-203: Interactive Shell v1
//! - INV-SH-001: Bounded panic-free parsing
//! - INV-SH-002: Pipelines as capability-mediated IPC chains

pub const KERNEL_MAX_SHELL_ARGS: usize = 8;
pub const KERNEL_MAX_PIPELINE_STAGES: usize = 4;
pub const KERNEL_SHELL_OUTPUT_BUF_SIZE: usize = 512;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KernelShellParseError {
    EmptyLine,
    TooManyArguments,
    UnclosedQuote,
    InvalidPipelineSyntax,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KernelShellExecError {
    CommandNotFound,
    BufferOverflow,
    PipelineFailed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KernelCommandInvocation<'a> {
    pub name: &'a str,
    pub args: [&'a str; KERNEL_MAX_SHELL_ARGS],
    pub arg_count: usize,
}

impl<'a> KernelCommandInvocation<'a> {
    pub const fn empty() -> Self {
        Self {
            name: "",
            args: [""; KERNEL_MAX_SHELL_ARGS],
            arg_count: 0,
        }
    }
}

pub struct KernelShellParser;

impl KernelShellParser {
    pub fn parse_command<'a>(input: &'a str) -> Result<KernelCommandInvocation<'a>, KernelShellParseError> {
        let trimmed = input.trim();
        if trimmed.is_empty() {
            return Err(KernelShellParseError::EmptyLine);
        }

        let mut inv = KernelCommandInvocation::empty();
        let bytes = trimmed.as_bytes();
        let len = bytes.len();
        let mut idx = 0;
        let mut in_name = true;

        while idx < len {
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
                    return Err(KernelShellParseError::UnclosedQuote);
                }
                end = idx;
                idx += 1;
            } else {
                start = idx;
                while idx < len && bytes[idx] != b' ' && bytes[idx] != b'\t' && bytes[idx] != b'|' {
                    idx += 1;
                }
                end = idx;
            }

            let token = &trimmed[start..end];
            if in_name {
                inv.name = token;
                in_name = false;
            } else {
                if inv.arg_count >= KERNEL_MAX_SHELL_ARGS {
                    return Err(KernelShellParseError::TooManyArguments);
                }
                inv.args[inv.arg_count] = token;
                inv.arg_count += 1;
            }
        }

        if inv.name.is_empty() {
            return Err(KernelShellParseError::EmptyLine);
        }

        Ok(inv)
    }

    pub fn parse_pipeline<'a>(
        input: &'a str,
        stages: &mut [KernelCommandInvocation<'a>; KERNEL_MAX_PIPELINE_STAGES],
    ) -> Result<usize, KernelShellParseError> {
        let trimmed = input.trim();
        if trimmed.is_empty() {
            return Err(KernelShellParseError::EmptyLine);
        }

        let mut count = 0;
        let mut start = 0;
        let bytes = trimmed.as_bytes();
        let len = bytes.len();

        for i in 0..=len {
            if i == len || bytes[i] == b'|' {
                let part = trimmed[start..i].trim();
                if part.is_empty() {
                    return Err(KernelShellParseError::InvalidPipelineSyntax);
                }
                if count >= KERNEL_MAX_PIPELINE_STAGES {
                    return Err(KernelShellParseError::TooManyArguments);
                }
                stages[count] = Self::parse_command(part)?;
                count += 1;
                start = i + 1;
            }
        }

        Ok(count)
    }
}

pub struct KernelShellSession;

impl KernelShellSession {
    pub fn execute_builtin(
        cmd: &KernelCommandInvocation,
        stdin_str: Option<&str>,
        stdout: &mut [u8],
    ) -> Result<usize, KernelShellExecError> {
        let mut out_len = 0;
        let write_str = |text: &str, out: &mut [u8], len: &mut usize| -> Result<(), KernelShellExecError> {
            let b = text.as_bytes();
            if *len + b.len() > out.len() {
                return Err(KernelShellExecError::BufferOverflow);
            }
            out[*len..*len + b.len()].copy_from_slice(b);
            *len += b.len();
            Ok(())
        };

        match cmd.name {
            "help" => {
                write_str("Ind OS Shell v1: help, version, ps, ls, cat, echo, caps, clear\n", stdout, &mut out_len)?;
            }
            "version" => {
                write_str("Ind OS v0.1.0-dogfood (aarch64 microkernel)\n", stdout, &mut out_len)?;
            }
            "ps" => {
                write_str("PID  SERVICE\n1    auditd\n2    vfs-indfs\n3    devmgr\n4    shell\n", stdout, &mut out_len)?;
            }
            "ls" => {
                let path = if cmd.arg_count > 0 { cmd.args[0] } else { "/" };
                if path.starts_with("/sys") {
                    write_str("manifest  devices  capabilities  telemetry\n", stdout, &mut out_len)?;
                } else {
                    write_str("bin/  etc/  sys/  tmp/\n", stdout, &mut out_len)?;
                }
            }
            "cat" => {
                if let Some(stdin) = stdin_str {
                    write_str(stdin, stdout, &mut out_len)?;
                } else if cmd.arg_count > 0 && (cmd.args[0] == "/sys/manifest" || cmd.args[0] == "manifest") {
                    write_str("service: devmgr\nrunlevel: SystemServices\ncapabilities: [MMIO, IRQ, DMA]\n", stdout, &mut out_len)?;
                } else {
                    write_str("(empty)\n", stdout, &mut out_len)?;
                }
            }
            "echo" => {
                for i in 0..cmd.arg_count {
                    if i > 0 {
                        write_str(" ", stdout, &mut out_len)?;
                    }
                    write_str(cmd.args[i], stdout, &mut out_len)?;
                }
                write_str("\n", stdout, &mut out_len)?;
            }
            "caps" => {
                write_str("CAP 101: READ+WRITE (Terminal)\nCAP 102: READ (Process)\nCAP 103: READ (VFS)\n", stdout, &mut out_len)?;
            }
            "clear" => {
                write_str("\x1b[2J\x1b[H", stdout, &mut out_len)?;
            }
            _ => return Err(KernelShellExecError::CommandNotFound),
        }

        Ok(out_len)
    }

    pub fn execute_pipeline(line: &str, final_output: &mut [u8]) -> Result<usize, KernelShellExecError> {
        let mut stages = [
            KernelCommandInvocation::empty(),
            KernelCommandInvocation::empty(),
            KernelCommandInvocation::empty(),
            KernelCommandInvocation::empty(),
        ];

        let count = KernelShellParser::parse_pipeline(line, &mut stages)
            .map_err(|_| KernelShellExecError::PipelineFailed)?;

        if count == 0 {
            return Ok(0);
        }

        let mut prev_buf = [0u8; KERNEL_SHELL_OUTPUT_BUF_SIZE];
        let mut prev_len = 0;
        let mut has_prev = false;

        for (i, cmd) in stages.iter().take(count).enumerate() {
            let is_last = i == count - 1;
            if is_last {
                let stdin = if has_prev {
                    Some(core::str::from_utf8(&prev_buf[..prev_len]).unwrap_or(""))
                } else {
                    None
                };
                return Self::execute_builtin(cmd, stdin, final_output);
            } else {
                let mut current_buf = [0u8; KERNEL_SHELL_OUTPUT_BUF_SIZE];
                let out_len = {
                    let stdin = if has_prev {
                        Some(core::str::from_utf8(&prev_buf[..prev_len]).unwrap_or(""))
                    } else {
                        None
                    };
                    Self::execute_builtin(cmd, stdin, &mut current_buf)?
                };
                prev_buf[..out_len].copy_from_slice(&current_buf[..out_len]);
                prev_len = out_len;
                has_prev = true;
            }
        }

        Ok(0)
    }
}
