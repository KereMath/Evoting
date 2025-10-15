use axum::{
    extract::State,
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub success: bool,
    pub message: String,
    pub token: Option<String>,
}

pub async fn login(
    State(_state): State<Arc<AppState>>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, StatusCode> {
    // Hardcoded admin credentials
    if payload.username == "admin" && payload.password == "admin" {
        Ok(Json(LoginResponse {
            success: true,
            message: "Login successful".to_string(),
            token: Some("admin-token-12345".to_string()), // Hardcoded token
        }))
    } else {
        Ok(Json(LoginResponse {
            success: false,
            message: "Invalid credentials".to_string(),
            token: None,
        }))
    }
}
