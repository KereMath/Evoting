-- E-Voting Database Schema
-- This script initializes the database for the e-voting system

-- Enable UUID extension
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Elections table
CREATE TABLE IF NOT EXISTS elections (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(255) NOT NULL,
    description TEXT,
    threshold INTEGER NOT NULL,
    total_trustees INTEGER NOT NULL,
    status VARCHAR(50) NOT NULL DEFAULT 'setup',
    phase INTEGER NOT NULL DEFAULT 1,
    docker_network VARCHAR(255),
    ttp_port INTEGER,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    started_at TIMESTAMP WITH TIME ZONE,
    ended_at TIMESTAMP WITH TIME ZONE,

    CONSTRAINT valid_threshold CHECK (threshold > 0 AND threshold <= total_trustees),
    CONSTRAINT valid_status CHECK (status IN ('setup', 'key_generation', 'active', 'voting', 'tallying', 'completed', 'cancelled', 'credential_issuance')),
    CONSTRAINT valid_phase CHECK (phase >= 1 AND phase <= 10)
);

-- Trustees (Election Authorities) table
CREATE TABLE IF NOT EXISTS trustees (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    election_id UUID NOT NULL REFERENCES elections(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    public_key TEXT,
    verification_key TEXT,
    status VARCHAR(50) NOT NULL DEFAULT 'pending',
    docker_type VARCHAR(20) NOT NULL DEFAULT 'auto',
    ip_address VARCHAR(50),
    docker_port INTEGER,
    container_id VARCHAR(255),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,

    CONSTRAINT valid_trustee_status CHECK (status IN ('pending', 'active', 'inactive', 'removed')),
    CONSTRAINT valid_docker_type CHECK (docker_type IN ('auto', 'manual'))
);

-- Voters table
CREATE TABLE IF NOT EXISTS voters (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    election_id UUID NOT NULL REFERENCES elections(id) ON DELETE CASCADE,
    voter_id VARCHAR(255) NOT NULL,
    -- did TEXT,  ← REMOVED! DID never stored on server
    did_generated BOOLEAN DEFAULT FALSE,  -- Only flag: has voter completed DID generation?
    prepare_blindsign_done BOOLEAN DEFAULT FALSE,  -- Has voter completed PrepareBlindSign?
    credential TEXT,  -- Blind signature credential (after unblinding)
    status VARCHAR(50) NOT NULL DEFAULT 'registered',
    docker_port INTEGER,
    container_id VARCHAR(255),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    voted_at TIMESTAMP WITH TIME ZONE,

    CONSTRAINT valid_voter_status CHECK (status IN ('registered', 'credential_issued', 'voted', 'revoked')),
    CONSTRAINT unique_voter_per_election UNIQUE (election_id, voter_id)
);

-- Blind signatures table (for tracking the signature process)
CREATE TABLE IF NOT EXISTS blind_signatures (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    voter_id UUID NOT NULL REFERENCES voters(id) ON DELETE CASCADE,
    trustee_id UUID NOT NULL REFERENCES trustees(id) ON DELETE CASCADE,
    blind_signature TEXT,
    unblind_signature TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,

    CONSTRAINT unique_signature_per_voter_trustee UNIQUE (voter_id, trustee_id)
);

-- Votes table (anonymized)
CREATE TABLE IF NOT EXISTS votes (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    election_id UUID NOT NULL REFERENCES elections(id) ON DELETE CASCADE,
    encrypted_vote TEXT NOT NULL,
    proof TEXT NOT NULL,
    aggregate_signature TEXT NOT NULL,
    verification_key TEXT NOT NULL,
    submitted_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    verified BOOLEAN DEFAULT FALSE
);

-- Cryptographic parameters table
CREATE TABLE IF NOT EXISTS crypto_parameters (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    election_id UUID NOT NULL UNIQUE REFERENCES elections(id) ON DELETE CASCADE,

    -- Bilinear group parameters
    prime_order TEXT NOT NULL,        -- p (asal sayı)
    g1 TEXT NOT NULL,                 -- G1 grubu üreteci
    g2 TEXT NOT NULL,                 -- G2 grubu üreteci
    h1 TEXT NOT NULL,                 -- G1 grubundan ikinci üreteci

    -- Master verification key (aggregated from trustees)
    mvk_alpha2 TEXT,                  -- Master verification key alpha component (G2)
    mvk_beta1 TEXT,                   -- Master verification key beta component (G1)
    mvk_beta2 TEXT,                   -- Master verification key beta component (G2)

    -- Pairing parameters (serialized)
    pairing_params TEXT NOT NULL,     -- PBC pairing parametreleri

    -- Metadata
    security_level INTEGER NOT NULL DEFAULT 256,  -- λ güvenlik parametresi (bit)
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- System events log
CREATE TABLE IF NOT EXISTS system_events (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    event_type VARCHAR(100) NOT NULL,
    entity_type VARCHAR(50),
    entity_id UUID,
    data JSONB,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Create indexes for better query performance
CREATE INDEX IF NOT EXISTS idx_elections_status ON elections(status);
CREATE INDEX IF NOT EXISTS idx_trustees_election ON trustees(election_id);
CREATE INDEX IF NOT EXISTS idx_voters_election ON voters(election_id);
CREATE INDEX IF NOT EXISTS idx_voters_status ON voters(status);
CREATE INDEX IF NOT EXISTS idx_votes_election ON votes(election_id);
CREATE INDEX IF NOT EXISTS idx_blind_signatures_voter ON blind_signatures(voter_id);
CREATE INDEX IF NOT EXISTS idx_crypto_parameters_election ON crypto_parameters(election_id);
CREATE INDEX IF NOT EXISTS idx_system_events_type ON system_events(event_type);
CREATE INDEX IF NOT EXISTS idx_system_events_created ON system_events(created_at);

-- Insert a sample election for testing
INSERT INTO elections (name, description, threshold, total_trustees, status)
VALUES
    ('Sample Election 2024', 'A test election for system verification', 3, 5, 'setup');

COMMENT ON TABLE elections IS 'Stores election configurations and metadata';
COMMENT ON TABLE trustees IS 'Election authorities participating in distributed key generation';
COMMENT ON TABLE voters IS 'Registered voters with their DIDs and credentials';
COMMENT ON TABLE blind_signatures IS 'Tracks blind signature issuance from trustees to voters';
COMMENT ON TABLE votes IS 'Anonymized votes with zero-knowledge proofs';
COMMENT ON TABLE system_events IS 'Audit log for system events';

-- ============================================================================
-- DKG (Distributed Key Generation) Tables
-- SECURITY NOTE: Private keys (sgk) are NEVER stored in database!
-- They remain ONLY in trustee containers, encrypted at rest.
-- ============================================================================

-- Master Verification Key (Election Public Key)
CREATE TABLE IF NOT EXISTS master_verification_keys (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    election_id UUID NOT NULL UNIQUE REFERENCES elections(id) ON DELETE CASCADE,
    alpha2 TEXT NOT NULL,  -- g2^x (aggregated from all qualified trustees)
    beta2 TEXT NOT NULL,   -- g2^y (aggregated from all qualified trustees)
    beta1 TEXT NOT NULL,   -- g1^y (aggregated from all qualified trustees)
    qualified_trustee_count INTEGER NOT NULL,
    threshold INTEGER NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Trustee Verification Keys (Public keys for each trustee)
CREATE TABLE IF NOT EXISTS trustee_verification_keys (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    trustee_id UUID NOT NULL UNIQUE REFERENCES trustees(id) ON DELETE CASCADE,
    election_id UUID NOT NULL REFERENCES elections(id) ON DELETE CASCADE,
    vk1 TEXT NOT NULL,  -- g2^x_i
    vk2 TEXT NOT NULL,  -- g2^y_i
    vk3 TEXT NOT NULL,  -- g1^y_i
    is_qualified BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- DKG Session Tracking
CREATE TABLE IF NOT EXISTS dkg_sessions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    election_id UUID NOT NULL REFERENCES elections(id) ON DELETE CASCADE,
    status VARCHAR(50) NOT NULL,  -- 'in_progress', 'completed', 'failed'
    current_step INTEGER DEFAULT 0,
    total_trustees INTEGER NOT NULL,
    threshold INTEGER NOT NULL,
    qualified_trustees JSONB,
    disqualified_trustees JSONB,
    started_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    completed_at TIMESTAMP WITH TIME ZONE,
    error_message TEXT,
    UNIQUE(election_id)
);

-- DKG Trustee Status
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

-- DKG Complaints
CREATE TABLE IF NOT EXISTS dkg_complaints (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    session_id UUID NOT NULL REFERENCES dkg_sessions(id) ON DELETE CASCADE,
    complainer_trustee_id UUID NOT NULL REFERENCES trustees(id) ON DELETE CASCADE,
    accused_trustee_id UUID NOT NULL REFERENCES trustees(id) ON DELETE CASCADE,
    complaint_type VARCHAR(100) NOT NULL,
    details TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Indexes for DKG tables
CREATE INDEX IF NOT EXISTS idx_mvk_election ON master_verification_keys(election_id);
CREATE INDEX IF NOT EXISTS idx_tvk_trustee ON trustee_verification_keys(trustee_id);
CREATE INDEX IF NOT EXISTS idx_tvk_election ON trustee_verification_keys(election_id);
CREATE INDEX IF NOT EXISTS idx_dkg_session_election ON dkg_sessions(election_id);
CREATE INDEX IF NOT EXISTS idx_dkg_session_status ON dkg_sessions(status);
CREATE INDEX IF NOT EXISTS idx_dkg_trustee_status_session ON dkg_trustee_status(session_id);
CREATE INDEX IF NOT EXISTS idx_dkg_trustee_status_trustee ON dkg_trustee_status(trustee_id);
CREATE INDEX IF NOT EXISTS idx_dkg_complaints_session ON dkg_complaints(session_id);
CREATE INDEX IF NOT EXISTS idx_dkg_complaints_accused ON dkg_complaints(accused_trustee_id);

-- Comments
COMMENT ON TABLE master_verification_keys IS 'Election public key (MVK) - PUBLIC DATA';
COMMENT ON TABLE trustee_verification_keys IS 'Trustee public verification keys - PUBLIC DATA';
COMMENT ON TABLE dkg_sessions IS 'DKG protocol execution tracking';
COMMENT ON TABLE dkg_trustee_status IS 'Individual trustee DKG progress';
COMMENT ON TABLE dkg_complaints IS 'DKG share verification complaints';
