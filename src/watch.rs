use futures::{
    channel::mpsc::{channel, Receiver},
    SinkExt,
};
use std::fs;
use std::path::Path;
use std::path::PathBuf;

use miniquad::*;
use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};

use crate::toy::shader;
use crate::toy::Toy;

pub fn async_watcher<P: AsRef<Path>>(
    path: P,
) -> notify::Result<(RecommendedWatcher, Receiver<notify::Result<Event>>)> {
    let (mut tx, rx) = channel(1);

    let mut watcher = RecommendedWatcher::new(
        move |res| {
            futures::executor::block_on(async {
                tx.send(res).await.unwrap();
            })
        },
        Config::default(),
    )?;
    watcher
        .watch(path.as_ref(), RecursiveMode::Recursive)
        .expect("can always watch");

    Ok((watcher, rx))
}

pub fn create_toy(path: &String) {
    fs::create_dir_all(path).expect("directory accessible");
    let path = PathBuf::from(path);
    // TODO: don't overwrite existing data
    fs::write(path.join("toy.glsl"), shader::MAIN_IMAGE).expect("toy writeable");
}

pub fn run(path: PathBuf) {
    // Create initial files
    create_toy(
        &path
            .clone()
            .into_os_string()
            .into_string()
            .expect("Path is valid"),
    );

    // Start watch
    let (_watcher, rx) = async_watcher(path).expect("Can watch");

    // Start graphics
    let mut conf = conf::Conf::default();
    conf.platform.apple_gfx_api = conf::AppleGfxApi::OpenGl;

    miniquad::start(conf, move || Box::new(Toy::new(rx)));
}
