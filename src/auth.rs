use axum_extra::extract::cookie::{Cookie, SameSite};
use chrono::{Duration as ChronoDuration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use time::Duration;

pub const AUTH_COOKIE_NAME: &str = "ra_auth";
const TOKEN_TTL_DAYS: i64 = 7;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub iat: usize,
    pub exp: usize,
}

pub fn normalize_call_sign(input: &str) -> String {
    input.trim().to_uppercase()
}

pub fn encode_token(call_sign: &str, secret: &str) -> Result<String, jsonwebtoken::errors::Error> {
    let now = Utc::now();
    let exp = now + ChronoDuration::days(TOKEN_TTL_DAYS);
    let claims = Claims {
        sub: call_sign.to_string(),
        iat: now.timestamp() as usize,
        exp: exp.timestamp() as usize,
    };
    jsonwebtoken::encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
}

pub fn decode_token(token: &str, secret: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    let mut validation = Validation::default();
    validation.validate_exp = true;
    let data = jsonwebtoken::decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &validation,
    )?;
    Ok(data.claims)
}

pub fn build_auth_cookie(token: String) -> Cookie<'static> {
    Cookie::build((AUTH_COOKIE_NAME, token))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax)
        .max_age(Duration::days(TOKEN_TTL_DAYS))
        .build()
}

pub fn build_logout_cookie() -> Cookie<'static> {
    Cookie::build((AUTH_COOKIE_NAME, ""))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax)
        .max_age(Duration::seconds(0))
        .build()
}
