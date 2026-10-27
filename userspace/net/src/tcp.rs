//! TCP Segment Parser, Flags & Finite State Machine (SP-51, REQ-KRN-110, INV-NET-002)

use crate::types::{NetError, TcpState};

pub const TCP_MIN_HEADER_LEN: usize = 20;

pub const TCP_FLAG_FIN: u8 = 0x01;
pub const TCP_FLAG_SYN: u8 = 0x02;
pub const TCP_FLAG_RST: u8 = 0x04;
pub const TCP_FLAG_PSH: u8 = 0x08;
pub const TCP_FLAG_ACK: u8 = 0x10;
pub const TCP_FLAG_URG: u8 = 0x20;

#[derive(Debug, Clone, Copy)]
pub struct TcpHeader {
    pub src_port: u16,
    pub dst_port: u16,
    pub seq_num: u32,
    pub ack_num: u32,
    pub data_offset: u8,
    pub flags: u8,
    pub window_size: u16,
    pub checksum: u16,
    pub urgent_ptr: u16,
}

impl TcpHeader {
    pub fn is_syn(&self) -> bool {
        (self.flags & TCP_FLAG_SYN) != 0
    }

    pub fn is_ack(&self) -> bool {
        (self.flags & TCP_FLAG_ACK) != 0
    }

    pub fn is_fin(&self) -> bool {
        (self.flags & TCP_FLAG_FIN) != 0
    }

    pub fn is_rst(&self) -> bool {
        (self.flags & TCP_FLAG_RST) != 0
    }

    pub fn parse(buf: &[u8]) -> Result<(Self, usize), NetError> {
        if buf.len() < TCP_MIN_HEADER_LEN {
            return Err(NetError::BufferTooShort);
        }

        let src_port = ((buf[0] as u16) << 8) | (buf[1] as u16);
        let dst_port = ((buf[2] as u16) << 8) | (buf[3] as u16);
        let seq_num = ((buf[4] as u32) << 24)
            | ((buf[5] as u32) << 16)
            | ((buf[6] as u32) << 8)
            | (buf[7] as u32);
        let ack_num = ((buf[8] as u32) << 24)
            | ((buf[9] as u32) << 16)
            | ((buf[10] as u32) << 8)
            | (buf[11] as u32);

        let data_offset = buf[12] >> 4;
        let header_len = (data_offset as usize) * 4;
        if header_len < TCP_MIN_HEADER_LEN || buf.len() < header_len {
            return Err(NetError::InvalidHeaderLength);
        }

        let flags = buf[13];
        let window_size = ((buf[14] as u16) << 8) | (buf[15] as u16);
        let checksum = ((buf[16] as u16) << 8) | (buf[17] as u16);
        let urgent_ptr = ((buf[18] as u16) << 8) | (buf[19] as u16);

        Ok((
            Self {
                src_port,
                dst_port,
                seq_num,
                ack_num,
                data_offset,
                flags,
                window_size,
                checksum,
                urgent_ptr,
            },
            header_len,
        ))
    }

    pub fn serialize(&self, buf: &mut [u8]) -> Result<usize, NetError> {
        let header_len = TCP_MIN_HEADER_LEN;
        if buf.len() < header_len {
            return Err(NetError::BufferTooShort);
        }

        buf[0] = (self.src_port >> 8) as u8;
        buf[1] = (self.src_port & 0xFF) as u8;
        buf[2] = (self.dst_port >> 8) as u8;
        buf[3] = (self.dst_port & 0xFF) as u8;

        buf[4] = (self.seq_num >> 24) as u8;
        buf[5] = (self.seq_num >> 16) as u8;
        buf[6] = (self.seq_num >> 8) as u8;
        buf[7] = (self.seq_num & 0xFF) as u8;

        buf[8] = (self.ack_num >> 24) as u8;
        buf[9] = (self.ack_num >> 16) as u8;
        buf[10] = (self.ack_num >> 8) as u8;
        buf[11] = (self.ack_num & 0xFF) as u8;

        buf[12] = 5 << 4; // 20 bytes = 5 32-bit words
        buf[13] = self.flags;
        buf[14] = (self.window_size >> 8) as u8;
        buf[15] = (self.window_size & 0xFF) as u8;
        buf[16] = (self.checksum >> 8) as u8;
        buf[17] = (self.checksum & 0xFF) as u8;
        buf[18] = (self.urgent_ptr >> 8) as u8;
        buf[19] = (self.urgent_ptr & 0xFF) as u8;

        Ok(header_len)
    }
}

/// TCP Connection State Machine Transition Engine
pub struct TcpStateMachine;

impl TcpStateMachine {
    /// Process incoming segment flags and transition socket state
    pub fn step(
        current_state: TcpState,
        flags: u8,
        is_active_opener: bool,
    ) -> Result<TcpState, NetError> {
        let has_syn = (flags & TCP_FLAG_SYN) != 0;
        let has_ack = (flags & TCP_FLAG_ACK) != 0;
        let has_fin = (flags & TCP_FLAG_FIN) != 0;
        let has_rst = (flags & TCP_FLAG_RST) != 0;

        if has_rst {
            return Ok(TcpState::Closed);
        }

        match current_state {
            TcpState::Closed => {
                if has_syn && is_active_opener {
                    Ok(TcpState::SynSent)
                } else {
                    Err(NetError::StateMismatch)
                }
            }
            TcpState::Listen => {
                if has_syn && !has_ack {
                    Ok(TcpState::SynReceived)
                } else {
                    Err(NetError::StateMismatch)
                }
            }
            TcpState::SynSent => {
                if has_syn && has_ack {
                    Ok(TcpState::Established)
                } else if has_syn {
                    Ok(TcpState::SynReceived) // Simultaneous open
                } else {
                    Err(NetError::StateMismatch)
                }
            }
            TcpState::SynReceived => {
                if has_ack {
                    Ok(TcpState::Established)
                } else {
                    Err(NetError::StateMismatch)
                }
            }
            TcpState::Established => {
                if has_fin {
                    Ok(TcpState::CloseWait)
                } else {
                    Ok(TcpState::Established)
                }
            }
            TcpState::FinWait1 => {
                if has_fin && has_ack {
                    Ok(TcpState::TimeWait)
                } else if has_ack {
                    Ok(TcpState::FinWait2)
                } else if has_fin {
                    Ok(TcpState::Closing)
                } else {
                    Err(NetError::StateMismatch)
                }
            }
            TcpState::FinWait2 => {
                if has_fin {
                    Ok(TcpState::TimeWait)
                } else {
                    Err(NetError::StateMismatch)
                }
            }
            TcpState::CloseWait => {
                // Application closes -> sends FIN -> LastAck
                Ok(TcpState::LastAck)
            }
            TcpState::Closing => {
                if has_ack {
                    Ok(TcpState::TimeWait)
                } else {
                    Err(NetError::StateMismatch)
                }
            }
            TcpState::LastAck => {
                if has_ack {
                    Ok(TcpState::Closed)
                } else {
                    Err(NetError::StateMismatch)
                }
            }
            TcpState::TimeWait => {
                // 2MSL timeout passes
                Ok(TcpState::Closed)
            }
        }
    }
}
