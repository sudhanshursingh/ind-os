# Cold Boot Animation Architecture Specification (120Hz/60Hz Double-Buffered Pipeline)

## 1. Executive Summary & Provenance
- **Subsystem:** Ind OS Visual Identity & Cold Boot Sequence (SP-77, INV-COMP-002, INV-BOOT-001)
- **Target Hardware:** Motorola Moto G60 (`hanoip` / Qualcomm Snapdragon 732G / SM7150)
- **Target Panel:** Tianma ILI7807S FHD+ (1080x2460, 120Hz/60Hz)
- **Scanout Buffers:** Double-buffered DRAM scanout (`0x9c000000` / `0x9d000000`)
- **Memory Constraint:** Zero dynamic heap allocation (`INV-COMP-002`); pre-allocated frame arrays and procedural rasterization.
- **Timing Constraint:** Cold-boot animation duration $\le 2.5\text{s}$ (shell transition complete at $2.8\text{s}$).

## 2. Chronological Timeline & Subsystem Correlation

```
0.0s ──────────────── 0.8s ──────────────── 1.8s ──────────────── 2.5s ───> 2.8s
 [Microchip Circuit]     [Vector Mascot Assembly]   [Sovereignty Pulse]    [Status Bar Transition]
   (aarch64-IndK)           (IndUS Services)          (AIS Sandbox)           (Ind Surface UI)
```

### Stage 1: Silicon Ignition ($0.0\text{s} \to 0.8\text{s}$)
- **Visuals:** Background `#0A0D12` (`0xFF0A0D12`). Central microchip square outline illuminates in thin cyan lines (`#00F0FF` / `0xFF00F0FF`). Bright data pulses travel outward along PCB circuit traces across the screen.
- **Kernel Lifecycle:** IndK initializes ARM64 MMU page tables, exception vectors, and Device Tree DRAM maps (`INV-BOOT-001`).

### Stage 2: Mascot Assembly ($0.8\text{s} \to 1.8\text{s}$)
- **Visuals:** The circuit traces converge smoothly upward, drawing the glowing line-art silhouette of Gaja the Elephant. The Mandala ear geometry glows in warm gold (`#FFB300` / `0xFFFFB300`), symbolizing intelligence waking up.
- **Userspace Lifecycle:** `indus-init` (PID 1) spawns core services (`logd`, `stored`, `devmgr`) and mounts IndFS on UFS storage.

### Stage 3: The Sovereignty Pulse ($1.8\text{s} \to 2.5\text{s}$)
- **Visuals:** A soft circular energy ring expands outward from Gaja's forehead node across the entire display. The brand text fades in below in crisp vector typography:
  $$\text{\textbf{Ind OS}}$$
  $$\text{FAST • SECURE • PRIVATE • INTELLIGENT}$$
- **Security Lifecycle:** The Enforcement Layer (ENF) locks down capability tables and verifies the signed AIS model sandbox (`INV-AIS-001`).

### Stage 4: Transition to Shell ($2.5\text{s} \to 2.8\text{s}$)
- **Visuals:** The mascot logo smoothly morphs and shrinks upward into the top-left status bar `(60, 32)`, locking into position as the 2D monochrome Status Bar Icon. Simultaneously, the dark background clears to reveal the central Intent Bar and interactive home surface.
- **Interactive UI Lifecycle:** Shell event loop activates FocalTech FT3518 touch digitizer and begins 120Hz compositing.

## 3. Invariants
- `INV-ANIM-001`: Total cold-boot animation duration $\le 2800\text{ms}$.
- `INV-ANIM-002`: Zero dynamic heap allocation in animation rendering loop.
- `INV-ANIM-003`: Deterministic state transitions across all 4 stages.
