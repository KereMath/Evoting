use axum::{
    extract::{State, Query},
    http::StatusCode,
    Json,
};
use std::sync::Arc;
use serde::Deserialize;
use uuid::Uuid;

use crate::{AppState, models::SystemEvent};

#[derive(Deserialize)]
pub struct EventQuery {
    pub event_type: Option<String>,
    pub entity_id: Option<Uuid>,
    pub limit: Option<i64>,
}

pub async fn list_events(
    State(state): State<Arc<AppState>>,
    Query(params): Query<EventQuery>,
) -> Result<Json<Vec<SystemEvent>>, StatusCode> {
    let limit = params.limit.unwrap_or(100).min(1000); // Max 1000 events

    let events = if let Some(entity_id) = params.entity_id {
        sqlx::query_as::<_, SystemEvent>(
            "SELECT * FROM system_events WHERE entity_id = $1 ORDER BY created_at DESC LIMIT $2"
        )
        .bind(entity_id)
        .bind(limit)
        .fetch_all(&state.db)
        .await
    } else if let Some(event_type) = params.event_type {
        sqlx::query_as::<_, SystemEvent>(
            "SELECT * FROM system_events WHERE event_type = $1 ORDER BY created_at DESC LIMIT $2"
        )
        .bind(event_type)
        .bind(limit)
        .fetch_all(&state.db)
        .await
    } else {
        sqlx::query_as::<_, SystemEvent>(
            "SELECT * FROM system_events ORDER BY created_at DESC LIMIT $1"
        )
        .bind(limit)
        .fetch_all(&state.db)
        .await
    }
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(events))
}
