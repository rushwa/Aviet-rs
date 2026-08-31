use axum::{
    extract::{State, Path},
    Json,
};
use std::sync::Arc;
use uuid::Uuid;

use crate::state::AppState;
use crate::strategy::StrategyGenerator;
use aviet_shared::models::*;

pub async fn generate(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<StrategyRequest>,
) -> Json<StrategyResponse> {
    let (tuples, expected) = StrategyGenerator::generate(
        req.start,
        req.end,
        req.profitier,
        req.multiplier,
        req.item_no,
        req.choice,
    );

    let strategy_id = Uuid::new_v4();
    let name = req.name.unwrap_or_else(|| format!("Strategy {}", strategy_id));

    Json(StrategyResponse {
        id: strategy_id,
        name,
        tuples,
        expected_amount: expected,
        created_at: chrono::Utc::now(),
    })
}

pub async fn list_strategies(
    State(_state): State<Arc<AppState>>,
) -> Json<Vec<StrategyResponse>> {
    Json(vec![])
}

pub async fn get_strategy(
    State(_state): State<Arc<AppState>>,
    Path(_id): Path<Uuid>,
) -> Json<Option<StrategyResponse>> {
    Json(None)
}
