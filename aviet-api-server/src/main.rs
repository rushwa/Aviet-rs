use axum::{
    routing::{get, post},
    Router,
    http::Method,
};
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tracing::info;

mod config;
mod database;
mod state;
mod routes;
mod browser;
mod game;
mod strategy;
mod auth;

use config::AppConfig;
use state::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter("aviet_api_server=debug,tower_http=debug")
        .init();

    info!("Starting Aviet API Server...");

    let config = AppConfig::from_env()?;
    info!("Configuration loaded");

    let db_pool = database::init_db(&config.database_url).await?;
    info!("Database connected");

    sqlx::migrate!("./migrations")
        .run(&db_pool)
        .await?;
    info!("Database migrations applied");

    let state = Arc::new(AppState::new(db_pool, config.clone()));

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers(Any);

    let app = Router::new()
        .route("/api/status", get(routes::status::get_status))
        .route("/api/auth/login", post(routes::auth::login))
        .route("/api/auth/logout/:session_id", post(routes::auth::logout))
        .route("/api/auth/session/:session_id", get(routes::auth::get_session))
        .route("/api/game/state/:session_id", get(routes::game::get_game_state))
        .route("/api/game/auto-play/start", post(routes::game::start_auto_play))
        .route("/api/game/auto-play/stop", post(routes::game::stop_auto_play))
        .route("/api/game/payouts/:session_id", get(routes::game::get_payouts))
        .route("/api/game/balance/:session_id", get(routes::game::get_balance))
        .route("/api/game/history/:session_id", get(routes::game::get_history))
        .route("/api/game/bet", post(routes::game::place_bet))
        .route("/api/game/cashout", post(routes::game::cashout))
        .route("/api/strategy/generate", post(routes::strategy::generate))
        .route("/api/strategy/list", get(routes::strategy::list_strategies))
        .route("/api/strategy/:id", get(routes::strategy::get_strategy))
        .route("/api/sites", get(routes::sites::get_sites))
        .route("/api/ws/:session_id", get(routes::ws::ws_handler))
        .layer(cors)
        .with_state(state.clone());

    let addr: SocketAddr = format!("{}:{}", config.host, config.port).parse()?;
    info!("Server listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
