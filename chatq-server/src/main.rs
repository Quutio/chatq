use chrono::Utc;
use dotenv::dotenv;
use lib::data::models::{MessageAudience, MessageSource, MessageStub, Server};
use lib::ChatQDao;
use sqlx::PgPool;

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
    let pool = PgPool::connect(db_url)
        .await
        .expect("database connect failure");
    let db = ChatQDao::with_pool(pool);

    let stub = MessageStub {
        timestamp: Utc::now().naive_utc(),
        source: MessageSource::Players(vec![uuid::Uuid::new_v4()]),
        audience: MessageAudience::Servers(vec![
            Server {
                value: "Helloz".to_owned(),
            },
            Server {
                value: "Yez".to_owned(),
            },
        ]),
        content: "Blah Blah Ba Ba Lah".to_owned(),
    };

    let yez = db.insert_message(stub).await.unwrap();

    println!("{:#?}", yez);

    println!("Hello, world!");
}
