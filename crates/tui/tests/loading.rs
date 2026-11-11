#[allow(dead_code)]
mod support;

use multi_link_tui::{Fade, LoadPhase, LoadingOptions, render_loading, theme::Theme};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
};
use std::{ffi::OsStr, fs, path::PathBuf};

fn options() -> LoadingOptions {
    LoadingOptions {
        time: 4.0,
        phase: LoadPhase::Reading,
        files_done: 612,
        files_total: 871,
        fade: Fade::None,
        theme: Theme::from_env(Some("truecolor"), None),
    }
}
fn render(o: &LoadingOptions, w: u16, h: u16) -> Buffer {
    let mut b = Buffer::empty(Rect::new(0, 0, w, h));
    render_loading(o, b.area, &mut b);
    b
}
fn text(b: &Buffer) -> String {
    (b.area.y..b.area.bottom())
        .map(|y| {
            let line: String = (b.area.x..b.area.right())
                .map(|x| b[(x, y)].symbol())
                .collect();
            format!("{}\n", line.trim_end())
        })
        .collect()
}
fn golden(b: &Buffer, name: &str, glyphs: bool) {
    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/expected/tui")
        .join(name);
    if glyphs {
        assert_eq!(
            text(b),
            fs::read_to_string(base.with_extension("txt")).unwrap(),
            "text {name}"
        );
    }
    // Do not use the snapshot updater: these files are immutable answer keys.
    assert_eq!(
        support::dump_styles(b),
        fs::read_to_string(base.with_extension("styles.txt")).unwrap(),
        "styles {name}"
    );
}
#[test]
fn all_loading_goldens() {
    for (w, h) in [
        (120, 40),
        (76, 22),
        (60, 16),
        (59, 16),
        (50, 12),
        (29, 6),
        (28, 5),
    ] {
        golden(&render(&options(), w, h), &format!("loading-{w}x{h}"), true);
    }
    let mut o = options();
    o.theme = Theme::from_env(None, None);
    golden(&render(&o, 76, 22), "loading-76x22-256", false);
    o.theme = Theme::from_env(None, Some(OsStr::new("1")));
    golden(&render(&o, 76, 22), "loading-76x22-nocolor", true);
    for (suffix, phase, done, total, fade) in [
        ("ready", LoadPhase::Done, 612, 871, Fade::None),
        ("linking", LoadPhase::Linking, 612, 871, Fade::None),
        ("zero", LoadPhase::Reading, 0, 0, Fade::None),
        ("fade1", LoadPhase::Done, 612, 871, Fade::School),
    ] {
        let o = LoadingOptions {
            phase,
            files_done: done,
            files_total: total,
            fade,
            ..options()
        };
        golden(
            &render(&o, 76, 22),
            &format!("loading-76x22-{suffix}"),
            true,
        );
    }
}
#[test]
fn two_hundred_frames_use_only_f6_roles_and_no_color_preserves_glyphs() {
    for frame in 0..200 {
        let o = LoadingOptions {
            time: frame as f64 * 0.05,
            ..options()
        };
        let b = render(&o, 76, 22);
        let t = o.theme;
        for (i, c) in b.content.iter().enumerate() {
            let y = i / 76;
            if y < 15 {
                assert!(matches!(
                    c.symbol(),
                    " " | "." | "·" | ">" | "<" | "›" | "‹" | "=" | "-" | "*"
                ));
            }
            assert!(
                [
                    Color::Reset,
                    t.claude,
                    t.codex,
                    t.claude_dim,
                    t.codex_dim,
                    t.text,
                    t.secondary,
                    t.border
                ]
                .contains(&c.fg)
            );
            assert_eq!(c.bg, t.screen);
            assert!((c.modifier & !Modifier::BOLD).is_empty());
        }
        let plain = render(
            &LoadingOptions {
                theme: Theme::from_env(None, Some(OsStr::new("1"))),
                ..o
            },
            76,
            22,
        );
        assert_eq!(text(&b), text(&plain));
        for c in plain.content {
            assert_eq!(
                (c.fg, c.bg, c.modifier),
                (Color::Reset, Color::Reset, Modifier::empty())
            );
        }
    }
}
#[test]
fn sizes_origins_redraw_and_six_digit_counts_are_bounded() {
    for w in [0, 1, 29, 30, 59, 60, 76] {
        for h in [0, 1, 5, 6, 15, 16, 22] {
            for no_color in [false, true] {
                let o = LoadingOptions {
                    files_done: 123456,
                    files_total: 654321,
                    theme: Theme::from_env(Some("truecolor"), no_color.then_some(OsStr::new("1"))),
                    ..options()
                };
                let want = render(&o, w, h);
                let mut b = Buffer::filled(
                    Rect::new(0, 0, w + 6, h + 8),
                    ratatui::buffer::Cell::new("X"),
                );
                b.set_style(
                    b.area,
                    Style::default()
                        .fg(Color::Red)
                        .add_modifier(Modifier::BOLD | Modifier::ITALIC),
                );
                let before = b.clone();
                let area = Rect::new(3, 4, w, h);
                render_loading(&o, area, &mut b);
                for y in 0..b.area.height {
                    for x in 0..b.area.width {
                        if area.contains((x, y).into()) {
                            assert_eq!(b[(x, y)], want[(x - 3, y - 4)]);
                        } else {
                            assert_eq!(b[(x, y)], before[(x, y)]);
                        }
                    }
                }
            }
        }
    }
    let b = render(
        &LoadingOptions {
            files_done: 123456,
            files_total: 654321,
            ..options()
        },
        60,
        16,
    );
    assert_eq!(
        text(&b).lines().nth(13).unwrap().trim(),
        "reading transcripts  123456 / 654321"
    );
}
#[test]
fn listing_padding_bar_rounding_and_second_fade() {
    let listing = render(
        &LoadingOptions {
            phase: LoadPhase::Listing,
            ..options()
        },
        76,
        22,
    );
    assert_eq!(
        text(&listing).lines().nth(19).unwrap().trim(),
        "listing transcripts…"
    );
    let b = render(
        &LoadingOptions {
            files_done: 1,
            files_total: 12,
            ..options()
        },
        76,
        22,
    );
    assert_eq!(
        text(&b).lines().nth(19).unwrap().trim(),
        "reading transcripts   1 / 12"
    );
    assert_eq!(
        text(&b).lines().nth(17).unwrap().trim(),
        format!("{}{}", "━".repeat(3), "─".repeat(27))
    );
    let dim = render(
        &LoadingOptions {
            fade: Fade::Text,
            ..options()
        },
        76,
        22,
    );
    let normal = render(&options(), 76, 22);
    assert_eq!(text(&dim), text(&normal));
    for y in 0..15 {
        for x in 0..76 {
            if dim[(x, y)].symbol() != " " {
                assert_eq!(dim[(x, y)].fg, options().theme.secondary);
            }
            assert!(dim[(x, y)].modifier.is_empty());
        }
    }
    for y in 15..22 {
        for x in 0..76 {
            assert_eq!(dim[(x, y)], normal[(x, y)]);
        }
    }
}

#[test]
fn bar_clamps_counts_without_overflow() {
    for (done, total, filled) in [
        (0, 0, 0),
        (1, 0, 0),
        (0, 12, 0),
        (1, 12, 3),
        (12, 12, 30),
        (13, 12, 30),
        (usize::MAX, usize::MAX, 30),
        (usize::MAX, 1, 30),
    ] {
        let b = render(
            &LoadingOptions {
                files_done: done,
                files_total: total,
                ..options()
            },
            76,
            22,
        );
        assert_eq!(
            text(&b).lines().nth(17).unwrap().trim(),
            format!("{}{}", "━".repeat(filled), "─".repeat(30 - filled))
        );
    }
}
