use chrono::Utc;
use dotenv::dotenv;
use lib::data::models::{
    AudienceFilter, CompositeFilter, FilterItem, MessageAudience, MessageFilter,
    MessageFilterPattern, MessageSource, MessageStub, Plugin, Server, SourceFilter,
};
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

    let uuid1 = uuid::Uuid::from_u128(1);
    let uuid2 = uuid::Uuid::from_u128(2);

    let stub = MessageStub {
        timestamp: Utc::now().naive_utc(),
        source: MessageSource::Players(vec![uuid1.clone()]),
        audience: MessageAudience::Servers(vec![Server::new("Helloz"), Server::new("Yez")]),
        content: "Blah Blah Ba Ba Lah".to_owned(),
    };

    let stub2 = MessageStub {
        timestamp: Utc::now().naive_utc(),
        source: MessageSource::Players(vec![uuid2.clone()]),
        audience: MessageAudience::Players(vec![uuid1, uuid2]),
        content: "Blah Blah Bssa Ba Lah".to_owned(),
    };

    let stub3 = MessageStub {
        timestamp: Utc::now().naive_utc(),
        source: MessageSource::Plugins(vec![Plugin::new("qkernel"), Plugin::new("reportas")]),
        audience: MessageAudience::Servers(vec![Server::new("myctophids")]),
        content: "aasddsd".to_owned(),
    };

    let yez1 = db.insert_message(stub).await.unwrap();
    let yez2 = db.insert_message(stub2).await.unwrap();
    let yez3 = db.insert_message(stub3).await.unwrap();

    println!("{:#?}", yez1);
    println!("{:#?}", yez2);
    println!("{:#?}", yez3);

    let filter = MessageFilterPattern::Composite(CompositeFilter::And(vec![
        FilterItem::Composite(CompositeFilter::Or(vec![
            FilterItem::Single(MessageFilter::Audience(AudienceFilter::Server(Server {
                value: "Hellozaaa".to_string(),
            }))),
            FilterItem::Single(MessageFilter::Audience(AudienceFilter::Server(Server {
                value: "myctophids".to_string(),
            }))),
        ])),
        FilterItem::Single(MessageFilter::Source(SourceFilter::Plugin(Plugin {
            value: "qkernel".to_string(),
        }))),
    ]));

    let res = db.query_messages(&filter).await.unwrap();

    println!("{:#?}", res);

    println!("Hello, world!");
}
