use axum::{
    extract::{State, Path},
    http::StatusCode,
    Json,
};
use std::sync::Arc;
use uuid::Uuid;

use crate::{AppState, models::{Election, CreateElectionRequest}};

pub async fn list_elections(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<Election>>, StatusCode> {
    let elections = sqlx::query_as::<_, Election>(
        "SELECT * FROM elections ORDER BY created_at DESC"
    )
    .fetch_all(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(elections))
}

pub async fn create_election(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateElectionRequest>,
) -> Result<Json<Election>, StatusCode> {
    let election = sqlx::query_as::<_, Election>(
        r#"
        INSERT INTO elections (name, description, threshold, total_trustees, status)
        VALUES ($1, $2, $3, $4, 'setup')
        RETURNING *
        "#
    )
    .bind(&payload.name)
    .bind(&payload.description)
    .bind(payload.threshold)
    .bind(payload.total_trustees)
    .fetch_one(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Log event
    let _ = sqlx::query(
        "INSERT INTO system_events (event_type, entity_type, entity_id, data) VALUES ($1, $2, $3, $4)"
    )
    .bind("election_created")
    .bind("election")
    .bind(election.id)
    .bind(serde_json::json!({
        "name": &election.name,
        "threshold": election.threshold,
        "total_trustees": election.total_trustees
    }))
    .execute(&state.db)
    .await;

    Ok(Json(election))
}

pub async fn get_election(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<Election>, StatusCode> {
    let election = sqlx::query_as::<_, Election>(
        "SELECT * FROM elections WHERE id = $1"
    )
    .bind(id)
    .fetch_one(&state.db)
    .await
    .map_err(|_| StatusCode::NOT_FOUND)?;

    Ok(Json(election))
}

pub async fn delete_election(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    // First cleanup all Docker containers and network
    tracing::info!("🗑️  Deleting election: {}, cleaning up containers first...", id);

    let cleanup_result = crate::handlers::orchestration::cleanup_election(
        State(state.clone()),
        Path(id),
    ).await;

    match cleanup_result {
        Ok(_) => tracing::info!("✅ Container cleanup successful for election: {}", id),
        Err(e) => tracing::warn!("⚠️  Container cleanup encountered issues: {:?}", e),
    }

    // Now delete the election from database (CASCADE will delete related records)
    let result = sqlx::query("DELETE FROM elections WHERE id = $1")
        .bind(id)
        .execute(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if result.rows_affected() == 0 {
        return Err(StatusCode::NOT_FOUND);
    }

    // Log event
    let _ = sqlx::query(
        "INSERT INTO system_events (event_type, entity_type, entity_id) VALUES ($1, $2, $3)"
    )
    .bind("election_deleted")
    .bind("election")
    .bind(id)
    .execute(&state.db)
    .await;

    tracing::info!("🎉 Election deleted successfully: {}", id);

    Ok(StatusCode::NO_CONTENT)
}

pub async fn advance_to_did_phase(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    // Check if election exists and is in phase 6
    let election: crate::models::Election = sqlx::query_as(
        "SELECT * FROM elections WHERE id = $1"
    )
    .bind(&id)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)?;

    if election.phase != 6 {
        tracing::warn!("Cannot advance to DID phase: election {} is in phase {}, expected 6", id, election.phase);
        return Err(StatusCode::BAD_REQUEST);
    }

    // Advance to phase 7
    sqlx::query("UPDATE elections SET phase = 7 WHERE id = $1")
        .bind(&id)
        .execute(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    tracing::info!("✅ Election {} advanced to phase 7 (DID Generation)", id);

    Ok(StatusCode::OK)
}
