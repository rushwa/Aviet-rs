use axum::{
    extract::{State, Path, WebSocketUpgrade},
    response::Response,
};
use std::sync::Arc;
use uuid::Uuid;

use crate::state::AppState;

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(_state): State<Arc<AppState>>,
    Path(session_id): Path<Uuid>,
) -> Response {
    ws.on_upgrade(move |socket| handle_socket(socket, session_id))
}

async fn handle_socket(mut socket: axum::extract::ws::WebSocket, session_id: Uuid) {
    use axum::extract::ws::Message;
    use tokio::time::{sleep, Duration};

    let _ = socket.send(Message::Text(format!("Connected to session {}", session_id))).await;

    loop {
        sleep(Duration::from_secs(5)).await;

        let ping_msg: Message = Message::Ping(vec![]);
        if socket.send(ping_msg).await.is_err() {
            break;
        }
    }
}
