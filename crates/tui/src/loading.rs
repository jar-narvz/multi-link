//! Pure first-load presentation. The CLI translates scanner progress into these inputs.
use crate::theme::{ColorMode, Theme};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LoadPhase {
    Listing,
    Reading,
    Linking,
    Done,
}

/// The caller selects a fade stage from its deadline; rendering never reads a clock.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fade {
    None,
    School,
    Text,
}

#[derive(Clone, Copy, Debug)]
pub struct LoadingOptions {
    /// Seconds since loading began (the board's drawC time, without its +1 offset).
    pub time: f64,
    pub phase: LoadPhase,
    pub files_done: usize,
    pub files_total: usize,
    pub fade: Fade,
    pub theme: Theme,
}

/// Replace only `area`, including old glyphs and modifiers, at any origin.
pub fn render_loading(options: &LoadingOptions, area: Rect, buf: &mut Buffer) {
    if area.is_empty() {
        return;
    }
    let mut scene = Scene { options, area, buf };
    for y in 0..area.height {
        for x in 0..area.width {
            scene.put(x.into(), y.into(), " ", options.theme.text, false);
        }
    }
    let counter = match options.phase {
        LoadPhase::Listing => "listing transcripts…".to_string(),
        LoadPhase::Reading => format!(
            "reading transcripts  {:>width$} / {}",
            options.files_done,
            options.files_total,
            width = options.files_total.to_string().len()
        ),
        LoadPhase::Linking => "linking sessions…".to_string(),
        LoadPhase::Done => "ready ✓  opening home".to_string(),
    };
    if area.width < 30 || area.height < 6 {
        scene.center((area.height - 1) / 2, &counter, options.theme.text);
    } else if area.width < 60 || area.height < 16 {
        let row = (area.height - 2) / 2;
        scene.center(row, &counter, options.theme.text);
        scene.center(row + 1, "any key skips · q quits", options.theme.secondary);
    } else {
        scene.art();
        let title = "m u l t i · l i n k";
        let x = (area.width - title.chars().count() as u16) / 2;
        for (i, ch) in title.chars().enumerate() {
            let fg = match i {
                0..=8 => options.theme.claude,
                12.. => options.theme.codex,
                _ => options.theme.text,
            };
            scene.put(
                i32::from(x) + i as i32,
                i32::from(area.height - 7),
                &ch.to_string(),
                fg,
                true,
            );
        }
        // Widen before multiplying, and clamp first so even usize::MAX counts are safe.
        let filled = match options.phase {
            LoadPhase::Listing => 0,
            LoadPhase::Reading if options.files_total == 0 => 0,
            LoadPhase::Reading => {
                let total = options.files_total as u128;
                ((60 * options.files_done.min(options.files_total) as u128 + total) / (2 * total))
                    as usize
            }
            LoadPhase::Linking | LoadPhase::Done => 30,
        };
        for i in 0..30 {
            let (glyph, fg) = if i < filled {
                (
                    "━",
                    if i < 15 {
                        options.theme.claude
                    } else {
                        options.theme.codex
                    },
                )
            } else {
                ("─", options.theme.border)
            };
            scene.put(
                i32::from((area.width - 30) / 2) + i as i32,
                i32::from(area.height - 5),
                glyph,
                fg,
                false,
            );
        }
        scene.center(area.height - 3, &counter, options.theme.text);
        scene.center(
            area.height - 2,
            "any key skips · q quits",
            options.theme.secondary,
        );
    }
}

struct Scene<'a> {
    options: &'a LoadingOptions,
    area: Rect,
    buf: &'a mut Buffer,
}

impl Scene<'_> {
    fn put(&mut self, x: i32, y: i32, glyph: &str, fg: Color, bold: bool) {
        if x < 0 || y < 0 || x >= i32::from(self.area.width) || y >= i32::from(self.area.height) {
            return;
        }
        let plain = self.options.theme.mode == ColorMode::NoColor;
        let mut style = Style::default()
            .fg(if plain { Color::Reset } else { fg })
            .bg(if plain {
                Color::Reset
            } else {
                self.options.theme.screen
            });
        if bold && !plain {
            style = style.add_modifier(Modifier::BOLD);
        }
        let cell = &mut self.buf[(self.area.x + x as u16, self.area.y + y as u16)];
        cell.reset();
        cell.set_symbol(glyph).set_style(style);
    }

    fn center(&mut self, row: u16, text: &str, color: Color) {
        let width = usize::from(self.area.width);
        let line = if Span::raw(text).width() > width {
            let mut cut = String::new();
            for ch in text.chars() {
                if Span::raw(format!("{cut}{ch}…")).width() > width {
                    break;
                }
                cut.push(ch);
            }
            cut.push('…');
            cut
        } else {
            text.to_string()
        };
        let start = (width - Line::raw(&line).width()) / 2;
        let mut x = start;
        for ch in line.chars() {
            let glyph = ch.to_string();
            self.put(x as i32, i32::from(row), &glyph, color, false);
            x += Span::raw(&glyph).width();
        }
    }

    fn art(&mut self) {
        let t = self.options.time;
        let w = f64::from(self.area.width);
        let rows = self.area.height - 7;
        let a_rows = f64::from(rows);
        let theme = self.options.theme;
        for i in 0..46 {
            let speed = 0.3 + 0.12 * f64::from(i % 5);
            let x = round((37.3 * f64::from(i) % w - 2.2 * speed * (0.6 * t)).rem_euclid(w));
            let y =
                round((11.7 * f64::from(i) % a_rows - 0.5 * speed * (0.6 * t)).rem_euclid(a_rows));
            if y >= i32::from(rows) {
                continue;
            }
            let color = if self.options.fade == Fade::Text {
                theme.secondary
            } else if i % 3 == 0 {
                theme.claude_dim
            } else {
                theme.codex_dim
            };
            self.put(x, y, if speed > 0.6 { "·" } else { "." }, color, false);
        }
        // Heads persist here even when a later tail or dot overwrites the visible cell.
        let mut heads = vec![None; usize::from(self.area.width) * usize::from(rows)];
        for i in 0..96 {
            let school = i % 2;
            let ph = f64::from(i) * 2.399;
            let radius = 0.55 + f64::from((7 * i) % 10) / 22.0;
            let speed = 0.9 + f64::from((3 * i) % 7) / 20.0;
            let a = ph + 0.42 * speed * t;
            let b = -a;
            let (x, y, velocity) = if school == 0 {
                (
                    w / 2.0 + 27.0 * (w / 76.0) * radius * a.cos(),
                    7.2 * a_rows / 15.0
                        + 5.6 * (a_rows / 15.0) * radius * a.sin()
                        + 0.4 * (1.3 * t + ph).sin(),
                    -a.sin(),
                )
            } else {
                (
                    w / 2.0 + 30.0 * (w / 76.0) * radius * b.sin(),
                    7.2 * a_rows / 15.0
                        + 6.2 * (a_rows / 15.0) * radius * (2.0 * b).sin()
                        + 0.4 * (1.1 * t + ph).sin(),
                    -b.cos(),
                )
            };
            let (x, y) = (round(x), round(y));
            if y < 0 || y >= i32::from(rows) {
                continue;
            }
            let depth = 0.5 + 0.5 * (a + ph / 2.0).sin();
            let dir = if velocity >= 0.0 { 1 } else { -1 };
            let role = if school == 0 {
                theme.claude
            } else {
                theme.codex
            };
            let dim = if school == 0 {
                theme.claude_dim
            } else {
                theme.codex_dim
            };
            let (head_color, tail_color, bold) = match self.options.fade {
                Fade::Text => (theme.secondary, theme.secondary, false),
                Fade::School => (dim, dim, false),
                Fade::None => (
                    if depth >= 0.33 { role } else { dim },
                    if depth > 0.66 { role } else { dim },
                    depth > 0.66,
                ),
            };
            let meet = if x >= 0 && x < i32::from(self.area.width) {
                let idx = y as usize * usize::from(self.area.width) + x as usize;
                if heads[idx].is_some_and(|g| g != school) {
                    true
                } else {
                    heads[idx] = Some(school);
                    false
                }
            } else {
                false
            };
            if meet {
                self.put(
                    x,
                    y,
                    "*",
                    if self.options.fade == Fade::None {
                        theme.text
                    } else {
                        theme.secondary
                    },
                    self.options.fade == Fade::None,
                );
            } else {
                let glyph = match (dir, depth > 0.66) {
                    (1, true) => ">",
                    (1, false) => "›",
                    (_, true) => "<",
                    _ => "‹",
                };
                self.put(x, y, glyph, head_color, bold);
            }
            self.put(
                x - dir,
                y,
                if depth > 0.5 { "=" } else { "-" },
                tail_color,
                false,
            );
            if depth > 0.8 {
                self.put(
                    x - 2 * dir,
                    y,
                    "·",
                    if self.options.fade == Fade::Text {
                        theme.secondary
                    } else {
                        dim
                    },
                    false,
                );
            }
        }
    }
}

/// Match JavaScript Math.round, including negative half values.
fn round(value: f64) -> i32 {
    (value + 0.5).floor() as i32
}
