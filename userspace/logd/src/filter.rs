//! Ind OS Privacy Redaction Filter Engine
//!
//! Standards:
//! - REQ-USE-204: Redaction filters
//! - INV-LOG-001: Privacy redaction guarantee (zero cleartext secrets leaked)

use crate::types::{LogMessage, MAX_LOG_MESSAGE_LEN};

pub struct RedactionFilter;

impl RedactionFilter {
    pub fn redact(input: &str) -> LogMessage {
        let mut out = LogMessage::empty();
        let bytes = input.as_bytes();
        let len = bytes.len();
        let mut i = 0;

        while i < len && out.len < MAX_LOG_MESSAGE_LEN {
            if Self::match_prefix(bytes, i, b"token:0x") {
                Self::append_str(&mut out, "token:[REDACTED]");
                i += 8;
                while i < len && Self::is_hex_digit(bytes[i]) {
                    i += 1;
                }
            } else if Self::match_prefix(bytes, i, b"key:0x") {
                Self::append_str(&mut out, "key:[REDACTED]");
                i += 6;
                while i < len && Self::is_hex_digit(bytes[i]) {
                    i += 1;
                }
            } else if Self::match_prefix(bytes, i, b"secret:") {
                Self::append_str(&mut out, "secret:[REDACTED]");
                i += 7;
                while i < len && bytes[i] != b' ' && bytes[i] != b'\n' && bytes[i] != b'\t' {
                    i += 1;
                }
            } else {
                if out.len < MAX_LOG_MESSAGE_LEN {
                    out.bytes[out.len] = bytes[i];
                    out.len += 1;
                }
                i += 1;
            }
        }

        out
    }

    fn match_prefix(bytes: &[u8], idx: usize, prefix: &[u8]) -> bool {
        if idx + prefix.len() > bytes.len() {
            return false;
        }
        let mut j = 0;
        while j < prefix.len() {
            if bytes[idx + j] != prefix[j] {
                return false;
            }
            j += 1;
        }
        true
    }

    fn is_hex_digit(b: u8) -> bool {
        (b >= b'0' && b <= b'9') || (b >= b'a' && b <= b'f') || (b >= b'A' && b <= b'F')
    }

    fn append_str(out: &mut LogMessage, text: &str) {
        let b = text.as_bytes();
        let mut j = 0;
        while j < b.len() && out.len < MAX_LOG_MESSAGE_LEN {
            out.bytes[out.len] = b[j];
            out.len += 1;
            j += 1;
        }
    }
}
