use axum::{
    extract::State,
    http::StatusCode,
    Json,
};
use std::sync::Arc;

use crate::{AppState, models::{Trustee, CreateTrusteeRequest}};

pub async fn list_trustees(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<Trustee>>, StatusCode> {
    let trustees = sqlx::query_as::<_, Trustee>(
        "SELECT * FROM trustees ORDER BY created_at DESC"
    )
    .fetch_all(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(trustees))
}

pub async fn create_trustee(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateTrusteeRequest>,
) -> Result<Json<Trustee>, StatusCode> {
    let trustee = sqlx::query_as::<_, Trustee>(
        r#"
        INSERT INTO trustees (election_id, name, status, docker_type, ip_address)
        VALUES ($1, $2, 'active', $3, $4)
        RETURNING *
        "#
    )
    .bind(&payload.election_id)
    .bind(&payload.name)
    .bind(&payload.docker_type)
    .bind(&payload.ip_address)
    .fetch_one(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Log event
    let _ = sqlx::query(
        r#"
        INSERT INTO system_events (event_type, entity_type, entity_id, data)
        VALUES ($1, $2, $3, $4)
        "#
    )
    .bind("trustee_created")
    .bind("trustee")
    .bind(&trustee.id)
    .bind(serde_json::json!({
        "name": &trustee.name,
        "docker_type": &trustee.docker_type,
        "ip_address": &trustee.ip_address
    }))
    .execute(&state.db)
    .await;

    // Check if we should advance election phase
    let election_id = payload.election_id;
    let trustee_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM trustees WHERE election_id = $1"
    )
    .bind(&election_id)
    .fetch_one(&state.db)
    .await
    .unwrap_or(0);

    let election: Option<crate::models::Election> = sqlx::query_as(
        "SELECT * FROM elections WHERE id = $1"
    )
    .bind(&election_id)
    .fetch_optional(&state.db)
    .await
    .ok()
    .flatten();

    if let Some(election) = election {
        if trustee_count >= election.total_trustees as i64 && election.phase == 1 {
            // Advance to phase 2
            let _ = sqlx::query(
                "UPDATE elections SET phase = 2 WHERE id = $1"
            )
            .bind(&election_id)
            .execute(&state.db)
            .await;
        }
    }

    Ok(Json(trustee))
}
