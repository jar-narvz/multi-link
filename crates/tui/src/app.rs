//! Pure interaction state. Effects request work; they never contain a cached open decision.
use std::collections::HashSet;

use crossterm::event::KeyCode;
use multi_link_model::{LinksReport, OverviewReport, State};
use ratatui::{buffer::Buffer, layout::Rect};

use crate::{
    RepoOptions, Target, TimeWindow,
    timeline::{lanes, render_repo},
};

pub struct Refresh {
    pub overview: OverviewReport,
    pub links: LinksReport,
    pub diagnostics: String,
}

pub enum Event {
    Refresh(Result<Box<Refresh>, String>),
    Key(KeyCode),
    /// Completion of the fresh lookup / external command, never an executable action.
    Opened(String),
}

#[derive(Debug, PartialEq, Eq)]
pub enum Effect {
    Bell,
    FreshOpen { id: String, fork: bool },
    Quit,
}

#[derive(Default)]
pub struct App {
    pub report: Option<Box<Refresh>>,
    /// Open result, dismissed by the next key; progress remains until Opened.
    pub footer: String,
    refresh_error: Option<String>,
    pub selection: usize,
    pub theme: Option<crate::theme::Theme>,
    offset: Option<time::UtcOffset>,
    timeline: Option<(String, TimeWindow)>,
    home_selection: Option<SelectionIdentity>,
    explicit_range: bool,
    unfold: bool,
    imports: bool,
    pub history_window: Option<std::time::Duration>,
    pub history_all: bool,
    waiting: Option<HashSet<String>>,
    healthy: bool,
    opening: bool,
}

/// Stable identity shared by refresh and returning from the timeline.
struct SelectionIdentity {
    repo: String,
    id: Option<String>,
    harness: Option<multi_link_model::Harness>,
}

impl SelectionIdentity {
    fn from_target(repo: &str, target: Target<'_>) -> Self {
        let (id, harness) = match target {
            Target::Session(row) => (Some(row.id.clone()), Some(row.harness)),
            Target::Merged(row) => (None, Some(row.harness)),
            Target::Repo(_) | Target::Hidden(_) => (None, None),
        };
        Self {
            repo: repo.to_string(),
            id,
            harness,
        }
    }

    fn restore(&self, targets: &[(&str, Target<'_>)]) -> Option<usize> {
        targets
            .iter()
            .position(|(repo, target)| {
                *repo == self.repo
                    && match target {
                        Target::Session(row) => {
                            self.id.as_deref() == Some(&row.id) && self.harness == Some(row.harness)
                        }
                        Target::Merged(row) => {
                            self.id.is_none() && self.harness == Some(row.harness)
                        }
                        Target::Repo(_) | Target::Hidden(_) => self.id.is_none(),
                    }
            })
            .or_else(|| {
                // The selected row is gone from the targets (e.g. hidden): fall back to its
                // nearest listed ancestor, at any depth (the one with the smallest subtree).
                let id = self.id.as_deref()?;
                targets
                    .iter()
                    .enumerate()
                    .filter_map(|(i, (repo, target))| match target {
                        Target::Session(row)
                            if *repo == self.repo
                                && crate::view::subtree(row).skip(1).any(|c| c.id == id) =>
                        {
                            Some((crate::view::subtree(row).count(), i))
                        }
                        _ => None,
                    })
                    .min()
                    .map(|(_, i)| i)
            })
            .or_else(|| targets.iter().position(|(repo, _)| *repo == self.repo))
    }
}

impl App {
    fn report_window(
        report: &OverviewReport,
        length: Option<std::time::Duration>,
        all: bool,
    ) -> TimeWindow {
        if all {
            let first = report
                .repos
                .iter()
                .flat_map(crate::view::sessions)
                .flat_map(|r| &r.activity)
                .map(|b| b.from)
                .min()
                .unwrap_or(report.now);
            TimeWindow::new(first, report.now)
        } else {
            TimeWindow::ending_at(report.now, length.unwrap_or(crate::DEFAULT_WINDOW))
        }
    }

    /// Restrict the repo screen without changing core grouping or placement.
    fn view(&self) -> Option<OverviewReport> {
        let mut report = self.report.as_ref()?.overview.clone();
        if let Some((repo, _)) = &self.timeline {
            report.repos.retain(|group| &group.repo == repo);
        }
        Some(report)
    }

    fn targets<'a>(&self, view: &'a OverviewReport) -> Vec<(&'a str, Target<'a>)> {
        if self.timeline.is_some() {
            view.repos
                .iter()
                .flat_map(|g| {
                    lanes(g, self.imports)
                        .into_iter()
                        .map(|l| (g.repo.as_str(), Target::Session(l.session)))
                })
                .collect()
        } else {
            crate::home::home_targets(view, &self.home_options())
        }
    }

    fn home_options(&self) -> crate::HomeOptions<'_> {
        crate::HomeOptions {
            links: self.report.as_ref().map(|r| &r.links),
            window: self
                .report
                .as_ref()
                .map(|r| Self::report_window(&r.overview, self.history_window, self.history_all)),
            theme: self
                .theme
                .unwrap_or_else(|| crate::theme::Theme::from_env(Some("truecolor"), None)),
            offset: self.offset.unwrap_or(time::UtcOffset::UTC),
            ..crate::HomeOptions::default()
        }
    }
    fn selection_identity(&self) -> Option<SelectionIdentity> {
        let view = self.view()?;
        let targets = self.targets(&view);
        let (repo, target) = targets.get(self.selection)?;
        Some(SelectionIdentity::from_target(repo, *target))
    }

    fn restore_selection(&mut self, selected: Option<&SelectionIdentity>, fallback: usize) {
        if let Some(view) = self.view() {
            let targets = self.targets(&view);
            self.selection = selected
                .and_then(|identity| identity.restore(&targets))
                .unwrap_or(fallback)
                .min(targets.len().saturating_sub(1));
        }
    }

    fn selected(&self) -> Option<(String, Option<String>)> {
        let view = self.view()?;
        let targets = self.targets(&view);
        let (repo, target) = targets.get(self.selection)?;
        Some((
            repo.to_string(),
            match target {
                Target::Session(row) => Some(row.id.clone()),
                Target::Merged(_) | Target::Repo(_) | Target::Hidden(_) => None,
            },
        ))
    }

    pub fn update(&mut self, event: Event) -> Vec<Effect> {
        if matches!(&event, Event::Key(_)) && !self.opening {
            self.footer.clear();
        }
        match event {
            Event::Refresh(Err(error)) => {
                self.healthy = false;
                self.refresh_error = Some(format!("refresh failed: {error}; keeping last report"));
            }
            Event::Refresh(Ok(report)) => {
                let selected = self.selection_identity();
                let waiting = waiting_ids(&report.overview);
                let bells = self
                    .waiting
                    .as_ref()
                    .map_or(0, |previous| waiting.difference(previous).count());
                self.waiting = Some(waiting);
                self.refresh_error = None;
                self.offset = Some(crate::view::local_offset(report.overview.now));
                if !self.explicit_range
                    && let Some((_, window)) = &mut self.timeline
                {
                    *window = Self::report_window(
                        &report.overview,
                        self.history_window,
                        self.history_all,
                    );
                }
                self.report = Some(report);
                self.healthy = true;
                self.restore_selection(selected.as_ref(), self.selection);
                return (0..bells).map(|_| Effect::Bell).collect();
            }
            Event::Opened(message) => {
                self.opening = false;
                self.footer = message;
            }
            Event::Key(key) => match key {
                KeyCode::Char('q') => return vec![Effect::Quit],
                KeyCode::Up => self.selection = self.selection.saturating_sub(1),
                KeyCode::Down => {
                    let count = self.view().map_or(0, |view| self.targets(&view).len());
                    self.selection = (self.selection + 1).min(count.saturating_sub(1));
                }
                KeyCode::Esc => {
                    if self.timeline.take().is_some() {
                        let selected = self.home_selection.take();
                        self.restore_selection(selected.as_ref(), 0);
                    } else {
                        self.selection = 0;
                    }
                }
                KeyCode::Enter | KeyCode::Char('t') if self.timeline.is_none() => {
                    if let Some((repo, id)) = self.selected() {
                        self.home_selection = self.selection_identity();
                        self.timeline = Some((
                            repo,
                            Self::report_window(
                                &self.report.as_ref().unwrap().overview,
                                self.history_window,
                                self.history_all,
                            ),
                        ));
                        self.explicit_range = false;
                        self.unfold = false;
                        self.imports = false;
                        self.selection = 0;
                        if let Some(id) = id {
                            let view = self.view().unwrap();
                            self.selection = self.targets(&view).iter().position(|(_, target)| matches!(target, Target::Session(row) if row.id == id)).unwrap_or(0);
                        }
                    }
                }
                KeyCode::Char('o' | 'f') if self.healthy && !self.opening => {
                    if let Some((_, Some(id))) = self.selected() {
                        self.opening = true;
                        self.footer = "checking current session…".into();
                        return vec![Effect::FreshOpen {
                            id,
                            fork: key == KeyCode::Char('f'),
                        }];
                    } else {
                        self.footer = "enter opens the repo view; select a session there".into();
                    }
                }
                KeyCode::Char('i') if self.timeline.is_some() => {
                    let selected = self.selection_identity();
                    self.imports = !self.imports;
                    self.restore_selection(selected.as_ref(), self.selection);
                }
                KeyCode::Char('z') if self.timeline.is_some() => self.unfold = !self.unfold,
                KeyCode::Char('0') if self.timeline.is_some() => {
                    let window = Self::report_window(
                        &self.report.as_ref().unwrap().overview,
                        self.history_window,
                        self.history_all,
                    );
                    self.timeline.as_mut().unwrap().1 = window;
                    self.explicit_range = false;
                }
                // Zoom around the right edge; pan by half a window. First use the fitted range.
                KeyCode::Char('+' | '-') | KeyCode::Left | KeyCode::Right => {
                    self.adjust_window(key);
                }
                _ => {}
            },
        }
        Vec::new()
    }

    fn adjust_window(&mut self, key: KeyCode) {
        let Some(report) = self.report.as_ref() else {
            return;
        };
        let bounds = Self::report_window(&report.overview, self.history_window, self.history_all);
        let Some((repo, window)) = &mut self.timeline else {
            return;
        };
        if !self.explicit_range
            && let Some(group) = self
                .report
                .as_ref()
                .unwrap()
                .overview
                .repos
                .iter()
                .find(|g| &g.repo == repo)
            && let Some(first) = lanes(group, self.imports)
                .iter()
                .flat_map(|l| &l.session.activity)
                .map(|b| b.from)
                .min()
        {
            window.start = first.max(window.start);
        }
        let end = window.end.epoch_ms();
        let length = (end - window.start.epoch_ms()).max(1000);
        let (end, length) = match key {
            KeyCode::Char('+') => (end, (length / 2).max(1000)),
            KeyCode::Char('-') => (end, length.saturating_mul(2)),
            KeyCode::Left => (end - length / 2, length),
            KeyCode::Right => (end + length / 2, length),
            _ => return,
        };
        let length = length.min((bounds.end.epoch_ms() - bounds.start.epoch_ms()).max(0));
        let end = end.clamp(bounds.start.epoch_ms() + length, bounds.end.epoch_ms());
        if let Ok(end) = multi_link_model::Timestamp::from_epoch_ms(end) {
            *window = TimeWindow::ending_at(end, std::time::Duration::from_millis(length as u64));
            self.explicit_range = true;
        }
    }

    pub fn render(&self, area: Rect, buf: &mut Buffer) {
        let area = area.intersection(buf.area);
        if area.is_empty() {
            return;
        }
        if let (Some(view), Some(report)) = (self.view(), &self.report) {
            if let Some((repo, window)) = &self.timeline {
                let options = RepoOptions {
                    fitted: !self.explicit_range,
                    fold: !self.unfold,
                    imports: self.imports,
                    theme: self.home_options().theme,
                    offset: self.offset.unwrap_or(time::UtcOffset::UTC),
                };
                render_repo(
                    &view,
                    &report.links,
                    repo,
                    *window,
                    Some(self.selection),
                    &options,
                    area,
                    buf,
                );
            } else {
                let options = self.home_options();
                crate::render_home_with_options(&view, Some(self.selection), &options, area, buf);
            }
        }
        if self.report.is_some()
            && self.footer.is_empty()
            && self.refresh_error.is_none()
            && self
                .report
                .as_ref()
                .is_some_and(|r| r.diagnostics.is_empty())
        {
            return;
        }
        let style = if self.timeline.is_none() {
            let t = self.home_options().theme;
            t.text_style(t.text, t.screen)
        } else {
            self.home_options().theme.text_style(
                self.home_options().theme.text,
                self.home_options().theme.screen,
            )
        };
        buf.set_style(Rect::new(area.x, area.bottom() - 1, area.width, 1), style);
        for x in area.x..area.right() {
            buf[(x, area.bottom() - 1)].set_symbol(" ");
        }
        let footer = self.footer_text();
        buf.set_stringn(
            area.x,
            area.bottom() - 1,
            &footer,
            area.width as usize,
            style,
        );
    }

    fn footer_text(&self) -> String {
        if !self.footer.is_empty() {
            return self.footer.clone();
        }
        if let Some(error) = &self.refresh_error {
            return error.clone();
        }
        let Some(report) = &self.report else {
            return "loading sessions… · q quit".into();
        };
        let help = if self.timeline.is_some() {
            "↑↓ select · +/- zoom · ←→ pan · 0 reset · esc home · o open · f fork · q quit"
        } else {
            "↑↓ move · enter repo view · o open session · q quit"
        };
        if report.diagnostics.is_empty() {
            help.into()
        } else {
            format!("{help} · {}", report.diagnostics)
        }
    }
}

fn waiting_ids(report: &OverviewReport) -> HashSet<String> {
    report
        .repos
        .iter()
        .flat_map(crate::view::sessions)
        .filter(|row| row.state == State::Waiting)
        .map(|row| row.id.clone())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use multi_link_model::Row;
    fn report() -> Box<Refresh> {
        Box::new(Refresh {
            overview: serde_json::from_str(include_str!(
                "../../../fixtures/expected/overview.json"
            ))
            .unwrap(),
            links: serde_json::from_str(include_str!("../../../fixtures/expected/links.json"))
                .unwrap(),
            diagnostics: String::new(),
        })
    }
    fn state(report: &mut Refresh, state: State) {
        let Row::Session(row) = &mut report.overview.repos[0].rows[0] else {
            panic!("session")
        };
        row.state = state;
    }
    fn rendered_footer(app: &App, width: u16) -> String {
        let area = Rect::new(0, 0, width, 1);
        let mut buffer = Buffer::empty(area);
        app.render(area, &mut buffer);
        (0..width)
            .map(|x| buffer[(x, 0)].symbol())
            .collect::<String>()
            .trim_end()
            .to_owned()
    }

    #[test]
    fn open_results_survive_successful_refresh_until_a_key_press() {
        for message in [
            "running_without_app_link; in /repo: claude --resume session-id",
            "session no longer found",
            "claude exited with exit status: 1",
        ] {
            let mut app = App::default();
            app.update(Event::Refresh(Ok(report())));
            app.update(Event::Opened(message.into()));
            let mut refreshed = report();
            refreshed.diagnostics = "incomplete: 1".into();
            app.update(Event::Refresh(Ok(refreshed)));
            assert_eq!(app.footer, message);
            assert_eq!(rendered_footer(&app, 200), message);
            app.update(Event::Key(KeyCode::Up));
            assert!(app.footer.is_empty());
            assert_eq!(
                rendered_footer(&app, 200),
                "↑↓ move · enter repo view · o open session · q quit · incomplete: 1"
            );
        }
    }

    #[test]
    fn checking_message_survives_refreshes_and_keys_until_opened() {
        let mut app = App::default();
        app.update(Event::Refresh(Ok(report())));
        assert!(matches!(
            app.update(Event::Key(KeyCode::Char('o'))).as_slice(),
            [Effect::FreshOpen { .. }]
        ));
        app.update(Event::Refresh(Ok(report())));
        app.update(Event::Refresh(Err("denied".into())));
        app.update(Event::Key(KeyCode::Down));
        assert_eq!(rendered_footer(&app, 200), "checking current session…");
        app.update(Event::Opened("session no longer found".into()));
        assert_eq!(rendered_footer(&app, 200), "session no longer found");
        app.update(Event::Key(KeyCode::Up));
        assert_eq!(
            rendered_footer(&app, 200),
            "refresh failed: denied; keeping last report"
        );
    }

    #[test]
    fn footer_prioritizes_open_then_error_then_help_with_diagnostics() {
        let mut app = App::default();
        let mut initial = report();
        initial.diagnostics = "bad records: 2 · incomplete: 1".into();
        app.update(Event::Refresh(Ok(initial)));
        assert_eq!(
            rendered_footer(&app, 200),
            "↑↓ move · enter repo view · o open session · q quit · bad records: 2 · incomplete: 1"
        );
        assert_eq!(rendered_footer(&app, 9), "↑↓ move ·");
        app.update(Event::Key(KeyCode::Char('t')));
        assert_eq!(
            rendered_footer(&app, 200),
            "↑↓ select · +/- zoom · ←→ pan · 0 reset · esc home · o open · f fork · q quit · bad records: 2 · incomplete: 1"
        );
        app.update(Event::Refresh(Err("denied".into())));
        assert_eq!(
            rendered_footer(&app, 200),
            "refresh failed: denied; keeping last report"
        );
        app.update(Event::Opened("child failed".into()));
        assert_eq!(rendered_footer(&app, 200), "child failed");
        app.update(Event::Refresh(Err("still denied".into())));
        assert_eq!(rendered_footer(&app, 200), "child failed");
        app.update(Event::Key(KeyCode::Up));
        assert_eq!(
            rendered_footer(&app, 200),
            "refresh failed: still denied; keeping last report"
        );
        app.update(Event::Refresh(Ok(report())));
        assert_eq!(
            app.footer_text(),
            "↑↓ select · +/- zoom · ←→ pan · 0 reset · esc home · o open · f fork · q quit"
        );
    }

    #[test]
    fn bell_only_on_entry_to_waiting_and_not_on_first_report() {
        let mut app = App::default();
        let mut idle = report();
        state(&mut idle, State::Idle);
        assert!(app.update(Event::Refresh(Ok(idle))).is_empty());
        let mut waiting = report();
        state(&mut waiting, State::Waiting);
        assert_eq!(app.update(Event::Refresh(Ok(waiting))), [Effect::Bell]);
        let mut waiting = report();
        state(&mut waiting, State::Waiting);
        assert!(app.update(Event::Refresh(Ok(waiting))).is_empty());
        let mut first_waiting = report();
        state(&mut first_waiting, State::Waiting);
        assert!(
            App::default()
                .update(Event::Refresh(Ok(first_waiting)))
                .is_empty()
        );
        assert!(app.update(Event::Refresh(Err("denied".into()))).is_empty());
        let mut waiting = report();
        state(&mut waiting, State::Waiting);
        assert!(app.update(Event::Refresh(Ok(waiting))).is_empty());
        let mut idle = report();
        state(&mut idle, State::Idle);
        app.update(Event::Refresh(Ok(idle)));
        let mut waiting = report();
        state(&mut waiting, State::Waiting);
        assert_eq!(app.update(Event::Refresh(Ok(waiting))), [Effect::Bell]);
    }
    #[test]
    fn each_new_waiting_id_gets_one_bell() {
        let mut app = App::default();
        let mut idle = report();
        state(&mut idle, State::Idle);
        app.update(Event::Refresh(Ok(idle)));
        let mut waiting = report();
        let row = waiting
            .overview
            .repos
            .iter_mut()
            .flat_map(|group| &mut group.rows)
            .find_map(|row| match row {
                Row::Session(row) if !row.children.is_empty() => Some(row),
                _ => None,
            })
            .unwrap();
        row.state = State::Waiting;
        row.children[0].state = State::Waiting;
        assert_eq!(
            app.update(Event::Refresh(Ok(waiting))),
            [Effect::Bell, Effect::Bell]
        );
    }

    #[test]
    fn failed_refresh_keeps_report_and_blocks_open_until_success() {
        let mut app = App::default();
        app.update(Event::Refresh(Ok(report())));
        let previous = app.report.as_ref().unwrap().overview.clone();
        app.update(Event::Refresh(Err("permission denied".into())));
        assert_eq!(app.report.as_ref().unwrap().overview, previous);
        assert_eq!(
            app.footer_text(),
            "refresh failed: permission denied; keeping last report"
        );
        for key in ['o', 'f'] {
            assert!(app.update(Event::Key(KeyCode::Char(key))).is_empty());
        }
        app.update(Event::Refresh(Ok(report())));
        let id = app.selected().unwrap().1.unwrap();
        assert_eq!(
            app.update(Event::Key(KeyCode::Char('o'))),
            [Effect::FreshOpen { id, fork: false }]
        );
        assert!(app.update(Event::Key(KeyCode::Char('o'))).is_empty());
        app.update(Event::Opened("not found".into()));
        assert!(!app.update(Event::Key(KeyCode::Char('f'))).is_empty());
    }
    #[test]
    fn refresh_preserves_selected_full_id_when_order_changes() {
        let mut app = App::default();
        app.update(Event::Refresh(Ok(report())));
        let selected = app.selected().unwrap();
        let mut reordered = report();
        reordered.overview.repos.reverse();
        app.update(Event::Refresh(Ok(reordered)));
        assert_eq!(app.selected().unwrap(), selected);
    }
}

#[cfg(test)]
mod phase6_tests {
    use super::*;
    use multi_link_model::Row;
    fn refresh() -> Box<Refresh> {
        Box::new(Refresh {
            overview: serde_json::from_str(include_str!(
                "../../../fixtures/expected/phase6/overview.json"
            ))
            .unwrap(),
            links: LinksReport {
                links: vec![],
                unparented: vec![],
            },
            diagnostics: String::new(),
        })
    }
    fn app() -> App {
        let mut app = App::default();
        app.update(Event::Refresh(Ok(refresh())));
        app
    }
    fn select_repo(app: &mut App, suffix: &str) {
        let view = app.view().unwrap();
        app.selection = app
            .targets(&view)
            .iter()
            .position(|(repo, _)| repo.ends_with(suffix))
            .unwrap();
    }
    #[test]
    fn home_window_follows_history_settings_after_refresh() {
        for (length, all, start, refreshed_start, period, refreshed_period) in [
            (
                Some(std::time::Duration::from_secs(16 * 3600)),
                false,
                "2026-11-02T14:25:00Z",
                "2026-11-02T15:25:00Z",
                "16h",
                "16h",
            ),
            (
                None,
                true,
                "2026-11-03T05:05:00Z",
                "2026-11-03T05:05:00Z",
                "1h20",
                "2h20",
            ),
        ] {
            let mut app = App {
                history_window: length,
                history_all: all,
                ..App::default()
            };
            app.update(Event::Refresh(Ok(refresh())));
            for (expected_start, expected_period) in
                [(start, period), (refreshed_start, refreshed_period)]
            {
                let report = &app.report.as_ref().unwrap().overview;
                assert_eq!(
                    app.home_options().window,
                    Some(TimeWindow::new(expected_start.parse().unwrap(), report.now))
                );
                let mut buf = Buffer::empty(Rect::new(0, 0, 160, 80));
                app.render(buf.area, &mut buf);
                let header: String = (0..160).map(|x| buf[(x, 0)].symbol()).collect();
                assert!(
                    header.contains(&format!("All repos · last {expected_period}")),
                    "{header}"
                );
                let mut next = refresh();
                next.overview.now = "2026-11-03T07:25:00Z".parse().unwrap();
                app.update(Event::Refresh(Ok(next)));
            }
        }
    }
    #[test]
    fn enter_opens_cards_summaries_and_quiet_repos_and_escape_restores_identity() {
        for suffix in ["repo-p", "repo-r", "repo-s", "repo-t"] {
            let mut app = app();
            select_repo(&mut app, suffix);
            let selected = app.selected().unwrap();
            app.update(Event::Key(KeyCode::Enter));
            assert_eq!(app.timeline.as_ref().unwrap().0, selected.0);
            let view = app.view().unwrap();
            assert!(
                app.targets(&view)
                    .iter()
                    .all(|(_, t)| !matches!(t, Target::Merged(_)))
            );
            app.update(Event::Key(KeyCode::Esc));
            assert_eq!(app.selected().unwrap(), selected);
        }
    }
    #[test]
    fn refresh_hides_and_reveals_imported_attention_rows_without_leaving_repo() {
        let mut app = app();
        app.selection = 1;
        let original = app.selected().unwrap();
        let mut hidden = refresh();
        let Row::Session(row) = &mut hidden.overview.repos[0].rows[1] else {
            panic!("session")
        };
        row.imported = true;
        app.update(Event::Refresh(Ok(hidden)));
        assert_eq!(app.selected().unwrap().0, original.0);
        assert_eq!(
            app.selected().unwrap().1.as_deref(),
            Some("60010000-0000-4000-8000-000000006001")
        );
        app.update(Event::Refresh(Ok(refresh())));
        assert_eq!(app.selection, 0);
        app.update(Event::Key(KeyCode::Down));
        assert_eq!(app.selected().unwrap(), original);
    }
    #[test]
    fn refresh_collapses_attention_to_summary_and_reveals_it_on_return_from_repo() {
        let mut app = app();
        select_repo(&mut app, "repo-q");
        let selected = app.selected().unwrap();
        app.update(Event::Key(KeyCode::Enter));
        let mut quiet = refresh();
        let Row::Session(row) = &mut quiet.overview.repos[1].rows[0] else {
            panic!("session")
        };
        row.state = State::Idle;
        app.update(Event::Refresh(Ok(quiet)));
        app.update(Event::Key(KeyCode::Esc));
        assert_eq!(app.selected().unwrap(), (selected.0.clone(), None));
        app.update(Event::Refresh(Ok(refresh())));
        assert_eq!(app.selected().unwrap(), selected);
    }
    #[test]
    fn navigation_open_refresh_and_zoom_preserve_identity() {
        let mut app = app();
        app.update(Event::Key(KeyCode::Down));
        let selected = app.selected().unwrap();
        assert_eq!(
            app.update(Event::Key(KeyCode::Char('o'))),
            [Effect::FreshOpen {
                id: selected.1.clone().unwrap(),
                fork: false
            }]
        );
        app.update(Event::Opened(String::new()));
        app.update(Event::Key(KeyCode::Enter));
        let window = app.timeline.as_ref().unwrap().1;
        app.update(Event::Key(KeyCode::Char('+')));
        assert_eq!(
            app.timeline.as_ref().unwrap().1,
            TimeWindow::new(
                multi_link_model::Timestamp::parse("2026-11-03T05:47:30Z").unwrap(),
                window.end
            )
        );
        app.update(Event::Key(KeyCode::Left));
        assert!(app.timeline.as_ref().unwrap().1.end < window.end);
        app.update(Event::Key(KeyCode::Char('0')));
        assert_eq!(app.timeline.as_ref().unwrap().1, window);
        let mut refresh = refresh();
        refresh.overview.repos.reverse();
        app.update(Event::Refresh(Ok(refresh)));
        app.update(Event::Key(KeyCode::Esc));
        assert_eq!(app.selected().unwrap(), selected);
        assert_eq!(app.update(Event::Key(KeyCode::Char('q'))), [Effect::Quit]);
    }
}

#[cfg(test)]
mod repo_tests {
    use super::*;
    use multi_link_model::Row;
    fn refresh() -> Box<Refresh> {
        let overview: OverviewReport = serde_json::from_str(include_str!(
            "../../../fixtures/expected/phase6/overview.json"
        ))
        .unwrap();
        let links = LinksReport {
            links: overview
                .repos
                .iter()
                .flat_map(crate::view::sessions)
                .flat_map(|r| r.handoffs.clone())
                .collect(),
            unparented: vec![],
        };
        Box::new(Refresh {
            overview,
            links,
            diagnostics: String::new(),
        })
    }
    fn app() -> App {
        let mut app = App::default();
        app.update(Event::Refresh(Ok(refresh())));
        let view = app.view().unwrap();
        app.selection = app
            .targets(&view)
            .iter()
            .position(|(repo, _)| repo.ends_with("repo-r"))
            .unwrap();
        app.update(Event::Key(KeyCode::Enter));
        app.offset = Some(time::UtcOffset::from_hms(-8, 0, 0).unwrap());
        app
    }
    fn text(app: &App) -> String {
        let mut b = Buffer::empty(Rect::new(0, 0, 120, 40));
        app.render(b.area, &mut b);
        (0..40)
            .map(|y| (0..120).map(|x| b[(x, y)].symbol()).collect::<String>())
            .collect::<Vec<_>>()
            .join("\n")
    }
    #[test]
    fn explicit_range_after_pan_and_back_with_zero() {
        let mut app = app();
        assert!(text(&app).contains("42m"));
        app.update(Event::Key(KeyCode::Left));
        assert!(app.explicit_range);
        assert!(!text(&app).contains("42m"));
        assert!(!text(&app).lines().nth(3).unwrap().contains("now"));
        assert!(text(&app).lines().nth(3).unwrap().contains("21:54"));
        app.update(Event::Key(KeyCode::Char('0')));
        assert!(!app.explicit_range);
        assert!(text(&app).contains("42m"));
        assert!(text(&app).lines().nth(3).unwrap().contains("22:25 now"));
        app.update(Event::Key(KeyCode::Right));
        let axis = text(&app).lines().nth(3).unwrap().to_owned();
        assert!(axis.contains("22:25 now"));
        assert!(axis.trim_end().ends_with("22:25 now"));
        let fitted = app.timeline.as_ref().unwrap().1;
        let bounds = TimeWindow::default_for(&app.report.as_ref().unwrap().overview);
        for _ in 0..10 {
            app.update(Event::Key(KeyCode::Left));
        }
        let left = app.timeline.as_ref().unwrap().1;
        assert_eq!(left.start, bounds.start);
        assert_eq!(
            left.end.epoch_ms() - left.start.epoch_ms(),
            fitted.end.epoch_ms() - fitted.start.epoch_ms()
        );
        for _ in 0..10 {
            app.update(Event::Key(KeyCode::Right));
        }
        assert_eq!(app.timeline.as_ref().unwrap().1, fitted);
        for _ in 0..10 {
            app.update(Event::Key(KeyCode::Char('-')));
        }
        assert_eq!(app.timeline.as_ref().unwrap().1, bounds);
        for key in [KeyCode::Left, KeyCode::Right, KeyCode::Char('-')] {
            app.update(Event::Key(key));
            assert_eq!(app.timeline.as_ref().unwrap().1, bounds);
        }
    }
    #[test]
    fn selection_and_open_after_import_toggle() {
        let mut app = app();
        let selected = app.selected().unwrap();
        app.update(Event::Key(KeyCode::Char('i')));
        assert!(app.imports);
        assert_eq!(app.selected().unwrap(), selected);
        assert!(text(&app).contains("import ·"));
        assert_eq!(
            app.update(Event::Key(KeyCode::Char('o'))),
            [Effect::FreshOpen {
                id: selected.1.unwrap(),
                fork: false
            }]
        );
        app.update(Event::Opened(String::new()));
        app.selection = 0;
        let imported = app.selected().unwrap().1.unwrap();
        assert_eq!(imported, "60140000-0000-4000-8000-000000006014");
        assert_eq!(
            app.update(Event::Key(KeyCode::Char('f'))),
            [Effect::FreshOpen {
                id: imported,
                fork: true
            }]
        );
        app.update(Event::Opened(String::new()));
        app.update(Event::Key(KeyCode::Char('i')));
        assert!(!app.imports);
        assert!(!text(&app).contains("import ·"));
        let id = app.selected().unwrap().1.unwrap();
        assert_eq!(id, "60110000-0000-4000-8000-000000006011");
        assert_eq!(
            app.update(Event::Key(KeyCode::Char('o'))),
            [Effect::FreshOpen { id, fork: false }]
        );
    }
    #[test]
    fn nesting_and_refresh_keep_full_selected_id() {
        let mut app = app();
        app.update(Event::Key(KeyCode::Down));
        let selected = app.selected().unwrap();
        assert_eq!(
            selected.1.as_deref(),
            Some("60130000-0000-4000-8000-000000006013")
        );
        let mut r = refresh();
        r.overview.repos.reverse();
        let group = r
            .overview
            .repos
            .iter_mut()
            .find(|g| g.repo.ends_with("repo-r"))
            .unwrap();
        group.rows.reverse();
        // Move the selected child beneath a different parent on refresh.
        let Row::Session(old) = &mut group.rows[2] else {
            panic!()
        };
        let child = old.children.pop().unwrap();
        let Row::Session(new) = &mut group.rows[1] else {
            panic!()
        };
        new.children.push(child);
        app.update(Event::Refresh(Ok(r)));
        assert_eq!(app.selected().unwrap(), selected);
        assert_eq!(
            app.update(Event::Key(KeyCode::Char('o'))),
            [Effect::FreshOpen {
                id: selected.1.unwrap(),
                fork: false
            }]
        );
    }
    #[test]
    fn fold_toggle_and_reset_keep_explicit_range_rules() {
        let mut app = app();
        app.update(Event::Key(KeyCode::Char('z')));
        assert!(app.unfold);
        assert!(!text(&app).contains("42m"));
        app.update(Event::Key(KeyCode::Char('z')));
        assert!(text(&app).contains("42m"));
        app.update(Event::Key(KeyCode::Char('+')));
        assert!(app.explicit_range);
        app.update(Event::Key(KeyCode::Char('z')));
        assert!(!text(&app).contains("42m"));
        app.update(Event::Key(KeyCode::Char('0')));
        assert!(!app.explicit_range);
        app.update(Event::Key(KeyCode::Char('z')));
        assert!(text(&app).contains("42m"));
    }
    #[test]
    fn refresh_advances_fitted_but_preserves_explicit_range() {
        let mut app = app();
        let mut r = refresh();
        r.overview.now = multi_link_model::Timestamp::parse("2026-11-03T06:26:00Z").unwrap();
        app.update(Event::Refresh(Ok(r)));
        assert_eq!(
            app.timeline.as_ref().unwrap().1.end,
            app.report.as_ref().unwrap().overview.now
        );
        app.update(Event::Key(KeyCode::Left));
        let window = app.timeline.as_ref().unwrap().1;
        app.update(Event::Refresh(Ok(refresh())));
        assert_eq!(app.timeline.as_ref().unwrap().1, window);
    }
    #[test]
    fn scroll_keeps_selected_lane_visible() {
        let mut app = app();
        app.update(Event::Key(KeyCode::Down));
        app.update(Event::Key(KeyCode::Down));
        let id = app.selected().unwrap().1.unwrap();
        assert_eq!(id, "60120000-0000-4000-8000-000000006012");
        for height in [3, 8, 16, 40] {
            let mut b = Buffer::empty(Rect::new(0, 0, 120, height));
            app.render(b.area, &mut b);
            assert!(
                b.content.iter().any(|c| c.symbol() == "▌"),
                "height {height}"
            );
        }
    }
    #[test]
    fn app_repo_matches_immutable_target() {
        let app = app();
        let mut b = Buffer::empty(Rect::new(0, 0, 120, 40));
        app.render(b.area, &mut b);
        let actual = (0..40)
            .map(|y| {
                let line = (0..120).map(|x| b[(x, y)].symbol()).collect::<String>();
                format!("{}\n", line.trim_end())
            })
            .collect::<String>();
        assert_eq!(
            actual,
            include_str!("../../../fixtures/expected/tui/repo-phase6.txt")
        );
    }
}
