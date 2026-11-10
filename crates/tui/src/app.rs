//! Pure interaction state. Effects request work; they never contain a cached open decision.
use std::{cell::Cell, collections::HashSet};

use crossterm::event::KeyCode;
use multi_link_model::{
    GraphReport, Harness, Link, LinksReport, OverviewReport, RepoGroup, SessionRow, State,
};
use ratatui::{buffer::Buffer, layout::Rect};

use crate::{
    GraphOptions, RepoOptions, Target, TimeWindow, graph_first_slot, render_graph,
    timeline::{lanes, render_repo},
};

pub struct Refresh {
    pub overview: OverviewReport,
    pub links: LinksReport,
    pub diagnostics: String,
    /// Graph of the step the App asked for, built on this refresh; None when none is open.
    pub graph: Option<GraphReport>,
    /// Live part indices belonging only to this refresh's graph.
    pub graph_live: Vec<usize>,
    /// Step ids `[` `]` walk for that graph's repo, in G9 order; empty when none is open.
    pub graph_steps: Vec<String>,
    /// Per Codex session id: latest tagged incoming hand-off step from a same-repo parent, all-time.
    pub codex_steps: std::collections::HashMap<String, String>,
    /// Per Claude session id: (repo, step) of each direct started child's latest tagged
    /// hand-off, all-time, in the order `s` lists them (G9 amendment).
    pub started_steps: std::collections::HashMap<String, Vec<(String, String)>>,
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
    FreshOpen {
        id: String,
        fork: bool,
    },
    Quit,
    /// Ask the worker to build this (repo, step) on every refresh from now on; None stops it.
    Graph(Option<(String, String)>),
}

/// Below this drawn width the graph is a list and `↑↓` move along it (G10).
const GRAPH_WIDE: u16 = 120;

/// The open step graph (G9). The repo view's state stays untouched underneath it.
struct GraphView {
    repo: String,
    step: String,
    /// None until a refresh brings the report for exactly this (repo, step).
    report: Option<GraphReport>,
    live: Vec<usize>,
    steps: Vec<String>,
    selected: usize,
    first_slot: usize,
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
    graph: Option<GraphView>,
    /// Steps of a lane's direct started children (G9 amendment) and the chosen entry.
    picker: Option<(Vec<(String, String)>, usize)>,
    /// Width of the last render; 0 means never drawn and counts as wide.
    drawn_width: Cell<u16>,
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
            Event::Refresh(Ok(mut report)) => {
                let graph = report.graph.take();
                let steps = std::mem::take(&mut report.graph_steps);
                let live = std::mem::take(&mut report.graph_live);
                self.take_graph(graph, steps, live);
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
            Event::Key(KeyCode::Char('q')) => return vec![Effect::Quit],
            Event::Key(key) if self.picker.is_some() => return self.picker_key(key),
            Event::Key(key) if self.graph.is_some() => return self.graph_key(key),
            Event::Key(key) => match key {
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
                // An open check owns the footer until `Opened`; a picker would be hidden (review #5).
                KeyCode::Char('s') if self.timeline.is_some() && !self.opening => {
                    return self.step_key();
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

    /// A one-shot footer message; never hides `checking current session…`.
    fn say(&mut self, message: &str) {
        if !self.opening {
            self.footer = message.into();
        }
    }

    fn open_graph(&mut self, repo: String, step: String, steps: Vec<String>) -> Vec<Effect> {
        self.graph = Some(GraphView {
            repo: repo.clone(),
            step: step.clone(),
            report: None,
            live: Vec::new(),
            steps,
            selected: 0,
            first_slot: 0,
        });
        vec![Effect::Graph(Some((repo, step)))]
    }

    /// `s` on a repo-view lane (G9 and its amendment).
    fn step_key(&mut self) -> Vec<Effect> {
        let Some(view) = self.view() else {
            return Vec::new();
        };
        let Some(report) = self.report.as_ref() else {
            return Vec::new();
        };
        let found = match self.targets(&view).get(self.selection) {
            Some((repo, Target::Session(row))) => {
                let group = view.repos.iter().find(|g| g.repo == *repo);
                let all_time = (row.harness == Harness::Codex)
                    .then(|| report.codex_steps.get(&row.id).cloned())
                    .flatten();
                match all_time
                    .or_else(|| group.and_then(|group| own_step(group, row, &report.links.links)))
                {
                    Some(step) => Ok((repo.to_string(), step)),
                    None if row.harness == Harness::Claude => Err(report
                        .started_steps
                        .get(&row.id)
                        .cloned()
                        .unwrap_or_default()),
                    None => Err(Vec::new()),
                }
            }
            _ => Err(Vec::new()),
        };
        match found {
            Ok((repo, step)) => self.open_graph(repo, step, Vec::new()),
            Err(entries) if !entries.is_empty() => {
                self.picker = Some((entries, 0));
                Vec::new()
            }
            Err(_) => {
                self.say("no step on this session");
                Vec::new()
            }
        }
    }

    fn picker_key(&mut self, key: KeyCode) -> Vec<Effect> {
        let Some((entries, chosen)) = &mut self.picker else {
            return Vec::new();
        };
        match key {
            KeyCode::Left => *chosen = chosen.saturating_sub(1),
            KeyCode::Right => *chosen = (*chosen + 1).min(entries.len().saturating_sub(1)),
            KeyCode::Enter => {
                let (repo, step) = entries[*chosen].clone();
                self.picker = None;
                return self.open_graph(repo, step, Vec::new());
            }
            KeyCode::Esc => self.picker = None,
            _ => {}
        }
        Vec::new()
    }

    fn graph_key(&mut self, key: KeyCode) -> Vec<Effect> {
        let width = self.drawn_width.get();
        let wide = width == 0 || width >= GRAPH_WIDE;
        match key {
            KeyCode::Esc => {
                self.graph = None;
                return vec![Effect::Graph(None)];
            }
            KeyCode::Left | KeyCode::Right if wide => self.move_part(key == KeyCode::Right),
            KeyCode::Up | KeyCode::Down if !wide => self.move_part(key == KeyCode::Down),
            KeyCode::Char(c @ ('[' | ']')) => {
                let graph = self.graph.as_ref().unwrap();
                let position = graph.steps.iter().position(|id| *id == graph.step);
                let next = position
                    .and_then(|i| {
                        if c == '[' {
                            i.checked_sub(1)
                        } else {
                            Some(i + 1)
                        }
                    })
                    .and_then(|i| graph.steps.get(i));
                match next {
                    Some(step) => {
                        let (repo, step, steps) =
                            (graph.repo.clone(), step.clone(), graph.steps.clone());
                        return self.open_graph(repo, step, steps);
                    }
                    None if c == '[' => self.say("no earlier step"),
                    None => self.say("no later step"),
                }
            }
            KeyCode::Char('o' | 'f') => return self.open_part(key == KeyCode::Char('f')),
            _ => {}
        }
        Vec::new()
    }

    fn move_part(&mut self, forward: bool) {
        let Some(graph) = &mut self.graph else { return };
        let Some(report) = &graph.report else { return };
        if report.parts.is_empty() {
            return;
        }
        graph.selected = if forward {
            (graph.selected + 1).min(report.parts.len() - 1)
        } else {
            graph.selected.saturating_sub(1)
        };
        graph.first_slot = graph_first_slot(report, graph.selected, graph.first_slot);
    }

    /// `o` / `f` on the selected part: the panel's rule for drawing `o open · f fork`.
    fn open_part(&mut self, fork: bool) -> Vec<Effect> {
        let Some(part) = self
            .graph
            .as_ref()
            .and_then(|g| g.report.as_ref()?.parts.get(g.selected))
        else {
            return Vec::new();
        };
        let available = part.harness != Harness::Codex || crate::graph::has_run(part);
        let session = part.session.as_deref().filter(|id| !id.is_empty());
        match (available, session) {
            (true, Some(id)) => {
                if self.healthy && !self.opening {
                    let id = id.to_owned();
                    self.opening = true;
                    self.footer = "checking current session…".into();
                    return vec![Effect::FreshOpen { id, fork }];
                }
            }
            (false, _) => {
                let words = crate::graph::missing_run(part);
                self.say(&words);
            }
            (true, None) => self.say("run unavailable"),
        }
        Vec::new()
    }

    /// Take a refresh's graph when it is for the open (repo, step); keep the selected part by
    /// session and position, else clamp the old index.
    fn take_graph(&mut self, report: Option<GraphReport>, steps: Vec<String>, live: Vec<usize>) {
        let (Some(graph), Some(report)) = (&mut self.graph, report) else {
            return;
        };
        if report.repo != graph.repo || report.step.id != graph.step {
            return;
        }
        let last = report.parts.len().saturating_sub(1);
        let kept = graph.report.as_ref().and_then(|old| {
            let part = old.parts.get(graph.selected)?;
            let nth = old.parts[..graph.selected]
                .iter()
                .filter(|p| p.session == part.session)
                .count();
            report
                .parts
                .iter()
                .enumerate()
                .filter(|(_, p)| p.session == part.session)
                .nth(nth)
                .map(|(i, _)| i)
        });
        graph.selected = kept.unwrap_or(graph.selected).min(last);
        graph.first_slot = graph_first_slot(&report, graph.selected, graph.first_slot);
        graph.live = live;
        graph.report = Some(report);
        graph.steps = steps;
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
        self.drawn_width.set(area.width);
        let graph = self
            .graph
            .as_ref()
            .and_then(|g| Some((g, g.report.as_ref()?)));
        if let Some((graph, report)) = graph {
            let options = GraphOptions {
                live: &graph.live,
                selected: graph.selected,
                first_slot: graph.first_slot,
                theme: self.home_options().theme,
                offset: self.offset.unwrap_or(time::UtcOffset::UTC),
            };
            render_graph(report, &options, area, buf);
        } else if let (Some(view), Some(report)) = (self.view(), &self.report) {
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
        let loading = self.graph.as_ref().is_some_and(|g| g.report.is_none());
        if self.report.is_some()
            && !loading
            && self.picker.is_none()
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
        if let Some((entries, chosen)) = &self.picker {
            let steps: Vec<String> = entries.iter().map(|(_, s)| format!("step {s}")).collect();
            return format!(
                "started: {} · ←→ choose · enter opens {} · esc cancel",
                steps.join(", "),
                steps[*chosen]
            );
        }
        if let Some(error) = &self.refresh_error {
            return error.clone();
        }
        let Some(report) = &self.report else {
            return "loading sessions… · q quit".into();
        };
        if let Some(graph) = self.graph.as_ref().filter(|g| g.report.is_none()) {
            return format!("loading step {}…", graph.step);
        }
        let width = self.drawn_width.get();
        let help = if self.graph.is_some() && width != 0 && width < GRAPH_WIDE {
            "↑↓ move along the step · [ ] prev / next step · o open · f fork · esc repo view · q quit"
        } else if self.graph.is_some() {
            "←→ move along the step · [ ] prev / next step · o open · f fork · esc repo view · q quit"
        } else if self.timeline.is_some() {
            "↑↓ select · +/- zoom · ←→ pan · 0 reset · esc home · o open · f fork · s step · q quit"
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

/// A lane's latest tagged hand-off step within the window (Codex fallback when the
/// worker has no all-time entry): a Claude lane's own hand-offs (it is the parent), a
/// Codex lane's incoming ones (it is the child) whose parent is a session of the lane's repo
/// group, as overview placement and the G1 graph require. Latest = max `at`, ties by later
/// position.
fn own_step(group: &RepoGroup, row: &SessionRow, links: &[Link]) -> Option<String> {
    let in_group: HashSet<&str> = match row.harness {
        Harness::Claude => HashSet::new(),
        Harness::Codex => crate::view::sessions(group)
            .map(|s| s.id.as_str())
            .collect(),
    };
    links
        .iter()
        .enumerate()
        .filter(|(_, link)| match row.harness {
            Harness::Claude => link.parent == row.id,
            Harness::Codex => {
                link.child.as_deref() == Some(row.id.as_str())
                    && in_group.contains(link.parent.as_str())
            }
        })
        .filter_map(|(i, link)| Some((link.at, i, link.step.clone()?)))
        .max_by_key(|(at, i, _)| (*at, *i))
        .map(|(_, _, step)| step)
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
            graph: None,
            graph_live: Vec::new(),
            graph_steps: Vec::new(),
            codex_steps: Default::default(),
            started_steps: Default::default(),
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
            "↑↓ select · +/- zoom · ←→ pan · 0 reset · esc home · o open · f fork · s step · q quit · bad records: 2 · incomplete: 1"
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
            "↑↓ select · +/- zoom · ←→ pan · 0 reset · esc home · o open · f fork · s step · q quit"
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
            graph: None,
            graph_live: Vec::new(),
            graph_steps: Vec::new(),
            codex_steps: Default::default(),
            started_steps: Default::default(),
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
            graph: None,
            graph_live: Vec::new(),
            graph_steps: Vec::new(),
            codex_steps: Default::default(),
            started_steps: Default::default(),
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
        assert!(text(&app).lines().nth(3).unwrap().contains("9:54pm"));
        app.update(Event::Key(KeyCode::Char('0')));
        assert!(!app.explicit_range);
        assert!(text(&app).contains("42m"));
        assert!(text(&app).lines().nth(3).unwrap().contains("10:25pm now"));
        app.update(Event::Key(KeyCode::Right));
        let axis = text(&app).lines().nth(3).unwrap().to_owned();
        assert!(axis.contains("10:25pm now"));
        assert!(axis.trim_end().ends_with("10:25pm now"));
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

#[cfg(test)]
mod graph_tests {
    use super::*;
    use multi_link_model::{GraphReport, Link, Timestamp};

    const REPO: &str = "/home/dev/multi-link-sandbox/repo-a";
    const REPO_K: &str = "/home/dev/repo-k";
    const K: &str = "84c00001";
    const B: &str = "84c00003";
    const A2: &str = "84c00004";

    fn read(path: &str) -> String {
        std::fs::read_to_string(format!("{}/../../{path}", env!("CARGO_MANIFEST_DIR"))).unwrap()
    }
    fn base() -> Box<Refresh> {
        Box::new(Refresh {
            overview: serde_json::from_str(&read("fixtures/expected/overview.json")).unwrap(),
            links: serde_json::from_str(&read("fixtures/expected/links.json")).unwrap(),
            diagnostics: String::new(),
            graph: None,
            graph_live: Vec::new(),
            graph_steps: Vec::new(),
            codex_steps: Default::default(),
            started_steps: Default::default(),
        })
    }
    fn started() -> Box<Refresh> {
        Box::new(Refresh {
            overview: serde_json::from_str(&read("fixtures/expected/started/overview.json"))
                .unwrap(),
            links: serde_json::from_str(&read("fixtures/expected/started/links.json")).unwrap(),
            diagnostics: String::new(),
            graph: None,
            graph_live: Vec::new(),
            graph_steps: Vec::new(),
            codex_steps: Default::default(),
            started_steps: Default::default(),
        })
    }
    fn graph(key: &str, repo: &str, step: &str) -> GraphReport {
        let mut report: GraphReport =
            serde_json::from_str(&read(&format!("fixtures/expected/phase8/graph-{key}.json")))
                .unwrap();
        report.repo = repo.into();
        report.step.id = step.into();
        report
    }
    fn at(text: &str) -> Timestamp {
        Timestamp::parse(text).unwrap()
    }
    /// Tag every link from `parent` (prefix) with `step`.
    fn tag(refresh: &mut Refresh, parent: &str, step: &str) {
        for link in &mut refresh.links.links {
            if link.parent.starts_with(parent) {
                link.step = Some(step.into());
            }
        }
    }
    /// A new tagged hand-off from `parent` (full id taken from an existing row), at `when`.
    fn handoff(refresh: &mut Refresh, parent: &str, step: &str, when: &str) {
        let full = refresh
            .overview
            .repos
            .iter()
            .flat_map(crate::view::sessions)
            .find(|row| row.id.starts_with(parent))
            .unwrap()
            .id
            .clone();
        let mut link: Link = refresh.links.links[0].clone();
        link.parent = full;
        link.step = Some(step.into());
        link.at = at(when);
        link.invocation = format!("toolu_{parent}_{step}");
        refresh.links.links.push(link);
    }
    fn with_graph(mut refresh: Box<Refresh>, graph: GraphReport, steps: &[&str]) -> Box<Refresh> {
        refresh.graph = Some(graph);
        refresh.graph_steps = steps.iter().map(|s| s.to_string()).collect();
        refresh
    }
    fn repo_view(app: &mut App, repo: &str, lane: &str) {
        let view = app.view().unwrap();
        app.selection = app
            .targets(&view)
            .iter()
            .position(|(r, _)| *r == repo)
            .unwrap();
        app.update(Event::Key(KeyCode::Enter));
        assert!(app.timeline.is_some());
        select_lane(app, lane);
    }
    fn select_lane(app: &mut App, lane: &str) {
        let view = app.view().unwrap();
        app.selection = app
            .targets(&view)
            .iter()
            .position(|(_, t)| matches!(t, Target::Session(row) if row.id.starts_with(lane)))
            .unwrap_or_else(|| panic!("lane {lane}"));
    }
    fn key(app: &mut App, code: KeyCode) -> Vec<Effect> {
        app.update(Event::Key(code))
    }
    fn tagged() -> Box<Refresh> {
        let mut r = base();
        tag(&mut r, "bb792d9c", "1.3");
        r
    }
    /// Repo view on lane bb792d9c (tagged 1.3), `s` pressed, graph `graph_key` delivered.
    fn graph_app(graph_key: &str, steps: &[&str]) -> App {
        let mut app = App::default();
        app.update(Event::Refresh(Ok(tagged())));
        repo_view(&mut app, REPO, "bb792d9c");
        assert_eq!(
            key(&mut app, KeyCode::Char('s')),
            [Effect::Graph(Some((REPO.into(), "1.3".into())))]
        );
        app.update(Event::Refresh(Ok(with_graph(
            tagged(),
            graph(graph_key, REPO, "1.3"),
            steps,
        ))));
        assert!(app.graph.as_ref().unwrap().report.is_some());
        app
    }
    fn draw(app: &App, width: u16, height: u16) -> Buffer {
        let mut b = Buffer::empty(Rect::new(0, 0, width, height));
        app.render(b.area, &mut b);
        b
    }
    fn row(b: &Buffer, y: u16) -> String {
        (0..b.area.width)
            .map(|x| b[(x, y)].symbol())
            .collect::<String>()
            .trim_end()
            .to_owned()
    }
    fn selected(app: &App) -> (usize, usize) {
        let g = app.graph.as_ref().unwrap();
        (g.selected, g.first_slot)
    }
    fn session_of(app: &App) -> String {
        let g = app.graph.as_ref().unwrap();
        g.report.as_ref().unwrap().parts[g.selected]
            .session
            .clone()
            .unwrap()
    }

    #[test]
    fn s_opens_latest_tagged_step_of_lane() {
        let mut r = tagged();
        tag(&mut r, "bb792d9c", "1.1");
        handoff(&mut r, "bb792d9c", "1.2", "2026-10-29T23:30:00Z");
        handoff(&mut r, "bb792d9c", "1.0", "2026-10-29T22:00:00Z");
        let mut app = App::default();
        app.update(Event::Refresh(Ok(r)));
        repo_view(&mut app, REPO, "bb792d9c");
        let before = row(&draw(&app, 120, 40), 20);
        assert_eq!(
            key(&mut app, KeyCode::Char('s')),
            [Effect::Graph(Some((REPO.into(), "1.2".into())))]
        );
        let b = draw(&app, 120, 40);
        assert_eq!(row(&b, 39), "loading step 1.2…");
        // The repo view stays on screen until a matching report arrives.
        assert_eq!(row(&b, 20), before);
    }

    #[test]
    fn s_on_codex_lane_uses_its_incoming_handoff() {
        let mut r = base();
        tag(&mut r, "bb792d9c", "1.3");
        tag(&mut r, "ef72378c", "1.4");
        for link in &mut r.links.links {
            if link.parent.starts_with("ef72378c") {
                link.at = at("2026-10-29T23:40:00Z");
            }
        }
        let mut app = App::default();
        app.update(Event::Refresh(Ok(r)));
        repo_view(&mut app, REPO, "bb792d9c");
        assert_eq!(
            key(&mut app, KeyCode::Char('s')),
            [Effect::Graph(Some((REPO.into(), "1.3".into())))]
        );
        key(&mut app, KeyCode::Esc);
        // The Codex child of both: its latest incoming tagged hand-off.
        select_lane(&mut app, "01a189e2");
        assert_eq!(
            key(&mut app, KeyCode::Char('s')),
            [Effect::Graph(Some((REPO.into(), "1.4".into())))]
        );
    }

    #[test]
    fn codex_all_time_step_wins_over_window_and_leaves_claude_unchanged() {
        let mut r = base();
        tag(&mut r, "bb792d9c", "1.3");
        let codex = r
            .overview
            .repos
            .iter()
            .flat_map(crate::view::sessions)
            .find(|row| row.id.starts_with("01a189e2"))
            .unwrap()
            .id
            .clone();
        let claude = r
            .overview
            .repos
            .iter()
            .flat_map(crate::view::sessions)
            .find(|row| row.id.starts_with("bb792d9c"))
            .unwrap()
            .id
            .clone();
        r.codex_steps.insert(codex, "1.4".into());
        r.codex_steps.insert(claude, "9.9".into());
        let mut app = App::default();
        app.update(Event::Refresh(Ok(r)));
        repo_view(&mut app, REPO, "01a189e2");
        assert_eq!(
            key(&mut app, KeyCode::Char('s')),
            [Effect::Graph(Some((REPO.into(), "1.4".into())))]
        );
        key(&mut app, KeyCode::Esc);
        select_lane(&mut app, "bb792d9c");
        assert_eq!(
            key(&mut app, KeyCode::Char('s')),
            [Effect::Graph(Some((REPO.into(), "1.3".into())))]
        );
    }

    #[test]
    fn s_without_step_says_no_step_on_this_session() {
        for lane in ["ef72378c", "01a189e2"] {
            let mut app = App::default();
            app.update(Event::Refresh(Ok(base())));
            repo_view(&mut app, REPO, lane);
            let selection = app.selection;
            assert!(key(&mut app, KeyCode::Char('s')).is_empty());
            assert!(app.graph.is_none() && app.picker.is_none());
            assert_eq!(row(&draw(&app, 120, 40), 39), "no step on this session");
            key(&mut app, KeyCode::Char('x'));
            assert!(app.footer.is_empty());
            assert_eq!(app.selection, selection);
        }
    }

    /// What the worker sends for K: its direct started children's steps, all-time.
    fn started_of_k(refresh: &mut Refresh, steps: &[(&str, &str)]) {
        refresh.started_steps.insert(
            "84c00001-0000-4000-8000-000000008401".into(),
            steps
                .iter()
                .map(|(repo, step)| (repo.to_string(), step.to_string()))
                .collect(),
        );
    }

    fn coordinator(r: Box<Refresh>) -> App {
        let mut app = App::default();
        app.update(Event::Refresh(Ok(r)));
        repo_view(&mut app, REPO_K, K);
        app
    }

    #[test]
    fn s_on_coordinator_lists_direct_started_children_steps() {
        // The picker lists exactly the worker's entries for K, in order; the overview's
        // `children` (2 h window, one repo) are not consulted.
        let mut r = started();
        handoff(&mut r, A2, "9.9", "2026-11-08T09:30:00Z");
        handoff(&mut r, B, "3.3", "2026-11-08T09:11:00Z");
        started_of_k(&mut r, &[(REPO_K, "2.1"), (REPO_K, "3.1")]);
        let mut app = coordinator(r);
        assert!(key(&mut app, KeyCode::Char('s')).is_empty());
        assert_eq!(
            row(&draw(&app, 120, 40), 39),
            "started: step 2.1, step 3.1 · ←→ choose · enter opens step 2.1 · esc cancel"
        );
    }

    #[test]
    fn own_tagged_handoff_wins_over_children() {
        let mut r = started();
        handoff(&mut r, K, "5.0", "2026-11-08T09:01:00Z");
        started_of_k(&mut r, &[(REPO_K, "2.1")]);
        let mut app = coordinator(r);
        assert_eq!(
            key(&mut app, KeyCode::Char('s')),
            [Effect::Graph(Some((REPO_K.into(), "5.0".into())))]
        );
        assert!(app.picker.is_none());
    }

    #[test]
    fn coordinator_with_no_graphed_children_says_no_step() {
        // A (K's child in the overview) has a tagged hand-off, but the worker sent no entry
        // for K (missing, then empty): no step.
        for entries in [None, Some(&[][..])] {
            let mut r = started();
            if let Some(entries) = entries {
                started_of_k(&mut r, entries);
            }
            assert!(r.links.links.iter().any(|l| l.step.is_some()));
            let mut app = coordinator(r);
            assert!(key(&mut app, KeyCode::Char('s')).is_empty());
            assert!(app.picker.is_none() && app.graph.is_none());
            assert_eq!(app.footer_text(), "no step on this session");
        }
    }

    #[test]
    fn picker_opens_the_childs_repo() {
        let mut r = started();
        started_of_k(&mut r, &[("/home/dev/repo-o", "6.1")]);
        let mut app = coordinator(r);
        key(&mut app, KeyCode::Char('s'));
        assert_eq!(
            key(&mut app, KeyCode::Enter),
            [Effect::Graph(Some((
                "/home/dev/repo-o".into(),
                "6.1".into()
            )))]
        );
    }

    #[test]
    fn s_while_open_check_pending_does_nothing() {
        let mut r = tagged();
        started_of_k(&mut r, &[(REPO_K, "2.1")]);
        let mut app = App::default();
        app.update(Event::Refresh(Ok(r)));
        repo_view(&mut app, REPO, "bb792d9c");
        assert!(matches!(
            key(&mut app, KeyCode::Char('o')).as_slice(),
            [Effect::FreshOpen { .. }]
        ));
        assert!(key(&mut app, KeyCode::Char('s')).is_empty());
        assert!(app.graph.is_none() && app.picker.is_none());
        assert_eq!(app.footer_text(), "checking current session…");
        // A coordinator lane does not open its picker either.
        let mut r = started();
        started_of_k(&mut r, &[(REPO_K, "2.1")]);
        let mut app = coordinator(r);
        assert!(matches!(
            key(&mut app, KeyCode::Char('o')).as_slice(),
            [Effect::FreshOpen { .. }]
        ));
        assert!(key(&mut app, KeyCode::Char('s')).is_empty());
        assert!(app.picker.is_none());
        assert_eq!(app.footer_text(), "checking current session…");
        // Once the check is answered, `s` works again.
        app.update(Event::Opened(String::new()));
        assert!(key(&mut app, KeyCode::Char('s')).is_empty());
        assert!(app.picker.is_some());
    }

    #[test]
    fn codex_lane_ignores_handoffs_from_another_repo() {
        let mut r = base();
        let other = r
            .overview
            .repos
            .iter()
            .find(|g| g.repo != REPO)
            .and_then(|g| crate::view::sessions(g).next())
            .unwrap()
            .id
            .clone();
        for link in &mut r.links.links {
            if link
                .child
                .as_deref()
                .is_some_and(|c| c.starts_with("01a189e2"))
            {
                link.parent = other.clone();
                link.step = Some("1.4".into());
            }
        }
        let mut app = App::default();
        app.update(Event::Refresh(Ok(r)));
        repo_view(&mut app, REPO, "01a189e2");
        assert!(key(&mut app, KeyCode::Char('s')).is_empty());
        assert!(app.graph.is_none() && app.picker.is_none());
        assert_eq!(app.footer_text(), "no step on this session");
    }

    fn picker_app() -> App {
        let mut r = started();
        started_of_k(&mut r, &[(REPO_K, "2.1"), (REPO_K, "3.1")]);
        let mut app = coordinator(r);
        key(&mut app, KeyCode::Char('s'));
        assert!(app.picker.is_some());
        app
    }

    #[test]
    fn picker_esc_keeps_repo_view() {
        let mut app = picker_app();
        let selection = app.selection;
        let window = app.timeline.clone();
        for code in [
            KeyCode::Down,
            KeyCode::Up,
            KeyCode::Char('+'),
            KeyCode::Char('o'),
            KeyCode::Char('s'),
            KeyCode::Char('i'),
        ] {
            assert!(key(&mut app, code).is_empty());
        }
        assert_eq!(app.selection, selection);
        assert_eq!(app.timeline, window);
        assert!(!app.imports);
        assert!(key(&mut app, KeyCode::Esc).is_empty());
        assert!(app.picker.is_none() && app.graph.is_none());
        assert_eq!(app.timeline, window);
        assert_eq!(app.selection, selection);
        assert!(app.footer_text().starts_with("↑↓ select"));
        let mut app = picker_app();
        assert_eq!(key(&mut app, KeyCode::Char('q')), [Effect::Quit]);
    }

    #[test]
    fn picker_enter_opens_chosen() {
        let mut app = picker_app();
        key(&mut app, KeyCode::Left);
        assert!(app.footer_text().contains("enter opens step 2.1"));
        key(&mut app, KeyCode::Right);
        key(&mut app, KeyCode::Right);
        assert_eq!(
            app.footer_text(),
            "started: step 2.1, step 3.1 · ←→ choose · enter opens step 3.1 · esc cancel"
        );
        assert_eq!(
            key(&mut app, KeyCode::Enter),
            [Effect::Graph(Some((REPO_K.into(), "3.1".into())))]
        );
        assert!(app.picker.is_none());
        assert_eq!(app.footer_text(), "loading step 3.1…");
    }

    #[test]
    fn arrows_clamp_at_both_ends() {
        let mut app = graph_app("1.3", &[]);
        let report = app.graph.as_ref().unwrap().report.clone().unwrap();
        assert_eq!(report.parts.len(), 7);
        draw(&app, 120, 40);
        key(&mut app, KeyCode::Left);
        assert_eq!(selected(&app), (0, 0));
        let mut first = 0;
        for i in 1..7 {
            key(&mut app, KeyCode::Right);
            first = crate::graph_first_slot(&report, i, first);
            assert_eq!(selected(&app), (i, first));
        }
        assert!(first > 0);
        key(&mut app, KeyCode::Right);
        assert_eq!(selected(&app), (6, first));
        for code in [KeyCode::Up, KeyCode::Down] {
            key(&mut app, code);
            assert_eq!(selected(&app), (6, first));
        }
        for _ in 0..10 {
            key(&mut app, KeyCode::Left);
        }
        assert_eq!(selected(&app), (0, 0));
    }

    #[test]
    fn narrow_uses_up_down_and_resize_switches() {
        let mut r = tagged();
        r.diagnostics = "incomplete: 1".into();
        let mut app = App::default();
        app.update(Event::Refresh(Ok(r.clone_box())));
        repo_view(&mut app, REPO, "bb792d9c");
        key(&mut app, KeyCode::Char('s'));
        app.update(Event::Refresh(Ok(with_graph(
            r,
            graph("1.3", REPO, "1.3"),
            &[],
        ))));
        let b = draw(&app, 130, 40);
        assert_eq!(
            row(&b, 39),
            "←→ move along the step · [ ] prev / next step · o open · f fork · esc repo view · q quit · incomplete: 1"
        );
        key(&mut app, KeyCode::Right);
        assert_eq!(selected(&app).0, 1);
        key(&mut app, KeyCode::Down);
        assert_eq!(selected(&app).0, 1);
        let b = draw(&app, 100, 40);
        assert!(row(&b, 39).starts_with("↑↓ move along the step · [ ]"));
        key(&mut app, KeyCode::Down);
        assert_eq!(selected(&app).0, 2);
        key(&mut app, KeyCode::Right);
        key(&mut app, KeyCode::Left);
        assert_eq!(selected(&app).0, 2);
        key(&mut app, KeyCode::Up);
        assert_eq!(selected(&app).0, 1);
    }

    #[test]
    fn brackets_walk_steps_and_stop_at_ends() {
        let mut app = graph_app("1.3", &["1.1", "1.3", "1.5"]);
        key(&mut app, KeyCode::Right);
        assert_eq!(
            key(&mut app, KeyCode::Char('[')),
            [Effect::Graph(Some((REPO.into(), "1.1".into())))]
        );
        let g = app.graph.as_ref().unwrap();
        assert_eq!((g.step.as_str(), g.selected, g.first_slot), ("1.1", 0, 0));
        assert!(g.report.is_none());
        assert!(key(&mut app, KeyCode::Char('[')).is_empty());
        assert_eq!(app.footer_text(), "no earlier step");
        assert_eq!(
            key(&mut app, KeyCode::Char(']')),
            [Effect::Graph(Some((REPO.into(), "1.3".into())))]
        );
        assert_eq!(
            key(&mut app, KeyCode::Char(']')),
            [Effect::Graph(Some((REPO.into(), "1.5".into())))]
        );
        assert!(key(&mut app, KeyCode::Char(']')).is_empty());
        assert_eq!(app.footer_text(), "no later step");
        // A step missing from the list has no neighbours.
        let mut app = graph_app("1.3", &["2.0"]);
        assert!(key(&mut app, KeyCode::Char('[')).is_empty());
        assert_eq!(app.footer_text(), "no earlier step");
        assert!(key(&mut app, KeyCode::Char(']')).is_empty());
        assert_eq!(app.footer_text(), "no later step");
    }

    #[test]
    fn live_indices_follow_only_the_accepted_graph() {
        let mut app = graph_app("1.3", &["1.3", "1.5"]);
        let mut fresh = with_graph(tagged(), graph("1.3", REPO, "1.3"), &["1.3", "1.5"]);
        fresh.graph_live = vec![1];
        app.update(Event::Refresh(Ok(fresh)));
        assert_eq!(app.graph.as_ref().unwrap().live, [1]);
        assert!(app.report.as_ref().unwrap().graph_live.is_empty());
        key(&mut app, KeyCode::Char(']'));
        assert!(app.graph.as_ref().unwrap().live.is_empty());
        let mut stale = with_graph(tagged(), graph("1.3", REPO, "1.3"), &["1.3", "1.5"]);
        stale.graph_live = vec![1];
        app.update(Event::Refresh(Ok(stale)));
        assert!(app.graph.as_ref().unwrap().live.is_empty());
        assert!(app.graph.as_ref().unwrap().report.is_none());
        let mut fresh = with_graph(tagged(), graph("1.5", REPO, "1.5"), &["1.3", "1.5"]);
        fresh.graph_live = vec![2];
        app.update(Event::Refresh(Ok(fresh)));
        assert_eq!(app.graph.as_ref().unwrap().live, [2]);
        // A stale response must not replace an already accepted graph or its live indices.
        let mut stale = with_graph(tagged(), graph("1.3", REPO, "1.3"), &[]);
        stale.graph_live = vec![0];
        app.update(Event::Refresh(Ok(stale)));
        assert_eq!(app.graph.as_ref().unwrap().live, [2]);
        app.update(Event::Refresh(Ok(with_graph(
            tagged(),
            graph("1.5", REPO, "1.5"),
            &[],
        ))));
        assert!(app.graph.as_ref().unwrap().live.is_empty());
    }

    #[test]
    fn stale_graph_refresh_is_ignored() {
        let mut app = graph_app("1.3", &["1.3", "1.5"]);
        key(&mut app, KeyCode::Char(']'));
        app.update(Event::Refresh(Ok(with_graph(
            tagged(),
            graph("1.3", REPO, "1.3"),
            &["1.3", "1.5"],
        ))));
        let g = app.graph.as_ref().unwrap();
        assert_eq!(g.step, "1.5");
        assert!(g.report.is_none());
        assert_eq!(app.footer_text(), "loading step 1.5…");
        // Same step id in another repo is another step.
        app.update(Event::Refresh(Ok(with_graph(
            tagged(),
            graph("1.5", "/home/dev/elsewhere", "1.5"),
            &[],
        ))));
        assert!(app.graph.as_ref().unwrap().report.is_none());
        app.update(Event::Refresh(Ok(with_graph(
            tagged(),
            graph("1.5", REPO, "1.5"),
            &["1.3", "1.5"],
        ))));
        assert_eq!(
            app.graph.as_ref().unwrap().report.as_ref().unwrap().step.id,
            "1.5"
        );
        // A refresh without a graph keeps the open one.
        app.update(Event::Refresh(Ok(tagged())));
        assert!(app.graph.as_ref().unwrap().report.is_some());
    }

    #[test]
    fn esc_restores_repo_selection_and_window() {
        let mut app = App::default();
        app.update(Event::Refresh(Ok(tagged())));
        repo_view(&mut app, REPO, "80cce3e4");
        key(&mut app, KeyCode::Char('+'));
        key(&mut app, KeyCode::Left);
        select_lane(&mut app, "bb792d9c");
        let before = (
            app.selection,
            app.timeline.clone(),
            app.explicit_range,
            app.unfold,
            app.imports,
        );
        let screen = draw(&app, 120, 40);
        key(&mut app, KeyCode::Char('s'));
        app.update(Event::Refresh(Ok(with_graph(
            tagged(),
            graph("1.3", REPO, "1.3"),
            &[],
        ))));
        for code in [
            KeyCode::Up,
            KeyCode::Down,
            KeyCode::Char('+'),
            KeyCode::Char('-'),
            KeyCode::Char('0'),
            KeyCode::Char('z'),
            KeyCode::Char('i'),
            KeyCode::Enter,
            KeyCode::Char('t'),
            KeyCode::Char('s'),
            KeyCode::Left,
            KeyCode::Right,
        ] {
            assert!(key(&mut app, code).is_empty(), "{code:?}");
        }
        assert_eq!(key(&mut app, KeyCode::Esc), [Effect::Graph(None)]);
        assert!(app.graph.is_none());
        assert_eq!(
            (
                app.selection,
                app.timeline.clone(),
                app.explicit_range,
                app.unfold,
                app.imports,
            ),
            before
        );
        assert_eq!(draw(&app, 120, 40), screen);
        assert_eq!(
            key(&mut graph_app("1.3", &[]), KeyCode::Char('q')),
            [Effect::Quit]
        );
    }

    #[test]
    fn open_and_fork_on_done_running_unlinked_ambiguous_parts() {
        // Done Codex part, then its Claude parent part (the panel draws o/f for both).
        let mut app = graph_app("1.1", &[]);
        key(&mut app, KeyCode::Right);
        let id = session_of(&app);
        assert!(id.starts_with("82010000"));
        assert_eq!(
            key(&mut app, KeyCode::Char('o')),
            [Effect::FreshOpen {
                id: id.clone(),
                fork: false
            }]
        );
        assert_eq!(app.footer_text(), "checking current session…");
        assert!(key(&mut app, KeyCode::Char('f')).is_empty());
        app.update(Event::Opened(String::new()));
        assert_eq!(
            key(&mut app, KeyCode::Char('f')),
            [Effect::FreshOpen { id, fork: true }]
        );
        app.update(Event::Opened(String::new()));
        key(&mut app, KeyCode::Left);
        let claude = session_of(&app);
        assert_eq!(
            key(&mut app, KeyCode::Char('o')),
            [Effect::FreshOpen {
                id: claude,
                fork: false
            }]
        );
        // Running part.
        let mut app = graph_app("1.5", &[]);
        key(&mut app, KeyCode::Right);
        key(&mut app, KeyCode::Right);
        let id = session_of(&app);
        assert!(id.starts_with("82520000"));
        assert_eq!(
            key(&mut app, KeyCode::Char('o')),
            [Effect::FreshOpen { id, fork: false }]
        );
        // Unlinked and ambiguous parts: the panel's words, no effect.
        let mut app = graph_app("1.2", &[]);
        for (index, words) in [(1, "no codex run found"), (3, "2 possible runs")] {
            while selected(&app).0 < index {
                key(&mut app, KeyCode::Right);
            }
            for k in ['o', 'f'] {
                assert!(key(&mut app, KeyCode::Char(k)).is_empty());
                assert_eq!(app.footer_text(), words);
                assert!(!app.opening);
            }
        }
        // Unavailable part.
        let mut app = graph_app("1.11", &[]);
        key(&mut app, KeyCode::Right);
        assert!(key(&mut app, KeyCode::Char('o')).is_empty());
        assert_eq!(app.footer_text(), "run unavailable");
    }

    #[test]
    fn open_blocked_after_failed_refresh() {
        let mut app = graph_app("1.1", &[]);
        key(&mut app, KeyCode::Right);
        app.update(Event::Refresh(Err("denied".into())));
        for k in ['o', 'f'] {
            assert!(key(&mut app, KeyCode::Char(k)).is_empty());
            assert_eq!(
                app.footer_text(),
                "refresh failed: denied; keeping last report"
            );
        }
        app.update(Event::Refresh(Ok(tagged())));
        assert!(matches!(
            key(&mut app, KeyCode::Char('o')).as_slice(),
            [Effect::FreshOpen { fork: false, .. }]
        ));
    }

    #[test]
    fn refresh_adding_a_part_keeps_selected_part() {
        let mut app = graph_app("1.3", &[]);
        for _ in 0..3 {
            key(&mut app, KeyCode::Right);
        }
        let old = app.graph.as_ref().unwrap().report.clone().unwrap();
        // Index 3 is the second part of Codex session 82030000.
        assert_eq!(old.parts[3].session, old.parts[1].session);
        let mut more = old.clone();
        let mut extra = more.parts[0].clone();
        extra.session = Some("new-session".into());
        more.parts.insert(0, extra);
        let old_first = selected(&app).1;
        app.update(Event::Refresh(Ok(with_graph(tagged(), more.clone(), &[]))));
        assert_eq!(
            selected(&app),
            (4, crate::graph_first_slot(&more, 4, old_first))
        );
        // The part is gone: clamp the old index.
        let mut fewer = old.clone();
        fewer.parts.truncate(2);
        app.update(Event::Refresh(Ok(with_graph(tagged(), fewer, &[]))));
        assert_eq!(selected(&app), (1, 0));
    }

    #[test]
    fn refresh_error_keeps_graph_screen() {
        let mut app = graph_app("1.3", &[]);
        key(&mut app, KeyCode::Right);
        let before = draw(&app, 120, 40);
        app.update(Event::Refresh(Err("denied".into())));
        let after = draw(&app, 120, 40);
        for y in 0..39 {
            assert_eq!(row(&after, y), row(&before, y), "row {y}");
        }
        assert_eq!(
            row(&after, 39),
            "refresh failed: denied; keeping last report"
        );
        assert!(app.graph.as_ref().unwrap().report.is_some());
    }

    #[test]
    fn keys_on_empty_graph_do_nothing() {
        let mut app = graph_app("1.13", &[]);
        assert!(
            app.graph
                .as_ref()
                .unwrap()
                .report
                .as_ref()
                .unwrap()
                .parts
                .is_empty()
        );
        for width in [120, 80] {
            draw(&app, width, 40);
            for code in [
                KeyCode::Left,
                KeyCode::Right,
                KeyCode::Up,
                KeyCode::Down,
                KeyCode::Char('o'),
                KeyCode::Char('f'),
            ] {
                assert!(key(&mut app, code).is_empty());
                assert_eq!(selected(&app), (0, 0));
            }
            draw(&app, width, 40);
        }
        app.update(Event::Refresh(Ok(with_graph(
            tagged(),
            graph("1.13", REPO, "1.3"),
            &[],
        ))));
        assert_eq!(selected(&app), (0, 0));
    }

    #[test]
    fn graph_render_has_no_sentinels() {
        for n in 1..=13 {
            let mut app = graph_app(&format!("1.{n}"), &[]);
            for width in [120, 80] {
                let mut moves = 0;
                loop {
                    let b = draw(&app, width, 40);
                    let text: String = b.content.iter().map(|c| c.symbol()).collect();
                    assert!(!text.contains("sentinel-"), "1.{n} at {width}");
                    let next = if width >= 120 {
                        KeyCode::Right
                    } else {
                        KeyCode::Down
                    };
                    let before = selected(&app).0;
                    key(&mut app, next);
                    moves += 1;
                    if selected(&app).0 == before || moves > 20 {
                        break;
                    }
                }
            }
        }
    }

    impl Refresh {
        fn clone_box(&self) -> Box<Refresh> {
            Box::new(Refresh {
                overview: self.overview.clone(),
                links: self.links.clone(),
                diagnostics: self.diagnostics.clone(),
                graph: self.graph.clone(),
                graph_live: self.graph_live.clone(),
                graph_steps: self.graph_steps.clone(),
                codex_steps: self.codex_steps.clone(),
                started_steps: self.started_steps.clone(),
            })
        }
    }
}
