//! Phase 6 palette. Environment access belongs to the CLI; selection here is deterministic.
use std::ffi::OsStr;

use ratatui::{
    buffer::Buffer,
    style::{Color, Modifier, Style},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ColorMode {
    Truecolor,
    Indexed,
    NoColor,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Theme {
    pub mode: ColorMode,
    pub screen: Color,
    pub waiting_card: Color,
    pub busy_card: Color,
    pub selected: Color,
    pub text: Color,
    pub secondary: Color,
    pub claude: Color,
    pub claude_dim: Color,
    pub codex: Color,
    pub codex_dim: Color,
    pub needs_you: Color,
    pub warning: Color,
    pub track: Color,
    pub border: Color,
    pub done: Color,
    pub child: Color,
}

impl Theme {
    /// A nonempty NO_COLOR takes precedence, regardless of COLORTERM.
    pub fn from_env(colorterm: Option<&str>, no_color: Option<&OsStr>) -> Self {
        let mode = if no_color.is_some_and(|value| !value.is_empty()) {
            ColorMode::NoColor
        } else if matches!(colorterm, Some("truecolor" | "24bit")) {
            ColorMode::Truecolor
        } else {
            ColorMode::Indexed
        };
        let color = |r, g, b, index| match mode {
            ColorMode::Truecolor => Color::Rgb(r, g, b),
            ColorMode::Indexed => Color::Indexed(index),
            ColorMode::NoColor => Color::Reset,
        };
        Self {
            mode,
            screen: color(0x0E, 0x10, 0x14, 233),
            waiting_card: color(0x17, 0x15, 0x0E, 234),
            busy_card: color(0x17, 0x13, 0x0F, 234),
            selected: color(0x26, 0x2C, 0x3A, 235),
            text: color(0xD7, 0xDA, 0xE0, 253),
            secondary: color(0x9A, 0xA1, 0xAE, 248),
            claude: color(0xF2, 0xA3, 0x6B, 215),
            claude_dim: color(0x8A, 0x64, 0x48, 137),
            codex: color(0x7D, 0xB3, 0xFF, 111),
            codex_dim: color(0x4F, 0x79, 0xB3, 67),
            needs_you: color(0xE8, 0xC4, 0x68, 221),
            warning: color(0xE8, 0x6F, 0x6F, 203),
            track: color(0x22, 0x26, 0x2F, 236),
            border: color(0x2A, 0x2F, 0x3A, 238),
            done: color(0x7C, 0xCB, 0x8E, 114),
            child: color(0x12, 0x15, 0x1B, 234),
        }
    }

    /// Text uses explicit backgrounds and no terminal-dependent DIM modifier.
    pub fn text_style(self, foreground: Color, background: Color) -> Style {
        Style::default().fg(foreground).bg(background)
    }

    /// Preserve glyphs and selection while stripping colours and DIM for NO_COLOR.
    pub fn apply_no_color(self, buffer: &mut Buffer) {
        if self.mode == ColorMode::NoColor {
            for cell in &mut buffer.content {
                cell.fg = Color::Reset;
                cell.bg = Color::Reset;
                cell.modifier.remove(Modifier::DIM);
            }
        }
    }
}
