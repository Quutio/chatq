use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use axum::{Json, Router};
use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::get;
use tokio::sync::Mutex;
use tonic::Request;
use tonic::transport::Channel;
use chatq_types::chatq::message_handler_client::MessageHandlerClient;
use chatq_types::chatq::MessageQueryRequest;
use chatq_types::data::Message;
use chatq_types::data::query::MessageQueryPattern;

#[derive(Debug, Clone, PartialEq)]
pub struct Snapshot {
    id: String,
    query: MessageQueryPattern,
    messages: Vec<Message>,
}

async fn generate_snapshot(State(state): State<AppState>, Json(query): Json<MessageQueryPattern>) -> Result<String, (StatusCode, String)> {
    let mut client = state.client.lock().await;

    let pattern: chatq_types::chatq::MessageQueryPattern = query.into();

    let request = Request::new(MessageQueryRequest {
        pattern: Some(pattern),
    });

    let result = client.query_messages(request).await
        .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?
        .into_inner()
        .messages
        .into_iter()
        .map(|x| Message::try_from(x))
        .flatten()
        .collect::<Vec<_>>();

    Ok("".to_string())
}

struct AppState {
    snapshots: Arc<RwLock<HashMap<String, Snapshot>>>,
    client: Arc<Mutex<MessageHandlerClient<Channel>>>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let app = Router::new().route("/", get(|| async { "Hello, World!" }));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    axum::serve(listener, app).await?;
    Ok(())
}
