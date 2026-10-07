use sqlx::{ConnectOptions, PgPool};
use sqlx::postgres::PgConnectOptions;
use chatq_types::chatq::message_handler_server::MessageHandlerServer;
use lib::grpc::message_handler::GrpcMessageHandler;

use std::str::FromStr;
use std::sync::Arc;
use anyhow::Context;
use tonic::transport::Server;
use tracing::info;
use tracing::log::LevelFilter;
use lib::{logic, ChatQDao};
use lib::event_channel::BroadcastMessageEventChannel;


#[tokio::main]
pub async fn main() {
    tracing_subscriber::fmt::fmt()
        .pretty()
        .with_file(false)
        .init();

    let db_url = &dotenv::var("CHATQ_DATABASE_URL").unwrap();
    let addr = &dotenv::var("CHATQ_ADDRESS").unwrap_or("[::1]:10000".to_string());

    let addr = addr.parse().unwrap();

    let pool = PgPool::connect_with(
        PgConnectOptions::from_str(db_url).context("failed to parse pool address").unwrap().log_statements(LevelFilter::Info),
    ).await.context("failed to connect to postgresql database").unwrap();

    let db = ChatQDao::with_pool(pool);

    let handler = GrpcMessageHandler::new(logic::Ctx {
        repo: Arc::new(db),
        channel: Arc::new(BroadcastMessageEventChannel::new(128)),
    });
    let handler_svc = MessageHandlerServer::new(handler);

    info!("Starting server {}", addr);

    Server::builder()
        .add_service(handler_svc)
        .serve(addr)
        .await
        .unwrap();
}
