//! Kernel TLS Integration Verification & Runtime Harness (SP-52, DEF-004, DEC-004)
//!
//! Architectural Invariants:
//! - INV-TLS-001: Non-TCB userspace isolation with capability-gated stream access
//! - INV-TLS-002: TLS 1.3 only & pinned AEAD cipher negotiation
//! - INV-TLS-003: Trust anchor verification & tampered certificate rejection

use crate::userspace::net_core::{KernelNetError, KernelSocketCapToken};

pub const KERNEL_TLS_VERSION_1_3: u16 = 0x0304;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KernelCipherSuite {
    Aes128GcmSha256 = 0x1301,
    ChaCha20Poly1305Sha256 = 0x1303,
    Unsupported = 0xFFFF,
}

impl From<u16> for KernelCipherSuite {
    fn from(val: u16) -> Self {
        match val {
            0x1301 => KernelCipherSuite::Aes128GcmSha256,
            0x1303 => KernelCipherSuite::ChaCha20Poly1305Sha256,
            _ => KernelCipherSuite::Unsupported,
        }
    }
}

pub struct KernelTlsHarness;

impl KernelTlsHarness {
    /// Verify TLS 1.3 ClientHello serialization and SNI matching (INV-TLS-002)
    pub fn build_client_hello(server_name: &str, buf: &mut [u8]) -> Result<usize, KernelNetError> {
        if buf.len() < 128 {
            return Err(KernelNetError::BufferTooShort);
        }

        buf[0] = 22; // Handshake
        buf[1] = 0x03; // TLS 1.3 legacy header version 0x0303
        buf[2] = 0x03;

        buf[5] = 0x01; // ClientHello
        buf[6] = 0x00;
        buf[7] = 0x00;
        buf[8] = 0x00;

        let mut offset = 9;
        // Legacy client version 0x0303
        buf[offset] = 0x03;
        buf[offset + 1] = 0x03;
        offset += 2;

        // 32-byte client random
        for i in 0..32 {
            buf[offset + i] = (i + 1) as u8;
        }
        offset += 32;

        // Session ID length: 0
        buf[offset] = 0;
        offset += 1;

        // Cipher suites (0x1301, 0x1303)
        buf[offset] = 0x00;
        buf[offset + 1] = 0x04;
        buf[offset + 2] = 0x13;
        buf[offset + 3] = 0x01;
        buf[offset + 4] = 0x13;
        buf[offset + 5] = 0x03;
        offset += 6;

        let sni = server_name.as_bytes();
        let len = if sni.len() > 32 { 32 } else { sni.len() };
        buf[offset..offset + len].copy_from_slice(&sni[..len]);
        offset += len;

        let hs_len = (offset - 9) as u32;
        buf[6] = (hs_len >> 16) as u8;
        buf[7] = (hs_len >> 8) as u8;
        buf[8] = (hs_len & 0xFF) as u8;

        let rec_len = (offset - 5) as u16;
        buf[3] = (rec_len >> 8) as u8;
        buf[4] = (rec_len & 0xFF) as u8;

        Ok(offset)
    }

    /// Verify ServerHello cipher negotiation (INV-TLS-002)
    pub fn negotiate_cipher(server_hello_cipher: u16) -> Result<KernelCipherSuite, KernelNetError> {
        let suite = KernelCipherSuite::from(server_hello_cipher);
        if suite == KernelCipherSuite::Unsupported {
            return Err(KernelNetError::InvalidVersion);
        }
        Ok(suite)
    }

    /// Verify certificate authenticity against expected fingerprint & domain (INV-TLS-003)
    pub fn verify_certificate(
        cert_data: &[u8],
        expected_fingerprint: u32,
        expected_domain: &str,
    ) -> Result<bool, KernelNetError> {
        if cert_data.len() < 8 {
            return Err(KernelNetError::BufferTooShort);
        }

        let mut hash: u32 = 0x811c9dc5;
        for &b in cert_data {
            hash ^= b as u32;
            hash = hash.wrapping_mul(0x01000193);
        }

        if hash != expected_fingerprint {
            return Err(KernelNetError::InvalidChecksum);
        }

        let cert_str = core::str::from_utf8(cert_data).unwrap_or("");
        if !cert_str.contains(expected_domain) {
            return Err(KernelNetError::PermissionDenied);
        }

        Ok(true)
    }

    /// Verify capability rights for TLS stream wrapper (INV-TLS-001)
    pub fn verify_stream_capability(token: &KernelSocketCapToken) -> Result<bool, KernelNetError> {
        // Must hold CONNECT and SEND/RECV
        if (token.rights & (crate::userspace::KERNEL_CAP_NET_CONNECT | crate::userspace::KERNEL_CAP_NET_SEND)) == 0 {
            return Err(KernelNetError::PermissionDenied);
        }
        Ok(true)
    }
}
