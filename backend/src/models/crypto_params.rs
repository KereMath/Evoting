use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

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
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
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
