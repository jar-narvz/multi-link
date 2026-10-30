//! Terminal front end. Render functions are pure: they draw a report into a ratatui `Buffer`
//! and never read the disk or the clock (design T3, T4). All grouping, ordering, nesting and
//! merging come from core's `OverviewReport` as given; this crate only lays it out.

mod home;
mod timeline;

pub use home::{Target, render_home, selectable};
pub use timeline::{DEFAULT_WINDOW, TimeWindow, render_timeline};
