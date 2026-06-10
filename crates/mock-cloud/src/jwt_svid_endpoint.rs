use axum::Json;
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Deserialize)]
pub struct JwtSvidRequest {
    pub audience: Vec<String>,
}

#[derive(Serialize)]
pub struct JwtSvidResponse {
    pub svid: String,
}

#[derive(Serialize)]
struct Claims {
    aud: Vec<String>,
    exp: usize,
    sub: String,
}

pub async fn handle_jwt_svid(
    Json(payload): Json<JwtSvidRequest>,
) -> Json<JwtSvidResponse> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as usize;

    // Issue a 1-hour token
    let claims = Claims {
        aud: payload.audience.clone(),
        exp: now + 3600,
        sub: "spiffe://mock-cloud/workload".to_string(),
    };

    // Use a hardcoded dummy secret for mock-cloud (HS256)
    let secret = b"super_secret_mock_key_for_testing";

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret),
    )
    .unwrap_or_else(|_| "dummy.jwt.token".to_string());

    Json(JwtSvidResponse { svid: token })
}
