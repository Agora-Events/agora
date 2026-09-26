-- Multi-Sig Escrow Lockup & DAO-Governed Fraud Dispute Mediation Protocol (Issue #1176)
--
-- This migration creates the disputes and mediation_votes tables that back the
-- governance dispute endpoints under /api/v1/admin/governance.

CREATE EXTENSION IF NOT EXISTS "pgcrypto";

CREATE TABLE IF NOT EXISTS disputes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    event_id UUID NOT NULL REFERENCES events(id),
    opened_by UUID NOT NULL REFERENCES users(id),
    status VARCHAR(32) NOT NULL DEFAULT 'open',
    closes_at TIMESTAMPTZ NOT NULL,
    resolved_at TIMESTAMPTZ,
    ruling VARCHAR(32),
    total_eligible_tickets INTEGER NOT NULL DEFAULT 0,
    buyer_votes INTEGER NOT NULL DEFAULT 0,
    organizer_votes INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS mediation_votes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    dispute_id UUID NOT NULL REFERENCES disputes(id) ON DELETE CASCADE,
    voter_id UUID NOT NULL REFERENCES users(id),
    vote VARCHAR(32) NOT NULL,
    voted_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(dispute_id, voter_id)
);

CREATE INDEX IF NOT EXISTS idx_disputes_event_id ON disputes(event_id);
CREATE INDEX IF NOT EXISTS idx_disputes_status ON disputes(status);
CREATE INDEX IF NOT EXISTS idx_mediation_votes_dispute_id ON mediation_votes(dispute_id);
