use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Election {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub threshold: i32,
    pub total_trustees: i32,
    pub status: String,
    pub phase: i32,
    pub docker_network: Option<String>,
    pub ttp_port: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub ended_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Trustee {
    pub id: Uuid,
    pub election_id: Uuid,
    pub name: String,
    pub public_key: Option<String>,
    pub status: String,
    pub docker_type: String,
    pub ip_address: Option<String>,
    pub docker_port: Option<i32>,
    pub container_id: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Voter {
    pub id: Uuid,
    pub election_id: Uuid,
    pub voter_id: String,
    // pub did: Option<String>,  ← REMOVED! DID NEVER stored on server
    pub did_generated: bool,  // Only flag: has voter completed DID generation?
    pub prepare_blindsign_done: bool,  // Has voter completed PrepareBlindSign?
    pub status: String,
    pub docker_port: Option<i32>,
    pub container_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub voted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateElectionRequest {
    pub name: String,
    pub description: Option<String>,
    pub threshold: i32,
    pub total_trustees: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateTrusteeRequest {
    pub election_id: Uuid,
    pub name: String,
    pub docker_type: String,  // "auto" or "manual"
    pub ip_address: Option<String>,  // Only for manual type
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateVoterRequest {
    pub election_id: Uuid,
    pub tc_id: String,  // TC Kimlik Numarası
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct SystemEvent {
    pub id: Uuid,
    pub event_type: String,
    pub entity_type: Option<String>,
    pub entity_id: Option<Uuid>,
    pub data: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct CryptoParameters {
    pub id: Uuid,
    pub election_id: Uuid,

    // Bilinear group parameters
    pub prime_order: String,        // p (prime number as hex string)
    pub g1: String,                 // G1 generator (serialized)
    pub g2: String,                 // G2 generator (serialized)
    pub h1: String,                 // Second G1 generator (serialized)

    // Master verification key (aggregated from trustees)
    pub mvk_alpha2: Option<String>, // Master verification key alpha (G2)
    pub mvk_beta1: Option<String>,  // Master verification key beta (G1)
    pub mvk_beta2: Option<String>,  // Master verification key beta (G2)

    // Pairing parameters
    pub pairing_params: String,     // PBC pairing params (serialized)

    // Metadata
    pub security_level: i32,        // λ security parameter (bits)
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CryptoSetupRequest {
    pub security_level: Option<i32>, // Default: 256
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CryptoSetupResponse {
    pub success: bool,
    pub message: String,
    pub parameters: Option<CryptoParametersPublic>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CryptoParametersPublic {
    pub election_id: String,
    pub prime_order: String,
    pub g1: String,
    pub g2: String,
    pub h1: String,
    pub pairing_params: String,
    pub security_level: i32,
}

impl From<CryptoParameters> for CryptoParametersPublic {
    fn from(params: CryptoParameters) -> Self {
        Self {
            election_id: params.election_id.to_string(),
            prime_order: params.prime_order,
            g1: params.g1,
            g2: params.g2,
            h1: params.h1,
            pairing_params: params.pairing_params,
            security_level: params.security_level,
        }
    }
}
