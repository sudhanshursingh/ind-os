//! Build script for IndK kernel crate
//! Extracts git hash and build metadata for Gate G1 build provenance.

use std::process::Command;

fn main() {
    // Re-run if git HEAD changes
    println!("cargo:rerun-if-changed=../.git/HEAD");
    println!("cargo:rerun-if-changed=../.git/refs");

    let git_hash = Command::new("git")
        .args(["rev-parse", "--short=8", "HEAD"])
        .output()
        .ok()
        .and_then(|out| {
            if out.status.success() {
                String::from_utf8(out.stdout).ok().map(|s| s.trim().to_string())
            } else {
                None
            }
        })
        .unwrap_or_else(|| "unknown".to_string());

    let git_dirty = Command::new("git")
        .args(["status", "--porcelain"])
        .output()
        .ok()
        .map(|out| if out.stdout.is_empty() { "" } else { "-dirty" })
        .unwrap_or("");

    println!("cargo:rerun-if-env-changed=INDK_PLATFORM");
    let platform = std::env::var("INDK_PLATFORM").unwrap_or_else(|_| "qemu-virt".to_string());
    println!("cargo:rustc-env=INDK_GIT_HASH={}{}", git_hash, git_dirty);
    println!("cargo:rustc-env=INDK_TARGET_ARCH=aarch64");
    println!("cargo:rustc-env=INDK_PLATFORM={}", platform);
}
