use crate::errors::ApiError;
use crate::models::*;
use reqwest::Client;
use uuid::Uuid;

const DEFAULT_BASE_URL: &str = "http://localhost:8080";

#[derive(Clone)]
pub struct AvietApiClient {
    client: Client,
    base_url: String,
    auth_token: Option<String>,
}

impl AvietApiClient {
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .expect("Failed to build HTTP client"),
            base_url: DEFAULT_BASE_URL.to_string(),
            auth_token: None,
        }
    }

    pub fn with_base_url(mut self, url: &str) -> Self {
        self.base_url = url.to_string();
        self
    }

    pub fn with_token(mut self, token: String) -> Self {
        self.auth_token = Some(token);
        self
    }

    fn url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }

    fn build_request(&self, method: reqwest::Method, path: &str) -> reqwest::RequestBuilder {
        let mut req = self.client.request(method, self.url(path));
        if let Some(token) = &self.auth_token {
            req = req.bearer_auth(token);
        }
        req
    }

    // ============================================
    // AUTH
    // ============================================

    pub async fn login(&self, phone: &str, password: &str, site_name: &str) -> Result<LoginResponse, ApiError> {
        let req = LoginRequest {
            phone: phone.to_string(),
            password: password.to_string(),
            site_name: site_name.to_string(),
        };
        let resp = self.build_request(reqwest::Method::POST, "/api/auth/login")
            .json(&req)
            .send()
            .await?;

        if !resp.status().is_success() {
            let err_text = resp.text().await.unwrap_or_default();
            return Err(ApiError::Server(err_text));
        }
        Ok(resp.json().await?)
    }

    pub async fn logout(&self, session_id: Uuid) -> Result<(), ApiError> {
        let resp = self.build_request(reqwest::Method::POST, &format!("/api/auth/logout/{}", session_id))
            .send()
            .await?;
        if !resp.status().is_success() {
            return Err(ApiError::Server(resp.text().await.unwrap_or_default()));
        }
        Ok(())
    }

    // ============================================
    // GAME STATE
    // ============================================

    pub async fn get_game_state(&self, session_id: Uuid) -> Result<GameStateResponse, ApiError> {
        let resp = self.build_request(reqwest::Method::GET, &format!("/api/game/state/{}", session_id))
            .send()
            .await?;
        if !resp.status().is_success() {
            return Err(ApiError::Server(resp.text().await.unwrap_or_default()));
        }
        Ok(resp.json().await?)
    }

    // ============================================
    // AUTO-PLAY
    // ============================================

    pub async fn start_auto_play(&self, req: AutoPlayRequest) -> Result<AutoPlayResponse, ApiError> {
        let resp = self.build_request(reqwest::Method::POST, "/api/game/auto-play/start")
            .json(&req)
            .send()
            .await?;
        if !resp.status().is_success() {
            return Err(ApiError::Server(resp.text().await.unwrap_or_default()));
        }
        Ok(resp.json().await?)
    }

    pub async fn stop_auto_play(&self, req: StopAutoPlayRequest) -> Result<(), ApiError> {
        let resp = self.build_request(reqwest::Method::POST, "/api/game/auto-play/stop")
            .json(&req)
            .send()
            .await?;
        if !resp.status().is_success() {
            return Err(ApiError::Server(resp.text().await.unwrap_or_default()));
        }
        Ok(())
    }

    // ============================================
    // PAYOUTS
    // ============================================

    pub async fn get_payouts(&self, session_id: Uuid) -> Result<PayoutsResponse, ApiError> {
        let resp = self.build_request(reqwest::Method::GET, &format!("/api/game/payouts/{}", session_id))
            .send()
            .await?;
        if !resp.status().is_success() {
            return Err(ApiError::Server(resp.text().await.unwrap_or_default()));
        }
        Ok(resp.json().await?)
    }

    // ============================================
    // BALANCE
    // ============================================

    pub async fn get_balance(&self, session_id: Uuid) -> Result<BalanceResponse, ApiError> {
        let resp = self.build_request(reqwest::Method::GET, &format!("/api/game/balance/{}", session_id))
            .send()
            .await?;
        if !resp.status().is_success() {
            return Err(ApiError::Server(resp.text().await.unwrap_or_default()));
        }
        Ok(resp.json().await?)
    }

    // ============================================
    // STRATEGY
    // ============================================

    pub async fn generate_strategy(&self, req: StrategyRequest) -> Result<StrategyResponse, ApiError> {
        let resp = self.build_request(reqwest::Method::POST, "/api/strategy/generate")
            .json(&req)
            .send()
            .await?;
        if !resp.status().is_success() {
            return Err(ApiError::Server(resp.text().await.unwrap_or_default()));
        }
        Ok(resp.json().await?)
    }

    pub async fn list_strategies(&self) -> Result<Vec<StrategyResponse>, ApiError> {
        let resp = self.build_request(reqwest::Method::GET, "/api/strategy/list")
            .send()
            .await?;
        if !resp.status().is_success() {
            return Err(ApiError::Server(resp.text().await.unwrap_or_default()));
        }
        Ok(resp.json().await?)
    }

    // ============================================
    // HISTORY
    // ============================================

    pub async fn get_history(&self, session_id: Uuid) -> Result<GameHistoryResponse, ApiError> {
        let resp = self.build_request(reqwest::Method::GET, &format!("/api/game/history/{}", session_id))
            .send()
            .await?;
        if !resp.status().is_success() {
            return Err(ApiError::Server(resp.text().await.unwrap_or_default()));
        }
        Ok(resp.json().await?)
    }

    // ============================================
    // SITES
    // ============================================

    pub async fn get_sites(&self) -> Result<SitesResponse, ApiError> {
        let resp = self.build_request(reqwest::Method::GET, "/api/sites")
            .send()
            .await?;
        if !resp.status().is_success() {
            return Err(ApiError::Server(resp.text().await.unwrap_or_default()));
        }
        Ok(resp.json().await?)
    }

    // ============================================
    // STATUS
    // ============================================

    pub async fn server_status(&self) -> Result<ServerStatus, ApiError> {
        let resp = self.build_request(reqwest::Method::GET, "/api/status")
            .send()
            .await?;
        if !resp.status().is_success() {
            return Err(ApiError::Server(resp.text().await.unwrap_or_default()));
        }
        Ok(resp.json().await?)
    }
}

impl Default for AvietApiClient {
    fn default() -> Self {
        Self::new()
    }
}
