//! Pure interaction state. Effects request work; they never contain a cached open decision.
use std::collections::HashSet;

use crossterm::event::KeyCode;
use multi_link_model::{LinksReport, OverviewReport, Row, State};
use ratatui::{buffer::Buffer, layout::Rect, style::Style};

use crate::{Target, TimeWindow, selectable, timeline::render_timeline_selected};

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
    expanded: HashSet<String>,
    timeline: Option<(String, TimeWindow)>,
    home_selection: usize,
    waiting: Option<HashSet<String>>,
    healthy: bool,
    opening: bool,
}

impl App {
    /// A presentation copy expands merged rows without changing core's grouping or placement.
    fn view(&self) -> Option<OverviewReport> {
        let mut report = self.report.as_ref()?.overview.clone();
        if let Some((repo, _)) = &self.timeline {
            report.repos.retain(|group| &group.repo == repo);
        }
        for group in &mut report.repos {
            if self.timeline.is_some() || self.expanded.contains(&group.repo) {
                group.rows = group
                    .rows
                    .iter()
                    .flat_map(|row| match row {
                        Row::Session(row) => vec![Row::Session(row.clone())],
                        Row::Merged(merged) => {
                            merged.rows.iter().cloned().map(Row::Session).collect()
                        }
                    })
                    .collect();
            }
        }
        Some(report)
    }

    fn selected(&self) -> Option<(String, Option<String>)> {
        let view = self.view()?;
        let targets = selectable(&view);
        let (repo, target) = targets.get(self.selection)?;
        Some((
            repo.to_string(),
            match target {
                Target::Session(row) => Some(row.id.clone()),
                Target::Merged(_) => None,
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
                let selected = self.view().and_then(|view| {
                    selectable(&view).get(self.selection).map(|(repo, target)| {
                        let (id, harness) = match target {
                            Target::Session(row) => (Some(row.id.clone()), row.harness),
                            Target::Merged(row) => (None, row.harness),
                        };
                        (repo.to_string(), id, harness)
                    })
                });
                let waiting = waiting_ids(&report.overview);
                let bells = self
                    .waiting
                    .as_ref()
                    .map_or(0, |previous| waiting.difference(previous).count());
                self.waiting = Some(waiting);
                self.refresh_error = None;
                self.report = Some(report);
                self.healthy = true;
                // Keep the full id selected when needs-you sorting moves rows.
                if let Some(view) = self.view() {
                    let targets = selectable(&view);
                    self.selection = selected
                        .and_then(|(repo, id, harness)| {
                            targets.iter().position(|(r, target)| {
                                *r == repo
                                    && match target {
                                        Target::Session(row) => id.as_deref() == Some(&row.id),
                                        Target::Merged(row) => {
                                            id.is_none() && row.harness == harness
                                        }
                                    }
                            })
                        })
                        .unwrap_or(self.selection)
                        .min(targets.len().saturating_sub(1));
                }
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
                    let count = self.view().map_or(0, |view| selectable(&view).len());
                    self.selection = (self.selection + 1).min(count.saturating_sub(1));
                }
                KeyCode::Esc => {
                    if self.timeline.take().is_some() {
                        self.selection = self.home_selection;
                    } else {
                        self.expanded.clear();
                        self.selection = 0;
                    }
                }
                KeyCode::Enter | KeyCode::Char('t') if self.timeline.is_none() => {
                    if let Some((repo, id)) = self.selected() {
                        if key == KeyCode::Enter && id.is_none() {
                            self.expanded.insert(repo);
                        } else {
                            self.home_selection = self.selection;
                            self.timeline = Some((
                                repo,
                                TimeWindow::default_for(&self.report.as_ref().unwrap().overview),
                            ));
                            self.selection = 0;
                            if let Some(id) = id {
                                let view = self.view().unwrap();
                                self.selection = selectable(&view).iter().position(|(_, target)| matches!(target, Target::Session(row) if row.id == id)).unwrap_or(0);
                            }
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
                        self.footer = "enter expands a merged row first".into();
                    }
                }
                // +/- zoom around the right edge; left/right pan by half a window.
                KeyCode::Char('+' | '-' | '0') | KeyCode::Left | KeyCode::Right => {
                    if let Some((_, window)) = &mut self.timeline {
                        let seconds = |stamp: multi_link_model::Timestamp| -> i64 {
                            use time::{OffsetDateTime, format_description::well_known::Rfc3339};
                            OffsetDateTime::parse(&stamp.to_string(), &Rfc3339)
                                .unwrap()
                                .unix_timestamp()
                        };
                        let end = seconds(window.end);
                        let length = (seconds(window.end) - seconds(window.start)).max(1);
                        let (end, length) = match key {
                            KeyCode::Char('+') => (end, (length / 2).max(1)),
                            KeyCode::Char('-') => (end, (length * 2).min(31_536_000)),
                            KeyCode::Left => (end - (length / 2).max(1), length),
                            KeyCode::Right => (end + (length / 2).max(1), length),
                            _ => (seconds(self.report.as_ref().unwrap().overview.now), 7200),
                        };
                        if let Ok(end) = multi_link_model::Timestamp::from_epoch_ms(end * 1000) {
                            *window = TimeWindow::ending_at(
                                end,
                                std::time::Duration::from_secs(length as u64),
                            );
                        }
                    }
                }
                _ => {}
            },
        }
        Vec::new()
    }

    pub fn render(&self, area: Rect, buf: &mut Buffer) {
        let area = area.intersection(buf.area);
        if area.is_empty() {
            return;
        }
        let body = Rect::new(area.x, area.y, area.width, area.height.saturating_sub(1));
        if let (Some(view), Some(report)) = (self.view(), &self.report) {
            if let Some((repo, window)) = &self.timeline {
                render_timeline_selected(
                    &view,
                    &report.links,
                    repo,
                    *window,
                    Some(self.selection),
                    body,
                    buf,
                );
            } else {
                crate::home::render_home_with_footer(&view, Some(self.selection), false, body, buf);
            }
        }
        let footer = self.footer_text();
        buf.set_stringn(
            area.x,
            area.bottom() - 1,
            &footer,
            area.width as usize,
            Style::default(),
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
            "↑↓ select · enter expand/timeline · t timeline · esc collapse · o open · f fork · q quit"
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
        .flat_map(|group| &group.rows)
        .flat_map(|row| match row {
            Row::Session(row) => std::slice::from_ref(row),
            Row::Merged(merged) => merged.rows.as_slice(),
        })
        .flat_map(|row| std::iter::once(row).chain(&row.children))
        .filter(|row| row.state == State::Waiting)
        .map(|row| row.id.clone())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
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
                "↑↓ select · enter expand/timeline · t timeline · esc collapse · o open · f fork · q quit · incomplete: 1"
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
            "↑↓ select · enter expand/timeline · t timeline · esc collapse · o open · f fork · q quit · bad records: 2 · incomplete: 1"
        );
        assert_eq!(rendered_footer(&app, 9), "↑↓ select");
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
            rendered_footer(&app, 200),
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
    fn merged_rows_expand_before_action_and_timeline_navigates() {
        let mut app = App::default();
        app.update(Event::Refresh(Ok(report())));
        let view = app.view().unwrap();
        app.selection = selectable(&view)
            .iter()
            .position(|(_, t)| matches!(t, Target::Merged(_)))
            .unwrap();
        let (repo, _) = app.selected().unwrap();
        assert!(app.update(Event::Key(KeyCode::Char('o'))).is_empty());
        app.update(Event::Key(KeyCode::Enter));
        assert!(app.selected().unwrap().1.is_some());
        let home_selection = app.selection;
        app.update(Event::Key(KeyCode::Char('t')));
        assert_eq!(app.timeline.as_ref().unwrap().0, repo);
        let window = app.timeline.as_ref().unwrap().1;
        app.update(Event::Key(KeyCode::Char('+')));
        assert_eq!(
            app.timeline.as_ref().unwrap().1,
            TimeWindow::ending_at(window.end, std::time::Duration::from_secs(3600))
        );
        app.update(Event::Key(KeyCode::Right));
        assert_ne!(app.timeline.as_ref().unwrap().1.end, window.end);
        app.update(Event::Key(KeyCode::Char('0')));
        assert_eq!(app.timeline.as_ref().unwrap().1, window);
        for _ in 0..100 {
            app.update(Event::Key(KeyCode::Down));
        }
        let view = app.view().unwrap();
        assert_eq!(app.selection, selectable(&view).len() - 1);
        for (width, height) in [(0, 0), (1, 1), (30, 8), (100, 16)] {
            let area = Rect::new(0, 0, width, height);
            app.render(area, &mut Buffer::empty(area));
        }
        app.update(Event::Key(KeyCode::Esc));
        assert_eq!(app.selection, home_selection);
        app.update(Event::Key(KeyCode::Esc));
        assert!(app.expanded.is_empty());
        assert_eq!(app.update(Event::Key(KeyCode::Char('q'))), [Effect::Quit]);
    }
    #[test]
    fn timeline_scroll_keeps_last_selected_lane_visible_even_in_one_body_row() {
        let mut app = App::default();
        app.update(Event::Refresh(Ok(report())));
        app.update(Event::Key(KeyCode::Char('t')));
        for _ in 0..100 {
            app.update(Event::Key(KeyCode::Down));
        }
        let view = app.view().unwrap();
        let targets = selectable(&view);
        let Target::Session(selected) = targets[app.selection].1 else {
            panic!("session")
        };
        let (repo, window) = app.timeline.as_ref().unwrap();
        for height in [3, 8, 16] {
            let area = Rect::new(0, 0, 100, height);
            let mut buffer = Buffer::empty(area);
            render_timeline_selected(
                &view,
                &app.report.as_ref().unwrap().links,
                repo,
                *window,
                Some(app.selection),
                area,
                &mut buffer,
            );
            let selected_lines = (0..height)
                .filter(|y| {
                    buffer[(0, *y)]
                        .modifier
                        .contains(ratatui::style::Modifier::REVERSED)
                })
                .map(|y| {
                    (0..100)
                        .map(|x| buffer[(x, y)].symbol())
                        .collect::<String>()
                })
                .collect::<Vec<_>>();
            assert_eq!(selected_lines.len(), 1, "height {height}");
            assert!(selected_lines[0].contains(&crate::home::short_id(&selected.id)));
        }
    }

    #[test]
    fn refresh_preserves_selected_merged_harness_and_enter_expands_both() {
        use multi_link_model::Harness;
        let mut refresh = report();
        refresh.overview = serde_json::from_str(include_str!(
            "../../../fixtures/expected/phase4/overview.json"
        ))
        .unwrap();
        refresh
            .overview
            .repos
            .retain(|group| group.repo == "/home/dev/repo-i");
        let mut app = App::default();
        app.update(Event::Refresh(Ok(refresh)));
        let mut reordered = report();
        reordered.overview = app.report.as_ref().unwrap().overview.clone();
        reordered.overview.repos[0].rows.reverse();
        assert_eq!(app.selection, 0);
        app.update(Event::Refresh(Ok(reordered)));
        assert_eq!(app.selection, 1);
        let view = app.view().unwrap();
        assert!(matches!(selectable(&view)[app.selection].1,
            Target::Merged(row) if row.harness == Harness::Codex));
        assert!(app.update(Event::Key(KeyCode::Char('o'))).is_empty());
        let area = Rect::new(0, 0, 140, 20);
        let mut buffer = Buffer::empty(area);
        app.render(area, &mut buffer);
        let rendered: String = buffer.content.iter().map(|cell| cell.symbol()).collect();
        assert!(rendered.contains("codex ×4"));
        assert!(rendered.contains("claude ×4"));
        app.update(Event::Key(KeyCode::Enter));
        let view = app.view().unwrap();
        assert_eq!(selectable(&view).len(), 8);
        assert!(
            selectable(&view)
                .iter()
                .all(|(_, target)| matches!(target, Target::Session(_)))
        );
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
