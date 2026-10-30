use std::{fmt::Write, fs, path::Path};

use ratatui::{
    buffer::{Buffer, Cell},
    style::{Color, Modifier},
};

pub const HEADER: &str = "# row first-last fg bg modifiers · 0-based inclusive columns · blank cells: fg - and no BOLD/DIM/ITALIC\n";

fn color(value: Color) -> String {
    match value {
        Color::Reset => "-".into(),
        Color::Rgb(r, g, b) => format!("#{r:02X}{g:02X}{b:02X}"),
        other => format!("{other:?}"),
    }
}

fn visible_style(cell: &Cell) -> (Color, Color, Modifier) {
    let mut fg = cell.fg;
    let mut modifiers = cell.modifier;
    if cell.symbol() == " " {
        fg = Color::Reset;
        modifiers.remove(Modifier::BOLD | Modifier::DIM | Modifier::ITALIC);
    }
    (fg, cell.bg, modifiers)
}

pub fn dump_styles(buf: &Buffer) -> String {
    let mut out = HEADER.to_string();
    for row in 0..buf.area.height {
        let mut first = 0;
        while first < buf.area.width {
            let style = visible_style(&buf[(buf.area.x + first, buf.area.y + row)]);
            let mut end = first + 1;
            while end < buf.area.width
                && visible_style(&buf[(buf.area.x + end, buf.area.y + row)]) == style
            {
                end += 1;
            }
            let modifiers = if style.2.is_empty() {
                "-".into()
            } else {
                style
                    .2
                    .iter_names()
                    .map(|(name, _)| name)
                    .collect::<Vec<_>>()
                    .join("+")
            };
            writeln!(
                out,
                "{row} {first}-{} {} {} {modifiers}",
                end - 1,
                color(style.0),
                color(style.1)
            )
            .unwrap();
            first = end;
        }
    }
    out
}

pub fn check_styles(buf: &Buffer, path: &Path) {
    let rendered = dump_styles(buf);
    if std::env::var_os("UPDATE_STYLE_SNAPSHOTS").is_some_and(|value| value == "1") {
        fs::write(path, &rendered).unwrap();
    }
    assert_eq!(
        rendered,
        fs::read_to_string(path).unwrap(),
        "{}",
        path.display()
    );
}
