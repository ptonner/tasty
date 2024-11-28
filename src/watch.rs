use futures::{
    channel::mpsc::{channel, Receiver, Sender},
    executor::ThreadPool,
    SinkExt, StreamExt,
};
use std::fs;
use std::path::Path;
use std::path::PathBuf;

use notify::event::{DataChange, ModifyKind};
use notify::{Config, Error, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};

use crate::runtime::{IRuntime, Runtime};
use crate::toy::Toy;
use crate::toy::{self, shader};

fn async_watcher<P: AsRef<Path>>(
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

async fn run_watch(
    mut file_event_chan: Receiver<Result<Event, Error>>,
    mut toy_chan: Sender<toy::Toy>,
) {
    while let Some(res) = file_event_chan.next().await {
        // dbg!(&res);
        match res {
            Ok(event) => match event {
                Event {
                    kind: EventKind::Modify(ModifyKind::Data(DataChange::Any)),
                    ref paths,
                    attrs: _,
                } => {
                    let p = &paths[0];
                    match p.file_name().unwrap().to_owned().to_str().unwrap() {
                        "toy.glsl" => match fs::read_to_string(p) {
                            Ok(toy) => toy_chan.send(Toy { main_image: toy }).await.unwrap(),
                            Err(err) => println!("Error reading {:?}: {:}", p, err),
                        },
                        _ => (),
                    }
                }
                _ => (),
            },
            Err(e) => println!("watch error: {:?}", e),
        }
    }
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
    let (tx, toy_chan) = channel(1);
    let pool = ThreadPool::new().unwrap();
    let _ = pool.spawn_ok(async { run_watch(rx, tx).await });

    // Start graphics
    Runtime::start(Toy::default(), Some(toy_chan));
}
