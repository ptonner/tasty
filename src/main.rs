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
    Debug,
    Watch {
        /// lists test values
        location: PathBuf,
    },
}

fn debug() {
    let mut toy = toy::Toy::default();
    let chan = toy::Channel::Builtin {
        name: toy::BuiltinName::RgbaNoiseSmall,
        config: toy::ChannelConfig::Texture { vflip: true },
    };
    toy.config.channels = vec![chan.clone(), chan];
    let _ = toy.write("dbg", true);
}

fn main() {
    env_logger::init();
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Debug) => debug(),
        Some(Commands::Watch { location }) => watch::run(location),
        None => (),
    }
}
