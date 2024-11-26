use std::path::PathBuf;

use clap::{Parser, Subcommand};

use futures::executor::ThreadPool;
use miniquad::*;

// tmp
use notify::{
    event::{DataChange, ModifyKind},
    Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher,
};

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
    let (mut watcher, mut rx) = watch::async_watcher(path).expect("Can watch");
    // match rx.try_next() {
    //     Ok(Some(t)) => println!("message: {:?}", t),
    //     Ok(None) => println!("closed"),
    //     Err(e) => println!("no messages yet"),
    // }
    // watcher
    //     .watch(path.as_ref(), RecursiveMode::Recursive)
    //     .expect("can always watch");

    // let pool = ThreadPool::new().unwrap();
    // pool.spawn_ok(async {
    //     if let Err(e) = watch::async_watch(path, watcher, rx).await {
    //         println!("error: {:?}", e)
    //     }
    // });

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
