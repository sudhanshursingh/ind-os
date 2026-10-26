//! IndOS Standard Driver Protocol over IPC
//! Compliant with Bible §8.10 (SP-26), INDOS-SPEC-DRV-01, Rule N-01, Rule N-04, and REQ-KRN-107.

use crate::ipc::IpcMessage;

/// Driver Protocol Operation Codes
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DriverOp {
    Register = 0x01,
    Bind = 0x02,
    IrqNotify = 0x03,
    IrqAck = 0x04,
    DmaMap = 0x05,
    DmaUnmap = 0x06,
    IoRead = 0x07,
    IoWrite = 0x08,
    Status = 0x09,
    Unknown = 0xFF,
}

impl DriverOp {
    pub fn from_u64(val: u64) -> Self {
        match val {
            0x01 => Self::Register,
            0x02 => Self::Bind,
            0x03 => Self::IrqNotify,
            0x04 => Self::IrqAck,
            0x05 => Self::DmaMap,
            0x06 => Self::DmaUnmap,
            0x07 => Self::IoRead,
            0x08 => Self::IoWrite,
            0x09 => Self::Status,
            _ => Self::Unknown,
        }
    }

    pub fn to_u64(&self) -> u64 {
        *self as u64
    }
}

/// Driver Protocol Status Codes
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DriverStatus {
    Ok = 0,
    Eperm = 1,
    Einval = 2,
    Ebusy = 3,
    Eio = 4,
    Enodev = 5,
}

impl DriverStatus {
    pub fn from_u64(val: u64) -> Self {
        match val {
            0 => Self::Ok,
            1 => Self::Eperm,
            2 => Self::Einval,
            3 => Self::Ebusy,
            4 => Self::Eio,
            _ => Self::Enodev,
        }
    }

    pub fn to_u64(&self) -> u64 {
        *self as u64
    }
}

/// Typed Driver Protocol Message multiplexed over register-based IpcMessage
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DriverMessage {
    pub op: DriverOp,
    pub correlation_id: u64,
    pub target_id: u64, // device_id, irq_line, or window_id
    pub arg1: u64,      // offset, length, or vendor_id
    pub arg2: u64,      // flags, status, or data payload
}

impl DriverMessage {
    pub const fn new(op: DriverOp, correlation_id: u64, target_id: u64, arg1: u64, arg2: u64) -> Self {
        Self {
            op,
            correlation_id,
            target_id,
            arg1,
            arg2,
        }
    }

    /// Encodes a driver protocol message into a microkernel IpcMessage
    pub fn to_ipc_message(&self) -> IpcMessage {
        IpcMessage::new(
            self.op.to_u64(),
            [self.correlation_id, self.target_id, self.arg1, self.arg2],
            None,
        )
    }

    /// Decodes a microkernel IpcMessage into a driver protocol message
    pub fn from_ipc_message(msg: &IpcMessage) -> Self {
        Self {
            op: DriverOp::from_u64(msg.label),
            correlation_id: msg.data[0],
            target_id: msg.data[1],
            arg1: msg.data[2],
            arg2: msg.data[3],
        }
    }

    /// Helper constructor for IRQ notification
    pub fn irq_notification(irq_line: u32, delivery_seq: u64) -> Self {
        Self::new(DriverOp::IrqNotify, delivery_seq, irq_line as u64, 0, 0)
    }

    /// Helper constructor for IRQ acknowledgment
    pub fn irq_ack(irq_line: u32, delivery_seq: u64) -> Self {
        Self::new(DriverOp::IrqAck, delivery_seq, irq_line as u64, 0, DriverStatus::Ok.to_u64())
    }

    /// Helper constructor for I/O response
    pub fn io_response(correlation_id: u64, status: DriverStatus, bytes_transferred: usize) -> Self {
        Self::new(DriverOp::Status, correlation_id, status.to_u64(), bytes_transferred as u64, 0)
    }
}
