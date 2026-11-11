use std::time::{Duration, Instant};

use crossterm::event::KeyCode;
use multi_link_model::{LinksReport, OverviewReport};
use multi_link_tui::{App, Effect, Event, Fade, LoadPhase, Refresh, theme::Theme};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier},
};

fn report() -> Box<Refresh> {
    Box::new(Refresh {
        live: None,
        overview: serde_json::from_str::<OverviewReport>(include_str!(
            "../../../fixtures/expected/overview.json"
        ))
        .unwrap(),
        links: serde_json::from_str::<LinksReport>(include_str!(
            "../../../fixtures/expected/links.json"
        ))
        .unwrap(),
        diagnostics: String::new(),
        graph: None,
        graph_live: vec![],
        graph_steps: vec![],
        codex_steps: Default::default(),
        started_steps: Default::default(),
    })
}
fn app() -> (App, Instant) {
    let start = Instant::now();
    let mut app = App::default();
    app.begin_loading(start);
    (app, start)
}
fn at(app: &mut App, start: Instant, ms: u64) {
    app.set_instant(start + Duration::from_millis(ms));
}
fn text(app: &App) -> String {
    let mut buf = Buffer::empty(Rect::new(0, 0, 76, 22));
    app.render(buf.area, &mut buf);
    buf.content.iter().map(|c| c.symbol()).collect()
}
#[test]
fn report_deadlines_drive_both_fade_stages_without_extra_frames() {
    for received in [300, 1600] {
        let (mut app, start) = app();
        at(&mut app, start, received);
        app.update(Event::Refresh(Ok(report())));
        let fade_start = received.max(1000);
        if received < 1000 {
            at(&mut app, start, 999);
            assert_eq!(app.loading_options().unwrap().fade, Fade::None);
        }
        for (ms, fade) in [
            (fade_start, Fade::School),
            (fade_start + 124, Fade::School),
            (fade_start + 125, Fade::Text),
            (fade_start + 249, Fade::Text),
        ] {
            at(&mut app, start, ms);
            let options = app.loading_options().unwrap();
            assert_eq!(options.fade, fade);
            assert_eq!(options.phase, LoadPhase::Done);
        }
        at(&mut app, start, fade_start + 250);
        assert!(app.loading_options().is_none());
    }
}
#[test]
fn skip_is_consumed_before_and_after_report_and_during_fade() {
    for received in [false, true] {
        let (mut app, start) = app();
        at(&mut app, start, 200);
        if received {
            app.update(Event::Refresh(Ok(report())));
        }
        assert!(app.update(Event::Key(KeyCode::Char('l'))).is_empty());
        assert!(app.loading_options().is_none());
        if !received {
            assert!(text(&app).contains("loading sessions…"));
            app.update(Event::Refresh(Ok(report())));
        }
        assert!(app.loading_options().is_none());
        assert!(
            app.update(Event::Key(KeyCode::Char('l')))
                .contains(&Effect::Live(true))
        );
    }
    let (mut app, start) = app();
    app.update(Event::Refresh(Ok(report())));
    at(&mut app, start, 1100);
    assert!(app.update(Event::Key(KeyCode::Enter)).is_empty());
    assert!(app.loading_options().is_none());
}
#[test]
fn errors_and_quit_end_loading_immediately() {
    for error in ["denied", "refresh worker stopped"] {
        let (mut app, start) = app();
        at(&mut app, start, 500);
        app.update(Event::Refresh(Err(error.into())));
        assert!(app.loading_options().is_none());
        assert!(text(&app).contains(&format!("refresh failed: {error}")));
        app.update(Event::Refresh(Ok(report())));
        assert!(app.loading_options().is_none());
    }
    let (mut app, _) = app();
    assert_eq!(app.update(Event::Key(KeyCode::Char('q'))), [Effect::Quit]);
}
#[test]
fn injected_counts_are_rendered_and_no_color_never_fades_or_adds_bold() {
    let (mut app, start) = app();
    app.set_load_progress(LoadPhase::Reading, 612, 871);
    assert!(text(&app).contains("reading transcripts  612 / 871"));
    app.theme = Some(Theme::from_env(None, Some(std::ffi::OsStr::new("1"))));
    let mut buf = Buffer::empty(Rect::new(0, 0, 76, 22));
    app.render(buf.area, &mut buf);
    app.theme.unwrap().apply_no_color(&mut buf);
    assert!(buf.content.iter().all(|c| c.fg == Color::Reset
        && c.bg == Color::Reset
        && !c.modifier.intersects(Modifier::BOLD | Modifier::DIM)));
    at(&mut app, start, 300);
    app.update(Event::Refresh(Ok(report())));
    at(&mut app, start, 999);
    assert_eq!(app.loading_options().unwrap().fade, Fade::None);
    at(&mut app, start, 1000);
    assert!(app.loading_options().is_none());
}
