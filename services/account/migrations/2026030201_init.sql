CREATE EXTENSION IF NOT EXISTS pgcrypto;

CREATE TABLE IF NOT EXISTS accounts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    customer_id UUID NOT NULL,
    type TEXT NOT NULL CHECK (type IN ('CHEQUING', 'SAVINGS')),
    currency TEXT NOT NULL DEFAULT 'CAD',
    status TEXT NOT NULL CHECK (status IN ('OPEN', 'CLOSED')),
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_accounts_customer_id ON accounts(customer_id);

CREATE TABLE IF NOT EXISTS account_balances (
    account_id UUID PRIMARY KEY REFERENCES accounts(id) ON DELETE CASCADE,
    available NUMERIC(14,2) NOT NULL DEFAULT 0,
    ledger NUMERIC(14,2) NOT NULL DEFAULT 0,
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

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

DROP TRIGGER IF EXISTS trg_accounts_updated_at ON accounts;
CREATE TRIGGER trg_accounts_updated_at
BEFORE UPDATE ON accounts
FOR EACH ROW
EXECUTE FUNCTION set_updated_at();

DROP TRIGGER IF EXISTS trg_account_balances_updated_at ON account_balances;
CREATE TRIGGER trg_account_balances_updated_at
BEFORE UPDATE ON account_balances
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
