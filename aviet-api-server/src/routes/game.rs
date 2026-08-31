use axum::{
    extract::{State, Path},
    Json,
};
use std::sync::Arc;
use uuid::Uuid;
use tracing::info;

use crate::state::AppState;
use crate::game::GameEngine;
use crate::browser::StrategyData;
use aviet_shared::models::*;

pub async fn get_game_state(
    State(state): State<Arc<AppState>>,
    Path(session_id): Path<Uuid>,
) -> Json<GameStateResponse> {
    let sessions = state.sessions.read().await;

    let game_state = match sessions.get(&session_id) {
        Some(session) => GameState {
            phase: session.game_phase.clone(),
            bet_button_class: String::new(),
            bet_button_text: String::new(),
            cashout_amount: None,
            input_value: None,
            is_waiting: false,
            is_cashout_available: false,
            current_balance: session.current_balance.clone(),
            payout_count: session.payouts.len(),
            latest_payout: session.payouts.last().cloned(),
            auto_play_active: session.auto_play_active,
            current_odd: session.strategy.as_ref().map(|s| s.tuples.get(s.current_index).map(|t| t.0).unwrap_or(0.0)).unwrap_or(0.0),
            current_multiplier: session.strategy.as_ref().map(|s| s.tuples.get(s.current_index).map(|t| t.1).unwrap_or(0.0)).unwrap_or(0.0),
            cashout_target: 0.0,
            strategy_index: session.game_history.current_strategy_index,
            total_wins: session.game_history.total_wins,
            total_losses: session.game_history.total_losses,
        },
        None => GameState {
            phase: GamePhase::Error("Session not found".to_string()),
            bet_button_class: String::new(),
            bet_button_text: String::new(),
            cashout_amount: None,
            input_value: None,
            is_waiting: false,
            is_cashout_available: false,
            current_balance: "0.00".to_string(),
            payout_count: 0,
            latest_payout: None,
            auto_play_active: false,
            current_odd: 0.0,
            current_multiplier: 0.0,
            cashout_target: 0.0,
            strategy_index: 0,
            total_wins: 0,
            total_losses: 0,
        }
    };

    Json(GameStateResponse {
        state: game_state,
        timestamp: chrono::Utc::now(),
    })
}

pub async fn start_auto_play(
    State(state): State<Arc<AppState>>,
    Json(req): Json<AutoPlayRequest>,
) -> Json<AutoPlayResponse> {
    let sessions = state.sessions.read().await;

    let session = match sessions.get(&req.session_id) {
        Some(s) => s,
        None => {
            return Json(AutoPlayResponse {
                success: false,
                message: "Session not found".to_string(),
                session_id: req.session_id,
            });
        }
    };

    if session.auto_play_active {
        return Json(AutoPlayResponse {
            success: false,
            message: "Auto-play already active".to_string(),
            session_id: req.session_id,
        });
    }

    drop(sessions);

    {
        let mut sessions = state.sessions.write().await;
        if let Some(session) = sessions.get_mut(&req.session_id) {
            if let Err(e) = session.navigate_to_aviator().await {
                return Json(AutoPlayResponse {
                    success: false,
                    message: format!("Failed to navigate to Aviator: {}", e),
                    session_id: req.session_id,
                });
            }
        }
    }

    let strategy_data = StrategyData {
        tuples: vec![(100.0, 2.0), (200.0, 2.0), (400.0, 2.0)],
        expected_amount: 1400.0,
        current_index: 0,
    };

    let engine = GameEngine::new(state.clone());
    match engine.start_auto_play(req.session_id, strategy_data, req.demo_mode).await {
        Ok(msg) => {
            info!("Auto-play started for session {}", req.session_id);
            Json(AutoPlayResponse {
                success: true,
                message: msg,
                session_id: req.session_id,
            })
        }
        Err(e) => {
            Json(AutoPlayResponse {
                success: false,
                message: e,
                session_id: req.session_id,
            })
        }
    }
}

pub async fn stop_auto_play(
    State(state): State<Arc<AppState>>,
    Json(req): Json<StopAutoPlayRequest>,
) -> Json<serde_json::Value> {
    let engine = GameEngine::new(state.clone());

    match engine.stop_auto_play(req.session_id).await {
        Ok(_) => {
            info!("Auto-play stopped for session {}", req.session_id);
            Json(serde_json::json!({"success": true, "message": "Auto-play stopped"}))
        }
        Err(e) => {
            Json(serde_json::json!({"success": false, "message": e}))
        }
    }
}

pub async fn get_payouts(
    State(state): State<Arc<AppState>>,
    Path(session_id): Path<Uuid>,
) -> Json<PayoutsResponse> {
    let mut sessions = state.sessions.write().await;

    let payouts = match sessions.get_mut(&session_id) {
        Some(session) => {
            match session.get_payouts().await {
                Ok(p) => p,
                Err(e) => {
                    info!("Failed to get payouts: {}", e);
                    session.payouts.clone()
                }
            }
        }
        None => Vec::new(),
    };

    let count = payouts.len();
    let latest = payouts.last().cloned();

    Json(PayoutsResponse {
        payouts,
        count,
        latest,
        timestamp: chrono::Utc::now(),
    })
}

pub async fn get_balance(
    State(state): State<Arc<AppState>>,
    Path(session_id): Path<Uuid>,
) -> Json<BalanceResponse> {
    let mut sessions = state.sessions.write().await;

    match sessions.get_mut(&session_id) {
        Some(session) => {
            match session.get_balance().await {
                Ok(balance) => {
                    Json(BalanceResponse {
                        success: balance.success,
                        balance: balance.balance.clone(),
                        currency: balance.currency,
                        source: balance.source,
                        error: balance.error,
                    })
                }
                Err(e) => {
                    Json(BalanceResponse {
                        success: false,
                        balance: None,
                        currency: "KES".to_string(),
                        source: "error".to_string(),
                        error: Some(e),
                    })
                }
            }
        }
        None => {
            Json(BalanceResponse {
                success: false,
                balance: None,
                currency: "KES".to_string(),
                source: "session_not_found".to_string(),
                error: Some("Session not found".to_string()),
            })
        }
    }
}

pub async fn get_history(
    State(state): State<Arc<AppState>>,
    Path(session_id): Path<Uuid>,
) -> Json<GameHistoryResponse> {
    let sessions = state.sessions.read().await;

    let history = match sessions.get(&session_id) {
        Some(session) => GameHistoryResponse {
            rounds: Vec::new(),
            total_wins: session.game_history.total_wins,
            total_losses: session.game_history.total_losses,
            current_strategy_index: session.game_history.current_strategy_index,
        },
        None => GameHistoryResponse {
            rounds: Vec::new(),
            total_wins: 0,
            total_losses: 0,
            current_strategy_index: 0,
        }
    };

    Json(history)
}

pub async fn place_bet(
    State(state): State<Arc<AppState>>,
    Json(req): Json<serde_json::Value>,
) -> Json<serde_json::Value> {
    let session_id = req.get("session_id")
        .and_then(|v| v.as_str())
        .and_then(|s| Uuid::parse_str(s).ok())
        .unwrap_or_default();

    let amount = req.get("amount").and_then(|v| v.as_str()).unwrap_or("100");
    let odd = req.get("odd").and_then(|v| v.as_str()).unwrap_or("2.0");
    let multiplier = req.get("multiplier").and_then(|v| v.as_str()).unwrap_or("1.5");

    let mut sessions = state.sessions.write().await;

    match sessions.get_mut(&session_id) {
        Some(session) => {
            match session.place_auto_bet(amount, odd, multiplier).await {
                Ok(msg) => Json(serde_json::json!({"success": true, "message": msg})),
                Err(e) => Json(serde_json::json!({"success": false, "message": e})),
            }
        }
        None => Json(serde_json::json!({"success": false, "message": "Session not found"})),
    }
}

pub async fn cashout(
    State(state): State<Arc<AppState>>,
    Json(req): Json<serde_json::Value>,
) -> Json<serde_json::Value> {
    let session_id = req.get("session_id")
        .and_then(|v| v.as_str())
        .and_then(|s| Uuid::parse_str(s).ok())
        .unwrap_or_default();

    let mut sessions = state.sessions.write().await;

    match sessions.get_mut(&session_id) {
        Some(session) => {
            match session.click_cashout().await {
                Ok(msg) => Json(serde_json::json!({"success": true, "message": msg})),
                Err(e) => Json(serde_json::json!({"success": false, "message": e})),
            }
        }
        None => Json(serde_json::json!({"success": false, "message": "Session not found"})),
    }
}
