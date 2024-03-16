use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::post;
use axum::{Json, Router};
use chatq_types::chatq::message_handler_client::MessageHandlerClient;
use chatq_types::chatq::{SnapshotFetchRequest, SnapshotGenerateRequest};
use chatq_types::data::query::MessageQueryPattern;
use chatq_types::data::Snapshot;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use tonic::transport::Channel;
use tonic::Request;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct SnapshotRecipe {
    target: Uuid,
    query: MessageQueryPattern,
}

async fn generate_snapshot(
    State(state): State<AppState>,
    Json(recipe): Json<SnapshotRecipe>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let mut client = state.client.lock().await;

    let request = Request::new(SnapshotGenerateRequest {
        target: Some(recipe.target.into()),
        query: Some(recipe.query.into()),
    });

    let result = client.generate_snapshot(request).await.map_err(|err| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("server error {}", err),
        )
    })?;

    let snapshot: Snapshot = result
        .into_inner()
        .snapshot
        .ok_or((StatusCode::NOT_FOUND, "not found".to_string()))?
        .try_into()
        .map_err(|err| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "snapshot invalid".to_string(),
            )
        })?;
    Ok(Json(snapshot))
}

async fn fetch_snapshot(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let mut client = state.client.lock().await;

    let request = Request::new(SnapshotFetchRequest { id });

    let result = client.fetch_snapshot(request).await.map_err(|err| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("snapshot fetch {}", err),
        )
    })?;

    let result = result
        .into_inner()
        .result
        .ok_or((StatusCode::NOT_FOUND, "not found".to_string()))?;

    return match result {
        chatq_types::chatq::fetch_snapshot_response::Result::Some(snapshot) => {
            let snapshot: Snapshot = snapshot.try_into().map_err(|err| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("snapshot invalid {}", err),
                )
            })?;
            Ok(Json(snapshot))
        }
        chatq_types::chatq::fetch_snapshot_response::Result::None(_) => {
            Err((StatusCode::NOT_FOUND, "not found".to_string()))
        }
    };
}

#[derive(Clone)]
struct AppState {
    client: Arc<Mutex<MessageHandlerClient<Channel>>>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let server_addr = dotenvy::var("CHATQ_SERVER_ADDRESS")?;
    let serve_addr = dotenvy::var("CHATQ_GATEWAY_ADDRESS")?;

    let client = MessageHandlerClient::connect(server_addr).await?;

    let state = AppState {
        client: Arc::new(Mutex::new(client)),
    };

    let app = Router::new()
        .route("/generate-snapshot", post(generate_snapshot))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(serve_addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
