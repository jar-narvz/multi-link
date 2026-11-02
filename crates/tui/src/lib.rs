//! Terminal front end. Render functions are pure: they draw a report into a ratatui `Buffer`
//! and never read the disk or the clock (design T3, T4). Home presentation derives windowed
//! lanes from core reports; the repo view orders recorded activity into swimlanes.

mod home;
mod timeline;
pub mod view;

pub use home::{
    HomeOptions, Target, render_home, render_home_with_options, selectable, selectable_home,
};
pub use timeline::{DEFAULT_WINDOW, RepoOptions, TimeWindow, render_repo, render_timeline};

mod app;
pub use app::{App, Effect, Event, Refresh};

/// Explicit Phase 5 palette and pure environment selection.
pub mod theme;
