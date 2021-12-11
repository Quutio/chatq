#[macro_use]
extern crate log;

#[tokio::main]
pub async fn main() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .is_test(true)
        .init();

    info!("jea");

    println!("Hello, world!");
}
