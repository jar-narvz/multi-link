//! Renders the home screen with its colours to a standalone HTML page, for screenshots.
//!
//! The report comes from the CLI's JSON, so the same example covers a fixture and real `~`:
//!
//! ```text
//! multi-link overview --json [--root R] --now T --since 2h > overview.json
//! multi-link links --json [--root R] --now T --since 2h > links.json
//! cargo run -p multi-link-tui --example home_html -- overview.json links.json out.html \
//!     [--size 120x40] [--offset -7 | --offset 5:30 | --offset local]
//! ```
//!
//! The theme is the truecolor palette (what `COLORTERM=truecolor` gives). `--offset` defaults to
//! `local`, the offset at the report's `now`, as the TUI uses.
use std::{env, fmt::Write, fs};

use multi_link_model::{LinksReport, OverviewReport};
use multi_link_tui::{HomeOptions, render_home_with_options, view};
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().skip(1).collect();
    let [overview, links, out] = match args.get(..3) {
        Some([a, b, c]) => [a, b, c],
        _ => return Err(
            "usage: home_html OVERVIEW.json LINKS.json OUT.html [--size WxH] [--offset H[:MM]|local]"
                .into(),
        ),
    };
    let (mut width, mut height, mut offset_arg) = (120, 40, "local".to_string());
    let mut rest = args[3..].iter();
    while let Some(flag) = rest.next() {
        let value = rest.next().ok_or_else(|| format!("{flag} needs a value"))?;
        match flag.as_str() {
            "--size" => {
                let (w, h) = value.split_once('x').ok_or("--size is WxH")?;
                (width, height) = (w.parse()?, h.parse()?);
            }
            "--offset" => offset_arg = value.clone(),
            _ => return Err(format!("unknown flag {flag}").into()),
        }
    }
    let report: OverviewReport = serde_json::from_str(&fs::read_to_string(overview)?)?;
    let links: LinksReport = serde_json::from_str(&fs::read_to_string(links)?)?;
    let offset = if offset_arg == "local" {
        view::local_offset(report.now)
    } else {
        parse_offset(&offset_arg)?
    };
    let options = HomeOptions {
        links: Some(&links),
        offset,
        ..HomeOptions::default()
    };
    let area = Rect::new(0, 0, width, height);
    let mut buf = Buffer::empty(area);
    render_home_with_options(&report, None, &options, area, &mut buf);
    fs::write(out, html(&buf, "multi-link home")?)?;
    Ok(())
}
