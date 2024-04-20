use chatq_types::chatq::message_handler_server::MessageHandlerServer;
use lib::grpc::message_handler::GrpcMessageHandler;

use tonic::transport::Server;
use tracing::info;

#[tokio::main]
pub async fn main() {
    tracing_subscriber::fmt::fmt()
        .pretty()
        .with_file(false)
        .init();

    let db_url = &dotenv::var("CHATQ_DATABASE_URL").unwrap();
    let addr = &dotenv::var("CHATQ_ADDRESS").unwrap_or("[::1]:10000".to_string());

    let addr = addr.parse().unwrap();

    let handler = GrpcMessageHandler::new(db_url).await.unwrap();
    let handler_svc = MessageHandlerServer::new(handler);

    info!("Starting server {}", addr);

    Server::builder()
        .add_service(handler_svc)
        .serve(addr)
        .await
        .unwrap();
}
