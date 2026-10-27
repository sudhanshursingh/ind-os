#!/usr/bin/env python3
"""
Ind OS Android Boot Image v2 Packager for Motorola Moto G60 (hanoip / SM7150)
Complies with Qualcomm ABOOT / LK Boot Image Standard (INV-BOOT-001, Rule N-06)
"""

import os
import sys
import struct
import hashlib

QUALCOMM_PAGE_SIZE = 4096
SM7150_DRAM_BASE = 0x80000000
KERNEL_OFFSET = 0x00080000
KERNEL_ADDR = SM7150_DRAM_BASE + KERNEL_OFFSET
TAGS_OFFSET = 0x00000100
TAGS_ADDR = SM7150_DRAM_BASE + TAGS_OFFSET

def pad_to_page(data: bytes, page_size: int = QUALCOMM_PAGE_SIZE) -> bytes:
    remainder = len(data) % page_size
    if remainder == 0:
        return data
    return data + b'\x00' * (page_size - remainder)

def package_boot_image(kernel_path: str, output_path: str):
    if not os.path.exists(kernel_path):
        print(f"❌ Error: Kernel binary not found at {kernel_path}")
        sys.exit(1)

    with open(kernel_path, "rb") as f:
        kernel_bytes = f.read()

    kernel_size = len(kernel_bytes)
    padded_kernel = pad_to_page(kernel_bytes)

    # Empty ramdisk padded to page
    ramdisk_bytes = b""
    ramdisk_size = 0
    padded_ramdisk = b""

    # SHA-1 hash for image ID
    hasher = hashlib.sha1()
    hasher.update(kernel_bytes)
    hasher.update(ramdisk_bytes)
    image_id = hasher.digest().ljust(32, b'\x00')

    cmdline = b"console=ttyMSM0,115200 earlycon=msm_geni_serial,0x0a880000 androidboot.hardware=qcom"
    cmdline_padded = cmdline.ljust(512, b'\x00')
    extra_cmdline_padded = b'\x00' * 1024
    name_padded = b"IndOS-v0.1-G9".ljust(16, b'\x00')

    # Boot image header v2 struct:
    # 8s: magic "ANDROID!"
    # 10I: kernel_size, kernel_addr, ramdisk_size, ramdisk_addr, second_size (0), second_addr (0), tags_addr, page_size, header_version (2), os_version (0)
    # 16s: name
    # 512s: cmdline
    # 32s: id
    # 1024s: extra_cmdline
    # I: recovery_dtbo_size (0)
    # Q: recovery_dtbo_offset (0)
    # I: header_size (1660)
    # I: dtb_size (0)
    # Q: dtb_addr (0)
    hdr_raw = struct.pack(
        '<8s10I16s512s32s1024sIQIIQ',
        b'ANDROID!',
        kernel_size,
        KERNEL_ADDR,
        ramdisk_size,
        0x81000000,
        0, 0,
        TAGS_ADDR,
        QUALCOMM_PAGE_SIZE,
        2,  # Header version 2
        0,  # OS version
        name_padded,
        cmdline_padded,
        image_id,
        extra_cmdline_padded,
        0,  # recovery_dtbo_size
        0,  # recovery_dtbo_offset
        1660, # header_size
        0,  # dtb_size
        0   # dtb_addr
    )

    padded_header = pad_to_page(hdr_raw)
    final_image = padded_header + padded_kernel + padded_ramdisk

    os.makedirs(os.path.dirname(os.path.abspath(output_path)), exist_ok=True)
    with open(output_path, "wb") as f:
        f.write(final_image)

    print("==================================================")
    print(" Ind OS Boot Image Packaging Successful          ")
    print("==================================================")
    print(f" Target Device:   Motorola Moto G60 (hanoip / SM7150)")
    print(f" Header Magic:    ANDROID! (Version 2)")
    print(f" Page Size:       {QUALCOMM_PAGE_SIZE} bytes")
    print(f" Kernel Load:     0x{KERNEL_ADDR:08X} ({kernel_size} bytes)")
    print(f" Output Image:    {output_path} ({len(final_image)} bytes)")
    print("==================================================")

if __name__ == "__main__":
    kernel = "target/indk.bin"
    output = "target/indos-hanoip-boot.img"
    if len(sys.argv) > 1:
        kernel = sys.argv[1]
    if len(sys.argv) > 2:
        output = sys.argv[2]
    package_boot_image(kernel, output)
