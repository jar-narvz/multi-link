#[allow(dead_code)]
mod support;

use multi_link_model::LiveReport;
use multi_link_tui::{LiveOptions, render_live, theme::Theme};
use ratatui::{buffer::Buffer, layout::Rect, text::Span};
use std::{fs, path::PathBuf};
use time::UtcOffset;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}
fn report() -> LiveReport {
    serde_json::from_str(
        &fs::read_to_string(root().join("fixtures/expected/phase14/live.json")).unwrap(),
    )
    .unwrap()
}
fn options(selected: usize, first_row: usize) -> LiveOptions {
    LiveOptions {
        selected,
        first_row,
        theme: Theme::from_env(Some("truecolor"), None),
        offset: UtcOffset::from_hms(-8, 0, 0).unwrap(),
    }
}
fn render(report: &LiveReport, options: &LiveOptions, w: u16, h: u16) -> Buffer {
    let mut buf = Buffer::empty(Rect::new(0, 0, w, h));
    render_live(report, options, buf.area, &mut buf);
    buf
}
fn text(buf: &Buffer) -> String {
    let mut out = String::new();
    for y in buf.area.y..buf.area.bottom() {
        let mut line = String::new();
        let mut x = buf.area.x;
        while x < buf.area.right() {
            let s = buf[(x, y)].symbol();
            line.push_str(s);
            x += Span::raw(s).width().max(1) as u16;
        }
        out.push_str(line.trim_end());
        out.push('\n');
    }
    out
}
#[test]
fn six_live_goldens_text_and_styles() {
    for (suffix, w, h, selected, first) in [
        ("", 120, 40, 0, 0),
        ("-100", 100, 40, 2, 1),
        ("-short", 120, 30, 0, 0),
        ("-tiny", 60, 20, 0, 0),
        ("-children", 120, 51, 6, 3),
        ("-stale", 120, 40, 11, 7),
    ] {
        let buf = render(&report(), &options(selected, first), w, h);
        let base = root().join(format!("fixtures/expected/tui/live-phase14{suffix}"));
        assert_eq!(
            text(&buf),
            fs::read_to_string(base.with_extension("txt")).unwrap(),
            "text {suffix}"
        );
        assert_eq!(
            support::dump_styles(&buf),
            fs::read_to_string(base.with_extension("styles.txt")).unwrap(),
            "styles {suffix}"
        );
    }
}

#[test]
fn waiting_card_without_selection_uses_waiting_background_and_light_border() {
    let buf = render(&report(), &options(1, 0), 120, 40);
    let theme = options(1, 0).theme;
    assert_eq!(buf[(1, 8)].symbol(), "╭");
    assert_eq!(buf[(1, 8)].fg, theme.needs_you);
    assert_eq!(buf[(1, 8)].bg, theme.waiting_card);
    assert!(
        !buf[(1, 8)]
            .modifier
            .contains(ratatui::style::Modifier::BOLD)
    );
    assert!(
        buf[(3, 10)]
            .modifier
            .contains(ratatui::style::Modifier::BOLD)
    );
}

#[test]
fn claude_handoff_link_variants_and_untagged_notable() {
    use multi_link_model::{CardLink, LiveOutcome, Match};
    let mut r = report();
    for (m, child, outcome, expected, color) in [
        (
            Match::Ambiguous,
            None,
            None,
            "2 possible runs",
            options(0, 0).theme.needs_you,
        ),
        (
            Match::Unlinked,
            None,
            None,
            "no run found",
            options(0, 0).theme.secondary,
        ),
        (
            Match::Exact,
            Some("abcdef12-3456".into()),
            None,
            "codex abcdef12",
            options(0, 0).theme.secondary,
        ),
        (
            Match::Exact,
            Some("abcdef12-3456".into()),
            Some(LiveOutcome::Running),
            "codex abcdef12",
            options(0, 0).theme.secondary,
        ),
    ] {
        r.groups[0].head.link = Some(CardLink::Handoff {
            at: r.now,
            r#match: m,
            child,
            candidates: 2,
            outcome,
            outcome_at: None,
        });
        r.groups[0].head.notable[0].label = "▸ codex exec".into();
        let buf = render(&r, &options(0, 0), 120, 40);
        assert_eq!(
            text(&buf).lines().nth(16).unwrap().trim_matches([' ', '┃']),
            expected
        );
        assert_eq!(buf[(3, 16)].fg, color);
        assert!(
            text(&buf)
                .lines()
                .nth(13)
                .unwrap()
                .contains("1:35pm  ▸ codex exec ")
        );
        assert_eq!(buf[(11, 13)].fg, options(0, 0).theme.claude);
    }
}

#[test]
fn wide_and_combining_prompt_is_cut_at_display_width_with_ellipsis() {
    let mut r = report();
    r.groups[0].head.last_asked = Some(format!("e\u{301}{}", "界".repeat(40)));
    let buf = render(&r, &options(0, 0), 60, 20);
    let row = text(&buf).lines().nth(4).unwrap().to_string();
    assert_eq!(row, format!(" ┃ “e\u{301}{}…  ┃", "界".repeat(25)));
    assert_eq!(Span::raw(&row).width(), 59);
    assert_eq!(buf[(3, 4)].symbol(), "“");
    assert_eq!(buf[(4, 4)].symbol(), "e\u{301}");
    assert_eq!(buf[(55, 4)].symbol(), "…");
    assert_eq!(buf[(58, 4)].symbol(), "┃");
}

#[test]
fn no_color_keeps_all_glyphs_and_words_in_each_layout() {
    use std::ffi::OsStr;
    for (w, h, selected, first) in [
        (120, 40, 0, 0),
        (100, 40, 2, 1),
        (60, 20, 0, 0),
        (120, 51, 6, 3),
        (120, 40, 11, 7),
    ] {
        let o = options(selected, first);
        let colored = render(&report(), &o, w, h);
        let plain = render(
            &report(),
            &LiveOptions {
                theme: Theme::from_env(Some("truecolor"), Some(OsStr::new("1"))),
                ..o
            },
            w,
            h,
        );
        assert_eq!(text(&plain), text(&colored));
        for cell in plain.content {
            assert_eq!(cell.fg, ratatui::style::Color::Reset);
            assert_eq!(cell.bg, ratatui::style::Color::Reset);
        }
    }
}

#[test]
fn empty_report_and_feed_have_explicit_messages() {
    let mut r = report();
    r.groups.clear();
    r.feed.clear();
    r.idle.count = 0;
    r.counts = Default::default();
    let buf = render(&r, &options(0, 0), 120, 40);
    assert_eq!(
        text(&buf).lines().nth(3).unwrap(),
        " no hand-offs in the last 2h"
    );
    assert_eq!(
        text(&buf).lines().nth(8).unwrap(),
        " nothing working right now · last hand-off 2:10pm"
    );
    r.window_from = None;
    r.last_handoff = None;
    let buf = render(&r, &options(0, 0), 120, 40);
    assert_eq!(text(&buf).lines().nth(3).unwrap(), " no hand-offs");
    assert_eq!(
        text(&buf).lines().nth(8).unwrap(),
        " nothing working right now"
    );
}

#[test]
fn worktree_place_and_promptless_card() {
    let mut r = report();
    r.groups[0].head.checkout = "/home/dev/repo-n-fix".into();
    r.groups[0].head.last_asked = None;
    let buf = render(&r, &options(0, 0), 120, 40);
    let rows = text(&buf);
    assert!(
        rows.lines()
            .nth(17)
            .unwrap()
            .contains("repo-n-fix worktree · 14130000")
    );
    assert!(
        rows.lines()
            .nth(32)
            .unwrap()
            .contains("repo-n, repo-n-fix worktree · started")
    );
    assert_eq!(
        rows.lines().nth(9).unwrap().trim_matches([' ', '┃']),
        "no prompt"
    );
}

#[test]
fn render_is_bounded_for_small_and_offset_areas() {
    use ratatui::style::Color;
    let r = report();
    for w in 0..12 {
        for h in 0..16 {
            let _ = render(&r, &options(0, 0), w, h);
        }
    }
    let mut buf = Buffer::empty(Rect::new(0, 0, 130, 50));
    buf[(0, 0)].set_symbol("X").set_fg(Color::Red);
    let area = Rect::new(3, 4, 120, 40);
    render_live(&r, &options(0, 0), area, &mut buf);
    assert_eq!(buf[(0, 0)].symbol(), "X");
    assert_eq!(buf[(0, 0)].fg, Color::Red);
    let plain = render(&r, &options(0, 0), 120, 40);
    for y in 0..40 {
        for x in 0..120 {
            assert_eq!(buf[(x + 3, y + 4)], plain[(x, y)]);
        }
    }
}

#[test]
fn fixture_wide_prompt_cuts_without_splitting_a_character() {
    let r = report();
    let buf = render(&r, &options(10, 10), 40, 30);
    assert_eq!(
        text(&buf).lines().nth(4).unwrap(),
        "     ┃ “Step 2.6: 表示幅で切る — cut… ┃"
    );
}

#[test]
fn no_color_respects_the_render_area() {
    use ratatui::style::Color;
    use std::ffi::OsStr;
    let mut buf = Buffer::empty(Rect::new(0, 0, 122, 42));
    buf[(0, 0)].set_symbol("X").set_fg(Color::Red);
    let o = LiveOptions {
        theme: Theme::from_env(None, Some(OsStr::new("1"))),
        ..options(0, 0)
    };
    render_live(&report(), &o, Rect::new(1, 1, 120, 40), &mut buf);
    assert_eq!(buf[(0, 0)].fg, Color::Red);
}

#[test]
fn clipped_selected_child_keeps_its_indent_and_connector() {
    let buf = render(&report(), &options(2, 2), 60, 20);
    let rows = text(&buf);
    assert!(rows.lines().nth(3).unwrap().starts_with("     ┏━● codex"));
    assert!(rows.lines().nth(7).unwrap().starts_with("  └▸ ┃"));
    assert_eq!(rows.lines().nth(9).unwrap(), " ↓ 10 more");
}

#[test]
fn redraw_replaces_previous_cells_and_modifiers() {
    use ratatui::style::{Color, Modifier, Style};
    let r = report();
    let o = options(0, 0);
    let mut buf = render(&r, &options(2, 1), 120, 40);
    buf.set_style(
        buf.area,
        Style::default()
            .fg(Color::Red)
            .add_modifier(Modifier::BOLD | Modifier::DIM | Modifier::ITALIC),
    );
    render_live(&r, &o, buf.area, &mut buf);
    assert_eq!(buf, render(&r, &o, 120, 40));
}
