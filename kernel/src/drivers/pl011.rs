//! ARM PrimeCell PL011 UART Driver
//! Physical base on QEMU virt: 0x09000000
//!
//! Provides polled transmission, reception, and core::fmt::Write formatting.

use core::fmt;

pub const DEFAULT_PL011_BASE: usize = 0x09000000;

#[allow(dead_code)]
// Register Offsets
const UARTDR: usize = 0x00;
#[allow(dead_code)]
const UARTRSR: usize = 0x04;
const UARTFR: usize = 0x18;
const UARTIBRD: usize = 0x24;
const UARTFBRD: usize = 0x28;
const UARTLCR_H: usize = 0x2C;
const UARTCR: usize = 0x30;
#[allow(dead_code)]
const UARTIFLS: usize = 0x34;
const UARTIMSC: usize = 0x38;
#[allow(dead_code)]
const UARTRIS: usize = 0x3C;
#[allow(dead_code)]
const UARTMIS: usize = 0x40;
const UARTICR: usize = 0x44;

// Flag Register (UARTFR) bitmasks
const FR_BUSY: u32 = 1 << 3;
const FR_RXFE: u32 = 1 << 4; // Receive FIFO Empty
const FR_TXFF: u32 = 1 << 5; // Transmit FIFO Full
#[allow(dead_code)]
const FR_RXFF: u32 = 1 << 6; // Receive FIFO Full
#[allow(dead_code)]
const FR_TXFE: u32 = 1 << 7; // Transmit FIFO Empty

// Line Control Register (UARTLCR_H) bitmasks
const LCR_H_FEN: u32 = 1 << 4;       // Enable FIFOs
const LCR_H_WLEN_8: u32 = 3 << 5;    // 8-bit word length

// Control Register (UARTCR) bitmasks
const CR_UARTEN: u32 = 1 << 0;       // UART Enable
const CR_TXE: u32 = 1 << 8;          // Transmit Enable
const CR_RXE: u32 = 1 << 9;          // Receive Enable

/// ARM PrimeCell PL011 UART controller
pub struct Pl011Uart {
    base: usize,
    initialized: bool,
}

impl Pl011Uart {
    /// Create a new PL011 driver instance at the specified physical base address
    pub const unsafe fn new(base: usize) -> Self {
        Self {
            base,
            initialized: false,
        }
    }

    #[inline(always)]
    unsafe fn read_reg(&self, offset: usize) -> u32 {
        core::ptr::read_volatile((self.base + offset) as *const u32)
    }

    #[inline(always)]
    unsafe fn write_reg(&mut self, offset: usize, val: u32) {
        core::ptr::write_volatile((self.base + offset) as *mut u32, val);
    }

    /// Initialize the PL011 UART hardware for polled 115200 8N1 operation
    pub fn init(&mut self) {
        unsafe {
            // 1. Disable UART before reprogramming
            self.write_reg(UARTCR, 0);

            // 2. Clear all pending interrupt flags
            self.write_reg(UARTICR, 0x7FF);

            // 3. Configure baud rate divisor for 115200 baud (assuming 24 MHz clock in QEMU virt)
            // Divisor = 24_000_000 / (16 * 115200) = 13.02083
            // IBRD = 13, FBRD = round(0.02083 * 64) = 1 or 2
            self.write_reg(UARTIBRD, 13);
            self.write_reg(UARTFBRD, 2);

            // 4. Set word length 8 bits and enable 32-byte FIFOs
            self.write_reg(UARTLCR_H, LCR_H_FEN | LCR_H_WLEN_8);

            // 5. Mask all hardware interrupts (polled early console)
            self.write_reg(UARTIMSC, 0);

            // 6. Enable UART, Transmit, and Receive
            self.write_reg(UARTCR, CR_UARTEN | CR_TXE | CR_RXE);
        }
        self.initialized = true;
    }

    /// Check if the transmit FIFO is full
    #[inline(always)]
    pub fn is_tx_full(&self) -> bool {
        unsafe { (self.read_reg(UARTFR) & FR_TXFF) != 0 }
    }

    /// Check if the receive FIFO is empty
    #[inline(always)]
    pub fn is_rx_empty(&self) -> bool {
        unsafe { (self.read_reg(UARTFR) & FR_RXFE) != 0 }
    }

    /// Transmit a single byte, blocking until the TX FIFO has available space
    pub fn write_byte(&mut self, byte: u8) {
        while self.is_tx_full() {
            core::hint::spin_loop();
        }
        unsafe {
            self.write_reg(UARTDR, byte as u32);
        }
    }

    /// Attempt to read a byte from the receive FIFO without blocking
    pub fn read_byte(&mut self) -> Option<u8> {
        if self.is_rx_empty() {
            None
        } else {
            unsafe { Some((self.read_reg(UARTDR) & 0xFF) as u8) }
        }
    }

    /// Write a string slice to the serial console, expanding '\n' to '\r\n'
    pub fn write_str(&mut self, s: &str) {
        for b in s.bytes() {
            if b == b'\n' {
                self.write_byte(b'\r');
            }
            self.write_byte(b);
        }
    }

    /// Flush the transmit FIFO until transmission is completely idle
    pub fn flush(&self) {
        unsafe {
            while (self.read_reg(UARTFR) & FR_BUSY) != 0 {
                core::hint::spin_loop();
            }
        }
    }
}

impl fmt::Write for Pl011Uart {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_str(s);
        Ok(())
    }
}
