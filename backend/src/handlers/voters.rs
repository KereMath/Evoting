use axum::{
    extract::{State, Path, Query, Multipart},
    http::StatusCode,
    Json,
};
use std::sync::Arc;
use uuid::Uuid;
use serde::{Deserialize, Serialize};

use crate::{AppState, models::{Voter, CreateVoterRequest}};

#[derive(Deserialize)]
pub struct VoterQuery {
    pub election_id: Option<Uuid>,
}

pub async fn list_voters(
    State(state): State<Arc<AppState>>,
    Query(params): Query<VoterQuery>,
) -> Result<Json<Vec<Voter>>, StatusCode> {
    let voters = if let Some(election_id) = params.election_id {
        sqlx::query_as::<_, Voter>(
            "SELECT * FROM voters WHERE election_id = $1 ORDER BY created_at DESC"
        )
        .bind(election_id)
        .fetch_all(&state.db)
        .await
    } else {
        sqlx::query_as::<_, Voter>(
            "SELECT * FROM voters ORDER BY created_at DESC"
        )
        .fetch_all(&state.db)
        .await
    }
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(voters))
}

pub async fn create_voter(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateVoterRequest>,
) -> Result<Json<Voter>, StatusCode> {
    let voter = sqlx::query_as::<_, Voter>(
        r#"
        INSERT INTO voters (election_id, voter_id, status)
        VALUES ($1, $2, 'registered')
        RETURNING *
        "#
    )
    .bind(&payload.election_id)
    .bind(&payload.tc_id)
    .fetch_one(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Log event
    let _ = sqlx::query(
        "INSERT INTO system_events (event_type, entity_type, entity_id, data) VALUES ($1, $2, $3, $4)"
    )
    .bind("voter_registered")
    .bind("voter")
    .bind(voter.id)
    .bind(serde_json::json!({
        "election_id": payload.election_id,
        "tc_id": &payload.tc_id
    }))
    .execute(&state.db)
    .await;

    // Check if we should advance election phase
    let election_id = payload.election_id;
    let voter_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM voters WHERE election_id = $1"
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
        if voter_count >= 1 && election.phase == 2 {
            // Advance to phase 3 when at least 1 voter is added
            let _ = sqlx::query(
                "UPDATE elections SET phase = 3 WHERE id = $1"
            )
            .bind(&election_id)
            .execute(&state.db)
            .await;
        }
    }

    Ok(Json(voter))
}

pub async fn delete_voter(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    let result = sqlx::query("DELETE FROM voters WHERE id = $1")
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
    .bind("voter_deleted")
    .bind("voter")
    .bind(id)
    .execute(&state.db)
    .await;

    Ok(StatusCode::NO_CONTENT)
}

#[derive(Serialize)]
pub struct CsvUploadResponse {
    pub success: bool,
    pub imported: usize,
    pub failed: usize,
    pub errors: Vec<String>,
}

pub async fn upload_voters_csv(
    State(state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> Result<Json<CsvUploadResponse>, StatusCode> {
    let mut imported = 0;
    let mut failed = 0;
    let mut errors = Vec::new();

    while let Some(field) = multipart.next_field().await.map_err(|_| StatusCode::BAD_REQUEST)? {
        let name = field.name().unwrap_or("").to_string();

        if name == "file" {
            let data = field.bytes().await.map_err(|_| StatusCode::BAD_REQUEST)?;
            let csv_content = String::from_utf8(data.to_vec())
                .map_err(|_| StatusCode::BAD_REQUEST)?;

            let mut reader = csv::Reader::from_reader(csv_content.as_bytes());

            for (line_num, result) in reader.records().enumerate() {
                match result {
                    Ok(record) => {
                        if record.len() < 2 {
                            errors.push(format!("Line {}: Invalid format, expected 2 columns", line_num + 2));
                            failed += 1;
                            continue;
                        }

                        let election_id = match Uuid::parse_str(record.get(0).unwrap_or("")) {
                            Ok(id) => id,
                            Err(_) => {
                                errors.push(format!("Line {}: Invalid election ID", line_num + 2));
                                failed += 1;
                                continue;
                            }
                        };

                        let tc_id = record.get(1).unwrap_or("").to_string();

                        if tc_id.len() != 11 || !tc_id.chars().all(|c| c.is_numeric()) {
                            errors.push(format!("Line {}: Invalid TC ID (must be 11 digits)", line_num + 2));
                            failed += 1;
                            continue;
                        }

                        // Try to insert voter
                        let insert_result = sqlx::query_as::<_, Voter>(
                            "INSERT INTO voters (election_id, voter_id, status) VALUES ($1, $2, 'registered') RETURNING *"
                        )
                        .bind(&election_id)
                        .bind(&tc_id)
                        .fetch_one(&state.db)
                        .await;

                        match insert_result {
                            Ok(voter) => {
                                imported += 1;

                                // Log event
                                let _ = sqlx::query(
                                    "INSERT INTO system_events (event_type, entity_type, entity_id, data) VALUES ($1, $2, $3, $4)"
                                )
                                .bind("voter_registered_csv")
                                .bind("voter")
                                .bind(voter.id)
                                .bind(serde_json::json!({
                                    "election_id": election_id,
                                    "tc_id": &tc_id,
                                    "source": "csv_upload"
                                }))
                                .execute(&state.db)
                                .await;
                            },
                            Err(_) => {
                                errors.push(format!("Line {}: TC ID already registered or election not found", line_num + 2));
                                failed += 1;
                            }
                        }
                    },
                    Err(e) => {
                        errors.push(format!("Line {}: CSV parse error - {}", line_num + 2, e));
                        failed += 1;
                    }
                }
            }
        }
    }

    let error_count = errors.len();
    Ok(Json(CsvUploadResponse {
        success: imported > 0,
        imported,
        failed,
        errors: if error_count > 10 {
            errors.into_iter().take(10).chain(vec![format!("... and {} more errors", error_count - 10)]).collect()
        } else {
            errors
        },
    }))
}

#[derive(Deserialize)]
pub struct DIDCompleteRequest {
    pub tc_id: String,
}

#[derive(Serialize)]
pub struct DIDCompleteResponse {
    pub success: bool,
    pub message: String,
    pub all_completed: bool,
}

/// Mark DID generation as complete for a voter
pub async fn mark_did_complete(
    State(state): State<Arc<AppState>>,
    Path(voter_id): Path<Uuid>,
    Json(payload): Json<DIDCompleteRequest>,
) -> Result<Json<DIDCompleteResponse>, StatusCode> {
    tracing::info!("Marking DID complete for voter {} (TC: {})", voter_id, payload.tc_id);

    // Get voter
    let voter = sqlx::query_as::<_, Voter>(
        "SELECT * FROM voters WHERE id = $1"
    )
    .bind(&voter_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| {
        tracing::error!("Database error: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?
    .ok_or(StatusCode::NOT_FOUND)?;

    // Verify TC ID matches
    if voter.voter_id != payload.tc_id {
        tracing::warn!("TC ID mismatch for voter {}: expected {}, got {}", voter_id, voter.voter_id, payload.tc_id);
        return Ok(Json(DIDCompleteResponse {
            success: false,
            message: "TC ID does not match".to_string(),
            all_completed: false,
        }));
    }

    // Check if already completed
    if voter.did_generated {
        return Ok(Json(DIDCompleteResponse {
            success: true,
            message: "DID already generated".to_string(),
            all_completed: false,
        }));
    }

    // Mark as complete
    sqlx::query(
        "UPDATE voters SET did_generated = TRUE WHERE id = $1"
    )
    .bind(&voter_id)
    .execute(&state.db)
    .await
    .map_err(|e| {
        tracing::error!("Failed to update voter: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    tracing::info!("✅ DID marked complete for TC: {}", payload.tc_id);

    // Check if all voters in this election have completed DID generation
    let total_voters: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM voters WHERE election_id = $1"
    )
    .bind(&voter.election_id)
    .fetch_one(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let completed_voters: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM voters WHERE election_id = $1 AND did_generated = TRUE"
    )
    .bind(&voter.election_id)
    .fetch_one(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let all_completed = completed_voters >= total_voters;

    if all_completed {
        tracing::info!("🎉 All voters completed DID generation for election {}", voter.election_id);

        // Update election phase to 8 (from 7 to 8 after DID generation)
        sqlx::query(
            "UPDATE elections SET phase = 8 WHERE id = $1 AND phase = 7"
        )
        .bind(&voter.election_id)
        .execute(&state.db)
        .await
        .map_err(|e| {
            tracing::error!("Failed to update election phase: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

        tracing::info!("✅ Election {} advanced to phase 8 (DID generation complete)", voter.election_id);
    } else {
        tracing::info!("DID generation progress: {}/{} voters completed", completed_voters, total_voters);
    }

    Ok(Json(DIDCompleteResponse {
        success: true,
        message: format!("DID generation marked complete ({}/{})", completed_voters, total_voters),
        all_completed,
    }))
}

#[derive(Serialize)]
pub struct VoterStatusResponse {
    pub voter_id: Uuid,
    pub election_id: Uuid,
    pub did_generated: bool,
    pub has_voted: bool,
    pub status: String,
    pub total_voters: i64,
    pub completed_voters: i64,
}

/// Get voter status (for checking DID generation status without localStorage)
pub async fn get_voter_status(
    State(state): State<Arc<AppState>>,
    Path(voter_id): Path<Uuid>,
) -> Result<Json<VoterStatusResponse>, StatusCode> {
    // Get voter
    let voter = sqlx::query_as::<_, Voter>(
        "SELECT * FROM voters WHERE id = $1"
    )
    .bind(&voter_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| {
        tracing::error!("Database error: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?
    .ok_or(StatusCode::NOT_FOUND)?;

    // Get total and completed voters count for this election
    let total_voters: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM voters WHERE election_id = $1"
    )
    .bind(&voter.election_id)
    .fetch_one(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let completed_voters: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM voters WHERE election_id = $1 AND did_generated = TRUE"
    )
    .bind(&voter.election_id)
    .fetch_one(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(VoterStatusResponse {
        voter_id: voter.id,
        election_id: voter.election_id,
        did_generated: voter.did_generated,
        has_voted: voter.voted_at.is_some(),
        status: voter.status,
        total_voters,
        completed_voters,
    }))
}

/// Mark voter's PrepareBlindSign as complete
pub async fn mark_prepare_blindsign_complete(
    State(state): State<Arc<AppState>>,
    Path(voter_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Update voter
    let voter = sqlx::query_as::<_, Voter>(
        "UPDATE voters SET prepare_blindsign_done = TRUE WHERE id = $1 RETURNING *"
    )
    .bind(&voter_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| {
        tracing::error!("Database error: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?
    .ok_or(StatusCode::NOT_FOUND)?;

    tracing::info!("Voter {} completed PrepareBlindSign", voter_id);

    // Check if all voters have completed PrepareBlindSign
    let total_voters: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM voters WHERE election_id = $1"
    )
    .bind(&voter.election_id)
    .fetch_one(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let completed_voters: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM voters WHERE election_id = $1 AND prepare_blindsign_done = TRUE"
    )
    .bind(&voter.election_id)
    .fetch_one(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // If all voters completed, advance election to phase 8
    if completed_voters >= total_voters && total_voters > 0 {
        let _ = sqlx::query(
            "UPDATE elections SET phase = 8, status = 'credential_issuance' WHERE id = $1"
        )
        .bind(&voter.election_id)
        .execute(&state.db)
        .await;

        tracing::info!("Election {} advanced to phase 8 (PrepareBlindSign complete)", voter.election_id);
    }

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "PrepareBlindSign marked as complete",
        "all_completed": completed_voters >= total_voters,
        "completed": completed_voters,
        "total": total_voters
    })))
}
