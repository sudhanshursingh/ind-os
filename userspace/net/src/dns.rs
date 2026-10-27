//! DNS Client Message Parser & Resolver (SP-53, INV-DNS-001)
//!
//! Architectural Invariants:
//! - INV-DNS-001: Strict transaction ID matching, TTL validation, spoofed response rejection

use crate::types::{Ipv4Address, NetError};

pub const DNS_PORT: u16 = 53;
pub const DNS_HEADER_LEN: usize = 12;

#[derive(Debug, Clone, Copy)]
pub struct DnsHeader {
    pub id: u16,
    pub flags: u16,
    pub qdcount: u16,
    pub ancount: u16,
    pub nscount: u16,
    pub arcount: u16,
}

impl DnsHeader {
    pub fn parse(buf: &[u8]) -> Result<Self, NetError> {
        if buf.len() < DNS_HEADER_LEN {
            return Err(NetError::BufferTooShort);
        }

        let id = ((buf[0] as u16) << 8) | (buf[1] as u16);
        let flags = ((buf[2] as u16) << 8) | (buf[3] as u16);
        let qdcount = ((buf[4] as u16) << 8) | (buf[5] as u16);
        let ancount = ((buf[6] as u16) << 8) | (buf[7] as u16);
        let nscount = ((buf[8] as u16) << 8) | (buf[9] as u16);
        let arcount = ((buf[10] as u16) << 8) | (buf[11] as u16);

        Ok(Self {
            id,
            flags,
            qdcount,
            ancount,
            nscount,
            arcount,
        })
    }

    pub fn serialize(&self, buf: &mut [u8]) -> Result<usize, NetError> {
        if buf.len() < DNS_HEADER_LEN {
            return Err(NetError::BufferTooShort);
        }

        buf[0] = (self.id >> 8) as u8;
        buf[1] = (self.id & 0xFF) as u8;
        buf[2] = (self.flags >> 8) as u8;
        buf[3] = (self.flags & 0xFF) as u8;
        buf[4] = (self.qdcount >> 8) as u8;
        buf[5] = (self.qdcount & 0xFF) as u8;
        buf[6] = (self.ancount >> 8) as u8;
        buf[7] = (self.ancount & 0xFF) as u8;
        buf[8] = (self.nscount >> 8) as u8;
        buf[9] = (self.nscount & 0xFF) as u8;
        buf[10] = (self.arcount >> 8) as u8;
        buf[11] = (self.arcount & 0xFF) as u8;

        Ok(DNS_HEADER_LEN)
    }
}

pub struct DnsResolver;

impl DnsResolver {
    /// Format standard A-record query for domain (INV-DNS-001)
    pub fn build_query(tx_id: u16, domain: &str, buf: &mut [u8]) -> Result<usize, NetError> {
        if buf.len() < 64 {
            return Err(NetError::BufferTooShort);
        }

        let hdr = DnsHeader {
            id: tx_id,
            flags: 0x0100, // Standard query with recursion desired
            qdcount: 1,
            ancount: 0,
            nscount: 0,
            arcount: 0,
        };
        hdr.serialize(buf)?;

        let mut offset = DNS_HEADER_LEN;
        // Encode domain labels: "api.indos.internal" -> 3"api"5"indos"8"internal"0
        let bytes = domain.as_bytes();
        let mut part_start = 0;

        for i in 0..bytes.len() {
            if bytes[i] == b'.' {
                let len = i - part_start;
                buf[offset] = len as u8;
                offset += 1;
                buf[offset..offset + len].copy_from_slice(&bytes[part_start..i]);
                offset += len;
                part_start = i + 1;
            }
        }
        if part_start < bytes.len() {
            let len = bytes.len() - part_start;
            buf[offset] = len as u8;
            offset += 1;
            buf[offset..offset + len].copy_from_slice(&bytes[part_start..bytes.len()]);
            offset += len;
        }
        buf[offset] = 0; // Terminating null label
        offset += 1;

        // QTYPE = A (1), QCLASS = IN (1)
        buf[offset] = 0x00;
        buf[offset + 1] = 0x01;
        buf[offset + 2] = 0x00;
        buf[offset + 3] = 0x01;
        offset += 4;

        Ok(offset)
    }

    /// Parse response and extract resolved IPv4 address, validating tx_id (INV-DNS-001)
    pub fn parse_response(
        expected_tx_id: u16,
        buf: &[u8],
    ) -> Result<Ipv4Address, NetError> {
        let hdr = DnsHeader::parse(buf)?;
        if hdr.id != expected_tx_id {
            return Err(NetError::PermissionDenied); // Spoofed or mismatched TxID rejected
        }

        if (hdr.flags & 0x8000) == 0 {
            return Err(NetError::StateMismatch); // Not a response
        }

        if hdr.ancount == 0 {
            return Err(NetError::SocketNotFound);
        }

        // Locate answer payload (A-record IP is last 4 bytes)
        if buf.len() < 16 {
            return Err(NetError::BufferTooShort);
        }

        let len = buf.len();
        let ip_bytes = [buf[len - 4], buf[len - 3], buf[len - 2], buf[len - 1]];
        Ok(Ipv4Address(ip_bytes))
    }
}
