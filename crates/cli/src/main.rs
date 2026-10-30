use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(name = "multi-link", about = "Inspect local Claude and Codex sessions")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    Sessions(CommonArgs),
    Links(CommonArgs),
    Steps {
        #[command(flatten)]
        common: CommonArgs,
        /// Repositories to read (repeatable; relative to --root).
        #[arg(long)]
        repo: Vec<PathBuf>,
    },
}

#[derive(Args)]
struct CommonArgs {
    /// Home-shaped root (default: HOME); enables fixture mode.
    #[arg(long)]
    root: Option<PathBuf>,
    /// Override the current time with an RFC 3339 timestamp.
    #[arg(long)]
    now: Option<String>,
    /// History duration, or 'all'.
    #[arg(long, default_value = "2h")]
    since: String,
    /// Print JSON.
    #[arg(long)]
    json: bool,
}

fn main() {
    let _ = Cli::parse();
    eprintln!("not implemented");
    std::process::exit(2);
}
