CREATE EXTENSION IF NOT EXISTS pgcrypto;

CREATE TABLE IF NOT EXISTS transfers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    customer_id UUID NOT NULL,
    from_account_id UUID NOT NULL,
    to_account_id UUID NOT NULL,
    amount NUMERIC(14,2) NOT NULL CHECK (amount > 0),
    status TEXT NOT NULL CHECK (status IN ('PENDING', 'COMPLETED', 'FAILED')),
    idempotency_key TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
    UNIQUE (customer_id, idempotency_key)
);

CREATE INDEX IF NOT EXISTS idx_transfers_customer_id ON transfers(customer_id);

CREATE TABLE IF NOT EXISTS audit_log (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    timestamp TIMESTAMP NOT NULL DEFAULT NOW(),
    actor_type TEXT NOT NULL CHECK (actor_type IN ('CUSTOMER', 'SYSTEM')),
    actor_id TEXT NOT NULL,
    action TEXT NOT NULL,
    entity_type TEXT NOT NULL,
    entity_id TEXT NOT NULL,
    metadata JSONB,
    trace_id TEXT
);

CREATE OR REPLACE FUNCTION set_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS trg_transfers_updated_at ON transfers;
CREATE TRIGGER trg_transfers_updated_at
BEFORE UPDATE ON transfers
FOR EACH ROW
EXECUTE FUNCTION set_updated_at();

CREATE OR REPLACE FUNCTION prevent_audit_log_mutation()
RETURNS TRIGGER AS $$
BEGIN
    RAISE EXCEPTION 'audit_log is append-only';
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS trg_audit_log_no_update ON audit_log;
CREATE TRIGGER trg_audit_log_no_update
BEFORE UPDATE ON audit_log
FOR EACH ROW
EXECUTE FUNCTION prevent_audit_log_mutation();

DROP TRIGGER IF EXISTS trg_audit_log_no_delete ON audit_log;
CREATE TRIGGER trg_audit_log_no_delete
BEFORE DELETE ON audit_log
FOR EACH ROW
EXECUTE FUNCTION prevent_audit_log_mutation();
