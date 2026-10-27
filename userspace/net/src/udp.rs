//! UDP Datagram Parser & Serializer (SP-51, REQ-KRN-110, INV-NET-002)

use crate::types::NetError;

pub const UDP_HEADER_LEN: usize = 8;

#[derive(Debug, Clone, Copy)]
pub struct UdpHeader {
    pub src_port: u16,
    pub dst_port: u16,
    pub length: u16,
    pub checksum: u16,
}

impl UdpHeader {
    pub fn parse(buf: &[u8]) -> Result<(Self, usize), NetError> {
        if buf.len() < UDP_HEADER_LEN {
            return Err(NetError::BufferTooShort);
        }

        let src_port = ((buf[0] as u16) << 8) | (buf[1] as u16);
        let dst_port = ((buf[2] as u16) << 8) | (buf[3] as u16);
        let length = ((buf[4] as u16) << 8) | (buf[5] as u16);
        let checksum = ((buf[6] as u16) << 8) | (buf[7] as u16);

        Ok((
            Self {
                src_port,
                dst_port,
                length,
                checksum,
            },
            UDP_HEADER_LEN,
        ))
    }

    pub fn serialize(&self, buf: &mut [u8]) -> Result<usize, NetError> {
        if buf.len() < UDP_HEADER_LEN {
            return Err(NetError::BufferTooShort);
        }

        buf[0] = (self.src_port >> 8) as u8;
        buf[1] = (self.src_port & 0xFF) as u8;
        buf[2] = (self.dst_port >> 8) as u8;
        buf[3] = (self.dst_port & 0xFF) as u8;
        buf[4] = (self.length >> 8) as u8;
        buf[5] = (self.length & 0xFF) as u8;
        buf[6] = (self.checksum >> 8) as u8;
        buf[7] = (self.checksum & 0xFF) as u8;

        Ok(UDP_HEADER_LEN)
    }
}
