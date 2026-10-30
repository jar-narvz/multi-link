//! Repo timeline snapshots (design T4) and window/clipping rules (PLAN Step 3.6). Goldens live
//! in `fixtures/expected/tui/timeline*.txt`; `UPDATE_SNAPSHOTS=1` rewrites them instead.

use std::{fs, path::PathBuf, time::Duration};

use multi_link_model::{Link, LinksReport, Match, OverviewReport, Row, SessionRow, Timestamp};
use multi_link_tui::{DEFAULT_WINDOW, TimeWindow, render_timeline};
use ratatui::{buffer::Buffer, layout::Rect, text::Span};

const REPO_A: &str = "/home/dev/multi-link-sandbox/repo-a";
const REPO_B: &str = "/home/dev/multi-link-sandbox/repo-b";
/// Width and the label column `render_timeline` uses at it (min(31, width / 2)).
const WIDTH: u16 = 100;
const LABEL: usize = 31;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn read(path: &str) -> String {
    fs::read_to_string(repo_root().join(path)).expect("read fixture")
}

fn overview() -> OverviewReport {
    serde_json::from_str(&read("fixtures/expected/overview.json")).expect("an OverviewReport")
}

fn load_links(path: &str) -> LinksReport {
    serde_json::from_str(&read(path)).expect("a LinksReport")
}

fn links() -> LinksReport {
    load_links("fixtures/expected/links.json")
}

fn ts(text: &str) -> Timestamp {
    Timestamp::parse(text).unwrap()
}

fn window(start: &str, end: &str) -> TimeWindow {
    TimeWindow::new(ts(start), ts(end))
}

/// The sandbox hand-offs happen 22:58–23:02; this window shows them one cell ≈ 3.5 s apart.
fn zoom() -> TimeWindow {
    window("2026-10-29T22:58:00Z", "2026-10-29T23:02:00Z")
}

fn render(
    report: &OverviewReport,
    links: &LinksReport,
    repo: &str,
    window: TimeWindow,
    width: u16,
    height: u16,
) -> Buffer {
    let area = Rect::new(0, 0, width, height);
    let mut buf = Buffer::empty(area);
    render_timeline(report, links, repo, window, area, &mut buf);
    buf
}

/// One line per buffer row, trailing spaces trimmed; a wide character's trailing cell skipped.
fn text(buf: &Buffer) -> String {
    let mut out = String::new();
    for y in 0..buf.area.height {
        let mut line = String::new();
        let mut x = 0;
        while x < buf.area.width {
            let symbol = buf[(x, y)].symbol();
            line.push_str(symbol);
            x += Span::raw(symbol).width().max(1) as u16;
        }
        out.push_str(line.trim_end());
        out.push('\n');
    }
    out
}

fn check_snapshot(rendered: &str, golden: &str) {
    let path = repo_root().join("fixtures/expected/tui").join(golden);
    if std::env::var_os("UPDATE_SNAPSHOTS").is_some_and(|value| value == "1") {
        fs::write(&path, rendered).expect("write golden");
        return;
    }
    let expected = fs::read_to_string(&path)
        .unwrap_or_else(|_| panic!("missing golden {golden}; run with UPDATE_SNAPSHOTS=1"));
    assert_eq!(rendered, expected, "{golden} differs from the render");
}

/// The screen line whose label holds `id_prefix`.
fn row<'a>(screen: &'a str, id_prefix: &str) -> &'a str {
    screen
        .lines()
        .find(|line| {
            line.chars()
                .take(LABEL)
                .collect::<String>()
                .contains(id_prefix)
        })
        .unwrap_or_else(|| panic!("no row for {id_prefix}:\n{screen}"))
}

/// The lane part of a row (after the label column), trailing spaces kept out.
fn lane(line: &str) -> String {
    line.chars()
        .skip(LABEL)
        .collect::<String>()
        .trim_end()
        .to_string()
}

/// The lane's non-blank cells, in order.
fn marks(line: &str) -> String {
    lane(line).chars().filter(|c| *c != ' ').collect()
}

/// `overview.json` was computed from `links.json`, where amb-1 (11bf1b58 → d501) and amb-2
/// (f4d83303 → d953) are both exact. In `links-amb-hidden.json` amb-1 is `ambiguous` (child
/// `null`, candidates d501 + d953) and amb-2 is `heuristic` with child d953. To render a
/// consistent pair, rebuild repo-b's rows the way core's overview would on the amb links
/// (`crates/core/src/overview.rs`, Codex placement): a Codex row is placed under the parent of
/// the latest link whose `child` is its id, whatever the `match`, so
/// - d953 stays under f4d83303, its `handoffs` now the one heuristic link;
/// - d501 has no link naming it, so it becomes a top-level Codex row with `handoffs: []`.
///   Codex rows never merge; sorted by state then activity it falls after 11bf1b58 and f4d83303,
///   which are merged, and the MergedRow takes the place of the first quiet row, so the rows are
///   `[claude ×5, d501]`.
/// - `latest_handoff` is the group's greatest (`at`, `invocation`) link: amb-2, now heuristic.
fn amb_overview(links: &LinksReport) -> OverviewReport {
    let mut report = overview();
    let group = report
        .repos
        .iter_mut()
        .find(|group| group.repo == REPO_B)
        .expect("repo-b group");
    assert_eq!(
        group.rows.len(),
        1,
        "repo-b is one merged row in overview.json"
    );
    let Row::Merged(merged) = &mut group.rows[0] else {
        panic!("repo-b row is merged")
    };
    let handoffs_of = |id: &str| -> Vec<Link> {
        let mut found: Vec<Link> = links
            .links
            .iter()
            .filter(|link| link.child.as_deref() == Some(id))
            .cloned()
            .collect();
        found.sort_by_key(|link| (link.at, link.invocation.clone()));
        found
    };
    let member = |merged: &[SessionRow], prefix: &str| -> usize {
        merged
            .iter()
            .position(|row| row.id.starts_with(prefix))
            .unwrap_or_else(|| panic!("{prefix} in the merged row"))
    };
    let amb1 = member(&merged.rows, "11bf1b58");
    let mut d501 = merged.rows[amb1]
        .children
        .pop()
        .expect("d501 under 11bf1b58");
    assert!(d501.id.starts_with("01a189e4-d501") && merged.rows[amb1].children.is_empty());
    d501.handoffs = handoffs_of(&d501.id);
    assert!(
        d501.handoffs.is_empty(),
        "no amb link names d501 as its child"
    );
    let amb2 = member(&merged.rows, "f4d83303");
    let d953 = &mut merged.rows[amb2].children[0];
    assert!(d953.id.starts_with("01a189e4-d953"));
    d953.handoffs = handoffs_of(&d953.id);
    assert_eq!(d953.handoffs.len(), 1);
    assert_eq!(d953.handoffs[0].r#match, Match::Heuristic);
    let parents: Vec<String> = merged.rows.iter().map(|row| row.id.clone()).collect();
    group.rows.push(Row::Session(d501));
    group.latest_handoff = links
        .links
        .iter()
        .filter(|link| parents.contains(&link.parent))
        .max_by_key(|link| (link.at, link.invocation.clone()))
        .cloned();
    report
}

fn amb_links() -> LinksReport {
    load_links("fixtures/expected/links-amb-hidden.json")
}

#[test]
fn timeline_repo_a_default_window_snapshot() {
    let report = overview();
    let window = TimeWindow::default_for(&report);
    let screen = text(&render(&report, &links(), REPO_A, window, WIDTH, 16));
    check_snapshot(&screen, "timeline-repo-a.txt");
}

#[test]
fn timeline_repo_a_zoom_snapshot() {
    let screen = text(&render(&overview(), &links(), REPO_A, zoom(), WIDTH, 16));
    check_snapshot(&screen, "timeline-repo-a-zoom.txt");
}

#[test]
fn timeline_repo_b_ambiguous_default_window_snapshot() {
    let links = amb_links();
    let report = amb_overview(&links);
    let window = TimeWindow::default_for(&report);
    let screen = text(&render(&report, &links, REPO_B, window, WIDTH, 16));
    check_snapshot(&screen, "timeline-repo-b-amb.txt");
}

#[test]
fn timeline_repo_b_ambiguous_zoom_snapshot() {
    let links = amb_links();
    let report = amb_overview(&links);
    let screen = text(&render(&report, &links, REPO_B, zoom(), WIDTH, 16));
    check_snapshot(&screen, "timeline-repo-b-amb-zoom.txt");
}

#[test]
fn default_window_is_two_hours_ending_at_now() {
    let report = overview();
    let window = TimeWindow::default_for(&report);
    assert_eq!(DEFAULT_WINDOW, Duration::from_secs(7_200));
    assert_eq!(window.end, report.now);
    assert_eq!(window.start, ts("2026-10-29T21:15:00Z"));
    let screen = text(&render(&report, &links(), REPO_A, window, WIDTH, 16));
    assert!(
        screen
            .lines()
            .next()
            .unwrap()
            .contains("2026-10-29 21:15:00 → 23:15:00 UTC (2h)")
    );
}

#[test]
fn markers_follow_the_link_match() {
    // links.json: every sandbox hand-off is exact, drawn on the parent's row only.
    let screen = text(&render(&overview(), &links(), REPO_B, zoom(), WIDTH, 16));
    for parent in ["11bf1b58", "f4d83303", "07c80c2e", "9d9a0ed8"] {
        assert_eq!(
            marks(row(&screen, parent)).matches('●').count(),
            1,
            "{parent}"
        );
    }
    assert!(
        !marks(row(&screen, "df403bfa")).contains('●'),
        "mention has no hand-off"
    );
    assert!(
        !lane(row(&screen, "01a189e4…ec5d")).contains('●'),
        "no marker on a child row"
    );

    // amb: amb-1 is `?2` on 11bf1b58, amb-2 heuristic `○` on f4d83303, no `●` on either.
    let links = amb_links();
    let screen = text(&render(
        &amb_overview(&links),
        &links,
        REPO_B,
        zoom(),
        WIDTH,
        16,
    ));
    let amb1 = marks(row(&screen, "11bf1b58"));
    assert!(amb1.contains("?2") && !amb1.contains('●'), "{amb1}");
    let amb2 = marks(row(&screen, "f4d83303"));
    assert!(amb2.contains('○') && !amb2.contains('●'), "{amb2}");
    // d501 is a top-level Codex row; d953 still sits under f4d83303.
    assert!(row(&screen, "01a189e4…ec5d").starts_with("unk   codex"));
    assert!(row(&screen, "01a189e4…2eb3").starts_with(" └~ unk   codex"));
}

/// Merged `claude ×N` rows are expanded: every session of the group gets its own row, once.
#[test]
fn merged_rows_are_expanded_once_each() {
    let report = overview();
    let screen = text(&render(&report, &links(), REPO_B, zoom(), WIDTH, 40));
    assert!(!screen.contains("claude ×"), "{screen}");
    for id in [
        "11bf1b58",
        "f4d83303",
        "df403bfa",
        "07c80c2e",
        "9d9a0ed8",
        "01a189e4…ec5d",
        "01a189e4…2eb3",
        "01a189e3…7480",
        "01a189e3…6d74",
    ] {
        let count = screen
            .lines()
            .filter(|line| line.chars().take(LABEL).collect::<String>().contains(id))
            .count();
        assert_eq!(count, 1, "{id}:\n{screen}");
    }
}

/// bb792d9c's span is 22:58:22.432–22:58:42.511 and its exec hand-off is at 22:58:26.346.
#[test]
fn span_clipped_by_the_window() {
    // Window starts mid-span: the span begins in cell 0 with `◂`.
    let starts_inside = window("2026-10-29T22:58:30Z", "2026-10-29T23:02:00Z");
    let screen = text(&render(
        &overview(),
        &links(),
        REPO_A,
        starts_inside,
        WIDTH,
        16,
    ));
    let bb = lane(row(&screen, "bb792d9c"));
    assert!(bb.starts_with('◂'), "{bb:?}");
    assert!(bb.chars().skip(1).all(|c| c == '━'), "{bb:?}");

    // Window ends mid-span: the span runs to the last cell and ends with `▸`.
    let ends_inside = window("2026-10-29T22:57:00Z", "2026-10-29T22:58:30Z");
    let screen = text(&render(
        &overview(),
        &links(),
        REPO_A,
        ends_inside,
        WIDTH,
        16,
    ));
    let bb = lane(row(&screen, "bb792d9c"));
    assert_eq!(bb.chars().count(), WIDTH as usize - LABEL, "{bb:?}");
    assert!(bb.ends_with('▸'), "{bb:?}");
    // Sessions that start after the window draw nothing.
    assert_eq!(lane(row(&screen, "80cce3e4")), "");

    // Window wholly after the span: label only.
    let after = window("2026-10-29T23:05:00Z", "2026-10-29T23:15:00Z");
    let screen = text(&render(&overview(), &links(), REPO_A, after, WIDTH, 16));
    assert_eq!(lane(row(&screen, "bb792d9c")), "");
}

#[test]
fn link_outside_the_window_is_not_drawn() {
    // bb792d9c's exec at 22:58:26.346 is before the window; its span (to 22:58:42) is inside.
    let late = window("2026-10-29T22:58:30Z", "2026-10-29T23:02:00Z");
    let screen = text(&render(&overview(), &links(), REPO_A, late, WIDTH, 16));
    let bb = marks(row(&screen, "bb792d9c"));
    assert!(!bb.contains('●') && bb.contains('━'), "{bb}");
    // ef72378c's resume at 22:58:53.566 is inside and drawn.
    assert!(marks(row(&screen, "ef72378c")).contains('●'));

    // The same link inside a window that includes it is drawn.
    let screen = text(&render(&overview(), &links(), REPO_A, zoom(), WIDTH, 16));
    assert!(marks(row(&screen, "bb792d9c")).contains('●'));
}

#[test]
fn zero_length_span_draws_one_cell() {
    let mut report = overview();
    let group = report.repos.iter_mut().find(|g| g.repo == REPO_A).unwrap();
    let Row::Session(bb) = group
        .rows
        .iter_mut()
        .find(|row| matches!(row, Row::Session(s) if s.id.starts_with("bb792d9c")))
        .unwrap()
    else {
        unreachable!()
    };
    bb.started_at = ts("2026-10-29T23:00:00Z");
    bb.last_activity = bb.started_at;
    let no_links = LinksReport {
        links: Vec::new(),
        unparented: Vec::new(),
    };
    let screen = text(&render(&report, &no_links, REPO_A, zoom(), WIDTH, 16));
    assert_eq!(marks(row(&screen, "bb792d9c")), "━");
    // On the window's very end it is still drawn, in the last cell.
    let ending = window("2026-10-29T22:58:00Z", "2026-10-29T23:00:00Z");
    let screen = text(&render(&report, &no_links, REPO_A, ending, WIDTH, 16));
    let bb = lane(row(&screen, "bb792d9c"));
    assert_eq!(bb.chars().filter(|c| *c != ' ').count(), 1);
    assert_eq!(bb.chars().count(), WIDTH as usize - LABEL);
}

/// repo-c's hand-offs are `unlinked`; inside a window they show as `×` on their parent's row.
#[test]
fn unlinked_handoffs_are_drawn_as_crosses() {
    let morning = window("2026-10-28T07:00:00Z", "2026-10-28T10:15:00Z");
    let screen = text(&render(
        &overview(),
        &links(),
        "/home/dev/repo-c",
        morning,
        WIDTH,
        16,
    ));
    assert_eq!(marks(row(&screen, "85a715ba")).matches('×').count(), 3);
    assert_eq!(marks(row(&screen, "11c1f13c")).matches('×').count(), 2);
    assert!(screen.contains("× unlinked"));
}

/// The bar is a session span, never busy time, and the legend says so.
#[test]
fn legend_names_the_session_span() {
    let screen = text(&render(&overview(), &links(), REPO_A, zoom(), WIDTH, 16));
    let lines: Vec<&str> = screen.lines().collect();
    let legend = lines[lines.len() - 3];
    assert!(
        legend.contains("session span") && legend.contains("idle gaps included"),
        "{legend}"
    );
    assert!(legend.contains("not busy time"));
    assert!(lines[lines.len() - 2].contains("● exact  ○ heuristic  ?N ambiguous"));
    assert!(lines[lines.len() - 2].contains("+ more"));
    assert!(lines[lines.len() - 1].contains("└~ by a heuristic one"));
}

#[test]
fn unknown_repo_and_empty_window_do_not_panic() {
    let report = overview();
    let screen = text(&render(&report, &links(), "/nowhere", zoom(), WIDTH, 16));
    assert!(screen.starts_with("/nowhere  not in this overview"));

    let now = report.now;
    for empty in [
        TimeWindow::new(now, now),
        window("2026-10-29T23:00:00Z", "2026-10-29T22:00:00Z"),
    ] {
        let screen = text(&render(&report, &links(), REPO_A, empty, WIDTH, 16));
        assert!(screen.lines().next().unwrap().contains("(empty window)"));
        // Rows are listed but nothing is drawn in their lanes.
        assert_eq!(lane(row(&screen, "bb792d9c")), "");
    }
}

#[test]
fn small_sizes_do_not_panic() {
    let report = overview();
    let links = links();
    let amb_links = amb_links();
    let amb = amb_overview(&amb_links);
    let now = report.now;
    let windows = [
        TimeWindow::default_for(&report),
        zoom(),
        TimeWindow::new(now, now),
        window("2026-10-29T23:00:00Z", "2026-10-29T22:00:00Z"),
        TimeWindow::ending_at(now, Duration::from_secs(10 * 365 * 86_400)),
        TimeWindow::ending_at(now, Duration::from_millis(1)),
    ];
    for (width, height) in [
        (0, 0),
        (1, 1),
        (0, 5),
        (5, 0),
        (2, 2),
        (3, 3),
        (20, 5),
        (40, 10),
        (100, 4),
        (100, 7),
    ] {
        for window in windows {
            for repo in [REPO_A, REPO_B, "/home/dev/repo-c", "/nowhere"] {
                render(&report, &links, repo, window, width, height);
                render(&amb, &amb_links, repo, window, width, height);
            }
        }
    }
}

#[test]
fn clipped_rows_are_counted() {
    let screen = text(&render(&overview(), &links(), REPO_B, zoom(), WIDTH, 10));
    // 10 rows: title, ticks, axis, 4 body rows (3 lanes + note), 3 legend lines; 9 lanes.
    assert!(screen.contains("↓ 6 more rows"), "{screen}");
}

/// With a single body row and more lanes than fit, that row is the note, not a lane.
#[test]
fn one_body_row_shows_the_note() {
    // Height 4: title, axis, one body row, one legend line; repo-a has 6 lanes.
    let screen = text(&render(&overview(), &links(), REPO_A, zoom(), WIDTH, 4));
    let lines: Vec<&str> = screen.lines().collect();
    assert_eq!(lines[2], "  ↓ 6 more rows", "{screen}");
}

/// Children placed by an exact hand-off hang from `└`, those placed by a heuristic one from `└~`.
#[test]
fn child_branch_shows_how_it_was_placed() {
    let amb = amb_links();
    let screen = text(&render(
        &amb_overview(&amb),
        &amb,
        REPO_B,
        zoom(),
        WIDTH,
        20,
    ));
    assert!(
        row(&screen, "01a189e4…2eb3").starts_with(" └~ "),
        "{screen}"
    );
    for exact in ["01a189e3…7480", "01a189e3…6d74"] {
        assert!(row(&screen, exact).starts_with(" └  unk"), "{screen}");
    }
    let screen = text(&render(&overview(), &links(), REPO_A, zoom(), WIDTH, 20));
    assert!(
        row(&screen, "01a189e2…a6a1").starts_with(" └  unk"),
        "{screen}"
    );
}

const BB: &str = "bb792d9c-7f69-410b-a571-b6d97d3aaffa";

/// A hand-off from bb792d9c at `at` with `match` and `candidates` candidates (a copy of its
/// real exec link with those fields changed).
fn handoff(at: &str, r#match: Match, candidates: usize) -> Link {
    let mut link = links()
        .links
        .into_iter()
        .find(|link| link.parent == BB)
        .expect("bb792d9c's exec link");
    link.at = ts(at);
    link.invocation = format!("toolu_{at}");
    link.r#match = r#match;
    link.child = None;
    link.candidates = (0..candidates)
        .map(|index| multi_link_model::Candidate {
            id: format!("c{index}"),
            gap_ms: index as i64,
        })
        .collect();
    link
}

/// bb792d9c's lane in the zoom window (69 cells over 240 s) with only `drawn` as links.
/// 22:59:00.000 is cell 17, 22:59:04 cell 18, 22:58:56 cell 16; the span fills cells 6..=12.
fn bb_lane(drawn: Vec<Link>) -> Vec<char> {
    let report = LinksReport {
        links: drawn,
        unparented: Vec::new(),
    };
    let screen = text(&render(&overview(), &report, REPO_A, zoom(), WIDTH, 16));
    let mut cells: Vec<char> = lane(row(&screen, "bb792d9c")).chars().collect();
    cells.resize(WIDTH as usize - LABEL, ' ');
    cells
}

#[test]
fn several_handoffs_in_one_cell_show_a_plus() {
    // Exact and heuristic share cell 17: the less certain `○` wins, `+` follows.
    let cells = bb_lane(vec![
        handoff("2026-10-29T22:59:00Z", Match::Exact, 0),
        handoff("2026-10-29T22:59:00.5Z", Match::Heuristic, 1),
    ]);
    assert_eq!(cells[17..19], ['○', '+'], "{cells:?}");
    // Next cell taken by another marker: the `+` goes before the shared cell instead.
    let cells = bb_lane(vec![
        handoff("2026-10-29T22:59:00Z", Match::Exact, 0),
        handoff("2026-10-29T22:59:00.5Z", Match::Unlinked, 0),
        handoff("2026-10-29T22:59:04Z", Match::Exact, 0),
    ]);
    assert_eq!(cells[16..19], ['+', '×', '●'], "{cells:?}");
    // Ambiguous wins a shared cell and keeps its count, then the `+`.
    let cells = bb_lane(vec![
        handoff("2026-10-29T22:59:00Z", Match::Ambiguous, 2),
        handoff("2026-10-29T22:59:00.5Z", Match::Exact, 0),
    ]);
    assert_eq!(cells[17..20], ['?', '2', '+'], "{cells:?}");
}

#[test]
fn ambiguous_count_never_covers_another_marker() {
    // Alone: `?2`.
    let cells = bb_lane(vec![handoff("2026-10-29T22:59:00Z", Match::Ambiguous, 2)]);
    assert_eq!(cells[17..19], ['?', '2'], "{cells:?}");
    // An exact marker in the next cell keeps its `●`; the ambiguous one shows just `?`.
    let cells = bb_lane(vec![
        handoff("2026-10-29T22:59:00Z", Match::Ambiguous, 2),
        handoff("2026-10-29T22:59:04Z", Match::Exact, 0),
    ]);
    assert_eq!(cells[17..20], ['?', '●', ' '], "{cells:?}");
}

#[test]
fn ambiguous_count_is_anchored_and_cut_at_the_right_edge() {
    // At the window's end: `?` in the last cell, no room for the digit, nothing shifted left.
    let cells = bb_lane(vec![handoff("2026-10-29T23:02:00Z", Match::Ambiguous, 2)]);
    let last = cells.len() - 1;
    assert_eq!(cells[last], '?', "{cells:?}");
    assert_eq!(cells[last - 1], ' ', "{cells:?}");
    assert!(!cells.contains(&'2'));
}

#[test]
fn ambiguous_without_candidates_is_a_bare_question_mark() {
    let cells = bb_lane(vec![handoff("2026-10-29T22:59:00Z", Match::Ambiguous, 0)]);
    assert_eq!(cells[17..19], ['?', ' '], "{cells:?}");
    assert!(!cells.contains(&'0'));
}

#[test]
fn area_outside_the_buffer_is_ignored() {
    let report = overview();
    let mut buf = Buffer::empty(Rect::new(0, 0, 10, 4));
    let window = zoom();
    render_timeline(
        &report,
        &links(),
        REPO_A,
        window,
        Rect::new(5, 2, 50, 50),
        &mut buf,
    );
    render_timeline(
        &report,
        &links(),
        REPO_A,
        window,
        Rect::new(20, 20, 5, 5),
        &mut buf,
    );
    for rect in [Rect::new(0, 65534, 100, 1), Rect::new(0, 65530, 100, 5)] {
        let mut buf = Buffer::empty(rect);
        render_timeline(&report, &links(), REPO_A, window, rect, &mut buf);
    }
}
