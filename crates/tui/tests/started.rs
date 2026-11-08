//! Step 8.4c (S3): sessions started by a session nest under it in the repo view, any depth.
//! Reads the answer keys for `fixtures/sessions-started`; they are never edited here.
use crossterm::event::KeyCode;
use multi_link_model::{LinksReport, OverviewReport, Row, SessionRow, State, Timestamp};
use multi_link_tui::{
    App, Effect, Event, Refresh, RepoOptions, TimeWindow, render_repo, theme::Theme, view,
};
use ratatui::{buffer::Buffer, layout::Rect, text::Span};
use time::UtcOffset;

const REPO: &str = "/home/dev/repo-k";
fn id(n: &str) -> String {
    format!("84c000{n}-0000-4000-8000-0000000084{n}")
}
fn report() -> OverviewReport {
    serde_json::from_str(include_str!(
        "../../../fixtures/expected/started/overview.json"
    ))
    .unwrap()
}
fn links() -> LinksReport {
    serde_json::from_str(include_str!(
        "../../../fixtures/expected/started/links.json"
    ))
    .unwrap()
}
fn window(r: &OverviewReport) -> TimeWindow {
    TimeWindow::new(Timestamp::parse("2026-11-08T09:00:00Z").unwrap(), r.now)
}
fn options(offset: UtcOffset) -> RepoOptions {
    RepoOptions {
        offset,
        ..RepoOptions::default()
    }
}
fn render(r: &OverviewReport, o: RepoOptions, width: u16, height: u16, sel: usize) -> Buffer {
    let mut b = Buffer::empty(Rect::new(0, 0, width, height));
    render_repo(r, &links(), REPO, window(r), Some(sel), &o, b.area, &mut b);
    b
}
fn line(b: &Buffer, y: u16) -> String {
    let mut out = String::new();
    let mut x = 0;
    while x < b.area.width {
        let s = b[(x, y)].symbol();
        out.push_str(s);
        x += Span::raw(s).width().max(1) as u16;
    }
    out.trim_end().to_owned()
}
/// Lane i's label row; lanes start at row 6, three rows apart.
fn lane_y(i: u16) -> u16 {
    6 + 3 * i
}
fn first_bar(b: &Buffer, y: u16) -> u16 {
    (44..b.area.width)
        .find(|&x| b[(x, y)].symbol() == "█")
        .expect("bar")
}
fn group(r: &mut OverviewReport) -> &mut multi_link_model::RepoGroup {
    r.repos.iter_mut().find(|g| g.repo == REPO).unwrap()
}
fn k(r: &mut OverviewReport) -> &mut SessionRow {
    group(r)
        .rows
        .iter_mut()
        .find_map(|row| match row {
            Row::Session(row) if row.id == id("01") => Some(row),
            _ => None,
        })
        .unwrap()
}
fn a2(r: &mut OverviewReport) -> &mut SessionRow {
    let a = k(r).children.iter_mut().find(|c| c.id == id("02")).unwrap();
    a.children.iter_mut().find(|c| c.id == id("04")).unwrap()
}

#[test]
fn sessions_walk_every_depth() {
    let r = report();
    let g = r.repos.iter().find(|g| g.repo == REPO).unwrap();
    let ids: Vec<_> = view::sessions(g).map(|s| s.id.clone()).collect();
    assert_eq!(ids.len(), 18);
    let at = |n: &str| ids.iter().position(|i| *i == id(n)).unwrap();
    // Pre-order: a parent comes before its whole subtree.
    assert!(at("01") < at("02") && at("02") < at("c2") && at("02") < at("04"));
    assert!(at("04") < at("03"));
    assert_eq!(view::home_label(g, g_row(g, "c2")), "step 2.1");
}
fn g_row<'a>(g: &'a multi_link_model::RepoGroup, n: &str) -> &'a SessionRow {
    view::sessions(g).find(|s| s.id == id(n)).unwrap()
}

#[test]
fn coordinator_tree_draws_nested_with_indent_and_started_sub_lines() {
    let r = report();
    // 13 merged quiet roots (newest first) come before K; then K, B, A, A2, Codex.
    let b = render(&r, options(UtcOffset::UTC), 160, 80, 16);
    let rows: Vec<_> = (13..18).map(|i| line(&b, lane_y(i))).collect();
    let subs: Vec<_> = (13..18).map(|i| line(&b, lane_y(i) + 1)).collect();
    assert!(rows[0].starts_with("  claude "), "K: {}", rows[0]);
    assert!(rows[1].starts_with("   └▸ claude "), "B: {}", rows[1]);
    assert!(
        rows[2].starts_with("   └▸ claude step 2.1"),
        "A: {}",
        rows[2]
    );
    assert!(rows[3].starts_with("▌    └▸ claude "), "A2: {}", rows[3]);
    assert!(
        rows[4].starts_with("     └▸ codex step 2.1"),
        "Codex: {}",
        rows[4]
    );
    assert!(subs[0].starts_with("    main · "), "K sub: {}", subs[0]);
    assert!(
        subs[1].starts_with("      started by this session · 09:10 "),
        "B sub: {}",
        subs[1]
    );
    assert!(subs[2].starts_with("      started by this session · 09:05 "));
    assert!(
        subs[3].starts_with("▌       started by this session · 09:25 "),
        "A2 sub: {}",
        subs[3]
    );
    assert!(
        subs[4].starts_with("        confirmed · 2 s after the call "),
        "Codex sub: {}",
        subs[4]
    );
    // Labels and sub lines stay out of the bar column.
    for i in 13..18 {
        for dy in 0..2 {
            let y = lane_y(i) + dy;
            assert!(
                (0..44).all(|x| b[(x, y)].symbol() != "█"),
                "row {y}: {}",
                line(&b, y)
            );
        }
    }
    // The ancestry mark: solid │ in the secondary colour at the child's started_at (the first
    // burst starts there in this fixture), from below the parent's label row to above the child's.
    let t = Theme::from_env(Some("truecolor"), None);
    for (parent, child) in [(13, 14), (13, 15), (15, 16)] {
        let x = first_bar(&b, lane_y(child));
        for y in lane_y(parent) + 1..lane_y(child) {
            assert_eq!(b[(x, y)].symbol(), "│", "({x}, {y}) {parent}->{child}");
            assert_eq!(b[(x, y)].fg, t.secondary, "({x}, {y}) {parent}->{child}");
        }
    }
    // Nested rows of every depth use the child background; the selected lane its own.
    assert_eq!(b[(1, lane_y(17))].bg, t.child);
    assert_eq!(b[(1, lane_y(16))].bg, t.selected);
}

#[test]
fn started_sub_line_uses_the_view_offset() {
    let r = report();
    let b = render(
        &r,
        options(UtcOffset::from_hms(-8, 0, 0).unwrap()),
        160,
        80,
        0,
    );
    assert!(line(&b, lane_y(16) + 1).starts_with("        started by this session · 01:25"));
}

#[test]
fn narrow_list_keeps_indent() {
    let r = report();
    let b = render(&r, options(UtcOffset::UTC), 70, 80, 0);
    assert!(line(&b, lane_y(16)).starts_with("     └▸ claude "));
    assert!(line(&b, lane_y(16) + 1).starts_with("        started by this session · 09:25"));
}

fn refresh(overview: OverviewReport) -> Result<Box<Refresh>, String> {
    Ok(Box::new(Refresh {
        overview,
        links: links(),
        diagnostics: String::new(),
    }))
}
fn open(app: &mut App) -> Vec<Effect> {
    let effects = app.update(Event::Key(KeyCode::Char('o')));
    app.update(Event::Opened(String::new()));
    effects
}

#[test]
fn waiting_nested_grandchild_rings_the_bell() {
    let mut app = App::default();
    app.update(Event::Refresh(refresh(report())));
    let mut waiting = report();
    a2(&mut waiting).state = State::Waiting;
    assert_eq!(app.update(Event::Refresh(refresh(waiting))), [Effect::Bell]);
}

#[test]
fn select_and_open_the_grandchild() {
    let mut app = App::default();
    app.update(Event::Refresh(refresh(report())));
    // Home stays flat by urgency: busy A2 is repo-k's one attention card, so enter opens the
    // repo view with A2 selected.
    app.update(Event::Key(KeyCode::Enter));
    assert_eq!(app.selection, 16);
    assert_eq!(
        open(&mut app),
        [Effect::FreshOpen {
            id: id("04"),
            fork: false
        }]
    );
    // Walking the selection reaches every nested lane once, in drawn order.
    app.update(Event::Key(KeyCode::Down));
    assert_eq!(
        open(&mut app),
        [Effect::FreshOpen {
            id: id("c2"),
            fork: false
        }]
    );
    app.update(Event::Key(KeyCode::Down));
    assert_eq!(app.selection, 17);
}

#[test]
fn refresh_that_reparents_keeps_the_selection_by_id() {
    // First report: A2 is top level (no started-by link yet).
    let mut flat = report();
    let a = k(&mut flat)
        .children
        .iter_mut()
        .find(|c| c.id == id("02"))
        .unwrap();
    let pos = a.children.iter().position(|c| c.id == id("04")).unwrap();
    let mut moved = a.children.remove(pos);
    moved.started_by = None;
    group(&mut flat).rows.push(Row::Session(moved));
    let mut app = App::default();
    app.update(Event::Refresh(refresh(flat)));
    app.update(Event::Key(KeyCode::Enter));
    assert_eq!(app.selection, 13, "A2 sorts among the roots");
    // Second report: A2 nested under A under K.
    app.update(Event::Refresh(refresh(report())));
    assert_eq!(app.selection, 16);
    assert_eq!(
        open(&mut app),
        [Effect::FreshOpen {
            id: id("04"),
            fork: false
        }]
    );
}

#[test]
fn hidden_grandchild_falls_back_to_its_nearest_ancestor() {
    let mut app = App::default();
    app.update(Event::Refresh(refresh(report())));
    app.update(Event::Key(KeyCode::Enter));
    assert_eq!(app.selection, 16);
    // A2 loses its activity, so it has no lane; the selection moves to A, not K.
    let mut quiet = report();
    a2(&mut quiet).activity.clear();
    app.update(Event::Refresh(refresh(quiet)));
    assert_eq!(app.selection, 15);
    assert_eq!(
        open(&mut app),
        [Effect::FreshOpen {
            id: id("02"),
            fork: false
        }]
    );
}

#[test]
fn deep_chains_indent_without_panicking() {
    let mut r = report();
    let mut leaf = a2(&mut r).clone();
    // Newest of all, so the chain is the first root.
    leaf.started_at = Timestamp::parse("2026-11-08T11:59:00Z").unwrap();
    for n in 0..30 {
        let mut parent = leaf.clone();
        parent.id = format!("deep-{n:02}");
        leaf.started_by = Some(parent.id.clone());
        parent.children = vec![leaf];
        leaf = parent;
    }
    leaf.started_by = None;
    group(&mut r).rows.push(Row::Session(leaf));
    for width in [70, 120, 160] {
        let b = render(&r, options(UtcOffset::UTC), width, 200, 0);
        assert!(line(&b, lane_y(1)).starts_with("   └▸ claude"));
        assert!(line(&b, lane_y(2)).starts_with("     └▸ claude"));
    }
}

#[test]
fn deep_indent_is_clamped_so_text_stays_out_of_the_bar_columns() {
    let mut r = report();
    let mut leaf = a2(&mut r).clone();
    leaf.started_at = Timestamp::parse("2026-11-08T11:59:00Z").unwrap();
    for n in 0..30 {
        let mut parent = leaf.clone();
        parent.id = format!("deep-{n:02}");
        leaf.started_by = Some(parent.id.clone());
        parent.children = vec![leaf];
        leaf = parent;
    }
    leaf.started_by = None;
    group(&mut r).rows.push(Row::Session(leaf));
    // Bars show from 120 columns; no lane row may hold text from the bar gap (col 42) on.
    let b = render(&r, options(UtcOffset::UTC), 160, 200, 0);
    let gap_and_bars =
        |y: u16| -> Vec<String> { (42..160).map(|x| b[(x, y)].symbol().to_owned()).collect() };
    for lane in 1..=30u16 {
        for y in [lane_y(lane), lane_y(lane) + 1] {
            for x in 42..160u16 {
                let symbol = b[(x, y)].symbol();
                assert!(
                    !symbol.chars().any(char::is_alphabetic),
                    "lane {lane} row {y} col {x} holds text {symbol:?}"
                );
            }
            // Columns 42 and 43 are the gap before the bars.
            assert_eq!(&gap_and_bars(y)[..2], [" ", " "], "lane {lane} row {y}");
        }
    }
    // The deepest lanes are still drawn and still named by their harness word.
    assert!(line(&b, lane_y(30)).contains("claude"));
    // Every chain member is a clone with the same activity: its bars sit in the same
    // columns at any depth.
    assert_eq!(first_bar(&b, lane_y(30)), first_bar(&b, lane_y(1)));
}
