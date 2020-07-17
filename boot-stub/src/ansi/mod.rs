//! Provides facilities related to ANSI video terminals.

use core::fmt;

/// Emits a reset code when the `Display` trait is used.
pub struct Reset;

impl fmt::Display for Reset {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "\x1B[0m")
    }
}

/// Provides the eight standard colors.
#[repr(u8)]
#[derive(Copy, Clone)]
pub enum StandardColor {
    Black = 0,
    Red = 1,
    Green = 2,
    Yellow = 3,
    Blue = 4,
    Magenta = 5,
    Cyan = 6,
    White = 7,
}

impl StandardColor {
    const FG_BASE: u8 = 30;
    const BG_BASE: u8 = 40;

    pub fn to_fg_code(self) -> u8 {
        Self::FG_BASE + (self as u8)
    }

    pub fn to_bg_code(self) -> u8 {
        Self::BG_BASE + (self as u8)
    }
}

/// Emits a color code when the `Display` trait is used.
pub struct Color(StandardColor, StandardColor);

impl Color {
    /// Constructs a color code from a foreground and a background color.
    pub fn from_fg_and_bg(fg: StandardColor, bg: StandardColor) -> Self {
        Color(fg, bg)
    }
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "\x1B[{};{}m", self.0.to_fg_code(), self.1.to_bg_code())
    }
}
