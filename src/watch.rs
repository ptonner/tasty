use futures::{
    channel::mpsc::{channel, Receiver, Sender},
    executor::ThreadPool,
    SinkExt, StreamExt,
};
use log;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

use notify::{
    event::{DataChange, ModifyKind},
    INotifyWatcher,
};
use notify::{Config, Error, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};

use crate::runtime::{IRuntime, Runtime};
use crate::toy::Toy;

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

async fn run_watch(mut file_event_chan: Receiver<Result<Event, Error>>, mut toy_chan: Sender<Toy>) {
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
                        "image.glsl" => match fs::read_to_string(p) {
                            Ok(toy) => toy_chan
                                .send(Toy {
                                    main_image: toy,
                                    ..Default::default()
                                })
                                .await
                                .unwrap(),
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

fn start_async_watch<P: AsRef<Path>>(path: P) -> (INotifyWatcher, Receiver<Toy>) {
    let (_watcher, rx) = async_watcher(path).expect("Can watch");
    let (tx, toy_chan) = channel(1);
    let pool = ThreadPool::new().unwrap();
    let _ = pool.spawn_ok(async { run_watch(rx, tx).await });
    return (_watcher, toy_chan);
}

pub fn run(path: PathBuf) {
    // Create initial files
    let toy = Toy::from_path(&path);
    if let Err(e) = toy.write(&path, false) {
        log::debug!("Error writing toy to path {:?}: {}", path, e);
    };

    // Start watch
    let (_w, toy_chan) = start_async_watch(&path);

    // Start graphics
    Runtime::start(toy, Some(toy_chan));
}

#[cfg(test)]
mod test {
    use super::*;
    use futures::executor;
    use tempdir::TempDir;

    #[test]
    fn watch_sends_updates() {
        let mut toy = Toy::default();
        let tmp_dir = TempDir::new("example").unwrap().into_path();
        let _ = toy.write(&tmp_dir, false);

        let (_w, mut toy_chan) = start_async_watch(&tmp_dir);

        // no messages initially
        let msg = toy_chan.try_next();
        assert!(msg.is_err()); // error means no message but still running

        // write toy and get new config
        toy.main_image = "test".into();
        toy.write(&tmp_dir.clone(), true).unwrap();
        let msg = executor::block_on(async { toy_chan.next().await });

        match msg {
            Some(cfg) => assert_eq!(cfg.main_image, toy.main_image),
            None => assert!(false, "Channel should not be closed"),
        }
    }
}
