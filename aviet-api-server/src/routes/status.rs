use axum::{extract::State, Json};
use std::sync::Arc;

use crate::state::AppState;
use aviet_shared::models::ServerStatus;

pub async fn get_status(State(state): State<Arc<AppState>>) -> Json<ServerStatus> {
    Json(ServerStatus {
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime_seconds: state.uptime_seconds(),
        active_sessions: state.sessions.read().await.len(),
        database_connected: true,
    })
}
