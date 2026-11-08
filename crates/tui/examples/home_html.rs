//! Renders the home screen, one repo view, or a step graph with its colours to a standalone HTML
//! page, for screenshots.
//!
//! The report comes from the CLI's JSON, so the same example covers a fixture and real `~`:
//!
//! ```text
//! multi-link overview --json [--root R] --now T --since 2h > overview.json
//! multi-link links --json [--root R] --now T --since 2h > links.json
//! cargo run -p multi-link-tui --example home_html -- overview.json links.json out.html \
//!     [--size 120x40] [--offset -7 | --offset 5:30 | --offset local] [--repo NAME]
//!     [--since 2h]
//! multi-link graph --repo R --step ID --json > graph.json
//! cargo run -p multi-link-tui --example home_html -- --graph graph.json out.html \
//!     [--size 120x40] [--offset -8 | --offset 5:30 | --offset local] [--selected N]
//! ```
//!
//! The theme is the truecolor palette (what `COLORTERM=truecolor` gives). `--offset` defaults to
//! `local`, the offset at the report's `now`, as the TUI uses. Home is drawn with nothing selected.
//! `--repo` draws that repo's view instead (a full repo path, or a name that ends one): fitted range,
//! folds on, imports hidden and the first lane selected, as the TUI opens it from the repo's title
//! line. Pass the report's
//! `--since` (`Ns`, `Nm`, `Nh` or `Nd`; default `2h`) so either view gets the window the TUI would
//! give it.
//! `--graph` draws a GraphReport instead; `--selected` is its zero-based part index (default 0).
use std::{env, fmt::Write, fs, time::Duration};

use multi_link_model::{GraphReport, LinksReport, OverviewReport};
use multi_link_tui::{
    DEFAULT_WINDOW, GraphOptions, HomeOptions, RepoOptions, TimeWindow, render_graph,
    render_home_with_options, render_repo, view,
};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier},
    text::Span,
};
use time::UtcOffset;

const SCREEN: (u8, u8, u8) = (0x0E, 0x10, 0x14);
const TEXT: (u8, u8, u8) = (0xD7, 0xDA, 0xE0);
/// One terminal row in pixels (13 px Menlo at line height ~1.23, a typical terminal setting).
const ROW: u32 = 16;

type Rgb = (u8, u8, u8);

/// The palette is truecolor only; any other colour would be drawn wrongly, so it is an error.
fn rgb(value: Color, reset: Rgb) -> Result<Rgb, String> {
    match value {
        Color::Rgb(r, g, b) => Ok((r, g, b)),
        Color::Reset => Ok(reset),
        other => Err(format!("non-truecolor colour {other:?} in the buffer")),
    }
}

fn hex((r, g, b): Rgb) -> String {
    format!("#{r:02X}{g:02X}{b:02X}")
}

/// A terminal draws DIM text closer to its background; the background itself stays.
fn dim(fg: Rgb, bg: Rgb) -> Rgb {
    let mix = |f: u8, b: u8| ((u16::from(f) * 6 + u16::from(b) * 4) / 10) as u8;
    (mix(fg.0, bg.0), mix(fg.1, bg.1), mix(fg.2, bg.2))
}

fn escape(symbol: &str) -> String {
    symbol
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

/// Each row is a block of full-height cells, so backgrounds and `█` touch the rows above and
/// below as they do in a terminal (an inline `<pre>` would leave page-colour strips between rows).
fn html(buf: &Buffer, title: &str) -> Result<String, String> {
    let mut body = String::new();
    for y in 0..buf.area.height {
        body.push_str("<div class=\"r\">");
        let mut x = 0;
        while x < buf.area.width {
            let cell = &buf[(x, y)];
            let bg = rgb(cell.bg, SCREEN)?;
            let mut fg = rgb(cell.fg, TEXT)?;
            if cell.modifier.contains(Modifier::DIM) {
                fg = dim(fg, bg);
            }
            // A wide symbol covers the next cell too; ratatui leaves a blank there.
            let width = Span::raw(cell.symbol()).width().max(1) as u16;
            let mut style = format!("color:{};background:{};width:{}ch", hex(fg), hex(bg), width);
            if cell.modifier.contains(Modifier::BOLD) {
                style.push_str(";font-weight:700");
            }
            if cell.modifier.contains(Modifier::ITALIC) {
                style.push_str(";font-style:italic");
            }
            if cell.modifier.contains(Modifier::UNDERLINED) {
                style.push_str(";text-decoration:underline");
            }
            write!(
                body,
                "<span style=\"{style}\">{}</span>",
                escape(cell.symbol())
            )
            .unwrap();
            x += width;
        }
        body.push_str("</div>\n");
    }
    Ok(format!(
        "<!doctype html><meta charset=\"utf-8\"><title>{title}</title>\
         <style>body{{margin:0;background:{screen}}}\
         .s{{margin:16px;font:13px Menlo,monospace}}\
         .r{{height:{ROW}px}}\
         .r span{{display:inline-block;height:{ROW}px;line-height:{ROW}px;\
         vertical-align:top;overflow:hidden;white-space:pre}}</style>\
         <div class=\"s\">{body}</div>",
        screen = hex(SCREEN)
    ))
}

/// `-7`, `5:30` or `-3:30` hours from UTC.
fn parse_offset(value: &str) -> Result<UtcOffset, Box<dyn std::error::Error>> {
    let (hours, minutes) = value.split_once(':').unwrap_or((value, "0"));
    let hours: i8 = hours.parse()?;
    let minutes: i8 = minutes.parse()?;
    let minutes = if value.starts_with('-') {
        -minutes
    } else {
        minutes
    };
    Ok(UtcOffset::from_hms(hours, minutes, 0)?)
}

/// `30m`, `2h` or `1d`, by the CLI's `--since` rule (`all` is not supported here).
fn parse_since(value: &str) -> Result<Duration, String> {
    let invalid = || format!("--since {value}: expected a duration such as 30m, 2h, or 1d");
    let multiplier = match value.as_bytes().last() {
        Some(b's') => 1,
        Some(b'm') => 60,
        Some(b'h') => 3600,
        Some(b'd') => 86_400,
        _ => return Err(invalid()),
    };
    let number = &value[..value.len() - 1];
    if number.is_empty() || !number.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(invalid());
    }
    let seconds = number
        .parse::<u64>()
        .ok()
        .and_then(|n| n.checked_mul(multiplier))
        .ok_or_else(invalid)?;
    Ok(Duration::from_secs(seconds))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().skip(1).collect();
    let (input, links, out) = match args.get(..3) {
        Some([mode, report, out]) if mode == "--graph" => (report, None, out),
        Some([a, b, c]) => (a, Some(b), c),
        _ => return Err(
            "usage: home_html OVERVIEW.json LINKS.json OUT.html [--size WxH] [--offset H[:MM]|local] \
             [--repo NAME] [--since 30m|2h|1d]\n\
             or: home_html --graph REPORT.json OUT.html [--size WxH] \
             [--offset H[:MM]|local] [--selected N]"
                .into(),
        ),
    };
    let (mut width, mut height, mut offset_arg) = (120, 40, "local".to_string());
    let mut repo_arg: Option<String> = None;
    let mut since = DEFAULT_WINDOW;
    let mut selected = 0;
    let mut rest = args[3..].iter();
    while let Some(flag) = rest.next() {
        let value = rest.next().ok_or_else(|| format!("{flag} needs a value"))?;
        match flag.as_str() {
            "--size" => {
                let (w, h) = value.split_once('x').ok_or("--size is WxH")?;
                (width, height) = (w.parse()?, h.parse()?);
            }
            "--offset" => offset_arg = value.clone(),
            "--repo" if links.is_some() => repo_arg = Some(value.clone()),
            "--since" if links.is_some() => since = parse_since(value)?,
            "--selected" if links.is_none() => {
                selected = value
                    .parse::<usize>()
                    .map_err(|error| format!("--selected: {error}"))?;
            }
            _ => return Err(format!("unknown flag {flag}").into()),
        }
    }
    let area = Rect::new(0, 0, width, height);
    let mut buf = Buffer::empty(area);
    let Some(links) = links else {
        let report: GraphReport = serde_json::from_str(&fs::read_to_string(input)?)?;
        if !report.parts.is_empty() && selected >= report.parts.len() {
            return Err(format!(
                "--selected: index {selected} is out of range for {} parts",
                report.parts.len()
            )
            .into());
        }
        let offset = if offset_arg == "local" {
            view::local_offset(report.now)
        } else {
            parse_offset(&offset_arg)?
        };
        let options = GraphOptions {
            selected,
            offset,
            ..GraphOptions::default()
        };
        render_graph(&report, &options, area, &mut buf);
        fs::write(out, html(&buf, "multi-link step graph")?)?;
        return Ok(());
    };
    let report: OverviewReport = serde_json::from_str(&fs::read_to_string(input)?)?;
    let links: LinksReport = serde_json::from_str(&fs::read_to_string(links)?)?;
    let offset = if offset_arg == "local" {
        view::local_offset(report.now)
    } else {
        parse_offset(&offset_arg)?
    };
    let window = TimeWindow::ending_at(report.now, since);
    let title = if let Some(name) = repo_arg {
        // An exact path wins; a bare name must end exactly one repo path.
        let repos: Vec<&str> = report.repos.iter().map(|g| g.repo.as_str()).collect();
        let suffix = format!("/{name}");
        let repo = match repos.iter().find(|r| **r == name) {
            Some(r) => *r,
            None => match repos
                .iter()
                .filter(|r| r.ends_with(&suffix))
                .collect::<Vec<_>>()[..]
            {
                [r] => *r,
                [] => return Err(format!("no repo {name} in the overview").into()),
                ref many => {
                    let list: Vec<&str> = many.iter().map(|r| **r).collect();
                    return Err(format!("{name} names several repos: {}", list.join(", ")).into());
                }
            },
        };
        let options = RepoOptions {
            offset,
            ..RepoOptions::default()
        };
        render_repo(
            &report,
            &links,
            repo,
            window,
            Some(0),
            &options,
            area,
            &mut buf,
        );
        "multi-link repo view"
    } else {
        let options = HomeOptions {
            links: Some(&links),
            window: Some(window),
            offset,
            ..HomeOptions::default()
        };
        render_home_with_options(&report, None, &options, area, &mut buf);
        "multi-link home"
    };
    fs::write(out, html(&buf, title)?)?;
    Ok(())
}
