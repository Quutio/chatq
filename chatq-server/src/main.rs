use dotenv::dotenv;
use lib::chatq::message_handler_server::MessageHandlerServer;
use lib::grpc::message_handler::GrpcMessageHandler;

use tonic::transport::Server;

#[macro_use]
extern crate log;

#[tokio::main]
pub async fn main() {
    dotenv().ok();

    env_logger::builder()
        .init();

    let db_url = &dotenv::var("DATABASE_URL").unwrap();

    let addr = "[::1]:10000".parse().unwrap();

    let handler = GrpcMessageHandler::new(db_url).await.unwrap();
    let handler_svc = MessageHandlerServer::new(handler);

    info!("Starting server {}", addr);

    Server::builder()
        .add_service(handler_svc)
        .serve(addr)
        .await
        .unwrap();
}
