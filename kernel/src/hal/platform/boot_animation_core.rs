//! Kernel Hardware Core for Cold Boot Animation (120Hz/60Hz Double-Buffered)
//!
//! Constitutional References:
//! - INV-ANIM-001: Total cold-boot animation duration <= 2800ms
//! - INV-ANIM-002: Zero dynamic heap allocation in rendering loop
//! - INV-BOOT-001: Microchip circuit ignition during MMU/DRAM init
//! - Rule N-01: Zero external crates

use super::hanoip_display::{HardwareScreenPainter, HANOIP_PANEL_HEIGHT, HANOIP_PANEL_WIDTH};

pub const KERNEL_CAP_BOOT_ANIM: u32 = 0x8000_00B7;

pub const COLOR_BG_IGNITION: u32 = 0xFF0A0D12;
pub const COLOR_CYAN_PULSE: u32 = 0xFF00F0FF;
pub const COLOR_GOLD_MANDALA: u32 = 0xFFFFB300;
pub const COLOR_WHITE_TEXT: u32 = 0xFFFFFFFF;
pub const COLOR_ORANGE_BRAND: u32 = 0xFFFF6D00;

pub struct KernelBootAnimationEngine {
    current_ms: u32,
    frame_count: u32,
}

impl KernelBootAnimationEngine {
    pub const fn new() -> Self {
        Self {
            current_ms: 0,
            frame_count: 0,
        }
    }

    pub fn set_time(&mut self, ms: u32) {
        self.current_ms = ms;
        self.frame_count += 1;
    }

    /// Renders the animation frame corresponding to `current_ms`
    pub unsafe fn render_frame(&self, painter: &mut HardwareScreenPainter) {
        let ms = self.current_ms;

        if ms < 800 {
            // ==========================================
            // STAGE 1: SILICON IGNITION (0.0s -> 0.8s)
            // ==========================================
            painter.fill_rect(0, 0, HANOIP_PANEL_WIDTH, HANOIP_PANEL_HEIGHT, COLOR_BG_IGNITION);

            let progress = (ms * 1000) / 800; // 0..1000
            let cx = 540;
            let cy = 1100;
            let chip_size = 180;

            // Central Microchip Square Outline
            painter.fill_rect(cx - chip_size / 2, cy - chip_size / 2, chip_size, 4, COLOR_CYAN_PULSE);
            painter.fill_rect(cx - chip_size / 2, cy + chip_size / 2 - 4, chip_size, 4, COLOR_CYAN_PULSE);
            painter.fill_rect(cx - chip_size / 2, cy - chip_size / 2, 4, chip_size, COLOR_CYAN_PULSE);
            painter.fill_rect(cx + chip_size / 2 - 4, cy - chip_size / 2, 4, chip_size, COLOR_CYAN_PULSE);

            // Microchip Inner Core
            painter.fill_rect(cx - 50, cy - 50, 100, 100, 0xFF1E293B);
            painter.fill_rect(cx - 30, cy - 30, 60, 60, COLOR_CYAN_PULSE);

            // Microchip Lead Pins
            for i in 0..6 {
                let offset = (i as usize) * 24 + 18;
                // Top & Bottom pins
                painter.fill_rect(cx - chip_size / 2 + offset, cy - chip_size / 2 - 20, 8, 20, COLOR_CYAN_PULSE);
                painter.fill_rect(cx - chip_size / 2 + offset, cy + chip_size / 2, 8, 20, COLOR_CYAN_PULSE);
                // Left & Right pins
                painter.fill_rect(cx - chip_size / 2 - 20, cy - chip_size / 2 + offset, 20, 8, COLOR_CYAN_PULSE);
                painter.fill_rect(cx + chip_size / 2, cy - chip_size / 2 + offset, 20, 8, COLOR_CYAN_PULSE);
            }

            // PCB Circuit Traces branching outward
            let trace_len = ((HANOIP_PANEL_WIDTH / 2) * progress as usize) / 1000;
            if trace_len > 0 {
                // Horizontal Bus Traces
                painter.fill_rect(cx + chip_size / 2 + 20, cy - 30, trace_len, 4, COLOR_CYAN_PULSE);
                painter.fill_rect(cx - chip_size / 2 - 20 - trace_len, cy - 30, trace_len, 4, COLOR_CYAN_PULSE);
                painter.fill_rect(cx + chip_size / 2 + 20, cy + 30, trace_len, 4, COLOR_CYAN_PULSE);
                painter.fill_rect(cx - chip_size / 2 - 20 - trace_len, cy + 30, trace_len, 4, COLOR_CYAN_PULSE);

                // Vertical Traces
                let vtrace_len = ((HANOIP_PANEL_HEIGHT / 3) * progress as usize) / 1000;
                painter.fill_rect(cx - 40, cy + chip_size / 2 + 20, 4, vtrace_len, COLOR_CYAN_PULSE);
                painter.fill_rect(cx + 40, cy + chip_size / 2 + 20, 4, vtrace_len, COLOR_CYAN_PULSE);
                painter.fill_rect(cx - 40, cy - chip_size / 2 - 20 - vtrace_len, 4, vtrace_len, COLOR_CYAN_PULSE);
                painter.fill_rect(cx + 40, cy - chip_size / 2 - 20 - vtrace_len, 4, vtrace_len, COLOR_CYAN_PULSE);

                // Data Packets Pulse
                let pulse_pos = trace_len * 3 / 4;
                if pulse_pos > 10 {
                    painter.fill_rect(cx + chip_size / 2 + 20 + pulse_pos - 10, cy - 33, 20, 10, COLOR_WHITE_TEXT);
                    painter.fill_rect(cx - chip_size / 2 - 20 - pulse_pos - 10, cy + 27, 20, 10, COLOR_WHITE_TEXT);
                }
            }

            painter.draw_string(cx - 180, cy + 280, "AARCH64-INDK SILICON IGNITION", COLOR_CYAN_PULSE, 2);

        } else if ms < 1800 {
            // ==========================================
            // STAGE 2: MASCOT ASSEMBLY (0.8s -> 1.8s)
            // ==========================================
            painter.fill_rect(0, 0, HANOIP_PANEL_WIDTH, HANOIP_PANEL_HEIGHT, COLOR_BG_IGNITION);

            let progress = ((ms - 800) * 1000) / 1000; // 0..1000
            let cx = 540;
            let cy = 1100;

            // Head Dome & Crown Arches
            painter.fill_rect(cx - 120, cy - 160, 240, 6, COLOR_CYAN_PULSE);
            painter.fill_rect(cx - 150, cy - 120, 300, 6, COLOR_CYAN_PULSE);

            // Gaja Elephant Ear Geometry with Gold Mandala Patterns
            let ear_glow = if progress > 400 { COLOR_GOLD_MANDALA } else { 0xFF64748B };
            // Left Mandala Ear
            painter.fill_rect(cx - 280, cy - 100, 100, 160, 0xFF0F172A);
            painter.fill_rect(cx - 280, cy - 100, 100, 4, ear_glow);
            painter.fill_rect(cx - 280, cy + 56, 100, 4, ear_glow);
            painter.fill_rect(cx - 280, cy - 100, 4, 160, ear_glow);
            painter.fill_rect(cx - 250, cy - 70, 40, 100, ear_glow);

            // Right Mandala Ear
            painter.fill_rect(cx + 180, cy - 100, 100, 160, 0xFF0F172A);
            painter.fill_rect(cx + 180, cy - 100, 100, 4, ear_glow);
            painter.fill_rect(cx + 180, cy + 56, 100, 4, ear_glow);
            painter.fill_rect(cx + 276, cy - 100, 4, 160, ear_glow);
            painter.fill_rect(cx + 210, cy - 70, 40, 100, ear_glow);

            // Sweeping Trunk
            let trunk_h = (180 * progress as usize) / 1000;
            painter.fill_rect(cx - 30, cy - 20, 60, trunk_h, COLOR_CYAN_PULSE);
            if progress > 700 {
                // Curved trunk tip
                painter.fill_rect(cx - 60, cy - 20 + trunk_h - 20, 40, 30, COLOR_ORANGE_BRAND);
            }

            // Forward Tusks
            painter.fill_rect(cx - 70, cy + 40, 16, 60, COLOR_WHITE_TEXT);
            painter.fill_rect(cx + 54, cy + 40, 16, 60, COLOR_WHITE_TEXT);

            // Forehead Node (Ajna Chakra / Intelligence Node)
            painter.fill_rect(cx - 16, cy - 80, 32, 32, COLOR_GOLD_MANDALA);
            painter.fill_rect(cx - 8, cy - 72, 16, 16, COLOR_WHITE_TEXT);

            painter.draw_string(cx - 190, cy + 280, "GAJA MASCOT VECTOR ASSEMBLY", COLOR_GOLD_MANDALA, 2);

        } else if ms < 2500 {
            // ==========================================
            // STAGE 3: SOVEREIGNTY PULSE (1.8s -> 2.5s)
            // ==========================================
            painter.fill_rect(0, 0, HANOIP_PANEL_WIDTH, HANOIP_PANEL_HEIGHT, COLOR_BG_IGNITION);

            let progress = ((ms - 1800) * 1000) / 700; // 0..1000
            let cx = 540;
            let cy = 1100;

            // Render Assembled Gaja Mascot
            painter.fill_rect(cx - 120, cy - 160, 240, 6, COLOR_CYAN_PULSE);
            painter.fill_rect(cx - 280, cy - 100, 100, 160, 0xFF0F172A);
            painter.fill_rect(cx - 280, cy - 100, 100, 4, COLOR_GOLD_MANDALA);
            painter.fill_rect(cx + 180, cy - 100, 100, 160, 0xFF0F172A);
            painter.fill_rect(cx + 180, cy - 100, 100, 4, COLOR_GOLD_MANDALA);
            painter.fill_rect(cx - 30, cy - 20, 60, 180, COLOR_CYAN_PULSE);
            painter.fill_rect(cx - 60, cy + 140, 40, 30, COLOR_ORANGE_BRAND);
            painter.fill_rect(cx - 70, cy + 40, 16, 60, COLOR_WHITE_TEXT);
            painter.fill_rect(cx + 54, cy + 40, 16, 60, COLOR_WHITE_TEXT);

            // Expanding Circular Energy Shockwave Pulse
            let radius = (480 * progress as usize) / 1000;
            if radius > 20 {
                // Radial ring approximation using box borders
                painter.fill_rect(cx - radius, cy - 80 - radius, radius * 2, 4, COLOR_CYAN_PULSE);
                painter.fill_rect(cx - radius, cy - 80 + radius, radius * 2, 4, COLOR_CYAN_PULSE);
                painter.fill_rect(cx - radius, cy - 80 - radius, 4, radius * 2, COLOR_CYAN_PULSE);
                painter.fill_rect(cx + radius, cy - 80 - radius, 4, radius * 2, COLOR_CYAN_PULSE);
            }

            // Brand Typography
            painter.draw_string(cx - 120, cy + 280, "IND OS", COLOR_WHITE_TEXT, 5);
            painter.draw_string(cx - 360, cy + 390, "FAST - SECURE - PRIVATE - INTELLIGENT", COLOR_CYAN_PULSE, 2);

        } else {
            // ==========================================
            // STAGE 4: TRANSITION TO SHELL (2.5s -> 2.8s)
            // ==========================================
            let progress = ((ms - 2500) * 1000) / 300; // 0..1000
            let start_cx = 540;
            let start_cy = 1100;
            let target_cx = 60;
            let target_cy = 45;

            let cur_cx = start_cx - ((start_cx - target_cx) * progress as usize) / 1000;
            let cur_cy = start_cy - ((start_cy - target_cy) * progress as usize) / 1000;
            let icon_size = 120 - ((90 * progress as usize) / 1000);

            painter.fill_rect(0, 0, HANOIP_PANEL_WIDTH, HANOIP_PANEL_HEIGHT, COLOR_BG_IGNITION);

            // Shrunk morphing icon
            painter.fill_rect(cur_cx, cur_cy, icon_size, icon_size, COLOR_ORANGE_BRAND);
            painter.fill_rect(cur_cx + 4, cur_cy + 4, icon_size - 8, icon_size - 8, COLOR_BG_IGNITION);
            painter.fill_rect(cur_cx + 8, cur_cy + 8, icon_size - 16, icon_size - 16, COLOR_CYAN_PULSE);

            // Fade in top status chrome
            if progress > 500 {
                painter.draw_string(140, 32, "IND OS", COLOR_WHITE_TEXT, 2);
                painter.draw_string(320, 32, "MOTO G60 HARDWARE V0.1", COLOR_ORANGE_BRAND, 2);
                painter.draw_string(820, 32, "[ONLINE", COLOR_CYAN_PULSE, 2);
                painter.fill_rect(950, 38, 14, 14, 0xFF10B981);
                painter.draw_string(975, 32, "]", COLOR_CYAN_PULSE, 2);
            }
        }
    }
}
