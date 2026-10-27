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
            let cx: usize = 540;
            let cy: usize = 1100;
            let chip_size: usize = 180;

            // Central Microchip Square Outline
            painter.fill_rect(cx.saturating_sub(chip_size / 2), cy.saturating_sub(chip_size / 2), chip_size, 4, COLOR_CYAN_PULSE);
            painter.fill_rect(cx.saturating_sub(chip_size / 2), cy + chip_size / 2 - 4, chip_size, 4, COLOR_CYAN_PULSE);
            painter.fill_rect(cx.saturating_sub(chip_size / 2), cy.saturating_sub(chip_size / 2), 4, chip_size, COLOR_CYAN_PULSE);
            painter.fill_rect(cx + chip_size / 2 - 4, cy.saturating_sub(chip_size / 2), 4, chip_size, COLOR_CYAN_PULSE);

            // Microchip Inner Core
            painter.fill_rect(cx.saturating_sub(50), cy.saturating_sub(50), 100, 100, 0xFF1E293B);
            painter.fill_rect(cx.saturating_sub(30), cy.saturating_sub(30), 60, 60, COLOR_CYAN_PULSE);

            // Microchip Lead Pins
            for i in 0..6 {
                let offset = (i as usize) * 24 + 18;
                // Top & Bottom pins
                painter.fill_rect(cx.saturating_sub(chip_size / 2) + offset, cy.saturating_sub(chip_size / 2 + 20), 8, 20, COLOR_CYAN_PULSE);
                painter.fill_rect(cx.saturating_sub(chip_size / 2) + offset, cy + chip_size / 2, 8, 20, COLOR_CYAN_PULSE);
                // Left & Right pins
                painter.fill_rect(cx.saturating_sub(chip_size / 2 + 20), cy.saturating_sub(chip_size / 2) + offset, 20, 8, COLOR_CYAN_PULSE);
                painter.fill_rect(cx + chip_size / 2, cy.saturating_sub(chip_size / 2) + offset, 20, 8, COLOR_CYAN_PULSE);
            }

            // PCB Circuit Traces branching outward
            let max_left = cx.saturating_sub(chip_size / 2 + 20);
            let raw_len = ((HANOIP_PANEL_WIDTH / 2) * progress as usize) / 1000;
            let trace_len = core::cmp::min(raw_len, max_left);
            if trace_len > 0 {
                // Horizontal Bus Traces
                let left_trace_x = max_left.saturating_sub(trace_len);
                painter.fill_rect(cx + chip_size / 2 + 20, cy.saturating_sub(30), trace_len, 4, COLOR_CYAN_PULSE);
                painter.fill_rect(left_trace_x, cy.saturating_sub(30), trace_len, 4, COLOR_CYAN_PULSE);
                painter.fill_rect(cx + chip_size / 2 + 20, cy + 30, trace_len, 4, COLOR_CYAN_PULSE);
                painter.fill_rect(left_trace_x, cy + 30, trace_len, 4, COLOR_CYAN_PULSE);

                // Vertical Traces
                let max_top = cy.saturating_sub(chip_size / 2 + 20);
                let vtrace_len = core::cmp::min(((HANOIP_PANEL_HEIGHT / 3) * progress as usize) / 1000, max_top);
                let top_trace_y = max_top.saturating_sub(vtrace_len);
                painter.fill_rect(cx.saturating_sub(40), cy + chip_size / 2 + 20, 4, vtrace_len, COLOR_CYAN_PULSE);
                painter.fill_rect(cx + 40, cy + chip_size / 2 + 20, 4, vtrace_len, COLOR_CYAN_PULSE);
                painter.fill_rect(cx.saturating_sub(40), top_trace_y, 4, vtrace_len, COLOR_CYAN_PULSE);
                painter.fill_rect(cx + 40, top_trace_y, 4, vtrace_len, COLOR_CYAN_PULSE);

                // Data Packets Pulse
                let pulse_pos = trace_len * 3 / 4;
                if pulse_pos > 10 {
                    painter.fill_rect(cx + chip_size / 2 + 20 + pulse_pos - 10, cy.saturating_sub(33), 20, 10, COLOR_WHITE_TEXT);
                    let left_pulse_x = max_left.saturating_sub(pulse_pos + 10);
                    painter.fill_rect(left_pulse_x, cy + 27, 20, 10, COLOR_WHITE_TEXT);
                }
            }

            painter.draw_string(cx.saturating_sub(180), cy + 280, "AARCH64-INDK SILICON IGNITION", COLOR_CYAN_PULSE, 2);

        } else if ms < 1800 {
            // ==========================================
            // STAGE 2: MASCOT ASSEMBLY (0.8s -> 1.8s)
            // ==========================================
            painter.fill_rect(0, 0, HANOIP_PANEL_WIDTH, HANOIP_PANEL_HEIGHT, COLOR_BG_IGNITION);

            let progress = ((ms - 800) * 1000) / 1000; // 0..1000
            let cx: usize = 540;
            let cy: usize = 1100;

            // Head Dome & Crown Arches
            painter.fill_rect(cx.saturating_sub(120), cy.saturating_sub(160), 240, 6, COLOR_CYAN_PULSE);
            painter.fill_rect(cx.saturating_sub(150), cy.saturating_sub(120), 300, 6, COLOR_CYAN_PULSE);

            // Gaja Elephant Ear Geometry with Gold Mandala Patterns
            let ear_glow = if progress > 400 { COLOR_GOLD_MANDALA } else { 0xFF64748B };
            // Left Mandala Ear
            painter.fill_rect(cx.saturating_sub(280), cy.saturating_sub(100), 100, 160, 0xFF0F172A);
            painter.fill_rect(cx.saturating_sub(280), cy.saturating_sub(100), 100, 4, ear_glow);
            painter.fill_rect(cx.saturating_sub(280), cy + 56, 100, 4, ear_glow);
            painter.fill_rect(cx.saturating_sub(280), cy.saturating_sub(100), 4, 160, ear_glow);
            painter.fill_rect(cx.saturating_sub(250), cy.saturating_sub(70), 40, 100, ear_glow);

            // Right Mandala Ear
            painter.fill_rect(cx + 180, cy.saturating_sub(100), 100, 160, 0xFF0F172A);
            painter.fill_rect(cx + 180, cy.saturating_sub(100), 100, 4, ear_glow);
            painter.fill_rect(cx + 180, cy + 56, 100, 4, ear_glow);
            painter.fill_rect(cx + 276, cy.saturating_sub(100), 4, 160, ear_glow);
            painter.fill_rect(cx + 210, cy.saturating_sub(70), 40, 100, ear_glow);

            // Sweeping Trunk
            let trunk_h = (180 * progress as usize) / 1000;
            painter.fill_rect(cx.saturating_sub(30), cy.saturating_sub(20), 60, trunk_h, COLOR_CYAN_PULSE);
            if progress > 700 && trunk_h >= 20 {
                // Curved trunk tip
                painter.fill_rect(cx.saturating_sub(60), cy.saturating_sub(20) + trunk_h - 20, 40, 30, COLOR_ORANGE_BRAND);
            }

            // Forward Tusks
            painter.fill_rect(cx.saturating_sub(70), cy + 40, 16, 60, COLOR_WHITE_TEXT);
            painter.fill_rect(cx + 54, cy + 40, 16, 60, COLOR_WHITE_TEXT);

            // Forehead Node (Ajna Chakra / Intelligence Node)
            painter.fill_rect(cx.saturating_sub(16), cy.saturating_sub(80), 32, 32, COLOR_GOLD_MANDALA);
            painter.fill_rect(cx.saturating_sub(8), cy.saturating_sub(72), 16, 16, COLOR_WHITE_TEXT);

            painter.draw_string(cx.saturating_sub(190), cy + 280, "GAJA MASCOT VECTOR ASSEMBLY", COLOR_GOLD_MANDALA, 2);

        } else if ms < 2500 {
            // ==========================================
            // STAGE 3: SOVEREIGNTY PULSE (1.8s -> 2.5s)
            // ==========================================
            painter.fill_rect(0, 0, HANOIP_PANEL_WIDTH, HANOIP_PANEL_HEIGHT, COLOR_BG_IGNITION);

            let progress = ((ms - 1800) * 1000) / 700; // 0..1000
            let cx: usize = 540;
            let cy: usize = 1100;

            // Render Assembled Gaja Mascot
            painter.fill_rect(cx.saturating_sub(120), cy.saturating_sub(160), 240, 6, COLOR_CYAN_PULSE);
            painter.fill_rect(cx.saturating_sub(280), cy.saturating_sub(100), 100, 160, 0xFF0F172A);
            painter.fill_rect(cx.saturating_sub(280), cy.saturating_sub(100), 100, 4, COLOR_GOLD_MANDALA);
            painter.fill_rect(cx + 180, cy.saturating_sub(100), 100, 160, 0xFF0F172A);
            painter.fill_rect(cx + 180, cy.saturating_sub(100), 100, 4, COLOR_GOLD_MANDALA);
            painter.fill_rect(cx.saturating_sub(30), cy.saturating_sub(20), 60, 180, COLOR_CYAN_PULSE);
            painter.fill_rect(cx.saturating_sub(60), cy + 140, 40, 30, COLOR_ORANGE_BRAND);
            painter.fill_rect(cx.saturating_sub(70), cy + 40, 16, 60, COLOR_WHITE_TEXT);
            painter.fill_rect(cx + 54, cy + 40, 16, 60, COLOR_WHITE_TEXT);

            // Expanding Circular Energy Shockwave Pulse
            let radius = (480 * progress as usize) / 1000;
            if radius > 20 {
                let ring_x = cx.saturating_sub(radius);
                let ring_y = (cy.saturating_sub(80)).saturating_sub(radius);
                let ring_size = radius * 2;
                painter.fill_rect(ring_x, ring_y, ring_size, 4, COLOR_CYAN_PULSE);
                painter.fill_rect(ring_x, cy.saturating_sub(80) + radius, ring_size, 4, COLOR_CYAN_PULSE);
                painter.fill_rect(ring_x, ring_y, 4, ring_size, COLOR_CYAN_PULSE);
                painter.fill_rect(cx + radius, ring_y, 4, ring_size, COLOR_CYAN_PULSE);
            }

            // Brand Typography
            painter.draw_string(cx.saturating_sub(120), cy + 280, "IND OS", COLOR_WHITE_TEXT, 5);
            painter.draw_string(cx.saturating_sub(360), cy + 390, "FAST - SECURE - PRIVATE - INTELLIGENT", COLOR_CYAN_PULSE, 2);

        } else {
            // ==========================================
            // STAGE 4: TRANSITION TO SHELL (2.5s -> 2.8s)
            // ==========================================
            let progress = ((ms.min(2800) - 2500) * 1000) / 300; // 0..1000
            let start_cx: usize = 540;
            let start_cy: usize = 1100;
            let target_cx: usize = 60;
            let target_cy: usize = 45;

            let cur_cx = start_cx.saturating_sub(((start_cx - target_cx) * progress as usize) / 1000);
            let cur_cy = start_cy.saturating_sub(((start_cy - target_cy) * progress as usize) / 1000);
            let icon_size = (120usize).saturating_sub((90 * progress as usize) / 1000).max(24);

            painter.fill_rect(0, 0, HANOIP_PANEL_WIDTH, HANOIP_PANEL_HEIGHT, COLOR_BG_IGNITION);

            // Shrunk morphing icon
            painter.fill_rect(cur_cx, cur_cy, icon_size, icon_size, COLOR_ORANGE_BRAND);
            painter.fill_rect(cur_cx + 4, cur_cy + 4, icon_size.saturating_sub(8), icon_size.saturating_sub(8), COLOR_BG_IGNITION);
            painter.fill_rect(cur_cx + 8, cur_cy + 8, icon_size.saturating_sub(16), icon_size.saturating_sub(16), COLOR_CYAN_PULSE);

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
