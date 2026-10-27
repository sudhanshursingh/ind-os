//! Ethernet Frame Parser & Serializer (SP-51, REQ-KRN-110, INV-NET-002)

use crate::types::{EtherType, MacAddress, NetError};

pub const ETHERNET_HEADER_LEN: usize = 14;

#[derive(Debug, Clone, Copy)]
pub struct EthernetHeader {
    pub dst_mac: MacAddress,
    pub src_mac: MacAddress,
    pub ether_type: EtherType,
}

impl EthernetHeader {
    pub fn parse(buf: &[u8]) -> Result<(Self, usize), NetError> {
        if buf.len() < ETHERNET_HEADER_LEN {
            return Err(NetError::BufferTooShort);
        }

        let mut dst = [0u8; 6];
        let mut src = [0u8; 6];
        dst.copy_from_slice(&buf[0..6]);
        src.copy_from_slice(&buf[6..12]);

        let eth_type_val = ((buf[12] as u16) << 8) | (buf[13] as u16);
        let ether_type = EtherType::from(eth_type_val);

        Ok((
            Self {
                dst_mac: MacAddress::new(dst),
                src_mac: MacAddress::new(src),
                ether_type,
            },
            ETHERNET_HEADER_LEN,
        ))
    }

    pub fn serialize(&self, buf: &mut [u8]) -> Result<usize, NetError> {
        if buf.len() < ETHERNET_HEADER_LEN {
            return Err(NetError::BufferTooShort);
        }

        buf[0..6].copy_from_slice(&self.dst_mac.0);
        buf[6..12].copy_from_slice(&self.src_mac.0);
        let eth_val = match self.ether_type {
            EtherType::Ipv4 => 0x0800u16,
            EtherType::Arp => 0x0806u16,
            EtherType::Unknown => 0x0000u16,
        };
        buf[12] = (eth_val >> 8) as u8;
        buf[13] = (eth_val & 0xFF) as u8;

        Ok(ETHERNET_HEADER_LEN)
    }
}
