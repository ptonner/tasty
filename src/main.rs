use futures::executor::ThreadPool;
use miniquad::*;

mod toy;
mod watch;

fn main() {
    let path = std::env::args()
        .nth(1)
        .expect("Argument 1 needs to be a path");

    toy::create_toy(&path);

    let pool = ThreadPool::new().unwrap();
    pool.spawn_ok(async {
        if let Err(e) = watch::async_watch(path).await {
            println!("error: {:?}", e)
        }
    });

    let mut conf = conf::Conf::default();
    conf.platform.apple_gfx_api = conf::AppleGfxApi::OpenGl;

    miniquad::start(conf, move || Box::new(toy::Stage::new()));
}
