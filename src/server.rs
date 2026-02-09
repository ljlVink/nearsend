use crate::protocol::{ActionRequest, ActionResponse, TransferRequest, TransferResponse};
use anyhow::{Context, Result};
use axum::{
    extract::{Multipart, Path, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower::ServiceBuilder;
use tower_http::cors::CorsLayer;

/// Server state
#[derive(Clone)]
pub struct ServerState {
    pub transfers: Arc<RwLock<HashMap<String, TransferRequest>>>,
    pub on_transfer_request: Arc<
        tokio::sync::mpsc::UnboundedSender<(String, TransferRequest)>,
    >,
}

/// Create the HTTP server router
pub fn create_router(state: ServerState) -> Router {
    Router::new()
        .route("/api/info", get(get_info))
        .route("/api/transfer", post(handle_transfer))
        .route("/api/transfer/:transfer_id", post(handle_action))
        .route("/api/transfer/:transfer_id/file/:file_id", get(handle_file_download))
        .layer(
            ServiceBuilder::new()
                .layer(CorsLayer::permissive())
                .into_inner(),
        )
        .with_state(state)
}

/// Get server info endpoint
async fn get_info() -> Json<serde_json::Value> {
    Json(json!({
        "alias": "NearSend",
        "version": "1.0.0",
        "deviceModel": "Desktop",
        "deviceType": "desktop",
        "fingerprint": "near-send",
        "port": 53317,
        "protocol": "http",
        "download": true
    }))
}

/// Handle transfer request
async fn handle_transfer(
    State(state): State<ServerState>,
    Json(request): Json<TransferRequest>,
) -> Result<Json<TransferResponse>, StatusCode> {
    let transfer_id = uuid::Uuid::new_v4().to_string();
    
    // Store the transfer request
    state.transfers.write().await.insert(transfer_id.clone(), request.clone());

    // Notify the application about the transfer request
    if let Err(_) = state.on_transfer_request.send((transfer_id.clone(), request.clone())) {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    Ok(Json(TransferResponse {
        transfer_id,
        files: request.files,
        text: request.text,
    }))
}

/// Handle action (accept/reject)
async fn handle_action(
    State(state): State<ServerState>,
    Path(transfer_id): Path<String>,
    Json(request): Json<ActionRequest>,
) -> Result<Json<ActionResponse>, StatusCode> {
    let transfers = state.transfers.read().await;
    
    if !transfers.contains_key(&transfer_id) {
        return Err(StatusCode::NOT_FOUND);
    }

    Ok(Json(ActionResponse {
        status: "success".to_string(),
        message: Some(format!("Transfer {} {}", transfer_id, request.action)),
    }))
}

/// Handle file download
async fn handle_file_download(
    State(_state): State<ServerState>,
    Path((transfer_id, file_id)): Path<(String, String)>,
) -> Result<String, StatusCode> {
    // TODO: Implement file download
    Err(StatusCode::NOT_IMPLEMENTED)
}
