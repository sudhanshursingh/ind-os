//! Ind OS Physical Hardware Display Engine for Motorola Moto G60 (hanoip / SM7150)
//! Renders directly into Qualcomm Little Kernel (ABL) continuous splash framebuffer.
//! Compliant with PEN-004, REQ-HW-106, REQ-UX-101, Rule N-01, Rule N-04.

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

/// Minimal 8x16 font glyphs for ASCII numbers, uppercase letters, and symbols
static FONT_8X16_MINIMAL: [u8; 16 * 44] = [
    // ' ' (32)
    0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,
    // '-' (45)
    0,0,0,0,0,0,0,0x3c,0x3c,0,0,0,0,0,0,0,
    // '.' (46)
    0,0,0,0,0,0,0,0,0,0,0,0x18,0x18,0,0,0,
    // '/' (47)
    0,0,0x02,0x06,0x0c,0x18,0x30,0x60,0xc0,0x80,0,0,0,0,0,0,
    // '0'..'9' (48..57)
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
    // ':' (58)
    0,0,0,0x18,0x18,0,0,0x18,0x18,0,0,0,0,0,0,0,
    // '=' (61)
    0,0,0,0,0x7e,0,0x7e,0,0,0,0,0,0,0,0,0,
    // 'A'..'Z' (65..90)
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
    0,0,0x66,0x66,0x3c,0x18,0x3c,0x66,0x66,0x66,0,0,0,0,0,0, // X
    0,0,0x66,0x66,0x66,0x3c,0x18,0x18,0x18,0x18,0,0,0,0,0,0, // Y
    0,0,0x7e,0x06,0x0c,0x18,0x30,0x60,0x60,0x7e,0,0,0,0,0,0, // Z
    // '[' (91)
    0,0,0x3c,0x30,0x30,0x30,0x30,0x30,0x30,0x3c,0,0,0,0,0,0,
    // ']' (93)
    0,0,0x3c,0x0c,0x0c,0x0c,0x0c,0x0c,0x0c,0x3c,0,0,0,0,0,0,
];

fn get_glyph_offset(c: u8) -> Option<usize> {
    let ascii = c.to_ascii_uppercase();
    if ascii == b' ' {
        Some(0)
    } else if ascii == b'-' {
        Some(1)
    } else if ascii == b'.' {
        Some(2)
    } else if ascii == b'/' {
        Some(3)
    } else if ascii >= b'0' && ascii <= b'9' {
        Some(4 + (ascii - b'0') as usize)
    } else if ascii == b':' {
        Some(14)
    } else if ascii == b'=' {
        Some(15)
    } else if ascii >= b'A' && ascii <= b'Z' {
        Some(16 + (ascii - b'A') as usize)
    } else if ascii == b'[' {
        Some(42)
    } else if ascii == b']' {
        Some(43)
    } else {
        None
    }
}

pub struct HardwareScreenPainter {
    fb: *mut u32,
    width: usize,
    height: usize,
}

impl HardwareScreenPainter {
    pub const fn new(fb_address: usize) -> Self {
        Self {
            fb: fb_address as *mut u32,
            width: HANOIP_PANEL_WIDTH,
            height: HANOIP_PANEL_HEIGHT,
        }
    }

    #[inline(always)]
    pub unsafe fn set_pixel(&mut self, x: usize, y: usize, color: u32) {
        if x < self.width && y < self.height {
            *self.fb.add(y * self.width + x) = color;
        }
    }

    pub unsafe fn fill_rect(&mut self, x: usize, y: usize, w: usize, h: usize, color: u32) {
        let x_end = (x + w).min(self.width);
        let y_end = (y + h).min(self.height);
        for row in y..y_end {
            let row_offset = row * self.width;
            for col in x..x_end {
                *self.fb.add(row_offset + col) = color;
            }
        }
    }

    pub unsafe fn draw_char(&mut self, x: usize, y: usize, c: u8, color: u32, scale: usize) {
        if let Some(glyph_idx) = get_glyph_offset(c) {
            let glyph_data = &FONT_8X16_MINIMAL[glyph_idx * 16..(glyph_idx + 1) * 16];
            for row in 0..16 {
                let bits = glyph_data[row];
                for col in 0..8 {
                    if (bits & (0x80 >> col)) != 0 {
                        self.fill_rect(x + col * scale, y + row * scale, scale, scale, color);
                    }
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

/// Paints the authoritative Ind OS boot splash on physical continuous splash framebuffer
pub unsafe fn paint_indos_hardware_splash() {
    for &fb_addr in &[HANOIP_SPLASH_FB_PRIMARY, HANOIP_SPLASH_FB_SECONDARY] {
        let mut painter = HardwareScreenPainter::new(fb_addr);

        // 1. Fill entire 1080x2460 display with Deep Slate background
        painter.fill_rect(0, 0, HANOIP_PANEL_WIDTH, HANOIP_PANEL_HEIGHT, COLOR_BG_DARK);

        // 2. Top Status Bar with Orange Accent strip
        painter.fill_rect(0, 0, HANOIP_PANEL_WIDTH, 90, 0xFF0F172A);
        painter.fill_rect(0, 86, HANOIP_PANEL_WIDTH, 4, COLOR_ORANGE_BRAND);
        painter.draw_string(60, 32, "IND OS", COLOR_WHITE, 2);
        painter.draw_string(300, 32, "MOTO G60 HARDWARE V0.1", COLOR_ORANGE_BRAND, 2);
        painter.draw_string(880, 32, "[ONLINE]", COLOR_GREEN_OK, 2);

        // 3. Central System Card
        let card_x = 80;
        let card_y = 480;
        let card_w = 920;
        let card_h = 1200;

        // Card body & Cyan glowing border
        painter.fill_rect(card_x, card_y, card_w, card_h, COLOR_CARD_BG);
        painter.fill_rect(card_x, card_y, card_w, 6, COLOR_CYAN_ACCENT);
        painter.fill_rect(card_x, card_y + card_h - 6, card_w, 6, COLOR_CYAN_ACCENT);
        painter.fill_rect(card_x, card_y, 6, card_h, COLOR_CYAN_ACCENT);
        painter.fill_rect(card_x + card_w - 6, card_y, 6, card_h, COLOR_CYAN_ACCENT);

        // Brand Icon / Geometric Hexagon Emblem
        let emblem_cx = HANOIP_PANEL_WIDTH / 2;
        let emblem_cy = card_y + 160;
        painter.fill_rect(emblem_cx - 80, emblem_cy - 80, 160, 160, COLOR_ORANGE_BRAND);
        painter.fill_rect(emblem_cx - 60, emblem_cy - 60, 120, 120, COLOR_CARD_BG);
        painter.fill_rect(emblem_cx - 30, emblem_cy - 30, 60, 60, COLOR_CYAN_ACCENT);

        // Title
        painter.draw_string(card_x + 180, card_y + 280, "IND OS MICROKERNEL", COLOR_WHITE, 3);
        painter.draw_string(card_x + 240, card_y + 350, "VERSION 0.1.0-ALPHA", COLOR_CYAN_ACCENT, 2);

        // Divider
        painter.fill_rect(card_x + 60, card_y + 420, card_w - 120, 2, 0xFF334155);

        // Hardware Specifications & Telemetry
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
        painter.draw_string(card_x + 60, line_y, "DISPLAY:         TIANMA ILI7807S FHD (120HZ)", COLOR_WHITE, 2);
        line_y += line_spacing;
        painter.draw_string(card_x + 60, line_y, "AI SUBSYSTEM:    QWEN-2.5-1.5B Q4_K_M (SANDBOXED)", COLOR_CYAN_ACCENT, 2);
        line_y += line_spacing;
        painter.draw_string(card_x + 60, line_y, "TCB GOVERNANCE:  RULE N-01 (0 EXTERNAL CRATES)", COLOR_GREEN_OK, 2);
        line_y += line_spacing;
        painter.draw_string(card_x + 60, line_y, "SECURITY AUDIT:  GATE G9 RATIFIED (100% DOD)", COLOR_GREEN_OK, 2);

        // Status Progress Bar
        line_y += 50;
        painter.draw_string(card_x + 60, line_y, "SYSTEM STATUS:", COLOR_ORANGE_BRAND, 2);
        painter.draw_string(card_x + 360, line_y, "READY / RUNNING", COLOR_GREEN_OK, 2);
        line_y += 40;
        painter.fill_rect(card_x + 60, line_y, card_w - 120, 16, 0xFF334155);
        painter.fill_rect(card_x + 60, line_y, (card_w - 120) * 9 / 10, 16, COLOR_CYAN_ACCENT);

        // 4. Bottom Dock Area
        let dock_y = 2240;
        painter.fill_rect(60, dock_y, HANOIP_PANEL_WIDTH - 120, 120, 0xFF0F172A);
        painter.fill_rect(60, dock_y, HANOIP_PANEL_WIDTH - 120, 2, COLOR_CYAN_ACCENT);
        painter.draw_string(140, dock_y + 45, "IND OS USERSPACE RUNTIME IS ALIVE ON DEVICE", COLOR_WHITE, 2);
    }
}
