#[allow(dead_code)] // This integration target only needs the dumper, not the snapshot updater.
mod support;

use multi_link_model::{LinksReport, OverviewReport};
use multi_link_tui::{
    TimeWindow, render_home, render_timeline,
    theme::{ColorMode, Theme},
};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
};
use std::{ffi::OsStr, fs, path::PathBuf};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn colors(t: Theme) -> [Color; 14] {
    [
        t.screen,
        t.waiting_card,
        t.busy_card,
        t.selected,
        t.text,
        t.secondary,
        t.claude,
        t.claude_dim,
        t.codex,
        t.codex_dim,
        t.needs_you,
        t.warning,
        t.track,
        t.border,
    ]
}

#[test]
fn environment_selection_and_exact_palette_table() {
    let rgb = Theme::from_env(Some("truecolor"), None);
    assert_eq!(rgb.mode, ColorMode::Truecolor);
    assert_eq!(rgb, Theme::from_env(Some("24bit"), None));
    let expected = [
        0x0E1014, 0x17150E, 0x17130F, 0x262C3A, 0xD7DAE0, 0x9AA1AE, 0xF2A36B, 0x8A6448, 0x7DB3FF,
        0x4F79B3, 0xE8C468, 0xE86F6F, 0x22262F, 0x2A2F3A,
    ];
    assert_eq!(
        colors(rgb),
        expected.map(|n| Color::Rgb((n >> 16) as u8, (n >> 8) as u8, n as u8))
    );
    let indexed = Theme::from_env(None, None);
    assert_eq!(indexed.mode, ColorMode::Indexed);
    assert_eq!(
        colors(indexed),
        [
            233, 234, 234, 235, 253, 248, 215, 137, 111, 67, 221, 203, 236, 238
        ]
        .map(Color::Indexed)
    );
    for value in [None, Some(""), Some("xterm-256color"), Some("TRUECOLOR")] {
        assert_eq!(Theme::from_env(value, None), indexed);
    }
    assert_eq!(
        Theme::from_env(Some("truecolor"), Some(OsStr::new(""))),
        rgb
    );
    for value in ["1", "0", "yes"] {
        let plain = Theme::from_env(Some("truecolor"), Some(OsStr::new(value)));
        assert_eq!(plain.mode, ColorMode::NoColor);
        assert_eq!(colors(plain), [Color::Reset; 14]);
    }
}

fn luminance(color: Color) -> f64 {
    let rgb = match color {
        Color::Rgb(r, g, b) => [r, g, b],
        Color::Indexed(i @ 232..=255) => [8 + 10 * (i - 232); 3],
        Color::Indexed(i @ 16..=231) => {
            let levels = [0, 95, 135, 175, 215, 255];
            let i = usize::from(i - 16);
            [levels[i / 36], levels[i / 6 % 6], levels[i % 6]]
        }
        other => panic!("not an explicit palette colour: {other:?}"),
    };
    let [r, g, b] = rgb.map(|c| {
        let c = f64::from(c) / 255.0;
        if c <= 0.04045 {
            c / 12.92
        } else {
            ((c + 0.055) / 1.055).powf(2.4)
        }
    });
    0.2126 * r + 0.7152 * g + 0.0722 * b
}

#[test]
fn every_text_pair_has_wcag_aa_contrast_in_both_palettes() {
    for t in [
        Theme::from_env(Some("truecolor"), None),
        Theme::from_env(None, None),
    ] {
        for fg in [
            t.text,
            t.secondary,
            t.claude,
            t.codex,
            t.needs_you,
            t.warning,
        ] {
            for bg in [t.screen, t.waiting_card, t.busy_card, t.selected] {
                let a = luminance(fg);
                let b = luminance(bg);
                let contrast = (a.max(b) + 0.05) / (a.min(b) + 0.05);
                assert!(contrast >= 4.5, "{fg:?} on {bg:?}: {contrast}");
                let style = t.text_style(fg, bg);
                assert_eq!(style.fg, Some(fg));
                assert_eq!(style.bg, Some(bg));
                assert!(style.add_modifier.is_empty());
            }
        }
    }
}

#[test]
fn no_color_preserves_home_and_timeline_glyphs_and_visible_tracks() {
    let report: OverviewReport = serde_json::from_str(
        &fs::read_to_string(root().join("fixtures/expected/overview.json")).unwrap(),
    )
    .unwrap();
    let links: LinksReport = serde_json::from_str(
        &fs::read_to_string(root().join("fixtures/expected/links.json")).unwrap(),
    )
    .unwrap();
    let plain = Theme::from_env(Some("24bit"), Some(OsStr::new("1")));
    for timeline in [false, true] {
        let mut buf = Buffer::empty(Rect::new(0, 0, 160, 80));
        if timeline {
            render_timeline(
                &report,
                &links,
                "/home/dev/multi-link-sandbox/repo-a",
                TimeWindow::default_for(&report),
                buf.area,
                &mut buf,
            );
        } else {
            render_home(&report, Some(0), buf.area, &mut buf);
        }
        let before = buf.clone();
        plain.apply_no_color(&mut buf);
        assert!(buf.content.iter().any(|c| c.symbol() == "·"));
        for (after, before) in buf.content.iter().zip(&before.content) {
            assert_eq!(after.symbol(), before.symbol());
            assert_eq!(after.fg, Color::Reset);
            assert_eq!(after.bg, Color::Reset);
            assert_eq!(after.modifier, before.modifier - Modifier::DIM);
        }
        let mut colored = before.clone();
        Theme::from_env(None, None).apply_no_color(&mut colored);
        assert_eq!(colored, before);
    }
}

fn parse_color(s: &str) -> Color {
    if s == "-" {
        return Color::Reset;
    }
    let value = u32::from_str_radix(s.strip_prefix('#').unwrap(), 16).unwrap();
    Color::Rgb((value >> 16) as u8, (value >> 8) as u8, value as u8)
}

#[test]
fn dumper_reproduces_the_fixed_phase6_styled_golden_byte_for_byte() {
    let path = root().join("fixtures/expected/tui/home-phase6.styles.txt");
    let styles = fs::read_to_string(&path).unwrap();
    let text = fs::read_to_string(root().join("fixtures/expected/tui/home-phase6.txt")).unwrap();
    let mut buf = Buffer::empty(Rect::new(0, 0, 120, 40));
    for (y, line) in text.lines().enumerate() {
        buf.set_string(0, y as u16, line, Style::default());
    }
    let mut covered = vec![false; buf.content.len()];
    for line in styles.lines().skip(1) {
        let fields: Vec<_> = line.split_whitespace().collect();
        assert_eq!(fields.len(), 5);
        let y: u16 = fields[0].parse().unwrap();
        let (first, last) = fields[1].split_once('-').unwrap();
        let modifiers = if fields[4] == "-" {
            Modifier::empty()
        } else {
            fields[4]
                .split('+')
                .map(|s| Modifier::from_name(s).unwrap())
                .fold(Modifier::empty(), |a, b| a | b)
        };
        for x in first.parse::<u16>().unwrap()..=last.parse().unwrap() {
            let index = usize::from(y) * 120 + usize::from(x);
            assert!(!covered[index]);
            covered[index] = true;
            let cell = &mut buf[(x, y)];
            cell.fg = parse_color(fields[2]);
            cell.bg = parse_color(fields[3]);
            cell.modifier = modifiers;
        }
    }
    assert!(covered.into_iter().all(|value| value));
    // Never update the hand-drawn target through the snapshot update path.
    assert_eq!(support::dump_styles(&buf), styles);
}

#[test]
fn dumper_normalizes_blanks_and_keeps_every_visible_modifier_and_color_kind() {
    let mut buf = Buffer::empty(Rect::new(7, 3, 3, 1));
    let all = Modifier::all();
    buf[(7, 3)].set_symbol("x").set_style(
        Style::default()
            .fg(Color::Cyan)
            .bg(Color::Indexed(236))
            .add_modifier(all),
    );
    buf[(8, 3)].set_style(
        Style::default()
            .fg(Color::Rgb(0xab, 0xcd, 0xef))
            .add_modifier(all),
    );
    buf[(9, 3)].set_style(
        Style::default().add_modifier(all - Modifier::BOLD - Modifier::DIM - Modifier::ITALIC),
    );
    assert_eq!(
        support::dump_styles(&buf),
        format!(
            "{}0 0-0 Cyan Indexed(236) BOLD+DIM+ITALIC+UNDERLINED+SLOW_BLINK+RAPID_BLINK+REVERSED+HIDDEN+CROSSED_OUT\n0 1-2 - - UNDERLINED+SLOW_BLINK+RAPID_BLINK+REVERSED+HIDDEN+CROSSED_OUT\n",
            support::HEADER
        )
    );
    buf[(7, 3)].fg = Color::Rgb(0xab, 0xcd, 0xef);
    assert!(support::dump_styles(&buf).contains("#ABCDEF"));
}
