use std::sync::Arc;
use tokio::time::{sleep, Duration};
use tracing::info;
use uuid::Uuid;

use crate::browser::StrategyData;
use crate::state::AppState;
use aviet_shared::models::*;

pub struct GameEngine {
    state: Arc<AppState>,
}

impl GameEngine {
    pub fn new(state: Arc<AppState>) -> Self {
        Self { state }
    }

    pub async fn start_auto_play(
        &self,
        session_id: Uuid,
        strategy: StrategyData,
        demo_mode: bool,
    ) -> Result<String, String> {
        let mut sessions = self.state.sessions.write().await;
        let session = sessions.get_mut(&session_id)
            .ok_or("Session not found")?;

        if session.auto_play_active {
            return Err("Auto-play already active".to_string());
        }

        session.auto_play_active = true;
        session.strategy = Some(strategy);
        session.is_demo_mode = demo_mode;
        session.game_phase = GamePhase::Idle;

        let state_clone = self.state.clone();
        tokio::spawn(async move {
            auto_play_loop(state_clone, session_id).await;
        });

        Ok("Auto-play started".to_string())
    }

    pub async fn stop_auto_play(&self, session_id: Uuid) -> Result<(), String> {
        let mut sessions = self.state.sessions.write().await;
        let session = sessions.get_mut(&session_id)
            .ok_or("Session not found")?;

        session.auto_play_active = false;
        session.game_phase = GamePhase::Idle;

        Ok(())
    }
}

async fn auto_play_loop(state: Arc<AppState>, session_id: Uuid) {
    loop {
        let (should_continue, phase) = {
            let sessions = state.sessions.read().await;
            if let Some(session) = sessions.get(&session_id) {
                if !session.auto_play_active {
                    info!("Auto-play stopped for session {}", session_id);
                    return;
                }
                (true, session.game_phase.clone())
            } else {
                info!("Session {} no longer exists", session_id);
                return;
            }
        };

        if !should_continue {
            return;
        }

        match phase {
            GamePhase::Idle => {
                handle_idle_phase(state.clone(), session_id).await;
                // Longer delay in Idle to prevent rapid retry on errors
                sleep(Duration::from_secs(2)).await;
            }
            GamePhase::WaitingRound | GamePhase::Flying => {
                handle_flying_phase(state.clone(), session_id).await;
                sleep(Duration::from_millis(500)).await;
            }
            GamePhase::CashoutPending => {
                sleep(Duration::from_millis(300)).await;
            }
            GamePhase::Settling => {
                sleep(Duration::from_secs(1)).await;
                {
                    let mut sessions = state.sessions.write().await;
                    if let Some(session) = sessions.get_mut(&session_id) {
                        session.game_phase = GamePhase::Idle;
                    }
                }
            }
            GamePhase::Error(ref _e) => {
                sleep(Duration::from_secs(3)).await;
                {
                    let mut sessions = state.sessions.write().await;
                    if let Some(session) = sessions.get_mut(&session_id) {
                        session.game_phase = GamePhase::Idle;
                    }
                }
            }
            _ => {
                sleep(Duration::from_millis(500)).await;
            }
        }
    }
}

async fn handle_idle_phase(state: Arc<AppState>, session_id: Uuid) {
    // Check if stopped before doing anything
    {
        let sessions = state.sessions.read().await;
        if let Some(session) = sessions.get(&session_id) {
            if !session.auto_play_active {
                return;
            }
        } else {
            return;
        }
    }

    let (odd, multiplier, _cashout_target) = {
        let sessions = state.sessions.read().await;
        let session = match sessions.get(&session_id) {
            Some(s) => s,
            None => return,
        };

        let strategy = match &session.strategy {
            Some(s) => s,
            None => return,
        };

        let (odd, mult) = match strategy.tuples.get(strategy.current_index) {
            Some(t) => *t,
            None => return,
        };

        let target = (odd * mult * 100.0).round() / 100.0;
        (odd, mult, target)
    };

    {
        let mut sessions = state.sessions.write().await;
        let session = match sessions.get_mut(&session_id) {
            Some(s) => s,
            None => return,
        };

        // Double-check stop flag before placing bet
        if !session.auto_play_active {
            return;
        }

        session.game_phase = GamePhase::Betting;

        let amount_str = format!("{:.2}", odd);
        let odd_str = format!("{:.2}", odd);
        let mult_str = format!("{:.2}", multiplier);

        let result = session.place_auto_bet(&amount_str, &odd_str, &mult_str).await;

        match result {
            Ok(msg) => {
                info!("[AUTO] Bet placed: {}", msg);
                session.game_phase = GamePhase::WaitingRound;
            }
            Err(e) => {
                info!("[AUTO] Bet failed: {}", e);
                session.game_phase = GamePhase::Idle;
                // Add delay on error to prevent rapid retry loop
                // (the outer loop delay will handle this)
            }
        }
    }
}

async fn handle_flying_phase(state: Arc<AppState>, session_id: Uuid) {
    let (cashout_target, should_cashout) = {
        let mut sessions = state.sessions.write().await;
        let session = match sessions.get_mut(&session_id) {
            Some(s) => s,
            None => return,
        };

        let strategy = match &session.strategy {
            Some(s) => s,
            None => return,
        };

        let (odd, mult) = match strategy.tuples.get(strategy.current_index) {
            Some(t) => *t,
            None => return,
        };

        let target = (odd * mult * 100.0).round() / 100.0;

        match session.get_game_state().await {
            Ok(game_state) => {
                if let Some(amount_str) = &game_state.cashout_amount {
                    let current: f64 = amount_str.chars()
                        .filter(|c| c.is_numeric() || *c == '.')
                        .collect::<String>()
                        .parse()
                        .unwrap_or(0.0);

                    if current >= target {
                        (target, true)
                    } else {
                        (target, false)
                    }
                } else {
                    (target, false)
                }
            }
            Err(_) => (target, false),
        }
    };

    if should_cashout {
        let mut sessions = state.sessions.write().await;
        let session = match sessions.get_mut(&session_id) {
            Some(s) => s,
            None => return,
        };

        session.game_phase = GamePhase::CashoutPending;

        match session.click_cashout().await {
            Ok(_) => {
                info!("[AUTO] Cashout clicked at target {}", cashout_target);
                session.game_phase = GamePhase::Settling;
                session.game_history.total_wins += 1;
                session.game_history.current_strategy_index = 0;
            }
            Err(e) => {
                info!("[AUTO] Cashout failed: {}", e);
                session.game_phase = GamePhase::Idle;
                session.game_history.total_losses += 1;

                if let Some(ref mut strategy) = session.strategy {
                    strategy.current_index = (strategy.current_index + 1) % strategy.tuples.len();
                }
            }
        }
    }
}
