use jsonwebtoken::{encode, decode, Header, Validation, Algorithm};
use jsonwebtoken::errors::Error as JwtError;
use serde::{Serialize, Deserialize};
use chrono::{Utc, Duration};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,      // user_id
    pub session_id: String,
    pub exp: i64,
    pub iat: i64,
}

pub fn create_token(user_id: Uuid, session_id: Uuid, secret: &str) -> Result<String, JwtError> {
    let now = Utc::now();
    let claims = Claims {
        sub: user_id.to_string(),
        session_id: session_id.to_string(),
        iat: now.timestamp(),
        exp: (now + Duration::hours(24)).timestamp(),
    };

    encode(&Header::default(), &claims, &jsonwebtoken::EncodingKey::from_secret(secret.as_bytes()))
}

pub fn verify_token(token: &str, secret: &str) -> Result<Claims, JwtError> {
    let validation = Validation::new(Algorithm::HS256);
    let token_data = decode::<Claims>(
        token,
        &jsonwebtoken::DecodingKey::from_secret(secret.as_bytes()),
        &validation,
    )?;
    Ok(token_data.claims)
}
