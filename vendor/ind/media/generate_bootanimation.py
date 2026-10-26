#!/usr/bin/env python3
"""
Ind OS — Boot Animation Generator
Generates zero-compression bootanimation.zip for Motorola Moto G60 (1080 x 2460 @ 60fps)
Reference: PRD v1.0.1-BIBLE (Sprint 16)
"""

import os
import shutil
import zipfile
from PIL import Image, ImageDraw

WIDTH = 1080
HEIGHT = 2460
FPS = 60

OUTPUT_DIR = os.path.dirname(os.path.abspath(__file__))
STAGING_DIR = os.path.join(OUTPUT_DIR, "bootanimation_staging")
ZIP_PATH = os.path.join(OUTPUT_DIR, "bootanimation.zip")

def create_frame(progress: float, is_loop: bool = False) -> Image.Image:
    # Deep obsidian navy background
    img = Image.new("RGB", (WIDTH, HEIGHT), color=(10, 14, 23))
    draw = ImageDraw.Draw(img)

    cx, cy = WIDTH // 2, HEIGHT // 2 - 120
    base_radius = 160

    # Dynamic breathing radius
    import math
    glow_pulse = math.sin(progress * math.pi * 2) if is_loop else min(1.0, progress * 1.5)
    radius = int(base_radius + 20 * glow_pulse)

    # Ambient Outer Glow rings
    draw.ellipse([cx - radius - 60, cy - radius - 60, cx + radius + 60, cy + radius + 60],
                 outline=(30, 45, 70), width=6)
    draw.ellipse([cx - radius - 30, cy - radius - 30, cx + radius + 30, cy + radius + 30],
                 outline=(40, 70, 110), width=4)

    # Tricolor Indian Sovereign Aura (Saffron, White, Green)
    draw.arc([cx - radius - 15, cy - radius - 15, cx + radius + 15, cy + radius + 15],
             start=200, end=340, fill=(255, 153, 51), width=10) # Saffron
    draw.arc([cx - radius - 15, cy - radius - 15, cx + radius + 15, cy + radius + 15],
             start=340, end=20, fill=(240, 245, 255), width=10)  # White/Ashoka Blue
    draw.arc([cx - radius - 15, cy - radius - 15, cx + radius + 15, cy + radius + 15],
             start=20, end=160, fill=(19, 136, 8), width=10)    # Green

    # Central Core Orb (Glowing Gradient Core)
    draw.ellipse([cx - radius, cy - radius, cx + radius, cy + radius],
                 fill=(18, 30, 52), outline=(100, 180, 255), width=6)

    # Center Pulse Iris
    inner_r = int(radius * 0.45)
    draw.ellipse([cx - inner_r, cy - inner_r, cx + inner_r, cy + inner_r],
                 fill=(220, 240, 255))

    # Branding text placeholders
    # In production AOSP bootanimation, simple geometry or prerendered bitmap is rendered
    return img

def main():
    if os.path.exists(STAGING_DIR):
        shutil.rmtree(STAGING_DIR)

    part0 = os.path.join(STAGING_DIR, "part0")
    part1 = os.path.join(STAGING_DIR, "part1")
    os.makedirs(part0, exist_ok=True)
    os.makedirs(part1, exist_ok=True)

    # Write desc.txt
    desc_path = os.path.join(STAGING_DIR, "desc.txt")
    with open(desc_path, "w") as f:
        f.write(f"{WIDTH} {HEIGHT} {FPS}\n")
        f.write("c 1 0 part0\n")
        f.write("c 0 0 part1\n")

    print(f"[*] Rendering Part 0 (Intro Ignition, 30 frames)...")
    for i in range(30):
        frame = create_frame(i / 30.0, is_loop=False)
        frame.save(os.path.join(part0, f"{i:05d}.png"), "PNG")

    print(f"[*] Rendering Part 1 (Breathing Ambient Loop, 30 frames)...")
    for i in range(30):
        frame = create_frame(i / 30.0, is_loop=True)
        frame.save(os.path.join(part1, f"{i:05d}.png"), "PNG")

    print(f"[*] Packaging zero-compression zip: {ZIP_PATH}...")
    if os.path.exists(ZIP_PATH):
        os.remove(ZIP_PATH)

    # Android requirement: ZIP_STORED (0 compression)
    with zipfile.ZipFile(ZIP_PATH, "w", compression=zipfile.ZIP_STORED) as zf:
        zf.write(desc_path, "desc.txt")
        for root, _, files in os.walk(part0):
            for file in sorted(files):
                zf.write(os.path.join(root, file), os.path.join("part0", file))
        for root, _, files in os.walk(part1):
            for file in sorted(files):
                zf.write(os.path.join(root, file), os.path.join("part1", file))

    shutil.rmtree(STAGING_DIR)
    print(f"[+] Boot animation package generated: {ZIP_PATH} ({os.path.getsize(ZIP_PATH)} bytes)")

if __name__ == "__main__":
    main()
