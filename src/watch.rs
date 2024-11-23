use futures::{
    channel::mpsc::{channel, Receiver},
    SinkExt, StreamExt,
};
use notify::{
    event::{DataChange, ModifyKind},
    Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher,
};
use std::path::Path;

pub async fn async_watch<P: AsRef<Path>>(path: P) -> notify::Result<()> {
    let (mut watcher, mut rx) = async_watcher()?;

    watcher.watch(path.as_ref(), RecursiveMode::NonRecursive)?;

    while let Some(res) = rx.next().await {
        match res {
            Ok(event) => handle_event(event),
            Err(e) => println!("watch error: {:?}", e),
        }
    }

    Ok(())
}

fn async_watcher() -> notify::Result<(RecommendedWatcher, Receiver<notify::Result<Event>>)> {
    let (mut tx, rx) = channel(1);

    // Automatically select the best implementation for your platform.
    // You can also access each implementation directly e.g. INotifyWatcher.
    let watcher = RecommendedWatcher::new(
        move |res| {
            futures::executor::block_on(async {
                tx.send(res).await.unwrap();
            })
        },
        Config::default(),
    )?;

    Ok((watcher, rx))
}

fn handle_event(event: Event) {
    dbg!(&event);
    match event {
        Event {
            kind: EventKind::Modify(ModifyKind::Data(DataChange::Any)),
            paths,
            attrs: _,
        } => {
            let p = &paths[0];
            match p.file_name().unwrap().to_owned().to_str().unwrap() {
                "toy.glsl" => println!("frag"),
                _ => (),
            }
        }
        _ => (),
    }
}
