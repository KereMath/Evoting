use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use std::sync::Arc;
use uuid::Uuid;
use std::ffi::{CString, CStr};
use std::os::raw::c_char;

use crate::models::*;
use crate::AppState;

#[repr(C)]
struct CryptoParams {
    prime_order: *mut c_char,
    g1: *mut c_char,
    g2: *mut c_char,
    h1: *mut c_char,
    pairing_params: *mut c_char,
    security_level: i32,
}

#[link(name = "evoting_crypto")]
extern "C" {
    fn setup_crypto_params(security_level: i32) -> *mut CryptoParams;
    fn free_crypto_params(params: *mut CryptoParams);
}

/// Generate cryptographic parameters for an election
/// This executes PBC-based parameter generation and stores results in database
pub async fn crypto_setup(
    State(state): State<Arc<AppState>>,
    Path(election_id): Path<Uuid>,
    Json(request): Json<CryptoSetupRequest>,
) -> Result<Json<CryptoSetupResponse>, StatusCode> {
    tracing::info!("Starting crypto setup for election {}", election_id);

    // Verify election exists and is in phase 4 (docker containers created)
    let election: Election = sqlx::query_as(
        "SELECT * FROM elections WHERE id = $1"
    )
    .bind(&election_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| {
        tracing::error!("Database error: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?
    .ok_or_else(|| {
        tracing::error!("Election not found: {}", election_id);
        StatusCode::NOT_FOUND
    })?;

    if election.phase != 4 {
        return Ok(Json(CryptoSetupResponse {
            success: false,
            message: format!("Election must be in phase 4 (setup complete). Current phase: {}", election.phase),
            parameters: None,
        }));
    }

    // Check if crypto parameters already exist
    let existing: Option<CryptoParameters> = sqlx::query_as(
        "SELECT * FROM crypto_parameters WHERE election_id = $1"
    )
    .bind(&election_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| {
        tracing::error!("Database error: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    if existing.is_some() {
        return Ok(Json(CryptoSetupResponse {
            success: false,
            message: "Crypto parameters already generated for this election".to_string(),
            parameters: existing.map(|p| p.into()),
        }));
    }

    let security_level = request.security_level.unwrap_or(256);

    tracing::info!("Generating PBC parameters with security level: {} bits", security_level);

    // Call C++ crypto library via FFI to generate real PBC parameters
    let params = generate_pbc_parameters(security_level)?;

    // Store parameters in database
    let crypto_params: CryptoParameters = sqlx::query_as(
        r#"
        INSERT INTO crypto_parameters
        (election_id, prime_order, g1, g2, h1, pairing_params, security_level)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING *
        "#
    )
    .bind(&election_id)
    .bind(&params.prime_order)
    .bind(&params.g1)
    .bind(&params.g2)
    .bind(&params.h1)
    .bind(&params.pairing_params)
    .bind(security_level)
    .fetch_one(&state.db)
    .await
    .map_err(|e| {
        tracing::error!("Failed to store crypto parameters: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Update election phase to 5 (crypto setup complete)
    sqlx::query(
        "UPDATE elections SET phase = 5, status = 'key_generation' WHERE id = $1"
    )
    .bind(&election_id)
    .execute(&state.db)
    .await
    .map_err(|e| {
        tracing::error!("Failed to update election phase: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Log event
    sqlx::query(
        r#"
        INSERT INTO system_events (event_type, entity_type, entity_id, data)
        VALUES ($1, $2, $3, $4)
        "#
    )
    .bind("crypto_setup_complete")
    .bind("election")
    .bind(&election_id)
    .bind(serde_json::json!({
        "security_level": security_level,
        "prime_order_length": params.prime_order.len(),
    }))
    .execute(&state.db)
    .await
    .ok();

    tracing::info!("Crypto setup completed successfully for election {}", election_id);

    Ok(Json(CryptoSetupResponse {
        success: true,
        message: format!(
            "Cryptographic parameters generated successfully with {}-bit security",
            security_level
        ),
        parameters: Some(crypto_params.into()),
    }))
}

/// Get crypto parameters for an election (public endpoint)
pub async fn get_crypto_parameters(
    State(state): State<Arc<AppState>>,
    Path(election_id): Path<Uuid>,
) -> Result<Json<CryptoParametersPublic>, StatusCode> {
    let params: CryptoParameters = sqlx::query_as(
        "SELECT * FROM crypto_parameters WHERE election_id = $1"
    )
    .bind(&election_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| {
        tracing::error!("Database error: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?
    .ok_or_else(|| {
        tracing::error!("Crypto parameters not found for election: {}", election_id);
        StatusCode::NOT_FOUND
    })?;

    Ok(Json(params.into()))
}

// Generate real PBC parameters via FFI call to C++ crypto library
fn generate_pbc_parameters(security_level: i32) -> Result<PbcCryptoParams, StatusCode> {
    unsafe {
        let params_ptr = setup_crypto_params(security_level);

        if params_ptr.is_null() {
            tracing::error!("Failed to generate crypto parameters");
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }

        let params = &*params_ptr;

        let prime_order = CStr::from_ptr(params.prime_order)
            .to_string_lossy()
            .into_owned();
        let g1 = CStr::from_ptr(params.g1)
            .to_string_lossy()
            .into_owned();
        let g2 = CStr::from_ptr(params.g2)
            .to_string_lossy()
            .into_owned();
        let h1 = CStr::from_ptr(params.h1)
            .to_string_lossy()
            .into_owned();
        let pairing_params = CStr::from_ptr(params.pairing_params)
            .to_string_lossy()
            .into_owned();

        let result = PbcCryptoParams {
            prime_order,
            g1,
            g2,
            h1,
            pairing_params,
        };

        free_crypto_params(params_ptr);

        Ok(result)
    }
}

struct PbcCryptoParams {
    prime_order: String,
    g1: String,
    g2: String,
    h1: String,
    pairing_params: String,
}
