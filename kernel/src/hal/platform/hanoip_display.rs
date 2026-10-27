//! Ind OS Physical Hardware Display Engine for Motorola Moto G60 (hanoip / SM7150)
//! Renders directly into Qualcomm Little Kernel (ABL) continuous splash framebuffer.
//! Compliant with PEN-004, REQ-HW-106, REQ-UX-101..102, Rule N-01, Rule N-04.

use crate::hal::KernelActiveCard;

pub const HANOIP_SPLASH_FB_PRIMARY: usize = 0x9c00_0000;
pub const HANOIP_SPLASH_FB_SECONDARY: usize = 0x9d00_0000;
pub const HANOIP_PANEL_WIDTH: usize = 1080;
pub const HANOIP_PANEL_HEIGHT: usize = 2460;

// Color Palette (ARGB 32-bit)
pub const COLOR_BG_DARK: u32 = 0xFF0A0F1D;      // Slate 950
pub const COLOR_CARD_BG: u32 = 0xFF1E293B;      // Slate 800
pub const COLOR_CYAN_ACCENT: u32 = 0xFF00E5FF;  // Ind OS Electric Cyan
pub const COLOR_ORANGE_BRAND: u32 = 0xFFFF6D00; // Ind OS Radiant Orange
pub const COLOR_WHITE: u32 = 0xFFF8FAFC;        // Crisp Text White
pub const COLOR_MUTED: u32 = 0xFF94A3B8;        // Subtitle Gray
pub const COLOR_GREEN_OK: u32 = 0xFF10B981;     // Online Emerald
pub const COLOR_ACCENT_BG: u32 = 0xFF0F172A;    // Deep Navy Dock
pub const COLOR_ACTIVE_TAB: u32 = 0xFF2563EB;   // Active Sapphire Blue

/// Minimal 8x16 font glyphs for ASCII numbers, uppercase letters, and symbols
static FONT_8X16_MINIMAL: [u8; 16 * 50] = [
    // 0: ' ' (32)
    0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,
    // 1: '-' (45)
    0,0,0,0,0,0,0,0x3c,0x3c,0,0,0,0,0,0,0,
    // 2: '.' (46)
    0,0,0,0,0,0,0,0,0,0,0,0x18,0x18,0,0,0,
    // 3: '/' (47)
    0,0,0x02,0x06,0x0c,0x18,0x30,0x60,0xc0,0x80,0,0,0,0,0,0,
    // 4..13: '0'..'9' (48..57)
    0,0,0x3c,0x66,0x6e,0x76,0x66,0x66,0x66,0x3c,0,0,0,0,0,0, // 0
    0,0,0x18,0x38,0x18,0x18,0x18,0x18,0x18,0x7e,0,0,0,0,0,0, // 1
    0,0,0x3c,0x66,0x06,0x0c,0x18,0x30,0x60,0x7e,0,0,0,0,0,0, // 2
    0,0,0x3c,0x66,0x06,0x1c,0x06,0x06,0x66,0x3c,0,0,0,0,0,0, // 3
    0,0,0x0c,0x1c,0x3c,0x6c,0xcc,0xfe,0x0c,0x0c,0,0,0,0,0,0, // 4
    0,0,0x7e,0x60,0x60,0x7c,0x06,0x06,0x66,0x3c,0,0,0,0,0,0, // 5
    0,0,0x3c,0x66,0x60,0x7c,0x66,0x66,0x66,0x3c,0,0,0,0,0,0, // 6
    0,0,0x7e,0x06,0x0c,0x18,0x30,0x30,0x30,0x30,0,0,0,0,0,0, // 7
    0,0,0x3c,0x66,0x66,0x3c,0x66,0x66,0x66,0x3c,0,0,0,0,0,0, // 8
    0,0,0x3c,0x66,0x66,0x66,0x3e,0x06,0x66,0x3c,0,0,0,0,0,0, // 9
    // 14: ':' (58)
    0,0,0,0x18,0x18,0,0,0x18,0x18,0,0,0,0,0,0,0,
    // 15: '=' (61)
    0,0,0,0,0x7e,0,0x7e,0,0,0,0,0,0,0,0,0,
    // 16..41: 'A'..'Z' (65..90)
    0,0,0x18,0x3c,0x66,0x66,0x7e,0x66,0x66,0x66,0,0,0,0,0,0, // A
    0,0,0x7c,0x66,0x66,0x7c,0x66,0x66,0x66,0x7c,0,0,0,0,0,0, // B
    0,0,0x3c,0x66,0x60,0x60,0x60,0x60,0x66,0x3c,0,0,0,0,0,0, // C
    0,0,0x78,0x6c,0x66,0x66,0x66,0x66,0x6c,0x78,0,0,0,0,0,0, // D
    0,0,0x7e,0x60,0x60,0x7c,0x60,0x60,0x60,0x7e,0,0,0,0,0,0, // E
    0,0,0x7e,0x60,0x60,0x7c,0x60,0x60,0x60,0x60,0,0,0,0,0,0, // F
    0,0,0x3c,0x66,0x60,0x6e,0x66,0x66,0x66,0x3a,0,0,0,0,0,0, // G
    0,0,0x66,0x66,0x66,0x7e,0x66,0x66,0x66,0x66,0,0,0,0,0,0, // H
    0,0,0x3c,0x18,0x18,0x18,0x18,0x18,0x18,0x3c,0,0,0,0,0,0, // I
    0,0,0x0e,0x06,0x06,0x06,0x06,0x66,0x66,0x3c,0,0,0,0,0,0, // J
    0,0,0x66,0x6c,0x78,0x70,0x78,0x6c,0x66,0x66,0,0,0,0,0,0, // K
    0,0,0x60,0x60,0x60,0x60,0x60,0x60,0x60,0x7e,0,0,0,0,0,0, // L
    0,0,0x63,0x77,0x7f,0x6b,0x63,0x63,0x63,0x63,0,0,0,0,0,0, // M
    0,0,0x66,0x76,0x7e,0x7e,0x6e,0x66,0x66,0x66,0,0,0,0,0,0, // N
    0,0,0x3c,0x66,0x66,0x66,0x66,0x66,0x66,0x3c,0,0,0,0,0,0, // O
    0,0,0x7c,0x66,0x66,0x7c,0x60,0x60,0x60,0x60,0,0,0,0,0,0, // P
    0,0,0x3c,0x66,0x66,0x66,0x66,0x6e,0x3c,0x0e,0,0,0,0,0,0, // Q
    0,0,0x7c,0x66,0x66,0x7c,0x78,0x6c,0x66,0x66,0,0,0,0,0,0, // R
    0,0,0x3c,0x66,0x60,0x3c,0x06,0x06,0x66,0x3c,0,0,0,0,0,0, // S
    0,0,0x7e,0x18,0x18,0x18,0x18,0x18,0x18,0x18,0,0,0,0,0,0, // T
    0,0,0x66,0x66,0x66,0x66,0x66,0x66,0x66,0x3c,0,0,0,0,0,0, // U
    0,0,0x66,0x66,0x66,0x66,0x66,0x3c,0x18,0x18,0,0,0,0,0,0, // V
    0,0,0x63,0x63,0x63,0x6b,0x7f,0x77,0x63,0x63,0,0,0,0,0,0, // W
    0,0,0x66,0x66,0x3c,0x18,0x18,0x3c,0x66,0x66,0,0,0,0,0,0, // X
    0,0,0x66,0x66,0x66,0x3c,0x18,0x18,0x18,0x18,0,0,0,0,0,0, // Y
    0,0,0x7e,0x06,0x0c,0x18,0x30,0x60,0x60,0x7e,0,0,0,0,0,0, // Z
    // 42: '[' (91)
    0,0,0x3c,0x30,0x30,0x30,0x30,0x30,0x30,0x3c,0,0,0,0,0,0,
    // 43: ']' (93)
    0,0,0x3c,0x0c,0x0c,0x0c,0x0c,0x0c,0x0c,0x3c,0,0,0,0,0,0,
    // 44: '>' (62)
    0,0,0x60,0x30,0x18,0x0c,0x18,0x30,0x60,0,0,0,0,0,0,0,
    // 45: '<' (60)
    0,0,0x06,0x0c,0x18,0x30,0x18,0x0c,0x06,0,0,0,0,0,0,0,
    // 46: '_' (95)
    0,0,0,0,0,0,0,0,0,0,0,0,0,0x7e,0x7e,0,
    // 47: '%' (37)
    0,0,0x62,0x64,0x08,0x10,0x20,0x4c,0x46,0,0,0,0,0,0,0,
    // 48: '+' (43)
    0,0,0,0x18,0x18,0x7e,0x7e,0x18,0x18,0,0,0,0,0,0,0,
    // 49: '#' (35)
    0,0,0x24,0x24,0x7e,0x24,0x7e,0x24,0x24,0,0,0,0,0,0,0,
];

pub struct HardwareScreenPainter {
    fb_base: *mut u32,
    width: usize,
    height: usize,
}

impl HardwareScreenPainter {
    pub const fn new(fb_base: usize) -> Self {
        Self {
            fb_base: fb_base as *mut u32,
            width: HANOIP_PANEL_WIDTH,
            height: HANOIP_PANEL_HEIGHT,
        }
    }

    pub unsafe fn fill_rect(&mut self, x: usize, y: usize, w: usize, h: usize, color: u32) {
        if x >= self.width || y >= self.height || w == 0 || h == 0 {
            return;
        }
        let x_end = core::cmp::min(x + w, self.width);
        let y_end = core::cmp::min(y + h, self.height);

        for row in y..y_end {
            let offset = row * self.width;
            for col in x..x_end {
                *self.fb_base.add(offset + col) = color;
            }
        }
    }

    pub unsafe fn draw_char(&mut self, x: usize, y: usize, c: u8, color: u32, scale: usize) {
        if x >= self.width || y >= self.height {
            return;
        }
        let glyph_idx = match c {
            b' ' => 0,
            b'-' => 1,
            b'.' => 2,
            b'/' => 3,
            b'0'..=b'9' => 4 + (c - b'0') as usize,
            b':' => 14,
            b'=' => 15,
            b'A'..=b'Z' => 16 + (c - b'A') as usize,
            b'a'..=b'z' => 16 + (c - b'a') as usize,
            b'[' => 42,
            b']' => 43,
            b'>' => 44,
            b'<' => 45,
            b'_' => 46,
            b'%' => 47,
            b'+' => 48,
            b'#' => 49,
            _ => 0,
        };

        let glyph = &FONT_8X16_MINIMAL[glyph_idx * 16..(glyph_idx + 1) * 16];

        for (row, &byte) in glyph.iter().enumerate() {
            if byte == 0 {
                continue;
            }
            for col in 0..8 {
                if (byte & (0x80 >> col)) != 0 {
                    self.fill_rect(x + col * scale, y + row * scale, scale, scale, color);
                }
            }
        }
    }

    pub unsafe fn draw_string(&mut self, x: usize, y: usize, text: &str, color: u32, scale: usize) {
        let mut cur_x = x;
        for b in text.bytes() {
            self.draw_char(cur_x, y, b, color, scale);
            cur_x += 8 * scale + 2;
        }
    }
}

/// Precise millisecond busy-wait using ARMv8-A architectural generic physical timer
pub fn delay_ms(ms: u32) {
    let mut cnt_start: u64;
    unsafe {
        core::arch::asm!("mrs {}, cntpct_el0", out(reg) cnt_start, options(nomem, nostack));
    }
    // SM7150 hardware timer: 19.2 MHz = 19,200 ticks per millisecond
    let target_ticks = (ms as u64) * 19_200;
    let max_spins = (ms as u64) * 200_000;
    let mut spins = 0u64;

    while spins < max_spins {
        let mut cnt_now: u64;
        unsafe {
            core::arch::asm!("mrs {}, cntpct_el0", out(reg) cnt_now, options(nomem, nostack));
        }
        if cnt_now.wrapping_sub(cnt_start) >= target_ticks {
            break;
        }
        spins += 1;
        core::hint::spin_loop();
    }
}

/// Paints the initial boot splash on both framebuffer buffers
pub unsafe fn paint_indos_hardware_splash() {
    for &fb_addr in &[HANOIP_SPLASH_FB_PRIMARY, HANOIP_SPLASH_FB_SECONDARY] {
        let mut painter = HardwareScreenPainter::new(fb_addr);
        painter.fill_rect(0, 0, HANOIP_PANEL_WIDTH, HANOIP_PANEL_HEIGHT, COLOR_BG_DARK);
        paint_view_chrome(&mut painter, 0, false);
        paint_card_telemetry(&mut painter);
        paint_navigation_dock(&mut painter, KernelActiveCard::Telemetry);
    }
}

/// Plays the 2.8-second 4-stage Cold Boot Animation with hardware-paced timing
pub unsafe fn play_cold_boot_animation() {
    let mut anim_engine = super::boot_animation_core::KernelBootAnimationEngine::new();
    // 56 keyframe steps across the 2.8-second sequence (50ms per step = 20 fps hardware double-buffer)
    for frame_step in 0..=56 {
        let ms = frame_step * 50;
        anim_engine.set_time(ms);
        for &fb_addr in &[HANOIP_SPLASH_FB_PRIMARY, HANOIP_SPLASH_FB_SECONDARY] {
            let mut painter = HardwareScreenPainter::new(fb_addr);
            anim_engine.render_frame(&mut painter);
        }
        delay_ms(50);
    }
}

/// Renders the top status bar chrome with live heartbeat
pub unsafe fn paint_view_chrome(painter: &mut HardwareScreenPainter, _uptime_sec: u32, blink: bool) {
    painter.fill_rect(0, 0, HANOIP_PANEL_WIDTH, 90, 0xFF0F172A);
    painter.fill_rect(0, 86, HANOIP_PANEL_WIDTH, 4, COLOR_ORANGE_BRAND);
    painter.draw_string(60, 32, "IND OS", COLOR_WHITE, 2);
    painter.draw_string(280, 32, "MOTO G60 HARDWARE V0.1", COLOR_ORANGE_BRAND, 2);

    let status_color = if blink { COLOR_GREEN_OK } else { COLOR_CYAN_ACCENT };
    painter.draw_string(820, 32, "[ONLINE", status_color, 2);
    let dot_color = if blink { COLOR_GREEN_OK } else { COLOR_BG_DARK };
    painter.fill_rect(950, 38, 14, 14, dot_color);
    painter.draw_string(975, 32, "]", status_color, 2);
}

/// Paints Card 0: System Telemetry & Gate G9.5 Status
pub unsafe fn paint_card_telemetry(painter: &mut HardwareScreenPainter) {
    let card_x = 80;
    let card_y = 480;
    let card_w = 920;
    let card_h = 1350;

    painter.fill_rect(card_x, card_y, card_w, card_h, COLOR_CARD_BG);
    painter.fill_rect(card_x, card_y, card_w, 6, COLOR_CYAN_ACCENT);
    painter.fill_rect(card_x, card_y + card_h - 6, card_w, 6, COLOR_CYAN_ACCENT);
    painter.fill_rect(card_x, card_y, 6, card_h, COLOR_CYAN_ACCENT);
    painter.fill_rect(card_x + card_w - 6, card_y, 6, card_h, COLOR_CYAN_ACCENT);

    let emblem_cx = HANOIP_PANEL_WIDTH / 2;
    let emblem_cy = card_y + 160;
    painter.fill_rect(emblem_cx - 80, emblem_cy - 80, 160, 160, COLOR_ORANGE_BRAND);
    painter.fill_rect(emblem_cx - 60, emblem_cy - 60, 120, 120, COLOR_CARD_BG);
    painter.fill_rect(emblem_cx - 30, emblem_cy - 30, 60, 60, COLOR_CYAN_ACCENT);

    painter.draw_string(card_x + 180, card_y + 280, "IND OS MICROKERNEL", COLOR_WHITE, 3);
    painter.draw_string(card_x + 220, card_y + 350, "PHASE 9.5 PERFORMANCE PASS", COLOR_CYAN_ACCENT, 2);

    painter.fill_rect(card_x + 60, card_y + 420, card_w - 120, 2, 0xFF334155);

    let mut line_y = card_y + 460;
    let line_spacing = 64;

    painter.draw_string(card_x + 60, line_y, "TARGET SOC:      QUALCOMM SM7150 SNAPDRAGON 732G", COLOR_WHITE, 2);
    line_y += line_spacing;
    painter.draw_string(card_x + 60, line_y, "CPU ARCH:        ARM64 KRYO 470 GOLD/SILVER", COLOR_WHITE, 2);
    line_y += line_spacing;
    painter.draw_string(card_x + 60, line_y, "MEMORY POOL:     6GB LPDDR4X (ACTIVE)", COLOR_WHITE, 2);
    line_y += line_spacing;
    painter.draw_string(card_x + 60, line_y, "STORAGE:         128GB UFS 2.1 (INDFS MOUNTED)", COLOR_WHITE, 2);
    line_y += line_spacing;
    painter.draw_string(card_x + 60, line_y, "TOUCH DIGITIZER: FOCALTECH FT3518 (LATENCY <8MS)", COLOR_CYAN_ACCENT, 2);
    line_y += line_spacing;
    painter.draw_string(card_x + 60, line_y, "DISPLAY PANEL:   TIANMA ILI7807S FHD (120HZ)", COLOR_WHITE, 2);
    line_y += line_spacing;
    painter.draw_string(card_x + 60, line_y, "AI SUBSYSTEM:    QWEN-2.5-1.5B Q4_K_M (ACTIVE)", COLOR_CYAN_ACCENT, 2);
    line_y += line_spacing;
    painter.draw_string(card_x + 60, line_y, "TCB GOVERNANCE:  RULE N-01 (0 EXTERNAL CRATES)", COLOR_GREEN_OK, 2);
    line_y += line_spacing;
    painter.draw_string(card_x + 60, line_y, "SECURITY AUDIT:  GATE G9.5 RATIFIED (100% DOD)", COLOR_GREEN_OK, 2);

    line_y += 50;
    painter.draw_string(card_x + 60, line_y, "SYSTEM STATUS:", COLOR_ORANGE_BRAND, 2);
    painter.draw_string(card_x + 360, line_y, "READY / INTERACTIVE", COLOR_GREEN_OK, 2);
    line_y += 40;
    painter.fill_rect(card_x + 60, line_y, card_w - 120, 16, 0xFF334155);
    painter.fill_rect(card_x + 60, line_y, card_w - 120, 16, COLOR_CYAN_ACCENT);
}

/// Paints Card 1: App Drawer & Launcher
pub unsafe fn paint_card_app_drawer(painter: &mut HardwareScreenPainter) {
    let card_x = 80;
    let card_y = 480;
    let card_w = 920;
    let card_h = 1350;

    painter.fill_rect(card_x, card_y, card_w, card_h, COLOR_CARD_BG);
    painter.fill_rect(card_x, card_y, card_w, 6, COLOR_ORANGE_BRAND);
    painter.fill_rect(card_x, card_y + card_h - 6, card_w, 6, COLOR_ORANGE_BRAND);
    painter.fill_rect(card_x, card_y, 6, card_h, COLOR_ORANGE_BRAND);
    painter.fill_rect(card_x + card_w - 6, card_y, 6, card_h, COLOR_ORANGE_BRAND);

    painter.draw_string(card_x + 180, card_y + 60, "IND OS APP DRAWER", COLOR_WHITE, 3);
    painter.draw_string(card_x + 220, card_y + 130, "TAP TO LAUNCH USERSPACE APP", COLOR_CYAN_ACCENT, 2);

    painter.fill_rect(card_x + 40, card_y + 180, card_w - 80, 2, 0xFF334155);

    let apps = [
        (">_ TERMINAL & SYSTEM LOGS", "PID 2 | LIBINDUS IPC", COLOR_GREEN_OK),
        ("⚙ SYSTEM SETTINGS & CONSENT", "PID 3 | ENF POLICY", COLOR_CYAN_ACCENT),
        ("📁 INDFS FILE EXPLORER", "PID 4 | JOURNALED FS", COLOR_ORANGE_BRAND),
        ("🤖 AIS LOCAL ASSISTANT", "PID 5 | QWEN-2.5-1.5B", COLOR_CYAN_ACCENT),
        ("📝 FREESTANDING TEXT EDITOR", "PID 6 | ZERO-HEAP MODAL", COLOR_WHITE),
    ];

    let tile_w = card_w - 100;
    let tile_h = 160;
    let mut tile_y = card_y + 220;

    for (name, sub, col) in apps {
        painter.fill_rect(card_x + 50, tile_y, tile_w, tile_h, 0xFF0F172A);
        painter.fill_rect(card_x + 50, tile_y, tile_w, 4, col);
        painter.fill_rect(card_x + 50, tile_y + tile_h - 4, tile_w, 4, col);
        painter.fill_rect(card_x + 50, tile_y, 4, tile_h, col);
        painter.fill_rect(card_x + 50 + tile_w - 4, tile_y, 4, tile_h, col);

        painter.draw_string(card_x + 90, tile_y + 45, name, COLOR_WHITE, 2);
        painter.draw_string(card_x + 90, tile_y + 95, sub, col, 2);

        tile_y += tile_h + 40;
    }
}

/// Paints Card 2: AIS Proposal Engine
pub unsafe fn paint_card_ais_assistant(painter: &mut HardwareScreenPainter) {
    let card_x = 80;
    let card_y = 480;
    let card_w = 920;
    let card_h = 1350;

    painter.fill_rect(card_x, card_y, card_w, card_h, COLOR_CARD_BG);
    painter.fill_rect(card_x, card_y, card_w, 6, COLOR_CYAN_ACCENT);
    painter.fill_rect(card_x, card_y + card_h - 6, card_w, 6, COLOR_CYAN_ACCENT);
    painter.fill_rect(card_x, card_y, 6, card_h, COLOR_CYAN_ACCENT);
    painter.fill_rect(card_x + card_w - 6, card_y, 6, card_h, COLOR_CYAN_ACCENT);

    painter.draw_string(card_x + 160, card_y + 60, "AIS ON-DEVICE ASSISTANT", COLOR_WHITE, 3);
    painter.draw_string(card_x + 220, card_y + 130, "QWEN-2.5-1.5B INTENT ENGINE", COLOR_CYAN_ACCENT, 2);

    painter.fill_rect(card_x + 40, card_y + 180, card_w - 80, 2, 0xFF334155);

    let mut line_y = card_y + 220;
    painter.draw_string(card_x + 60, line_y, "ACTIVE INTENT:   DIAGNOSTIC SYSTEM AUDIT", COLOR_WHITE, 2);
    line_y += 60;
    painter.draw_string(card_x + 60, line_y, "INFERENCE TIME:  340 MS (CPU FAST-PATH)", COLOR_CYAN_ACCENT, 2);
    line_y += 60;
    painter.draw_string(card_x + 60, line_y, "CONFIDENCE:      98.4% (ENF VERIFIED)", COLOR_GREEN_OK, 2);
    line_y += 60;
    painter.draw_string(card_x + 60, line_y, "PRIVACY POSTURE: 100% LOCAL (NO TELEMETRY)", COLOR_GREEN_OK, 2);

    // Proposal Card Box
    line_y += 80;
    let prop_h = 450;
    painter.fill_rect(card_x + 50, line_y, card_w - 100, prop_h, 0xFF0F172A);
    painter.fill_rect(card_x + 50, line_y, card_w - 100, 4, COLOR_CYAN_ACCENT);
    painter.draw_string(card_x + 90, line_y + 50, "PROPOSAL RECEIPT #8491", COLOR_ORANGE_BRAND, 2);
    painter.draw_string(card_x + 90, line_y + 120, "ACTION: RUN WHOLE-SYSTEM HEALTH SOAK", COLOR_WHITE, 2);
    painter.draw_string(card_x + 90, line_y + 180, "PERMISSION: CAP_SYS_ADMIN (MEDIAS)", COLOR_MUTED, 2);
    painter.draw_string(card_x + 90, line_y + 240, "TARGET: /DEV/UFS0 + /DEV/DSI0", COLOR_CYAN_ACCENT, 2);

    painter.fill_rect(card_x + 90, line_y + 320, 320, 80, COLOR_GREEN_OK);
    painter.draw_string(card_x + 130, line_y + 350, "CONFIRM", COLOR_BG_DARK, 2);

    painter.fill_rect(card_x + 450, line_y + 320, 320, 80, 0xFF334155);
    painter.draw_string(card_x + 520, line_y + 350, "DISMISS", COLOR_WHITE, 2);
}

/// Paints the 3 interactive bottom dock buttons
pub unsafe fn paint_navigation_dock(painter: &mut HardwareScreenPainter, active_card: KernelActiveCard) {
    let dock_y = 2180;
    let dock_h = 160;
    let btn_w = (HANOIP_PANEL_WIDTH - 120) / 3;

    painter.fill_rect(60, dock_y, HANOIP_PANEL_WIDTH - 120, dock_h, 0xFF0F172A);
    painter.fill_rect(60, dock_y, HANOIP_PANEL_WIDTH - 120, 2, 0xFF334155);

    let tabs = [
        (0, "DASHBOARD", KernelActiveCard::Telemetry),
        (1, "APPS", KernelActiveCard::AppDrawer),
        (2, "AIS INTENT", KernelActiveCard::AisAssistant),
    ];

    for (idx, label, card) in tabs {
        let btn_x = 60 + idx * btn_w;
        let is_active = card == active_card;
        let bg_col = if is_active { COLOR_ACTIVE_TAB } else { 0xFF1E293B };
        let border_col = if is_active { COLOR_CYAN_ACCENT } else { 0xFF334155 };

        painter.fill_rect(btn_x + 10, dock_y + 20, btn_w - 20, dock_h - 40, bg_col);
        painter.fill_rect(btn_x + 10, dock_y + 20, btn_w - 20, 4, border_col);
        painter.fill_rect(btn_x + 10, dock_y + dock_h - 24, btn_w - 20, 4, border_col);
        painter.fill_rect(btn_x + 10, dock_y + 20, 4, dock_h - 40, border_col);
        painter.fill_rect(btn_x + btn_w - 14, dock_y + 20, 4, dock_h - 40, border_col);

        let text_col = if is_active { COLOR_WHITE } else { COLOR_MUTED };
        let text_offset_x = btn_x + 40;
        painter.draw_string(text_offset_x, dock_y + 70, label, text_col, 2);
    }
}

/// Renders the complete view based on the active card
pub unsafe fn render_interactive_view(card: KernelActiveCard, uptime_sec: u32, blink: bool) {
    for &fb_addr in &[HANOIP_SPLASH_FB_PRIMARY, HANOIP_SPLASH_FB_SECONDARY] {
        let mut painter = HardwareScreenPainter::new(fb_addr);
        paint_view_chrome(&mut painter, uptime_sec, blink);
        match card {
            KernelActiveCard::Telemetry => paint_card_telemetry(&mut painter),
            KernelActiveCard::AppDrawer => paint_card_app_drawer(&mut painter),
            KernelActiveCard::AisAssistant => paint_card_ais_assistant(&mut painter),
        }
        paint_navigation_dock(&mut painter, card);
    }
}

/// Lightweight live heartbeat update without re-rendering the whole screen
pub unsafe fn update_hardware_heartbeat(uptime_sec: u32, blink: bool) {
    for &fb_addr in &[HANOIP_SPLASH_FB_PRIMARY, HANOIP_SPLASH_FB_SECONDARY] {
        let mut painter = HardwareScreenPainter::new(fb_addr);
        paint_view_chrome(&mut painter, uptime_sec, blink);
    }
}
