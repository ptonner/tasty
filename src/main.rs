use std::path::PathBuf;

use clap::{Parser, Subcommand};

mod runtime;
mod toy;
mod watch;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Watch {
        /// lists test values
        location: PathBuf,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Watch { location }) => watch::run(location),
        None => (),
    }
}
