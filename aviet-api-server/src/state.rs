use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::RwLock;
use sqlx::PgPool;
use uuid::Uuid;

use crate::config::AppConfig;
use crate::browser::BrowserSession;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub config: AppConfig,
    pub sessions: Arc<RwLock<HashMap<Uuid, BrowserSession>>>,
    pub start_time: std::time::Instant,
}

impl AppState {
    pub fn new(db: PgPool, config: AppConfig) -> Self {
        Self {
            db,
            config,
            sessions: Arc::new(RwLock::new(HashMap::new())),
            start_time: std::time::Instant::now(),
        }
    }

    pub fn uptime_seconds(&self) -> u64 {
        self.start_time.elapsed().as_secs()
    }
}
