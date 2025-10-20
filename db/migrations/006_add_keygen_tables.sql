-- Migration: Add tables for DKG public keys
-- IMPORTANT: Private keys (sgk) are NEVER stored in database!
-- They remain ONLY in trustee containers, encrypted at rest.

-- Master Verification Key (Election Public Key)
-- This is the aggregated public key used to encrypt votes
CREATE TABLE IF NOT EXISTS master_verification_keys (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    election_id UUID NOT NULL UNIQUE REFERENCES elections(id) ON DELETE CASCADE,
    alpha2 TEXT NOT NULL,  -- g2^x (aggregated from all qualified trustees)
    beta2 TEXT NOT NULL,   -- g2^y (aggregated from all qualified trustees)
    beta1 TEXT NOT NULL,   -- g1^y (aggregated from all qualified trustees)
    qualified_trustee_count INTEGER NOT NULL,  -- How many trustees contributed
    threshold INTEGER NOT NULL,  -- Minimum trustees needed for decryption
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_mvk_election ON master_verification_keys(election_id);

-- Trustee Verification Keys (Public keys for each trustee)
-- These allow verification of trustee's partial decryptions
CREATE TABLE IF NOT EXISTS trustee_verification_keys (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    trustee_id UUID NOT NULL UNIQUE REFERENCES trustees(id) ON DELETE CASCADE,
    election_id UUID NOT NULL REFERENCES elections(id) ON DELETE CASCADE,
    vk1 TEXT NOT NULL,  -- g2^x_i (verification key component 1)
    vk2 TEXT NOT NULL,  -- g2^y_i (verification key component 2)
    vk3 TEXT NOT NULL,  -- g1^y_i (verification key component 3)
    is_qualified BOOLEAN DEFAULT TRUE,  -- Was this trustee qualified in DKG?
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_tvk_trustee ON trustee_verification_keys(trustee_id);
CREATE INDEX idx_tvk_election ON trustee_verification_keys(election_id);

-- DKG Session Tracking
-- Tracks the progress of distributed key generation
CREATE TABLE IF NOT EXISTS dkg_sessions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    election_id UUID NOT NULL REFERENCES elections(id) ON DELETE CASCADE,
    status VARCHAR(50) NOT NULL,  -- 'in_progress', 'completed', 'failed'
    current_step INTEGER DEFAULT 0,  -- Which step of DKG (0-7)
    total_trustees INTEGER NOT NULL,
    threshold INTEGER NOT NULL,
    qualified_trustees JSONB,  -- Array of qualified trustee indices
    disqualified_trustees JSONB,  -- Array of disqualified trustee indices with reasons
    started_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    completed_at TIMESTAMP WITH TIME ZONE,
    error_message TEXT,
    UNIQUE(election_id)
);

CREATE INDEX idx_dkg_session_election ON dkg_sessions(election_id);
CREATE INDEX idx_dkg_session_status ON dkg_sessions(status);

-- DKG Trustee Status
-- Tracks individual trustee progress during DKG
CREATE TABLE IF NOT EXISTS dkg_trustee_status (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    session_id UUID NOT NULL REFERENCES dkg_sessions(id) ON DELETE CASCADE,
    trustee_id UUID NOT NULL REFERENCES trustees(id) ON DELETE CASCADE,
    current_step INTEGER DEFAULT 0,
    status VARCHAR(50) NOT NULL,  -- 'pending', 'in_progress', 'completed', 'failed'
    last_heartbeat TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    error_message TEXT,
    UNIQUE(session_id, trustee_id)
);

CREATE INDEX idx_dkg_trustee_status_session ON dkg_trustee_status(session_id);
CREATE INDEX idx_dkg_trustee_status_trustee ON dkg_trustee_status(trustee_id);

-- DKG Complaints
-- Records complaints during share verification
CREATE TABLE IF NOT EXISTS dkg_complaints (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    session_id UUID NOT NULL REFERENCES dkg_sessions(id) ON DELETE CASCADE,
    complainer_trustee_id UUID NOT NULL REFERENCES trustees(id) ON DELETE CASCADE,
    accused_trustee_id UUID NOT NULL REFERENCES trustees(id) ON DELETE CASCADE,
    complaint_type VARCHAR(100) NOT NULL,  -- 'invalid_share', 'invalid_commitment', etc.
    details TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_dkg_complaints_session ON dkg_complaints(session_id);
CREATE INDEX idx_dkg_complaints_accused ON dkg_complaints(accused_trustee_id);

-- System Events for DKG
INSERT INTO system_events (event_type, entity_type, entity_id, data)
SELECT
    'dkg_tables_created',
    'database',
    uuid_generate_v4(),
    jsonb_build_object(
        'migration', '006_add_keygen_tables',
        'timestamp', CURRENT_TIMESTAMP
    )
WHERE NOT EXISTS (
    SELECT 1 FROM system_events
    WHERE event_type = 'dkg_tables_created'
);

-- Comments for clarity
COMMENT ON TABLE master_verification_keys IS 'Election public key (MVK) - aggregated from qualified trustees. Used to encrypt votes.';
COMMENT ON TABLE trustee_verification_keys IS 'Individual trustee public keys (VK_m) - used to verify partial decryptions. PUBLIC DATA ONLY!';
COMMENT ON TABLE dkg_sessions IS 'Tracks DKG protocol execution for each election';
COMMENT ON TABLE dkg_trustee_status IS 'Individual trustee progress during DKG';
COMMENT ON TABLE dkg_complaints IS 'Complaints filed during DKG share verification';

COMMENT ON COLUMN master_verification_keys.alpha2 IS 'g2^x - PUBLIC - aggregated from all qualified trustees';
COMMENT ON COLUMN master_verification_keys.beta2 IS 'g2^y - PUBLIC - aggregated from all qualified trustees';
COMMENT ON COLUMN master_verification_keys.beta1 IS 'g1^y - PUBLIC - aggregated from all qualified trustees';

COMMENT ON COLUMN trustee_verification_keys.vk1 IS 'g2^x_i - PUBLIC - trustee verification key component 1';
COMMENT ON COLUMN trustee_verification_keys.vk2 IS 'g2^y_i - PUBLIC - trustee verification key component 2';
COMMENT ON COLUMN trustee_verification_keys.vk3 IS 'g1^y_i - PUBLIC - trustee verification key component 3';

-- SECURITY NOTE:
-- Private signing keys (sgk1, sgk2) are NEVER stored in this database!
-- They are stored ONLY in the trustee's container at /app/storage/signing_key.enc
-- Each trustee is responsible for their own private key security.
