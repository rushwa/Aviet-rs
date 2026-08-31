use anyhow::Result;

#[derive(Clone, Debug)]
pub struct AppConfig {
    pub host: String,
    pub port: u16,
    pub database_url: String,
    pub jwt_secret: String,
    pub geckodriver_url: String,
    pub headless: bool,
    pub max_concurrent_sessions: usize,
}

impl AppConfig {
    pub fn from_env() -> Result<Self> {
        dotenvy::dotenv().ok();

        Ok(Self {
            host: std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            port: std::env::var("PORT")
                .unwrap_or_else(|_| "8080".to_string())
                .parse()?,
            database_url: std::env::var("DATABASE_URL")
                .unwrap_or_else(|_| "postgres://postgres:postgres@localhost/aviet".to_string()),
            jwt_secret: std::env::var("JWT_SECRET")
                .unwrap_or_else(|_| "change-me-in-production".to_string()),
            geckodriver_url: std::env::var("GECKODRIVER_URL")
                .unwrap_or_else(|_| "http://localhost:4444".to_string()),
            headless: std::env::var("HEADLESS")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),
            max_concurrent_sessions: std::env::var("MAX_SESSIONS")
                .unwrap_or_else(|_| "5".to_string())
                .parse()
                .unwrap_or(5),
        })
    }
}
