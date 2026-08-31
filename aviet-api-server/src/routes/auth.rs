use axum::{
    extract::{State, Path},
    Json,
};
use std::sync::Arc;
use uuid::Uuid;
use tracing::info;

use crate::state::AppState;
use crate::browser::BrowserSession;
use aviet_shared::models::*;

pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(req): Json<LoginRequest>,
) -> Json<LoginResponse> {
    info!("Login attempt for phone: {} on site: {}", req.phone, req.site_name);

    let site = match get_site_profile(&req.site_name) {
        Some(s) => s,
        None => {
            return Json(LoginResponse {
                success: false,
                session_id: Uuid::nil(),
                message: format!("Unknown site: {}", req.site_name),
                balance: None,
            });
        }
    };

    let session_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();

    let (event_tx, mut event_rx) = tokio::sync::mpsc::unbounded_channel();

    let mut browser_session = match BrowserSession::new(
        session_id,
        user_id,
        &state.config.geckodriver_url,
        state.config.headless,
        site.clone(),
        event_tx,
    ).await {
        Ok(s) => s,
        Err(e) => {
            return Json(LoginResponse {
                success: false,
                session_id,
                message: format!("Browser error: {}", e),
                balance: None,
            });
        }
    };

    match browser_session.login(&req.phone, &req.password).await {
        Ok(msg) => {
            info!("Login successful for session {}", session_id);

            {
                let mut sessions = state.sessions.write().await;
                sessions.insert(session_id, browser_session);
            }

            let _state_clone = state.clone();
            tokio::spawn(async move {
                while let Some(event) = event_rx.recv().await {
                    let _ = event;
                }
            });

            Json(LoginResponse {
                success: true,
                session_id,
                message: msg,
                balance: Some("0.00".to_string()),
            })
        }
        Err(e) => {
            Json(LoginResponse {
                success: false,
                session_id,
                message: e,
                balance: None,
            })
        }
    }
}

pub async fn logout(
    State(state): State<Arc<AppState>>,
    Path(session_id): Path<Uuid>,
) -> Json<serde_json::Value> {
    let mut sessions = state.sessions.write().await;

    if let Some(mut session) = sessions.remove(&session_id) {
        let _ = session.quit().await;
        info!("Session {} logged out", session_id);
    }

    Json(serde_json::json!({"success": true, "message": "Logged out"}))
}

pub async fn get_session(
    State(state): State<Arc<AppState>>,
    Path(session_id): Path<Uuid>,
) -> Json<Option<SessionInfo>> {
    let sessions = state.sessions.read().await;

    let info = sessions.get(&session_id).map(|s| SessionInfo {
        session_id: s.session_id,
        user_id: s.user_id,
        is_active: true,
        site_url: s.site.url.clone(),
        created_at: chrono::Utc::now(),
    });

    Json(info)
}

fn get_site_profile(name: &str) -> Option<SiteProfile> {
    match name.to_lowercase().as_str() {
        "betika" => Some(SiteProfile {
            name: "Betika".to_string(),
            url: "https://www.betika.com/en-ke/".to_string(),
            login_url: "https://www.betika.com/en-ke/login".to_string(),
            aviator_url: "https://www.betika.com/en-ke/aviator".to_string(),
            phone_selector: "input[name='phone-number']".to_string(),
            password_selector: "input[type='password']".to_string(),
            login_button_selector: "button[type='submit']".to_string(),
            logged_in_indicator: Some("span.nav__item__label".to_string()),
        }),
        _ => None,
    }
}
