use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use axum::{Json, Router};
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{post};
use nanoid::nanoid;
use tokio::sync::Mutex;
use tonic::Request;
use tonic::transport::Channel;
use chatq_types::chatq::message_handler_client::MessageHandlerClient;
use chatq_types::chatq::MessageQueryRequest;
use chatq_types::data::Message;
use chatq_types::data::query::MessageQueryPattern;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Snapshot {
    id: String,
    query: MessageQueryPattern,
    messages: Vec<Message>,
}

async fn fetch_snapshot(State(state): State<AppState>, Path(id): Path<String>) -> Result<Json<Snapshot>, (StatusCode, String)> {
    let snapshot_read = state.snapshots.read()
        .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;

    let read = snapshot_read.get(&id)
        .ok_or((StatusCode::BAD_REQUEST, "Snapshot not found.".to_string()))?;

    Ok(Json(read.clone()))
}

async fn generate_snapshot(State(state): State<AppState>, Json(query): Json<MessageQueryPattern>) -> Result<impl IntoResponse, (StatusCode, String)> {
    let mut client = state.client.lock().await;

    let pattern: chatq_types::chatq::MessageQueryPattern = query.clone().into();

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

    let id = nanoid!();

    let snapshot = Snapshot {
        id: id.clone(),
        query,
        messages: result,
    };

    let mut write = state.snapshots.write().unwrap();
    let id = write.insert(id, snapshot)
        .ok_or((StatusCode::INTERNAL_SERVER_ERROR, "Insert error".to_string()))?.id;

    Ok(id)
}

#[derive(Clone)]
struct AppState {
    snapshots: Arc<RwLock<HashMap<String, Snapshot>>>,
    client: Arc<Mutex<MessageHandlerClient<Channel>>>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {

    let server_addr = dotenvy::var("CHATQ_SERVER_ADDRESS")?;
    let serve_addr = dotenvy::var("CHATQ_GATEWAY_ADDRESS")?;

    let client = MessageHandlerClient::connect(server_addr).await?;

    let state = AppState {
        snapshots: Arc::new(Default::default()),
        client: Arc::new(Mutex::new(client)),
    };

    let app = Router::new()
        .route("/generate-snapshot", post(generate_snapshot))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(serve_addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
