mod toy;
mod watch;

fn main() {
    let path = std::env::args()
        .nth(1)
        .expect("Argument 1 needs to be a path");

    toy::create_toy(&path);

    futures::executor::block_on(async {
        if let Err(e) = watch::async_watch(path).await {
            println!("error: {:?}", e)
        }
    });
}
