//! VFS Path Canonicalization & Resolution Layer
//! Compliant with Bible §8.10 (SP-24) and Invariant INV-VFS-002.

/// Path resolution and canonicalization utility
pub struct PathResolver;

impl PathResolver {
    /// Canonicalizes a path into a normalized form:
    /// - Strips redundant `/`
    /// - Resolves `.` (current directory)
    /// - Resolves `..` (parent directory, never escaping root `/`, INV-VFS-002)
    /// - Ensures leading `/`
    pub fn canonicalize<'a>(raw_path: &str, output: &'a mut [u8]) -> Result<&'a str, &'static str> {
        if raw_path.is_empty() {
            return Err("Path cannot be empty");
        }

        let mut stack: [&str; 16] = [""; 16];
        let mut depth: usize = 0;

        let bytes = raw_path.as_bytes();
        let mut start = 0;

        while start < bytes.len() {
            // Skip leading slashes
            while start < bytes.len() && bytes[start] == b'/' {
                start += 1;
            }
            if start >= bytes.len() {
                break;
            }

            let mut end = start;
            while end < bytes.len() && bytes[end] != b'/' {
                end += 1;
            }

            let part = core::str::from_utf8(&bytes[start..end]).map_err(|_| "Invalid UTF-8 path")?;
            start = end;

            if part == "." || part.is_empty() {
                continue;
            } else if part == ".." {
                // Pop directory, but never escape root (INV-VFS-002)
                if depth > 0 {
                    depth -= 1;
                }
            } else {
                if depth >= 16 {
                    return Err("Path depth exceeds maximum limit");
                }
                stack[depth] = part;
                depth += 1;
            }
        }

        // Render normalized path into output buffer
        let mut out_idx = 0;
        if depth == 0 {
            if output.is_empty() {
                return Err("Output buffer too small");
            }
            output[0] = b'/';
            out_idx = 1;
        } else {
            for i in 0..depth {
                if out_idx >= output.len() {
                    return Err("Output buffer too small");
                }
                output[out_idx] = b'/';
                out_idx += 1;

                let part_bytes = stack[i].as_bytes();
                if out_idx + part_bytes.len() > output.len() {
                    return Err("Output buffer too small");
                }
                output[out_idx..out_idx + part_bytes.len()].copy_from_slice(part_bytes);
                out_idx += part_bytes.len();
            }
        }

        core::str::from_utf8(&output[..out_idx]).map_err(|_| "Output UTF-8 conversion error")
    }
}
