use std::path::PathBuf;

use clap::{Parser, Subcommand};

use miniquad::*;

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

fn do_watch(path: PathBuf) {
    // Create initial files
    toy::create_toy(
        &path
            .clone()
            .into_os_string()
            .into_string()
            .expect("Path is valid"),
    );

    // Start watch
    let (_watcher, rx) = watch::async_watcher(path).expect("Can watch");

    // Start graphics
    let mut conf = conf::Conf::default();
    conf.platform.apple_gfx_api = conf::AppleGfxApi::OpenGl;

    miniquad::start(conf, move || Box::new(toy::Stage::new(rx)));
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Watch { location }) => do_watch(location),
        None => (),
    }
}
