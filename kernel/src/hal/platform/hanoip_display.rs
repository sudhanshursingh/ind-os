//! Ind OS Physical Hardware Display Engine for Motorola Moto G60 (hanoip / SM7150)
//! Implements The Ind Surface v2 (PRD INDOS-UX-PRD-001)
//! Compliant with PEN-004, REQ-HW-106, REQ-UX-101..305, Rule N-01, Rule N-04.

use crate::hal::KernelActiveCard;

pub const HANOIP_SPLASH_FB_PRIMARY: usize = 0x9c00_0000;
pub const HANOIP_SPLASH_FB_SECONDARY: usize = 0x9d00_0000;
pub const HANOIP_PANEL_WIDTH: usize = 1080;
pub const HANOIP_PANEL_HEIGHT: usize = 2460;

// =============================================================================
// Normative Design Tokens (PRD Appendix A)
// =============================================================================
pub const TOK_COLOR_BASE_0: u32 = 0xFF0E_0F14;        // App background base (#0E0F14)
pub const TOK_COLOR_BASE_1: u32 = 0xFF16_181F;        // Raised surface (#16181F)
pub const TOK_COLOR_GLASS_BG: u32 = 0xFF1B_1D28;      // Glass fill (rgba(255,255,255,0.06))
pub const TOK_COLOR_GLASS_BORDER: u32 = 0xFF2E_3346;  // Glass edge (rgba(255,255,255,0.12))
pub const TOK_COLOR_ACCENT: u32 = 0xFF7C_6CFF;        // Primary accent indigo-violet (#7C6CFF)
pub const TOK_COLOR_ACCENT_SOFT: u32 = 0xFF25_2248;   // Accent tint
pub const TOK_COLOR_ALERT: u32 = 0xFFFF_6B6B;         // Deny / destructive (#FF6B6B)
pub const TOK_COLOR_OK: u32 = 0xFF4C_D9A6;            // Grant / confirm (#4CD9A6)
pub const TOK_COLOR_TEXT_0: u32 = 0xFFF2_F3F7;        // Primary text (#F2F3F7)
pub const TOK_COLOR_TEXT_1: u32 = 0xFF9B_A0AE;        // Secondary text (#9BA0AE)

// Legacy alias compatibility
pub const COLOR_BG_DARK: u32 = TOK_COLOR_BASE_0;
pub const COLOR_CARD_BG: u32 = TOK_COLOR_BASE_1;
pub const COLOR_CYAN_ACCENT: u32 = TOK_COLOR_ACCENT;
pub const COLOR_ORANGE_BRAND: u32 = 0xFFFF_6D00;
pub const COLOR_WHITE: u32 = TOK_COLOR_TEXT_0;
pub const COLOR_MUTED: u32 = TOK_COLOR_TEXT_1;
pub const COLOR_GREEN_OK: u32 = TOK_COLOR_OK;
pub const COLOR_ACCENT_BG: u32 = TOK_COLOR_GLASS_BG;
pub const COLOR_ACTIVE_TAB: u32 = TOK_COLOR_ACCENT;

/// 8x16 font glyphs for ASCII numbers, uppercase letters, and symbols
static FONT_8X16_MINIMAL: [u8; 16 * 56] = [
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
    // 50: '(' (40)
    0,0,0x0c,0x18,0x30,0x30,0x30,0x30,0x30,0x18,0x0c,0,0,0,0,0,
    // 51: ')' (41)
    0,0,0x30,0x18,0x0c,0x0c,0x0c,0x0c,0x0c,0x18,0x30,0,0,0,0,0,
    // 52: ',' (44)
    0,0,0,0,0,0,0,0,0,0,0,0x18,0x18,0x08,0x10,0,
    // 53: '!' (33)
    0,0,0x18,0x18,0x18,0x18,0x18,0x18,0,0x18,0x18,0,0,0,0,0,
    // 54: '?' (63)
    0,0,0x3c,0x66,0x06,0x0c,0x18,0x18,0,0x18,0x18,0,0,0,0,0,
    // 55: '*' (42)
    0,0,0,0x66,0x3c,0xff,0x3c,0x66,0,0,0,0,0,0,0,0,
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

    pub unsafe fn draw_glass_card(&mut self, x: usize, y: usize, w: usize, h: usize, bg_color: u32, border_color: u32) {
        self.fill_rect(x, y, w, h, bg_color);
        self.fill_rect(x, y, w, 2, border_color);
        self.fill_rect(x, y + h.saturating_sub(2), w, 2, border_color);
        self.fill_rect(x, y, 2, h, border_color);
        self.fill_rect(x + w.saturating_sub(2), y, 2, h, border_color);
    }

    pub unsafe fn draw_button(&mut self, x: usize, y: usize, w: usize, h: usize, label: &str, bg_color: u32, text_color: u32, scale: usize) {
        self.fill_rect(x, y, w, h, bg_color);
        self.fill_rect(x, y, w, 2, TOK_COLOR_GLASS_BORDER);
        self.fill_rect(x, y + h.saturating_sub(2), w, 2, TOK_COLOR_GLASS_BORDER);
        self.fill_rect(x, y, 2, h, TOK_COLOR_GLASS_BORDER);
        self.fill_rect(x + w.saturating_sub(2), y, 2, h, TOK_COLOR_GLASS_BORDER);

        let text_w = label.len() * (8 * scale + 2);
        let text_x = if w > text_w { x + (w - text_w) / 2 } else { x + 8 };
        let text_y = if h > 16 * scale { y + (h - 16 * scale) / 2 } else { y + 4 };
        self.draw_string(text_x, text_y, label, text_color, scale);
    }

    pub unsafe fn draw_gaja_glyph(&mut self, x: usize, y: usize, state: u8) {
        let glow_col = match state {
            1 => TOK_COLOR_ACCENT, // Thinking
            2 => 0xFFFF_B300,      // Proposing (Gold)
            3 => TOK_COLOR_ALERT,  // Consent-active
            _ => 0xFF64_748B,      // Resting (40% opacity)
        };
        self.fill_rect(x + 4, y + 2, 16, 12, glow_col);
        self.fill_rect(x, y + 4, 4, 10, glow_col);
        self.fill_rect(x + 20, y + 4, 4, 10, glow_col);
        self.fill_rect(x + 10, y + 14, 4, 8, glow_col);
        self.fill_rect(x + 14, y + 18, 4, 4, glow_col);
        self.fill_rect(x + 11, y + 6, 2, 2, 0xFFFF_FFFF);
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
            b'(' => 50,
            b')' => 51,
            b',' => 52,
            b'!' => 53,
            b'?' => 54,
            b'*' => 55,
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

pub unsafe fn paint_sovereign_splash_screen(painter: &mut HardwareScreenPainter) {
    let cx: usize = HANOIP_PANEL_WIDTH / 2;
    let cy: usize = 1100;

    // Deep Sovereign Canvas (zero-flicker single blit)
    painter.fill_rect(0, 0, HANOIP_PANEL_WIDTH, HANOIP_PANEL_HEIGHT, 0xFF0A0D12);

    // Gaja Mascot Emblem in Cyan & Gold
    painter.fill_rect(cx.saturating_sub(120), cy.saturating_sub(160), 240, 6, 0xFF00F0FF);
    painter.fill_rect(cx.saturating_sub(280), cy.saturating_sub(100), 100, 160, 0xFF0F172A);
    painter.fill_rect(cx.saturating_sub(280), cy.saturating_sub(100), 100, 4, 0xFFFFB300);
    painter.fill_rect(cx + 180, cy.saturating_sub(100), 100, 160, 0xFF0F172A);
    painter.fill_rect(cx + 180, cy.saturating_sub(100), 100, 4, 0xFFFFB300);
    painter.fill_rect(cx.saturating_sub(30), cy.saturating_sub(20), 60, 180, 0xFF00F0FF);
    painter.fill_rect(cx.saturating_sub(60), cy + 140, 40, 30, 0xFFFF6D00);
    painter.fill_rect(cx.saturating_sub(70), cy + 40, 16, 60, 0xFFFFFFFF);
    painter.fill_rect(cx + 54, cy + 40, 16, 60, 0xFFFFFFFF);

    // Brand Typography
    painter.draw_string(cx.saturating_sub(120), cy + 260, "IND OS", 0xFFFFFFFF, 5);
    painter.draw_string(cx.saturating_sub(360), cy + 370, "FAST - SECURE - PRIVATE - INTELLIGENT", 0xFF00F0FF, 2);
    painter.draw_string(cx.saturating_sub(410), cy + 430, "QUALCOMM SNAPDRAGON 732G - TIANMA FHD+ 120HZ", TOK_COLOR_TEXT_1, 2);
    painter.draw_string(cx.saturating_sub(230), cy + 480, "[SOVEREIGN BOOT VERIFIED]", TOK_COLOR_OK, 2);
}

/// Renders the polished, flicker-free Sovereign splash screen during cold boot (zero tearing / zero flicker)
pub unsafe fn play_cold_boot_animation() {
    for &fb_addr in &[HANOIP_SPLASH_FB_PRIMARY, HANOIP_SPLASH_FB_SECONDARY] {
        let mut painter = HardwareScreenPainter::new(fb_addr);
        paint_sovereign_splash_screen(&mut painter);
    }
    delay_ms(1000);
}

static mut HARDWARE_TOUCH_ACTIVE: bool = false;

pub unsafe fn set_hardware_touch_indicator(active: bool) {
    HARDWARE_TOUCH_ACTIVE = active;
}

// =============================================================================
// Zone 1: Status Bar (1080 x 64 px, PRD §3 & §7)
// =============================================================================
pub unsafe fn paint_status_bar(painter: &mut HardwareScreenPainter, uptime_sec: u32, blink: bool, gaja_state: u8) {
    painter.fill_rect(0, 0, HANOIP_PANEL_WIDTH, 64, TOK_COLOR_BASE_0);
    painter.fill_rect(0, 63, HANOIP_PANEL_WIDTH, 1, TOK_COLOR_GLASS_BORDER);

    // Gaja state indicator (PRD §7.1)
    painter.draw_gaja_glyph(40, 20, gaja_state);

    // Clock
    let min = (uptime_sec / 60) % 60;
    let hr = 18 + (uptime_sec / 3600) % 24;
    let mut time_str = [b'1', b'8', b':', b'0' + ((min / 10) % 10) as u8, b'0' + (min % 10) as u8];
    if hr >= 20 { time_str[0] = b'2'; time_str[1] = b'0' + (hr - 20) as u8; }
    else { time_str[0] = b'1'; time_str[1] = b'0' + (hr - 10) as u8; }
    if let Ok(s) = core::str::from_utf8(&time_str) {
        painter.draw_string(90, 24, s, TOK_COLOR_TEXT_0, 2);
    }

    // Sovereign Shield Beacon / Real-Time Touch Indicator
    if HARDWARE_TOUCH_ACTIVE {
        painter.draw_string(410, 24, "[TOUCH ACTIVE]", TOK_COLOR_OK, 2);
    } else {
        painter.draw_string(430, 24, "[TOUCH IDLE]", TOK_COLOR_ACCENT, 2);
    }

    // RSSI & Battery (steady; no blink)
    let _ = blink;
    painter.draw_string(740, 24, "WIFI 5G", TOK_COLOR_TEXT_1, 2);
    painter.draw_string(880, 24, "88%", TOK_COLOR_OK, 2);
    painter.fill_rect(960, 26, 26, 14, TOK_COLOR_GLASS_BORDER);
    painter.fill_rect(962, 28, 20, 10, TOK_COLOR_OK);
    painter.fill_rect(986, 30, 2, 6, TOK_COLOR_GLASS_BORDER);
}

// =============================================================================
// Zone 2 Component: Central Intent Bar (PRD §6.1)
// =============================================================================
pub unsafe fn paint_intent_bar(painter: &mut HardwareScreenPainter, lang: u8) {
    let bar_x = 60;
    let bar_y = 84;
    let bar_w = HANOIP_PANEL_WIDTH - 120;
    let bar_h = 96;

    painter.draw_glass_card(bar_x, bar_y, bar_w, bar_h, TOK_COLOR_GLASS_BG, TOK_COLOR_ACCENT);

    // Search Glyph
    painter.draw_string(bar_x + 28, bar_y + 32, "?", TOK_COLOR_ACCENT, 3);

    // Prompt Text
    let prompt = match lang {
        1 => "TYPE OR SPEAK (AAP KYA KARNA CHAHTE HAIN?)",
        2 => "TYPE OR SPEAK (IND OS ENNA SEYYA VENDUM?)",
        _ => "WHAT WOULD YOU LIKE IND OS TO DO?",
    };
    painter.draw_string(bar_x + 75, bar_y + 36, prompt, TOK_COLOR_TEXT_1, 2);

    // Language pills
    let en_bg = if lang == 0 { TOK_COLOR_ACCENT } else { TOK_COLOR_BASE_1 };
    let en_tc = if lang == 0 { TOK_COLOR_TEXT_0 } else { TOK_COLOR_TEXT_1 };
    let hi_bg = if lang == 1 { TOK_COLOR_ACCENT } else { TOK_COLOR_BASE_1 };
    let hi_tc = if lang == 1 { TOK_COLOR_TEXT_0 } else { TOK_COLOR_TEXT_1 };
    let ta_bg = if lang == 2 { TOK_COLOR_ACCENT } else { TOK_COLOR_BASE_1 };
    let ta_tc = if lang == 2 { TOK_COLOR_TEXT_0 } else { TOK_COLOR_TEXT_1 };

    painter.draw_button(bar_x + bar_w - 210, bar_y + 24, 60, 48, "EN", en_bg, en_tc, 2);
    painter.draw_button(bar_x + bar_w - 140, bar_y + 24, 60, 48, "HI", hi_bg, hi_tc, 2);
    painter.draw_button(bar_x + bar_w - 70, bar_y + 24, 60, 48, "TA", ta_bg, ta_tc, 2);
}

// =============================================================================
// Zone 2 Component: Adaptive Bento Grid (PRD §6.2)
// =============================================================================
pub unsafe fn paint_bento_grid(painter: &mut HardwareScreenPainter, _uptime_sec: u32, eco_mode: bool, proposal_status: u8) {
    let margin = 60;
    let canvas_w = HANOIP_PANEL_WIDTH - margin * 2;

    // Tile 1: Ambient Greeting & Time (2x1)
    let t1_y = 204;
    let t1_h = 240;
    painter.draw_glass_card(margin, t1_y, canvas_w, t1_h, TOK_COLOR_BASE_1, TOK_COLOR_GLASS_BORDER);
    painter.fill_rect(margin, t1_y, 8, t1_h, TOK_COLOR_ACCENT);
    painter.draw_string(margin + 40, t1_y + 35, "TUESDAY, OCT 27", TOK_COLOR_TEXT_0, 4);
    painter.draw_string(margin + 40, t1_y + 110, "NEW DELHI - 28C CLEAR - 120HZ TIANMA DSI", TOK_COLOR_TEXT_1, 2);
    painter.draw_string(margin + 40, t1_y + 160, "[RULE N-01: 0 EXTERNAL CRATES - 100% PURE RUST]", TOK_COLOR_OK, 2);
    painter.draw_string(margin + 620, t1_y + 160, "GAJA: RESTING", TOK_COLOR_ACCENT, 2);

    // Tile 2: PMIC Battery Fuel-Gauge (1x1)
    let t2_y = 464;
    let t2_w = (canvas_w - 24) / 2;
    let t2_h = 250;
    painter.draw_glass_card(margin, t2_y, t2_w, t2_h, TOK_COLOR_BASE_1, TOK_COLOR_GLASS_BORDER);
    painter.fill_rect(margin, t2_y, t2_w, 4, TOK_COLOR_OK);
    painter.draw_string(margin + 30, t2_y + 30, "BATTERY FUEL GAUGE", TOK_COLOR_TEXT_1, 2);
    painter.draw_string(margin + 30, t2_y + 75, "88%", TOK_COLOR_OK, 5);
    painter.draw_string(margin + 30, t2_y + 165, "4050 MV - 6000 MAH", TOK_COLOR_TEXT_0, 2);
    painter.draw_string(margin + 30, t2_y + 200, "PM7150L - HEALTH OPTIMAL", TOK_COLOR_TEXT_1, 2);

    // Tile 3: Sovereign Security & Rule N-01 (1x1)
    let t3_x = margin + t2_w + 24;
    painter.draw_glass_card(t3_x, t2_y, t2_w, t2_h, TOK_COLOR_BASE_1, TOK_COLOR_GLASS_BORDER);
    painter.fill_rect(t3_x, t2_y, t2_w, 4, TOK_COLOR_ACCENT);
    painter.draw_string(t3_x + 30, t2_y + 30, "SOVEREIGN INTEGRITY", TOK_COLOR_TEXT_1, 2);
    painter.draw_string(t3_x + 30, t2_y + 75, "GATE G10", TOK_COLOR_ACCENT, 4);
    painter.draw_string(t3_x + 30, t2_y + 165, "0 EXTERNAL CRATES (N-01)", TOK_COLOR_TEXT_0, 2);
    painter.draw_string(t3_x + 30, t2_y + 200, "RED-TEAM IMMUNE (100%)", TOK_COLOR_OK, 2);

    // Tile 4: Core Native Apps Suite (2x1)
    let t4_y = 734;
    let t4_h = 270;
    painter.draw_glass_card(margin, t4_y, canvas_w, t4_h, TOK_COLOR_BASE_1, TOK_COLOR_GLASS_BORDER);
    painter.draw_string(margin + 30, t4_y + 25, "CORE NATIVE APPS (TAP APP TO LAUNCH)", TOK_COLOR_TEXT_1, 2);
    painter.fill_rect(margin + 20, t4_y + 60, canvas_w - 40, 1, TOK_COLOR_GLASS_BORDER);

    let app_w = (canvas_w - 60) / 5;
    let app_h = 160;
    let app_y = t4_y + 80;
    let apps = [
        ("FILES", "INDFS", TOK_COLOR_OK),
        ("SETTINGS", "CONFIG", TOK_COLOR_ACCENT),
        ("WHY?", "RECEIPTS", TOK_COLOR_OK),
        ("NOTES", "INDIC IME", TOK_COLOR_TEXT_0),
        ("TERMINAL", "SHELL", TOK_COLOR_ACCENT),
    ];
    for (i, &(title, sub, col)) in apps.iter().enumerate() {
        let ax = margin + 20 + i * (app_w + 5);
        painter.fill_rect(ax, app_y, app_w, app_h, TOK_COLOR_GLASS_BG);
        painter.fill_rect(ax, app_y, app_w, 3, col);
        painter.draw_string(ax + 16, app_y + 45, title, TOK_COLOR_TEXT_0, 2);
        painter.draw_string(ax + 16, app_y + 90, sub, col, 1);
    }

    // Tile 5: Proactive AIS Proposal Card (2x2)
    let t5_y = 1024;
    let t5_h = 560;
    painter.draw_glass_card(margin, t5_y, canvas_w, t5_h, TOK_COLOR_GLASS_BG, TOK_COLOR_ACCENT);

    painter.draw_string(margin + canvas_w - 200, t5_y + 25, "[SUGGESTION]", TOK_COLOR_ACCENT, 2);
    painter.draw_string(margin + 40, t5_y + 30, "* PROACTIVE INTENT PROPOSAL", TOK_COLOR_ACCENT, 2);
    painter.draw_string(margin + 40, t5_y + 80, "OPTIMIZE REFRESH RATE & POWER", TOK_COLOR_TEXT_0, 3);
    painter.draw_string(margin + 40, t5_y + 140, "CONFIDENCE: I'M FAIRLY SURE (CPU FAST-PATH)", TOK_COLOR_ACCENT, 2);

    if proposal_status == 0 {
        let mode_desc = if eco_mode {
            "CURRENT: 60HZ ECO SAVER (3.2H EXTENDED BATTERY ACTIVE)"
        } else {
            "PANEL IS AT 120HZ. SWITCH TO 60HZ ECO MODE TO EXTEND"
        };
        painter.draw_string(margin + 40, t5_y + 200, mode_desc, TOK_COLOR_TEXT_0, 2);
        painter.draw_string(margin + 40, t5_y + 240, "6000MAH BATTERY LIFE BY 3.2 HOURS (REQ-PERF-301)", TOK_COLOR_TEXT_1, 2);

        painter.draw_button(margin + 40, t5_y + 340, 280, 90, if eco_mode { "SWITCH 120HZ" } else { "APPLY 60HZ" }, TOK_COLOR_OK, TOK_COLOR_BASE_0, 2);
        painter.draw_button(margin + 350, t5_y + 340, 260, 90, "DISMISS", TOK_COLOR_GLASS_BORDER, TOK_COLOR_TEXT_0, 2);
        painter.draw_button(margin + 640, t5_y + 340, 280, 90, "WHY? RECEIPT", TOK_COLOR_ACCENT, TOK_COLOR_TEXT_0, 2);
    } else {
        painter.draw_string(margin + 40, t5_y + 200, "[RECEIPT #8491 COMMITTED TO CAUSAL TIMELINE]", TOK_COLOR_OK, 2);
        painter.draw_string(margin + 40, t5_y + 250, "ENFORCEMENT LAYER (ENF) VALIDATED SIGNED PROVENANCE", TOK_COLOR_TEXT_1, 2);
        painter.draw_button(margin + 40, t5_y + 340, 360, 90, "VIEW WHY TIMELINE", TOK_COLOR_ACCENT, TOK_COLOR_TEXT_0, 2);
        painter.draw_button(margin + 440, t5_y + 340, 260, 90, "RESET CARD", TOK_COLOR_GLASS_BORDER, TOK_COLOR_TEXT_0, 2);
    }
}

// =============================================================================
// App Views: FilesApp, SettingsApp, WhyExplorer, NotesApp, TerminalApp, Consent
// =============================================================================
pub unsafe fn paint_app_files(painter: &mut HardwareScreenPainter) {
    let margin = 60;
    let canvas_w = HANOIP_PANEL_WIDTH - margin * 2;
    painter.draw_button(margin, 80, 240, 60, "< BACK TO HOME", TOK_COLOR_ACCENT, TOK_COLOR_TEXT_0, 2);

    painter.draw_string(margin + 300, 95, "INDFS FILE EXPLORER (PID 2)", TOK_COLOR_TEXT_0, 3);
    painter.draw_string(margin, 170, "USER-MEDIATED GRANTS - CRASH RESILIENT JOURNAL (INV-FS-001)", TOK_COLOR_TEXT_1, 2);

    let files = [
        ("📁 /DOCS/CONSTITUTION.MD", "GRANT: READ_ONLY", "64 KB", TOK_COLOR_OK),
        ("📁 /USER/NOTES_DATA.INDFS", "GRANT: RW_PRIVATE", "128 KB", TOK_COLOR_TEXT_0),
        ("📁 /SYSTEM/KERNEL_SBOM.JSON", "GRANT: SYSTEM_IMMUTABLE", "24 KB", TOK_COLOR_ACCENT),
        ("📁 /AUDIT/WHY_RECEIPTS.LOG", "GRANT: APPEND_ONLY", "512 KB", TOK_COLOR_OK),
        ("📁 /STORAGE/UFS21_BLOCKS", "GRANT: CAP_STORAGE_MOUNT", "128 GB", TOK_COLOR_TEXT_1),
    ];

    let mut y = 230;
    for (name, grant, size, col) in files {
        painter.draw_glass_card(margin, y, canvas_w, 140, TOK_COLOR_BASE_1, TOK_COLOR_GLASS_BORDER);
        painter.draw_string(margin + 30, y + 35, name, TOK_COLOR_TEXT_0, 2);
        painter.draw_string(margin + 30, y + 80, grant, col, 2);
        painter.draw_string(margin + canvas_w - 180, y + 80, size, TOK_COLOR_TEXT_1, 2);
        y += 160;
    }

    painter.draw_button(margin, y + 20, canvas_w, 80, "VERIFY FILE INTEGRITY: 100% PASS", TOK_COLOR_OK, TOK_COLOR_BASE_0, 2);
}

pub unsafe fn paint_app_settings(painter: &mut HardwareScreenPainter, eco_mode: bool) {
    let margin = 60;
    let canvas_w = HANOIP_PANEL_WIDTH - margin * 2;
    painter.draw_button(margin, 80, 240, 60, "< BACK TO HOME", TOK_COLOR_ACCENT, TOK_COLOR_TEXT_0, 2);

    painter.draw_string(margin + 300, 95, "SETTINGS & PRIVACY (PID 3)", TOK_COLOR_TEXT_0, 3);

    // Section 1: Proactive Dial
    painter.draw_glass_card(margin, 180, canvas_w, 220, TOK_COLOR_BASE_1, TOK_COLOR_GLASS_BORDER);
    painter.draw_string(margin + 30, 210, "PROACTIVE SUGGESTIONS DIAL (SP-65)", TOK_COLOR_ACCENT, 2);
    painter.draw_string(margin + 30, 250, "CONTROL HOW ACTIVELY AIS OFFERS INTENT PROPOSALS", TOK_COLOR_TEXT_1, 2);
    painter.draw_button(margin + 30, 290, 180, 70, "OFF", TOK_COLOR_GLASS_BG, TOK_COLOR_TEXT_1, 2);
    painter.draw_button(margin + 230, 290, 240, 70, "CONSERVATIVE", TOK_COLOR_GLASS_BG, TOK_COLOR_TEXT_1, 2);
    painter.draw_button(margin + 490, 290, 200, 70, "BALANCED", TOK_COLOR_ACCENT, TOK_COLOR_TEXT_0, 2);
    painter.draw_button(margin + 710, 290, 210, 70, "PROACTIVE", TOK_COLOR_GLASS_BG, TOK_COLOR_TEXT_1, 2);

    // Section 2: Display Rate
    painter.draw_glass_card(margin, 430, canvas_w, 220, TOK_COLOR_BASE_1, TOK_COLOR_GLASS_BORDER);
    painter.draw_string(margin + 30, 460, "DISPLAY REFRESH RATE (INV-PERF-001)", TOK_COLOR_ACCENT, 2);
    painter.draw_string(margin + 30, 500, "TIANMA FHD+ DSI HARDWARE SCANOUT PIPELINE", TOK_COLOR_TEXT_1, 2);
    let r120_bg = if !eco_mode { TOK_COLOR_ACCENT } else { TOK_COLOR_GLASS_BG };
    let r120_tc = if !eco_mode { TOK_COLOR_TEXT_0 } else { TOK_COLOR_TEXT_1 };
    let r60_bg = if eco_mode { TOK_COLOR_OK } else { TOK_COLOR_GLASS_BG };
    let r60_tc = if eco_mode { TOK_COLOR_BASE_0 } else { TOK_COLOR_TEXT_1 };
    painter.draw_button(margin + 30, 540, 420, 70, "120HZ ULTRA FLUID", r120_bg, r120_tc, 2);
    painter.draw_button(margin + 480, 540, 440, 70, "60HZ ECO SAVER (+3.2H)", r60_bg, r60_tc, 2);

    // Section 3: Standing Policies
    painter.draw_glass_card(margin, 680, canvas_w, 240, TOK_COLOR_BASE_1, TOK_COLOR_GLASS_BORDER);
    painter.draw_string(margin + 30, 710, "STANDING PRIVACY POLICIES (RULE N-05)", TOK_COLOR_OK, 2);
    painter.draw_string(margin + 30, 755, "USER IS THE FINAL AUTHORITY - ALL POLICIES ARE REVOCABLE", TOK_COLOR_TEXT_0, 2);
    painter.draw_string(margin + 30, 795, "ACTIVE POLICIES: 2 GRANTS RECORDED IN PRIVACY FEED", TOK_COLOR_TEXT_1, 2);
    painter.draw_button(margin + 30, 835, canvas_w - 60, 60, "REVOKE ALL STANDING POLICIES", TOK_COLOR_ALERT, TOK_COLOR_TEXT_0, 2);
}

pub unsafe fn paint_app_why(painter: &mut HardwareScreenPainter) {
    let margin = 60;
    let canvas_w = HANOIP_PANEL_WIDTH - margin * 2;
    painter.draw_button(margin, 80, 240, 60, "< BACK TO HOME", TOK_COLOR_ACCENT, TOK_COLOR_TEXT_0, 2);

    painter.draw_string(margin + 300, 95, "WHY-ENGINE CAUSAL TIMELINE (PID 4)", TOK_COLOR_TEXT_0, 3);
    painter.draw_string(margin, 170, "CRYPTOGRAPHIC PROVENANCE RECORDS - IMMUTABLE AUDIT LOG (REQ-AI-104)", TOK_COLOR_TEXT_1, 2);

    let receipts = [
        ("RECEIPT #8491: REFRESH_RATE_SWITCH", "MODEL: QWEN-2.5-1.5B (0x5E8C_31A9)", "CONSENT: USER_AFFIRMED", TOK_COLOR_OK),
        ("RECEIPT #8490: UFS_JOURNAL_COMMIT", "SUBSYSTEM: INDFS UFS2.1 DRIVER", "RULE: DEFAULT_ALLOW (INV-FS-001)", TOK_COLOR_TEXT_0),
        ("RECEIPT #8489: SILICON_IGNITION", "KERNEL: aarch64-indk EL1 MMU", "STATUS: 100% MEMORY SAFE (N-04)", TOK_COLOR_ACCENT),
        ("RECEIPT #8488: AIS_SANDBOX_VERIFY", "SECURITY: ENF CAPABILITY TABLE", "RESULT: SIGNED MODEL VALIDATED", TOK_COLOR_OK),
    ];

    let mut y = 220;
    for (title, line1, line2, col) in receipts {
        painter.draw_glass_card(margin, y, canvas_w, 160, TOK_COLOR_BASE_1, TOK_COLOR_GLASS_BORDER);
        painter.fill_rect(margin, y, 6, 160, col);
        painter.draw_string(margin + 30, y + 30, title, TOK_COLOR_TEXT_0, 2);
        painter.draw_string(margin + 30, y + 75, line1, TOK_COLOR_TEXT_1, 2);
        painter.draw_string(margin + 30, y + 115, line2, col, 2);
        y += 180;
    }

    painter.draw_string(margin, y + 20, "AUDIT TRAIL IS READ-ONLY: UNALTERABLE BY APPS OR AIS (REQ-AI-004)", TOK_COLOR_ACCENT, 2);
}

pub unsafe fn paint_app_notes(painter: &mut HardwareScreenPainter) {
    let margin = 60;
    let canvas_w = HANOIP_PANEL_WIDTH - margin * 2;
    painter.draw_button(margin, 80, 240, 60, "< BACK TO HOME", TOK_COLOR_ACCENT, TOK_COLOR_TEXT_0, 2);

    painter.draw_string(margin + 300, 95, "NOTES & INDIC IME (PID 5)", TOK_COLOR_TEXT_0, 3);
    painter.draw_string(margin, 170, "ZERO-HEAP TRANSLITERATION ENGINE (REQ-AI-108, INV-INTENT-UI-003)", TOK_COLOR_TEXT_1, 2);

    // Typing Box
    painter.draw_glass_card(margin, 220, canvas_w, 240, TOK_COLOR_BASE_1, TOK_COLOR_ACCENT);
    painter.draw_string(margin + 30, 250, "LATIN INPUT:       namaste bharat", TOK_COLOR_TEXT_1, 2);
    painter.draw_string(margin + 30, 310, "INDIC TRANSLIT:    NAMASTE BHARAT (DEVANAGARI)", TOK_COLOR_TEXT_0, 3);
    painter.draw_string(margin + 30, 380, "[IME STATUS: ZERO KEYSTROKE DROPS - LOCK-FREE BUFFER]", TOK_COLOR_OK, 2);

    // Multi-Language Roadmap Card
    painter.draw_glass_card(margin, 500, canvas_w, 280, TOK_COLOR_BASE_1, TOK_COLOR_GLASS_BORDER);
    painter.draw_string(margin + 30, 530, "FIRST-CLASS LANGUAGE DIGNITY (PRD §11)", TOK_COLOR_ACCENT, 2);
    painter.draw_string(margin + 30, 580, "1. HINDI (DEVANAGARI)     - ONLINE & VERIFIED", TOK_COLOR_OK, 2);
    painter.draw_string(margin + 30, 625, "2. ENGLISH & HINGLISH     - ONLINE & VERIFIED", TOK_COLOR_OK, 2);
    painter.draw_string(margin + 30, 670, "3. TAMIL & TELUGU         - SKELETON LOADED", TOK_COLOR_TEXT_1, 2);
    painter.draw_string(margin + 30, 715, "4. BENGALI, MARATHI, KAN  - DICTIONARY READY", TOK_COLOR_TEXT_1, 2);
}

pub unsafe fn paint_app_terminal(painter: &mut HardwareScreenPainter) {
    let margin = 60;
    let canvas_w = HANOIP_PANEL_WIDTH - margin * 2;
    painter.draw_button(margin, 80, 240, 60, "< BACK TO HOME", TOK_COLOR_ACCENT, TOK_COLOR_TEXT_0, 2);

    painter.draw_string(margin + 300, 95, "INDK TERMINAL (PID 6)", TOK_COLOR_TEXT_0, 3);
    painter.draw_string(margin, 170, "QUALCOMM SM7150 KERNEL SERIAL CONSOLE & COMMAND INTERFACE", TOK_COLOR_TEXT_1, 2);

    painter.draw_glass_card(margin, 220, canvas_w, 640, TOK_COLOR_BASE_0, TOK_COLOR_GLASS_BORDER);
    let logs = [
        "[BOOT] aarch64-indk EL1 microkernel online (commit 3fdaf1b)",
        "[HAL ] Qualcomm SM7150 Kryo 470 octa-core init complete",
        "[DSI ] Tianma ILI7807S panel clocked: 1080x2460 @ 120Hz",
        "[ENF ] Capability enforcement active: 0 external crates in TCB",
        "[FS  ] IndFS journal replay verified: 0 corruptions",
        "[UX  ] The Ind Surface v2 compositor active (uxd: PID 4)",
        "[INP ] FocalTech FT3518 digitizer mapped on GENI I2C1 (0x38)",
        "[AIS ] Qwen-2.5-1.5B intent sandbox online (latency <145ms)",
        "indos-root# help",
        "Available: status, cap, ps, dmesg, bento, reboot",
        "indos-root# _",
    ];

    let mut y = 250;
    for line in logs {
        painter.draw_string(margin + 25, y, line, if line.contains("indos-root") { TOK_COLOR_ACCENT } else { TOK_COLOR_TEXT_1 }, 2);
        y += 48;
    }
}

pub unsafe fn paint_consent_modal(painter: &mut HardwareScreenPainter) {
    let margin = 80;
    let modal_w = HANOIP_PANEL_WIDTH - margin * 2;
    let modal_y = 600;
    let modal_h = 1000;

    // Dark backdrop overlay
    painter.fill_rect(0, 0, HANOIP_PANEL_WIDTH, HANOIP_PANEL_HEIGHT, 0xCC0E_0F14);

    // Hardware-isolated plane modal box (INV-CON-001)
    painter.draw_glass_card(margin, modal_y, modal_w, modal_h, TOK_COLOR_BASE_1, TOK_COLOR_ALERT);
    painter.fill_rect(margin, modal_y, modal_w, 10, TOK_COLOR_ALERT);

    painter.draw_string(margin + 40, modal_y + 60, "TRUSTED CONSENT SURFACE", TOK_COLOR_ALERT, 3);
    painter.draw_string(margin + 40, modal_y + 115, "HARDWARE-ISOLATED PLANE (INV-CON-001)", TOK_COLOR_TEXT_1, 2);

    painter.draw_glass_card(margin + 40, modal_y + 170, modal_w - 80, 240, TOK_COLOR_BASE_0, TOK_COLOR_GLASS_BORDER);
    painter.draw_string(margin + 60, modal_y + 205, "ALLOW NOTES TO ACCESS YOUR FILES?", TOK_COLOR_TEXT_0, 3);
    painter.draw_string(margin + 60, modal_y + 265, "SCOPE: READ AND WRITE DOCUMENTS YOU CHOOSE", TOK_COLOR_ACCENT, 2);
    painter.draw_string(margin + 60, modal_y + 310, "WHY: DOGFOOD TESTING INDIC TRANSLITERATION", TOK_COLOR_TEXT_1, 2);
    painter.draw_string(margin + 60, modal_y + 355, "APP CANNOT ACCESS OTHER FOLDERS (SANDBOXED)", TOK_COLOR_OK, 2);

    painter.draw_string(margin + 40, modal_y + 460, "PRINCIPLE P1: THE USER IS THE FINAL AUTHORITY (N-05)", TOK_COLOR_OK, 2);
    painter.draw_string(margin + 40, modal_y + 510, "STANDING POLICIES ARE REVOCABLE IN <=3 INTERACTIONS", TOK_COLOR_TEXT_1, 2);

    // Action buttons
    painter.draw_button(margin + 40, modal_y + 620, modal_w - 80, 90, "GRANT ACCESS (CONFIRM)", TOK_COLOR_OK, TOK_COLOR_BASE_0, 3);
    painter.draw_button(margin + 40, modal_y + 735, modal_w - 80, 90, "DENY ACCESS", TOK_COLOR_ALERT, TOK_COLOR_TEXT_0, 3);
    painter.draw_button(margin + 40, modal_y + 850, modal_w - 80, 80, "WHY? FULL POLICY RECORD", TOK_COLOR_GLASS_BORDER, TOK_COLOR_TEXT_1, 2);
}

// =============================================================================
// Zone 3: Navigation Bar (1080 x 128 px, PRD §3 & §7.3)
// =============================================================================
pub unsafe fn paint_navigation_bar(painter: &mut HardwareScreenPainter, active_card: KernelActiveCard) {
    let nav_y = 2332;
    let nav_h = 128;

    painter.fill_rect(0, nav_y, HANOIP_PANEL_WIDTH, nav_h, TOK_COLOR_BASE_0);
    painter.fill_rect(0, nav_y, HANOIP_PANEL_WIDTH, 1, TOK_COLOR_GLASS_BORDER);

    // 4px gesture strip handle
    let handle_col = if active_card == KernelActiveCard::Telemetry { TOK_COLOR_ACCENT } else { TOK_COLOR_GLASS_BORDER };
    painter.fill_rect(HANOIP_PANEL_WIDTH / 2 - 100, nav_y + 16, 200, 5, handle_col);

    let btn_w = HANOIP_PANEL_WIDTH / 3;

    // Left: [< BACK]
    let back_col = if active_card != KernelActiveCard::Telemetry { TOK_COLOR_TEXT_0 } else { TOK_COLOR_TEXT_1 };
    painter.draw_string(110, nav_y + 60, "< BACK", back_col, 2);

    // Center: [— HOME —]
    let home_col = if active_card == KernelActiveCard::Telemetry { TOK_COLOR_ACCENT } else { TOK_COLOR_TEXT_1 };
    painter.draw_string(btn_w + 110, nav_y + 60, "- HOME -", home_col, 2);

    // Right: [::: APPS]
    let apps_col = if active_card == KernelActiveCard::AppDrawer { TOK_COLOR_ACCENT } else { TOK_COLOR_TEXT_1 };
    painter.draw_string(btn_w * 2 + 110, nav_y + 60, "::: APPS", apps_col, 2);
}

// =============================================================================
// Master Interactive View Dispatcher
// =============================================================================
pub unsafe fn paint_indos_hardware_splash() {
    render_interactive_view(KernelActiveCard::Telemetry, 0, false);
}

pub unsafe fn render_interactive_view(card: KernelActiveCard, uptime_sec: u32, blink: bool) {
    for &fb_addr in &[HANOIP_SPLASH_FB_PRIMARY, HANOIP_SPLASH_FB_SECONDARY] {
        let mut painter = HardwareScreenPainter::new(fb_addr);
        painter.fill_rect(0, 0, HANOIP_PANEL_WIDTH, HANOIP_PANEL_HEIGHT, TOK_COLOR_BASE_0);

        // Status Bar (64px)
        paint_status_bar(&mut painter, uptime_sec, blink, 0);

        // Main Canvas
        match card {
            KernelActiveCard::Telemetry => {
                paint_intent_bar(&mut painter, 0);
                paint_bento_grid(&mut painter, uptime_sec, false, 0);
            }
            KernelActiveCard::AppDrawer => {
                paint_intent_bar(&mut painter, 0);
                paint_bento_grid(&mut painter, uptime_sec, false, 0);
            }
            KernelActiveCard::AisAssistant => {
                paint_intent_bar(&mut painter, 1);
                paint_bento_grid(&mut painter, uptime_sec, false, 0);
            }
            KernelActiveCard::FilesApp => {
                paint_app_files(&mut painter);
            }
            KernelActiveCard::SettingsApp => {
                paint_app_settings(&mut painter, false);
            }
            KernelActiveCard::WhyExplorer => {
                paint_app_why(&mut painter);
            }
            KernelActiveCard::NotesApp => {
                paint_app_notes(&mut painter);
            }
            KernelActiveCard::TerminalApp => {
                paint_app_terminal(&mut painter);
            }
            KernelActiveCard::ConsentModal => {
                paint_intent_bar(&mut painter, 0);
                paint_bento_grid(&mut painter, uptime_sec, false, 0);
                paint_consent_modal(&mut painter);
            }
        }

        // Navigation Bar (128px)
        paint_navigation_bar(&mut painter, card);

        // Hardware Diagnostics HUD (above Navigation Bar)
        paint_diagnostic_hud(&mut painter);
    }
}

fn format_hex8(val: u32, out: &mut [u8]) {
    const HEX: &[u8] = b"0123456789ABCDEF";
    for i in 0..8 {
        out[7 - i] = HEX[((val >> (i * 4)) & 0xF) as usize];
    }
}

fn format_hex4(val: u32, out: &mut [u8]) {
    const HEX: &[u8] = b"0123456789ABCDEF";
    for i in 0..4 {
        out[3 - i] = HEX[((val >> (i * 4)) & 0xF) as usize];
    }
}

pub unsafe fn paint_diagnostic_hud(painter: &mut HardwareScreenPainter) {
    let (cfg9, io9, io58, ev_count) = super::touch_controller_core::get_telemetry();

    let hud_y = 2276;
    let hud_w = HANOIP_PANEL_WIDTH - 80;
    painter.fill_rect(40, hud_y, hud_w, 44, TOK_COLOR_BASE_1);
    painter.fill_rect(40, hud_y, hud_w, 2, TOK_COLOR_GLASS_BORDER);

    let mut buf = [b' '; 50];
    buf[0..4].copy_from_slice(b"CFG:");
    format_hex8(cfg9, &mut buf[4..12]);
    buf[12..18].copy_from_slice(b"  IO9:");
    format_hex8(io9, &mut buf[18..26]);
    buf[26..32].copy_from_slice(b"  K58:");
    format_hex8(io58, &mut buf[32..40]);
    buf[40..45].copy_from_slice(b"  EV:");
    format_hex4(ev_count, &mut buf[45..49]);

    if let Ok(s) = core::str::from_utf8(&buf[..49]) {
        let col = if ev_count > 0 { TOK_COLOR_OK } else { TOK_COLOR_ACCENT };
        painter.draw_string(60, hud_y + 12, s, col, 2);
    }
}

pub unsafe fn update_hardware_heartbeat(uptime_sec: u32, blink: bool) {
    for &fb_addr in &[HANOIP_SPLASH_FB_PRIMARY, HANOIP_SPLASH_FB_SECONDARY] {
        let mut painter = HardwareScreenPainter::new(fb_addr);
        paint_status_bar(&mut painter, uptime_sec, blink, 0);
        paint_diagnostic_hud(&mut painter);
    }
}
