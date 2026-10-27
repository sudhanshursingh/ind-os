//! IPv4 Packet Parser, Serializer & Checksum (SP-51, REQ-KRN-110, INV-NET-002)

use crate::types::{Ipv4Address, IpProtocol, NetError};

pub const IPV4_MIN_HEADER_LEN: usize = 20;

#[derive(Debug, Clone, Copy)]
pub struct Ipv4Header {
    pub ihl: u8,
    pub tos: u8,
    pub total_length: u16,
    pub identification: u16,
    pub flags_and_offset: u16,
    pub ttl: u8,
    pub protocol: IpProtocol,
    pub checksum: u16,
    pub src_ip: Ipv4Address,
    pub dst_ip: Ipv4Address,
}

impl Ipv4Header {
    pub fn compute_checksum(header_bytes: &[u8]) -> u16 {
        let mut sum: u32 = 0;
        let mut i = 0;
        let len = header_bytes.len();

        while i + 1 < len {
            let word = ((header_bytes[i] as u32) << 8) | (header_bytes[i + 1] as u32);
            sum = sum.wrapping_add(word);
            i += 2;
        }

        if i < len {
            sum = sum.wrapping_add((header_bytes[i] as u32) << 8);
        }

        while (sum >> 16) != 0 {
            sum = (sum & 0xFFFF) + (sum >> 16);
        }

        !(sum as u16)
    }

    pub fn parse(buf: &[u8]) -> Result<(Self, usize), NetError> {
        if buf.len() < IPV4_MIN_HEADER_LEN {
            return Err(NetError::BufferTooShort);
        }

        let version = buf[0] >> 4;
        if version != 4 {
            return Err(NetError::InvalidVersion);
        }

        let ihl = (buf[0] & 0x0F) as usize;
        let header_len = ihl * 4;
        if header_len < IPV4_MIN_HEADER_LEN || buf.len() < header_len {
            return Err(NetError::InvalidHeaderLength);
        }

        let calc_cs = Self::compute_checksum(&buf[..header_len]);
        if calc_cs != 0 {
            return Err(NetError::InvalidChecksum);
        }

        let tos = buf[1];
        let total_length = ((buf[2] as u16) << 8) | (buf[3] as u16);
        let identification = ((buf[4] as u16) << 8) | (buf[5] as u16);
        let flags_and_offset = ((buf[6] as u16) << 8) | (buf[7] as u16);
        let ttl = buf[8];
        let protocol = IpProtocol::from(buf[9]);
        let checksum = ((buf[10] as u16) << 8) | (buf[11] as u16);

        let mut src = [0u8; 4];
        let mut dst = [0u8; 4];
        src.copy_from_slice(&buf[12..16]);
        dst.copy_from_slice(&buf[16..20]);

        Ok((
            Self {
                ihl: ihl as u8,
                tos,
                total_length,
                identification,
                flags_and_offset,
                ttl,
                protocol,
                checksum,
                src_ip: Ipv4Address(src),
                dst_ip: Ipv4Address(dst),
            },
            header_len,
        ))
    }

    pub fn serialize(&mut self, buf: &mut [u8]) -> Result<usize, NetError> {
        let header_len = IPV4_MIN_HEADER_LEN;
        if buf.len() < header_len {
            return Err(NetError::BufferTooShort);
        }

        buf[0] = (4 << 4) | 5; // Version 4, IHL 5 (20 bytes)
        buf[1] = self.tos;
        buf[2] = (self.total_length >> 8) as u8;
        buf[3] = (self.total_length & 0xFF) as u8;
        buf[4] = (self.identification >> 8) as u8;
        buf[5] = (self.identification & 0xFF) as u8;
        buf[6] = (self.flags_and_offset >> 8) as u8;
        buf[7] = (self.flags_and_offset & 0xFF) as u8;
        buf[8] = self.ttl;
        buf[9] = self.protocol as u8;
        buf[10] = 0; // Checksum zero for calculation
        buf[11] = 0;
        buf[12..16].copy_from_slice(&self.src_ip.0);
        buf[16..20].copy_from_slice(&self.dst_ip.0);

        let cs = Self::compute_checksum(&buf[..header_len]);
        self.checksum = cs;
        buf[10] = (cs >> 8) as u8;
        buf[11] = (cs & 0xFF) as u8;

        Ok(header_len)
    }
}
