use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;
use reqwest;

use crate::AppState;

#[derive(Debug, Serialize, Deserialize)]
pub struct KeygenStartRequest {
    pub threshold: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct KeygenStartResponse {
    pub session_id: Uuid,
    pub election_id: Uuid,
    pub total_trustees: i32,
    pub threshold: i32,
    pub status: String,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TrusteeReadyRequest {
    pub trustee_id: Uuid,
    pub vk1: String,  // Public verification key component 1
    pub vk2: String,  // Public verification key component 2
    pub vk3: String,  // Public verification key component 3
    pub mvk: MasterVerificationKey,  // Calculated MVK (should match others)
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MasterVerificationKey {
    pub alpha2: String,
    pub beta2: String,
    pub beta1: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TrusteeReadyResponse {
    pub success: bool,
    pub message: String,
    pub trustees_ready: i32,
    pub total_trustees: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct KeygenStatusResponse {
    pub session_id: Uuid,
    pub election_id: Uuid,
    pub status: String,  // 'in_progress', 'completed', 'failed'
    pub current_step: i32,
    pub total_trustees: i32,
    pub threshold: i32,
    pub trustees_ready: Vec<TrusteeStatus>,
    pub mvk: Option<MasterVerificationKey>,
    pub qualified_trustees: Option<Vec<i32>>,
    pub disqualified_trustees: Option<Vec<i32>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TrusteeStatus {
    pub trustee_id: Uuid,
    pub trustee_index: i32,
    pub name: String,
    pub status: String,
    pub current_step: i32,
    pub last_heartbeat: Option<String>,
    pub verification_key: Option<String>,  // VK for completed trustees
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PeerInfo {
    pub id: String,
    pub index: i32,
    pub name: String,
    pub hostname: String,
    pub port: i32,
}

/// Start distributed key generation process
pub async fn start_keygen(
    State(state): State<Arc<AppState>>,
    Path(election_id): Path<Uuid>,
    Json(payload): Json<KeygenStartRequest>,
) -> Result<Json<KeygenStartResponse>, StatusCode> {
    tracing::info!("🔐 Starting keygen for election: {}", election_id);

    // Get election details
    let election: crate::models::Election = sqlx::query_as(
        "SELECT * FROM elections WHERE id = $1"
    )
    .bind(&election_id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| {
        tracing::error!("Election not found: {}", e);
        StatusCode::NOT_FOUND
    })?;

    // Connect backend container to election network
    if let Some(ref network_name) = election.docker_network {
        tracing::info!("🔗 Connecting backend to election network: {}", network_name);

        use bollard::Docker;
        use bollard::network::ConnectNetworkOptions;

        let docker = Docker::connect_with_socket_defaults()
            .map_err(|e| {
                tracing::error!("Failed to connect to Docker: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

        // Get our own container ID from environment or hostname
        let container_id = std::env::var("HOSTNAME")
            .unwrap_or_else(|_| "evoting-main-server".to_string());

        let connect_opts = ConnectNetworkOptions {
            container: container_id.clone(),
            ..Default::default()
        };

        match docker.connect_network(network_name, connect_opts).await {
            Ok(_) => tracing::info!("✅ Backend connected to election network"),
            Err(e) => {
                // Check if already connected (not an error)
                let err_msg = e.to_string();
                if err_msg.contains("already exists") || err_msg.contains("already attached") {
                    tracing::info!("ℹ️  Backend already connected to election network");
                } else {
                    tracing::warn!("⚠️  Failed to connect to network: {}", e);
                }
            }
        }
    } else {
        tracing::warn!("⚠️  No docker network found for election");
    }

    // Validate election status
    if election.phase < 5 {
        return Err(StatusCode::BAD_REQUEST);
    }

    let threshold = payload.threshold.unwrap_or(election.threshold);

    // Get all trustees for this election
    let trustees: Vec<crate::models::Trustee> = sqlx::query_as(
        "SELECT * FROM trustees WHERE election_id = $1 AND docker_type = 'auto' ORDER BY id"
    )
    .bind(&election_id)
    .fetch_all(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if trustees.is_empty() {
        tracing::error!("No trustees found for election");
        return Err(StatusCode::BAD_REQUEST);
    }

    tracing::info!("Found {} trustees for keygen", trustees.len());

    // Check if a DKG session already exists for this election
    let existing_session: Option<(Uuid, String)> = sqlx::query_as(
        "SELECT id, status FROM dkg_sessions WHERE election_id = $1"
    )
    .bind(&election_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let session_id = if let Some((existing_id, existing_status)) = existing_session {
        tracing::warn!("⚠️  DKG session already exists for this election (status: {})", existing_status);

        // If session is in progress, reject to prevent duplicate start
        if existing_status == "in_progress" {
            tracing::error!("❌ DKG is already in progress for this election");
            return Ok(Json(KeygenStartResponse {
                session_id: existing_id,
                election_id,
                total_trustees: trustees.len() as i32,
                threshold,
                status: existing_status,
                message: "DKG is already in progress. Please wait for it to complete.".to_string(),
            }));
        }

        // If previous session failed, allow restart
        if existing_status == "failed" {
            tracing::info!("🔄 Restarting failed DKG session");

            // Delete old session and its related data
            sqlx::query("DELETE FROM dkg_trustee_status WHERE session_id = $1")
                .bind(&existing_id)
                .execute(&state.db)
                .await
                .ok();

            sqlx::query("DELETE FROM dkg_sessions WHERE id = $1")
                .bind(&existing_id)
                .execute(&state.db)
                .await
                .ok();

            // Create new session
            let new_session_id = Uuid::new_v4();
            sqlx::query(
                "INSERT INTO dkg_sessions (id, election_id, status, current_step, total_trustees, threshold)
                 VALUES ($1, $2, 'in_progress', 0, $3, $4)"
            )
            .bind(&new_session_id)
            .bind(&election_id)
            .bind(trustees.len() as i32)
            .bind(threshold)
            .execute(&state.db)
            .await
            .map_err(|e| {
                tracing::error!("Failed to create new DKG session: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

            new_session_id
        } else {
            // Session is completed, return the existing session
            tracing::info!("✅ DKG already completed for this election");
            return Ok(Json(KeygenStartResponse {
                session_id: existing_id,
                election_id,
                total_trustees: trustees.len() as i32,
                threshold,
                status: existing_status,
                message: "DKG already completed for this election".to_string(),
            }));
        }
    } else {
        // Create new DKG session
        let new_session_id = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO dkg_sessions (id, election_id, status, current_step, total_trustees, threshold)
             VALUES ($1, $2, 'in_progress', 0, $3, $4)"
        )
        .bind(&new_session_id)
        .bind(&election_id)
        .bind(trustees.len() as i32)
        .bind(threshold)
        .execute(&state.db)
        .await
        .map_err(|e| {
            tracing::error!("Failed to create DKG session: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

        new_session_id
    };

    // Create status entries for each trustee
    for trustee in &trustees {
        sqlx::query(
            "INSERT INTO dkg_trustee_status (session_id, trustee_id, current_step, status)
             VALUES ($1, $2, 0, 'pending')"
        )
        .bind(&session_id)
        .bind(&trustee.id)
        .execute(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }

    // Build peer list for each trustee - use IP addresses instead of hostnames for DNS reliability
    let peers: Vec<PeerInfo> = trustees.iter().enumerate().map(|(idx, t)| {
        PeerInfo {
            id: t.id.to_string(),
            index: (idx + 1) as i32,
            name: t.name.clone(),
            hostname: t.ip_address.clone().unwrap_or_else(|| format!("trustee-{}-{}", election_id, t.id)),
            port: 8000,  // DKG server port (internal, not exposed)
        }
    }).collect();

    // Get crypto parameters from database
    let crypto_params: Option<(String, String, String, String, String)> = sqlx::query_as(
        "SELECT prime_order, g1, g2, h1, pairing_params FROM crypto_parameters WHERE election_id = $1"
    )
    .bind(&election_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if crypto_params.is_none() {
        tracing::error!("❌ No crypto parameters found for election {}. Run crypto-setup first!", election_id);
        return Err(StatusCode::BAD_REQUEST);
    }

    let (prime_order, g1, g2, h1, pairing_params) = crypto_params.unwrap();
    tracing::info!("✅ Loaded crypto parameters from database");

    tracing::info!("Sending DKG start signal to {} trustees", trustees.len());

    // Send start signal to all trustee containers
    let client = reqwest::Client::new();
    let mut success_count = 0;

    for (idx, trustee) in trustees.iter().enumerate() {
        // Use IP address for direct communication, avoiding DNS resolution issues
        let container_host = trustee.ip_address.as_ref()
            .map(|ip| ip.as_str())
            .unwrap_or_else(|| {
                tracing::warn!("No IP address for trustee {}, using hostname", trustee.name);
                "unknown"
            });

        let container_url = format!("http://{}:8000/dkg/start", container_host);

        let start_payload = serde_json::json!({
            "session_id": session_id.to_string(),
            "election_id": election_id.to_string(),
            "my_index": idx + 1,
            "threshold": threshold,
            "total_trustees": trustees.len(),
            "peers": peers,
            "crypto_params": {
                "prime_order": prime_order,
                "g1": g1,
                "g2": g2,
                "h1": h1,
                "pairing_params": pairing_params
            }
        });

        tracing::info!("Sending DKG start to trustee {} at {}", trustee.name, container_url);

        match client.post(&container_url)
            .json(&start_payload)
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await
        {
            Ok(response) => {
                if response.status().is_success() {
                    tracing::info!("✅ Trustee {} accepted DKG start", trustee.name);
                    success_count += 1;
                } else {
                    tracing::warn!("⚠️  Trustee {} returned error: {}", trustee.name, response.status());
                }
            }
            Err(e) => {
                tracing::error!("❌ Failed to contact trustee {}: {}", trustee.name, e);
            }
        }
    }

    tracing::info!("DKG start signal sent: {}/{} trustees responded", success_count, trustees.len());

    // Update election phase to key_generation
    sqlx::query("UPDATE elections SET phase = 6, status = 'key_generation' WHERE id = $1")
        .bind(&election_id)
        .execute(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Log event
    let _ = sqlx::query(
        "INSERT INTO system_events (event_type, entity_type, entity_id, data)
         VALUES ($1, $2, $3, $4)"
    )
    .bind("keygen_started")
    .bind("election")
    .bind(&election_id)
    .bind(serde_json::json!({
        "session_id": session_id,
        "total_trustees": trustees.len(),
        "threshold": threshold,
        "trustees_contacted": success_count
    }))
    .execute(&state.db)
    .await;

    Ok(Json(KeygenStartResponse {
        session_id,
        election_id,
        total_trustees: trustees.len() as i32,
        threshold,
        status: "in_progress".to_string(),
        message: format!("DKG started with {} trustees (threshold: {})", trustees.len(), threshold),
    }))
}

/// Trustee reports completion and submits public keys
pub async fn trustee_ready(
    State(state): State<Arc<AppState>>,
    Path(election_id): Path<Uuid>,
    Json(payload): Json<TrusteeReadyRequest>,
) -> Result<Json<TrusteeReadyResponse>, StatusCode> {
    tracing::info!(
        "📝 Trustee {} reports ready for election {}",
        payload.trustee_id,
        election_id
    );

    // Get DKG session
    let session: (Uuid, i32) = sqlx::query_as(
        "SELECT id, total_trustees FROM dkg_sessions WHERE election_id = $1"
    )
    .bind(&election_id)
    .fetch_one(&state.db)
    .await
    .map_err(|_| StatusCode::NOT_FOUND)?;

    let session_id = session.0;
    let total_trustees = session.1;

    // Store trustee's verification keys (PUBLIC ONLY!)
    sqlx::query(
        "INSERT INTO trustee_verification_keys (trustee_id, election_id, vk1, vk2, vk3, is_qualified)
         VALUES ($1, $2, $3, $4, $5, true)
         ON CONFLICT (trustee_id) DO UPDATE SET vk1 = $3, vk2 = $4, vk3 = $5"
    )
    .bind(&payload.trustee_id)
    .bind(&election_id)
    .bind(&payload.vk1)
    .bind(&payload.vk2)
    .bind(&payload.vk3)
    .execute(&state.db)
    .await
    .map_err(|e| {
        tracing::error!("Failed to store verification keys: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Update trustee status
    sqlx::query(
        "UPDATE dkg_trustee_status
         SET status = 'completed', current_step = 7, last_heartbeat = CURRENT_TIMESTAMP
         WHERE session_id = $1 AND trustee_id = $2"
    )
    .bind(&session_id)
    .bind(&payload.trustee_id)
    .execute(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    tracing::info!("✅ Stored public keys for trustee {}", payload.trustee_id);

    // Check how many trustees are ready
    let trustees_ready: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM dkg_trustee_status
         WHERE session_id = $1 AND status = 'completed'"
    )
    .bind(&session_id)
    .fetch_one(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    tracing::info!("Progress: {}/{} trustees ready", trustees_ready, total_trustees);

    // If all trustees are ready, finalize DKG
    if trustees_ready as i32 >= total_trustees {
        tracing::info!("🎉 All trustees ready! Finalizing DKG...");
        finalize_keygen(&state.db, &session_id, &election_id, &payload.mvk).await?;
    }

    Ok(Json(TrusteeReadyResponse {
        success: true,
        message: "Verification keys stored successfully".to_string(),
        trustees_ready: trustees_ready as i32,
        total_trustees,
    }))
}

/// Finalize keygen when all trustees are ready
async fn finalize_keygen(
    db: &PgPool,
    session_id: &Uuid,
    election_id: &Uuid,
    mvk: &MasterVerificationKey,
) -> Result<(), StatusCode> {
    tracing::info!("🔐 Finalizing keygen for election {}", election_id);

    // Get session details
    let (total_trustees, threshold): (i32, i32) = sqlx::query_as(
        "SELECT total_trustees, threshold FROM dkg_sessions WHERE id = $1"
    )
    .bind(session_id)
    .fetch_one(db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Store Master Verification Key (PUBLIC!)
    sqlx::query(
        "INSERT INTO master_verification_keys
         (election_id, alpha2, beta2, beta1, qualified_trustee_count, threshold)
         VALUES ($1, $2, $3, $4, $5, $6)
         ON CONFLICT (election_id) DO UPDATE
         SET alpha2 = $2, beta2 = $3, beta1 = $4, qualified_trustee_count = $5, threshold = $6"
    )
    .bind(election_id)
    .bind(&mvk.alpha2)
    .bind(&mvk.beta2)
    .bind(&mvk.beta1)
    .bind(total_trustees)
    .bind(threshold)
    .execute(db)
    .await
    .map_err(|e| {
        tracing::error!("Failed to store MVK: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Update DKG session status
    sqlx::query(
        "UPDATE dkg_sessions
         SET status = 'completed', current_step = 7, completed_at = CURRENT_TIMESTAMP
         WHERE id = $1"
    )
    .bind(session_id)
    .execute(db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Update election phase
    sqlx::query("UPDATE elections SET phase = 7 WHERE id = $1")
        .bind(election_id)
        .execute(db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Log event
    let _ = sqlx::query(
        "INSERT INTO system_events (event_type, entity_type, entity_id, data)
         VALUES ($1, $2, $3, $4)"
    )
    .bind("keygen_completed")
    .bind("election")
    .bind(election_id)
    .bind(serde_json::json!({
        "session_id": session_id,
        "mvk": mvk,
        "qualified_trustee_count": total_trustees,
        "threshold": threshold
    }))
    .execute(db)
    .await;

    tracing::info!("✅ Keygen finalized! Election moved to phase 7");

    Ok(())
}

/// Get keygen status
pub async fn get_keygen_status(
    State(state): State<Arc<AppState>>,
    Path(election_id): Path<Uuid>,
) -> Result<Json<KeygenStatusResponse>, StatusCode> {
    // Get DKG session
    let session: (Uuid, String, i32, i32, i32) = sqlx::query_as(
        "SELECT id, status, current_step, total_trustees, threshold
         FROM dkg_sessions WHERE election_id = $1"
    )
    .bind(&election_id)
    .fetch_one(&state.db)
    .await
    .map_err(|_| StatusCode::NOT_FOUND)?;

    let session_id = session.0;
    let status = session.1;
    let current_step = session.2;
    let total_trustees = session.3;
    let threshold = session.4;

    // Get trustee statuses with verification keys
    let trustee_statuses: Vec<(Uuid, String, i32, Option<String>, Option<String>, Option<String>, Option<String>)> = sqlx::query_as(
        "SELECT t.id, t.name, dts.current_step, dts.last_heartbeat::text, tvk.vk1, tvk.vk2, tvk.vk3
         FROM dkg_trustee_status dts
         JOIN trustees t ON dts.trustee_id = t.id
         LEFT JOIN trustee_verification_keys tvk ON tvk.trustee_id = t.id AND tvk.election_id = $2
         WHERE dts.session_id = $1
         ORDER BY t.id"
    )
    .bind(&session_id)
    .bind(&election_id)
    .fetch_all(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let trustees_ready: Vec<TrusteeStatus> = trustee_statuses
        .iter()
        .enumerate()
        .map(|(idx, ts)| {
            // Combine vk1, vk2, vk3 into JSON string if they exist
            let verification_key = if let (Some(vk1), Some(vk2), Some(vk3)) = (&ts.4, &ts.5, &ts.6) {
                Some(serde_json::json!({
                    "vk1": vk1,
                    "vk2": vk2,
                    "vk3": vk3
                }).to_string())
            } else {
                None
            };

            TrusteeStatus {
                trustee_id: ts.0,
                trustee_index: (idx + 1) as i32,
                name: ts.1.clone(),
                status: if ts.2 >= 7 { "completed".to_string() } else { "in_progress".to_string() },
                current_step: ts.2,
                last_heartbeat: ts.3.clone(),
                verification_key,
            }
        })
        .collect();

    // Get MVK if completed
    let mvk: Option<MasterVerificationKey> = if status == "completed" {
        sqlx::query_as::<_, (String, String, String)>(
            "SELECT alpha2, beta2, beta1 FROM master_verification_keys WHERE election_id = $1"
        )
        .bind(&election_id)
        .fetch_optional(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .map(|(alpha2, beta2, beta1)| MasterVerificationKey { alpha2, beta2, beta1 })
    } else {
        None
    };

    Ok(Json(KeygenStatusResponse {
        session_id,
        election_id,
        status,
        current_step,
        total_trustees,
        threshold,
        trustees_ready,
        mvk,
        qualified_trustees: None,
        disqualified_trustees: None,
    }))
}

#[derive(Debug, Deserialize)]
pub struct ProgressReportRequest {
    pub trustee_id: Uuid,
    pub session_id: Uuid,
    pub current_step: i32,
    pub status: String,
    pub mvk: Option<serde_json::Value>,
    pub verification_key: Option<serde_json::Value>,
}

pub async fn report_progress(
    Path(election_id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    Json(req): Json<ProgressReportRequest>,
) -> Result<StatusCode, StatusCode> {
    tracing::info!(
        "📊 Progress report from trustee {} for election {}: Step {}, Status: {}",
        req.trustee_id,
        election_id,
        req.current_step,
        req.status
    );

    // Update trustee status in database
    sqlx::query(
        "UPDATE dkg_trustee_status SET current_step = $1, status = $2, last_heartbeat = NOW()
         WHERE session_id = $3 AND trustee_id = $4"
    )
    .bind(req.current_step)
    .bind(&req.status)
    .bind(&req.session_id)
    .bind(&req.trustee_id)
    .execute(&state.db)
    .await
    .map_err(|e| {
        tracing::error!("Failed to update trustee status: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    tracing::info!("✅ Updated trustee {} status to step {}, status: {}", req.trustee_id, req.current_step, req.status);

    // If trustee has MVK, save it (first trustee to complete will save it)
    if let Some(mvk_value) = &req.mvk {
        if let Ok(mvk) = serde_json::from_value::<MasterVerificationKey>(mvk_value.clone()) {
            // Check if MVK already exists
            let existing_mvk: Option<(String,)> = sqlx::query_as(
                "SELECT alpha2 FROM master_verification_keys WHERE election_id = $1"
            )
            .bind(&election_id)
            .fetch_optional(&state.db)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

            if existing_mvk.is_none() {
                // Get election threshold from database
                let election_threshold: (i32,) = sqlx::query_as(
                    "SELECT threshold FROM elections WHERE id = $1"
                )
                .bind(&election_id)
                .fetch_one(&state.db)
                .await
                .map_err(|e| {
                    tracing::error!("Failed to get election threshold: {}", e);
                    StatusCode::INTERNAL_SERVER_ERROR
                })?;

                // Count qualified trustees (those who completed DKG)
                let qualified_count: (i64,) = sqlx::query_as(
                    "SELECT COUNT(*) FROM dkg_trustee_status
                     WHERE session_id = $1 AND status = 'completed'"
                )
                .bind(&req.session_id)
                .fetch_one(&state.db)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

                // Insert MVK
                sqlx::query(
                    "INSERT INTO master_verification_keys (election_id, alpha2, beta2, beta1, qualified_trustee_count, threshold)
                     VALUES ($1, $2, $3, $4, $5, $6)
                     ON CONFLICT (election_id) DO NOTHING"
                )
                .bind(&election_id)
                .bind(&mvk.alpha2)
                .bind(&mvk.beta2)
                .bind(&mvk.beta1)
                .bind(qualified_count.0 as i32)
                .bind(election_threshold.0)
                .execute(&state.db)
                .await
                .map_err(|e| {
                    tracing::error!("Failed to save MVK: {}", e);
                    StatusCode::INTERNAL_SERVER_ERROR
                })?;

                tracing::info!("✅ Saved MVK for election {} (threshold: {}, qualified: {})",
                    election_id, election_threshold.0, qualified_count.0);
            }
        }
    }

    // If trustee has verification key, save it
    if let Some(vk_value) = &req.verification_key {
        // Try to parse as JSON object with vk1, vk2, vk3 fields
        if let Some(vk_obj) = vk_value.as_object() {
            let vk1 = vk_obj.get("vk1").and_then(|v| v.as_str()).unwrap_or("");
            let vk2 = vk_obj.get("vk2").and_then(|v| v.as_str()).unwrap_or("");
            let vk3 = vk_obj.get("vk3").and_then(|v| v.as_str()).unwrap_or("");

            // Save trustee verification key with separate vk1, vk2, vk3 columns
            sqlx::query(
                "INSERT INTO trustee_verification_keys (trustee_id, election_id, vk1, vk2, vk3, is_qualified)
                 VALUES ($1, $2, $3, $4, $5, true)
                 ON CONFLICT (trustee_id) DO UPDATE SET vk1 = $3, vk2 = $4, vk3 = $5, election_id = $2"
            )
            .bind(&req.trustee_id)
            .bind(&election_id)
            .bind(vk1)
            .bind(vk2)
            .bind(vk3)
            .execute(&state.db)
            .await
            .map_err(|e| {
                tracing::error!("Failed to save VK for trustee {}: {}", req.trustee_id, e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

            tracing::info!("✅ Saved VK for trustee {} (vk1: {}, vk2: {}, vk3: {})",
                req.trustee_id, &vk1[..20.min(vk1.len())], &vk2[..20.min(vk2.len())], &vk3[..20.min(vk3.len())]);
        }
    }

    tracing::info!("🔵 About to run completion check for session {}", req.session_id);

    // Always check if all trustees completed (regardless of current request status)
    let completed_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM dkg_trustee_status
         WHERE session_id = $1 AND status = 'completed'"
    )
    .bind(&req.session_id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| {
        tracing::error!("Failed to get completed count: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let total_trustees: i32 = sqlx::query_scalar(
        "SELECT total_trustees FROM dkg_sessions WHERE id = $1"
    )
    .bind(&req.session_id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| {
        tracing::error!("Failed to get total trustees: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    tracing::info!("🔍 DKG Completion Check: {}/{} trustees completed", completed_count, total_trustees as i64);

    // If all trustees completed, update session status
    if completed_count >= total_trustees as i64 {
        // Check if session is already completed to avoid duplicate updates
        let session_status: (String,) = sqlx::query_as(
            "SELECT status FROM dkg_sessions WHERE id = $1"
        )
        .bind(&req.session_id)
        .fetch_one(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        if session_status.0 != "completed" {
            tracing::info!("✅ All trustees completed DKG! Updating session status...");

            sqlx::query(
                "UPDATE dkg_sessions SET status = 'completed', current_step = 7, completed_at = NOW()
                 WHERE id = $1"
            )
            .bind(&req.session_id)
            .execute(&state.db)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

            // Update election status to active (DKG completed, ready for voting)
            sqlx::query(
                "UPDATE elections SET status = 'active', phase = 7 WHERE id = $1"
            )
            .bind(&election_id)
            .execute(&state.db)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

            tracing::info!("🎉 Session marked as completed! Election moved to phase 7 (active)");
        }
    }

    Ok(StatusCode::OK)
}
