// use macroquad::prelude::*;

use futures::{
    channel::mpsc::{channel, Receiver},
    SinkExt, StreamExt,
};
use notify::{
    event::{DataChange, ModifyKind},
    Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher,
};
// use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    let path = std::env::args()
        .nth(1)
        .expect("Argument 1 needs to be a path");

    create_toy(&path);

    futures::executor::block_on(async {
        if let Err(e) = async_watch(path).await {
            println!("error: {:?}", e)
        }
    });
}

fn create_toy(path: &String) {
    fs::create_dir_all(path).expect("directory accessible");
    let path = PathBuf::from(path);
    fs::write(path.join("toy.glsl"), DEFAULT_TOY_SHADER).expect("toy writeable");
    fs::write(path.join("vertex.glsl"), DEFAULT_VERTEX_SHADER).expect("toy writeable");
    fs::write(path.join("fragment.glsl"), DEFAULT_FRAGMENT_SHADER).expect("toy writeable");
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

async fn async_watch<P: AsRef<Path>>(path: P) -> notify::Result<()> {
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

// #[macroquad::main("toasty")]
// async fn main() {
//     let fragment_shader = DEFAULT_FRAGMENT_SHADER.to_string();
//     let vertex_shader = DEFAULT_VERTEX_SHADER.to_string();

//     let pipeline_params = PipelineParams {
//         depth_write: true,
//         depth_test: Comparison::LessOrEqual,
//         ..Default::default()
//     };

//     let material = load_material(
//         ShaderSource::Glsl {
//             vertex: &vertex_shader,
//             fragment: &fragment_shader,
//         },
//         MaterialParams {
//             pipeline_params,
//             ..Default::default()
//         },
//     )
//     .unwrap();

//     loop {
//         clear_background(WHITE);

//         gl_use_material(&material);
//         draw_rectangle(-1.0, -1.0, 2.0, 2.0, BLACK);
//         gl_use_default_material();

//         next_frame().await
//     }
// }

const DEFAULT_TOY_SHADER: &str = "void mainImage(out vec4 fragColor, in vec2 fragCoord)
{    
}";

const DEFAULT_FRAGMENT_SHADER: &str = "#version 100
precision highp float;
 
uniform vec2 u_resolution;
uniform vec2 u_mouse;
uniform float u_time;
 
out vec4 outColor;
 
void main() {
  mainImage(outColor, gl_FragCoord.xy);
}
";

const DEFAULT_VERTEX_SHADER: &str = "#version 100
precision lowp float;

attribute vec3 position;
attribute vec2 texcoord;

varying vec2 uv;

void main() {
    gl_Position = vec4(position, 1);
    uv = texcoord;
}
";
