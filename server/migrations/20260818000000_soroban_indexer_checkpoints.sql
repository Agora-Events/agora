-- Re-org resilient Soroban event indexer (Issue #1174)
--
-- Two tables back the new producer-consumer indexing pipeline:
--
-- 1. `blockchain_checkpoints` – a per-chain ledger cursor persisted in
--    PostgreSQL. Every write goes through a single atomic upsert so a crash
--    can never leave the cursor half-advanced. On a Stellar re-org the cursor
--    is rolled back to the fork ledger minus one.
--
-- 2. `indexer_ledger_events` – the sliding-window event buffer. Every raw RPC
--    event is parked here (deduplicated by its RPC `id`) before it has
--    reached finality. Once a ledger is `INDEXER_CONFIRMATIONS` ledgers below
--    the chain head (`latest_ledger`) its events are applied to the
--    application tables and marked `finalized`. The window is what lets the
--    indexer detect – and roll back – unfinalized state on re-orgs.

CREATE TABLE IF NOT EXISTS blockchain_checkpoints (
    chain_key        TEXT PRIMARY KEY,
    ledger_sequence  BIGINT NOT NULL DEFAULT 0,
    event_cursor     TEXT,
    created_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at       TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS indexer_ledger_events (
    id            TEXT PRIMARY KEY,
    ledger        BIGINT NOT NULL,
    contract_id   TEXT NOT NULL,
    topic         JSONB NOT NULL,
    value         JSONB NOT NULL,
    finalized     BOOLEAN NOT NULL DEFAULT FALSE,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_indexer_ledger_events_ledger
    ON indexer_ledger_events (ledger);
CREATE INDEX IF NOT EXISTS idx_indexer_ledger_events_contract
    ON indexer_ledger_events (contract_id);
CREATE INDEX IF NOT EXISTS idx_indexer_ledger_events_finalized
    ON indexer_ledger_events (ledger, finalized);