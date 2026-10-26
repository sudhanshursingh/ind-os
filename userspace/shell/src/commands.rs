//! Ind OS Built-in Shell Commands
//!
//! Standards:
//! - REQ-USE-201, REQ-USE-202, REQ-USE-203
//! - INV-SH-001: Panic-free execution

use crate::types::{CommandInvocation, ShellError};

pub struct CommandContext<'a> {
    pub stdout: &'a mut [u8],
    pub stdout_len: usize,
    pub stdin_data: Option<&'a str>,
}

impl<'a> CommandContext<'a> {
    pub fn new(stdout: &'a mut [u8], stdin_data: Option<&'a str>) -> Self {
        Self {
            stdout,
            stdout_len: 0,
            stdin_data,
        }
    }

    pub fn write_str(&mut self, text: &str) -> Result<(), ShellError> {
        let bytes = text.as_bytes();
        if self.stdout_len + bytes.len() > self.stdout.len() {
            return Err(ShellError::BufferOverflow);
        }
        self.stdout[self.stdout_len..self.stdout_len + bytes.len()].copy_from_slice(bytes);
        self.stdout_len += bytes.len();
        Ok(())
    }

    pub fn output_as_str(&self) -> &str {
        core::str::from_utf8(&self.stdout[..self.stdout_len]).unwrap_or("")
    }
}

pub struct BuiltinCommands;

impl BuiltinCommands {
    pub fn execute(cmd: &CommandInvocation, ctx: &mut CommandContext) -> Result<(), ShellError> {
        match cmd.name {
            "help" => Self::cmd_help(ctx),
            "version" => Self::cmd_version(ctx),
            "ps" => Self::cmd_ps(ctx),
            "ls" => Self::cmd_ls(cmd, ctx),
            "cat" => Self::cmd_cat(cmd, ctx),
            "echo" => Self::cmd_echo(cmd, ctx),
            "caps" => Self::cmd_caps(ctx),
            "clear" => Self::cmd_clear(ctx),
            _ => Err(ShellError::CommandNotFound),
        }
    }

    fn cmd_help(ctx: &mut CommandContext) -> Result<(), ShellError> {
        ctx.write_str("Ind OS Shell v1 Builtins: help, version, ps, ls, cat, echo, caps, clear\n")
    }

    fn cmd_version(ctx: &mut CommandContext) -> Result<(), ShellError> {
        ctx.write_str("Ind OS v0.1.0-dogfood (aarch64 microkernel, pure Rust, capability-first)\n")
    }

    fn cmd_ps(ctx: &mut CommandContext) -> Result<(), ShellError> {
        ctx.write_str("PID  STATE    RUNLEVEL       SERVICE\n")?;
        ctx.write_str("1    RUNNING  CoreSecurity   auditd\n")?;
        ctx.write_str("2    RUNNING  StorageDrivers vfs-indfs\n")?;
        ctx.write_str("3    RUNNING  SystemServices devmgr\n")?;
        ctx.write_str("4    RUNNING  Interactive    shell\n")
    }

    fn cmd_ls(cmd: &CommandInvocation, ctx: &mut CommandContext) -> Result<(), ShellError> {
        let path = if cmd.arg_count > 0 { cmd.args[0] } else { "/" };
        if path.starts_with("/sys") {
            ctx.write_str("manifest  devices  capabilities  telemetry\n")
        } else {
            ctx.write_str("bin/  etc/  sys/  tmp/  var/\n")
        }
    }

    fn cmd_cat(cmd: &CommandInvocation, ctx: &mut CommandContext) -> Result<(), ShellError> {
        if let Some(stdin) = ctx.stdin_data {
            return ctx.write_str(stdin);
        }
        if cmd.arg_count == 0 {
            return ctx.write_str("(empty)\n");
        }
        let target = cmd.args[0];
        if target == "/sys/manifest" || target == "manifest" {
            ctx.write_str("service: devmgr\nrunlevel: SystemServices\ncapabilities: [MMIO, IRQ, DMA]\nrestart: OnFailure\n")
        } else {
            ctx.write_str("[cat: file content streamed via capability channel]\n")
        }
    }

    fn cmd_echo(cmd: &CommandInvocation, ctx: &mut CommandContext) -> Result<(), ShellError> {
        for i in 0..cmd.arg_count {
            if i > 0 {
                ctx.write_str(" ")?;
            }
            ctx.write_str(cmd.args[i])?;
        }
        ctx.write_str("\n")
    }

    fn cmd_caps(ctx: &mut CommandContext) -> Result<(), ShellError> {
        ctx.write_str("CAP 101: READ+WRITE (UART Terminal Stream)\n")?;
        ctx.write_str("CAP 102: READ (Process Table Query)\n")?;
        ctx.write_str("CAP 103: READ (IndFS Read-Only VFS Subtree)\n")
    }

    fn cmd_clear(ctx: &mut CommandContext) -> Result<(), ShellError> {
        ctx.write_str("\x1b[2J\x1b[H")
    }
}
