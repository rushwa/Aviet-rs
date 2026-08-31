use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

// ============================================
// AUTH
// ============================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginRequest {
    pub phone: String,
    pub password: String,
    pub site_name: String, // "Betika", "SportPesa", etc.
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginResponse {
    pub success: bool,
    pub session_id: Uuid,
    pub message: String,
    pub balance: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInfo {
    pub session_id: Uuid,
    pub user_id: Uuid,
    pub is_active: bool,
    pub site_url: String,
    pub created_at: DateTime<Utc>,
}

// ============================================
// GAME STATE
// ============================================

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum GamePhase {
    Idle,
    Betting,
    WaitingRound,
    Flying,
    CashoutPending,
    Settling,
    Error(String),
}

impl GamePhase {
    pub fn can_fetch_balance(&self) -> bool {
        matches!(self, GamePhase::Idle | GamePhase::WaitingRound | GamePhase::Settling)
    }

    pub fn can_fetch_payouts(&self) -> bool {
        true
    }

    pub fn can_place_bet(&self) -> bool {
        *self == GamePhase::Idle
    }

    pub fn can_cashout(&self) -> bool {
        *self == GamePhase::Flying
    }

    pub fn is_busy(&self) -> bool {
        matches!(self, GamePhase::Betting | GamePhase::CashoutPending)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    pub phase: GamePhase,
    pub bet_button_class: String,
    pub bet_button_text: String,
    pub cashout_amount: Option<String>,
    pub input_value: Option<String>,
    pub is_waiting: bool,
    pub is_cashout_available: bool,
    pub current_balance: String,
    pub payout_count: usize,
    pub latest_payout: Option<String>,
    pub auto_play_active: bool,
    pub current_odd: f64,
    pub current_multiplier: f64,
    pub cashout_target: f64,
    pub strategy_index: usize,
    pub total_wins: u32,
    pub total_losses: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameStateResponse {
    pub state: GameState,
    pub timestamp: DateTime<Utc>,
}

// ============================================
// STRATEGY
// ============================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyRequest {
    pub start: f64,
    pub end: f64,
    pub profitier: f64,
    pub multiplier: f64,
    pub item_no: usize,
    pub choice: usize, // 0=Basic, 1=Advanced, 2=MultiAdvanced
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyResponse {
    pub id: Uuid,
    pub name: String,
    pub tuples: Vec<(f64, f64)>,
    pub expected_amount: f64,
    pub created_at: DateTime<Utc>,
}

// ============================================
// BETTING / AUTO-PLAY
// ============================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoPlayRequest {
    pub strategy_id: Uuid,
    pub bet_amount: Option<f64>, // override strategy odd
    pub demo_mode: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoPlayResponse {
    pub success: bool,
    pub message: String,
    pub session_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StopAutoPlayRequest {
    pub session_id: Uuid,
}

// ============================================
// PAYOUTS
// ============================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PayoutsResponse {
    pub payouts: Vec<String>,
    pub count: usize,
    pub latest: Option<String>,
    pub timestamp: DateTime<Utc>,
}

// ============================================
// HISTORY
// ============================================

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RoundResult {
    Win,
    Loss,
    Pending,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameRound {
    pub id: Uuid,
    pub round_id: String,
    pub timestamp: DateTime<Utc>,
    pub odd_used: f64,
    pub multiplier_used: f64,
    pub bet_amount: f64,
    pub cashout_target: f64,
    pub actual_cashout: Option<f64>,
    pub result: RoundResult,
    pub balance_before: String,
    pub balance_after: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameHistoryResponse {
    pub rounds: Vec<GameRound>,
    pub total_wins: u32,
    pub total_losses: u32,
    pub current_strategy_index: usize,
}

// ============================================
// BALANCE
// ============================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BalanceResponse {
    pub success: bool,
    pub balance: Option<String>,
    pub currency: String,
    pub source: String,
    pub error: Option<String>,
}

// ============================================
// SITE PROFILES
// ============================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SiteProfile {
    pub name: String,
    pub url: String,
    pub login_url: String,
    pub aviator_url: String,
    pub phone_selector: String,
    pub password_selector: String,
    pub login_button_selector: String,
    pub logged_in_indicator: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SitesResponse {
    pub sites: Vec<SiteProfile>,
}

// ============================================
// NETWORK / STATUS
// ============================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkStatus {
    pub online: bool,
    pub last_check: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerStatus {
    pub version: String,
    pub uptime_seconds: u64,
    pub active_sessions: usize,
    pub database_connected: bool,
}
