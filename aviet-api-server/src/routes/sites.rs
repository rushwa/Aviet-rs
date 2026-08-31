use axum::{extract::State, Json};
use std::sync::Arc;

use crate::state::AppState;
use aviet_shared::models::*;

pub async fn get_sites(State(_state): State<Arc<AppState>>) -> Json<SitesResponse> {
    Json(SitesResponse {
        sites: vec![
            SiteProfile {
                name: "Betika".to_string(),
                url: "https://www.betika.com/en-ke/".to_string(),
                login_url: "https://www.betika.com/en-ke/login".to_string(),
                aviator_url: "https://www.betika.com/en-ke/aviator".to_string(),
                phone_selector: "input[name='phone-number']".to_string(),
                password_selector: "input[type='password']".to_string(),
                login_button_selector: "button[type='submit']".to_string(),
                logged_in_indicator: Some("span.nav__item__label".to_string()),
            },
        ],
    })
}
