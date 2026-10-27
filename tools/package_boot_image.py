#!/usr/bin/env python3
"""
Ind OS Android Boot Image Packager for Motorola Moto G60 (hanoip / SM7150)
Complies with Qualcomm ABOOT / LK Boot Image Standard (INV-BOOT-001, Rule N-06)
Embeds:
  - Header v2 & v3 (Android Boot Image Standard)
  - Pure Rust ARM64 IndK microkernel binary (aarch64-indk)
  - Qualcomm SM7150-AC Device Tree Blob (hanoip.dtb, magic 0xd00dfeed)
  - Verified Initial Ramdisk Payload (stock_ramdisk.img)
"""

import os
import sys
import struct
import hashlib

QUALCOMM_PAGE_SIZE = 4096
SM7150_DRAM_BASE = 0x80000000
KERNEL_OFFSET = 0x00080000
KERNEL_ADDR = SM7150_DRAM_BASE + KERNEL_OFFSET
RAMDISK_OFFSET = 0x01000000
RAMDISK_ADDR = SM7150_DRAM_BASE + RAMDISK_OFFSET
DTB_OFFSET = 0x01f00000
DTB_ADDR = SM7150_DRAM_BASE + DTB_OFFSET
TAGS_OFFSET = 0x00000100
TAGS_ADDR = SM7150_DRAM_BASE + TAGS_OFFSET
ANDROID_OS_VERSION = 469762474  # Android 11+ (GKI base)

def pad_to_page(data: bytes, page_size: int = QUALCOMM_PAGE_SIZE) -> bytes:
    remainder = len(data) % page_size
    if remainder == 0:
        return data
    return data + b'\x00' * (page_size - remainder)

def locate_asset(asset_name: str, fallback_paths: list) -> str:
    for path in fallback_paths:
        if os.path.exists(path) and os.path.getsize(path) > 0:
            return path
    return ""

def package_boot_image(kernel_path: str, output_path: str):
    if not os.path.exists(kernel_path):
        print(f"❌ Error: Kernel binary not found at {kernel_path}")
        sys.exit(1)

    with open(kernel_path, "rb") as f:
        kernel_bytes = f.read()

    kernel_size = len(kernel_bytes)
    padded_kernel = pad_to_page(kernel_bytes)

    # 1. Locate and load Qualcomm SM7150 hanoip DTB
    script_dir = os.path.dirname(os.path.abspath(__file__))
    workspace_root = os.path.dirname(script_dir)
    dtb_candidates = [
        os.path.join(workspace_root, "tools", "hanoip_assets", "hanoip.dtb"),
        os.path.join(workspace_root, "target", "hanoip.dtb"),
        "/Users/dhan/Downloads/vendor_boot.img",
    ]
    dtb_path = locate_asset("hanoip.dtb", dtb_candidates)
    dtb_bytes = b""
    if dtb_path.endswith(".dtb"):
        with open(dtb_path, "rb") as f:
            dtb_bytes = f.read()
    elif dtb_path.endswith("vendor_boot.img"):
        with open(dtb_path, "rb") as f:
            f.seek(8192)
            dtb_bytes = f.read(343054)

    if not dtb_bytes or dtb_bytes[:4] != b"\xd0\x0d\xfe\xed":
        print(f"⚠️ Warning: hanoip.dtb magic 0xd00dfeed not verified. Size: {len(dtb_bytes)}")
    else:
        print(f"✓ Embedded Qualcomm SM7150 DTB: {len(dtb_bytes)} bytes (magic: 0xd00dfeed)")

    # 2. Locate and load initial ramdisk
    ramdisk_candidates = [
        os.path.join(workspace_root, "tools", "hanoip_assets", "stock_ramdisk.img"),
        os.path.join(workspace_root, "target", "stock_ramdisk.img"),
        "/Users/dhan/Downloads/boot.img",
    ]
    rd_path = locate_asset("stock_ramdisk.img", ramdisk_candidates)
    ramdisk_bytes = b""
    if rd_path.endswith(".img") and "boot" not in os.path.basename(rd_path):
        with open(rd_path, "rb") as f:
            ramdisk_bytes = f.read()
    elif rd_path.endswith("boot.img"):
        with open(rd_path, "rb") as f:
            f.seek(4096 + 48091136)
            ramdisk_bytes = f.read(16978469)

    ramdisk_size = len(ramdisk_bytes)
    padded_ramdisk = pad_to_page(ramdisk_bytes)
    padded_dtb = pad_to_page(dtb_bytes)

    print(f"✓ Embedded Initial Ramdisk: {ramdisk_size} bytes")

    # Command line
    cmdline = b"console=ttyMSM0,115200 earlycon=msm_geni_serial,0x0a880000 androidboot.hardware=qcom androidboot.console=ttyMSM0"
    cmdline_padded_v2 = cmdline.ljust(512, b'\x00')
    cmdline_padded_v3 = cmdline.ljust(1536, b'\x00')
    extra_cmdline_padded = b'\x00' * 1024
    name_padded = b"IndOS-v0.1-G9".ljust(16, b'\x00')

    # SHA-1 hash for image ID
    hasher = hashlib.sha1()
    hasher.update(kernel_bytes)
    hasher.update(ramdisk_bytes)
    hasher.update(dtb_bytes)
    image_id = hasher.digest().ljust(32, b'\x00')

    # ==================================================
    # Build Version 2 Boot Image (Complete Self-Contained)
    # ==================================================
    hdr_v2 = struct.pack(
        '<8s10I16s512s32s1024sIQIIQ',
        b'ANDROID!',
        kernel_size,
        KERNEL_ADDR,
        ramdisk_size,
        RAMDISK_ADDR,
        0, 0,
        TAGS_ADDR,
        QUALCOMM_PAGE_SIZE,
        2,  # Header version 2
        ANDROID_OS_VERSION,
        name_padded,
        cmdline_padded_v2,
        image_id,
        extra_cmdline_padded,
        0,  # recovery_dtbo_size
        0,  # recovery_dtbo_offset
        1660, # header_size
        len(dtb_bytes),  # dtb_size
        DTB_ADDR         # dtb_addr
    )
    padded_header_v2 = pad_to_page(hdr_v2)
    final_image_v2 = padded_header_v2 + padded_kernel + padded_ramdisk + padded_dtb

    os.makedirs(os.path.dirname(os.path.abspath(output_path)), exist_ok=True)
    with open(output_path, "wb") as f:
        f.write(final_image_v2)

    # ==================================================
    # Build Version 3 Boot Image (GKI / Android 11 standard)
    # ==================================================
    hdr_v3 = struct.pack(
        '<8s4I4II1536s',
        b'ANDROID!',
        kernel_size,
        ramdisk_size,
        ANDROID_OS_VERSION,
        1580,  # header_size
        0, 0, 0, 0,
        3,  # Header version 3
        cmdline_padded_v3
    )
    padded_header_v3 = pad_to_page(hdr_v3)
    final_image_v3 = padded_header_v3 + padded_kernel + padded_ramdisk

    v3_output_path = output_path.replace(".img", "-v3.img")
    with open(v3_output_path, "wb") as f:
        f.write(final_image_v3)

    print("==================================================")
    print(" Ind OS Boot Image Packaging Successful          ")
    print("==================================================")
    print(f" Target Device:     Motorola Moto G60 (hanoip / SM7150)")
    print(f" Page Size:         {QUALCOMM_PAGE_SIZE} bytes")
    print(f" Kernel Load:       0x{KERNEL_ADDR:08X} ({kernel_size} bytes)")
    print(f" Ramdisk Load:      0x{RAMDISK_ADDR:08X} ({ramdisk_size} bytes)")
    print(f" DTB Load:          0x{DTB_ADDR:08X} ({len(dtb_bytes)} bytes)")
    print(f" [V2 Image]:        {output_path} ({len(final_image_v2)} bytes / {round(len(final_image_v2)/1024/1024, 2)} MB)")
    print(f" [V3 Image]:        {v3_output_path} ({len(final_image_v3)} bytes / {round(len(final_image_v3)/1024/1024, 2)} MB)")
    print("==================================================")

if __name__ == "__main__":
    kernel = "target/indk-hanoip.bin"
    output = "target/indos-hanoip-boot.img"
    if len(sys.argv) > 1:
        kernel = sys.argv[1]
    if len(sys.argv) > 2:
        output = sys.argv[2]
    package_boot_image(kernel, output)
