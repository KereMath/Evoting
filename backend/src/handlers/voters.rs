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
