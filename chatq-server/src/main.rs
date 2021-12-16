use chrono::{NaiveDateTime, Utc};
use dotenv::dotenv;
use lib::{ChatQDb, data::models::{MessageStub, MessageSource, MessageAudience, Server}};
use lib::Db;

#[macro_use]
extern crate log;

#[tokio::main]
pub async fn main() {
    dotenv().ok();

    env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .is_test(true)
        .init();

    info!("jea");

    let db_url = &dotenv::var("DATABASE_URL").unwrap();

    let db = ChatQDb::new(db_url).await.unwrap();

    let stub = MessageStub {
        timestamp: Utc::now().naive_utc(),
        source: MessageSource::Players(vec![uuid::Uuid::new_v4()]),
        audience: MessageAudience::Servers(vec![Server { value: "Helloz".to_owned() }, Server { value: "Yez".to_owned() }]),
        content: "Blah Blah Ba Ba Lah".to_owned(),
    };

    db.insert_message(stub).await.unwrap();

    println!("Hello, world!");
}
