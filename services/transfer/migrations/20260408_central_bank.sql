-- Central Bank Payment Links
CREATE TABLE IF NOT EXISTS central_bank_payment_links (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    alias TEXT NOT NULL,
    account_id UUID NOT NULL,
    holder_name TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'PENDING' CHECK (status IN ('PENDING', 'ACTIVE', 'DEACTIVATED')),
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_cb_payment_links_alias ON central_bank_payment_links(alias);
CREATE INDEX IF NOT EXISTS idx_cb_payment_links_account_id ON central_bank_payment_links(account_id);

DROP TRIGGER IF EXISTS trg_cb_payment_links_updated_at ON central_bank_payment_links;
CREATE TRIGGER trg_cb_payment_links_updated_at
BEFORE UPDATE ON central_bank_payment_links
FOR EACH ROW
EXECUTE FUNCTION set_updated_at();

-- Central Bank Aliases (cache of alias lookups)
CREATE TABLE IF NOT EXISTS central_bank_aliases (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    alias TEXT NOT NULL,
    creditor_participant TEXT,
    masked_name TEXT,
    found BOOLEAN NOT NULL DEFAULT FALSE,
    looked_up_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_cb_aliases_alias ON central_bank_aliases(alias);

-- Central Bank Pending Alias Transfers
CREATE TABLE IF NOT EXISTS central_bank_pending_alias_transfers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    transfer_id TEXT NOT NULL UNIQUE,
    alias_value TEXT NOT NULL,
    account_id UUID NOT NULL,
    debtor_participant TEXT NOT NULL,
    new_debtor_participant TEXT NOT NULL,
    destination_email TEXT,
    status TEXT NOT NULL DEFAULT 'PENDING' CHECK (status IN ('PENDING', 'AWAITING_EMAIL_VALIDATION', 'APPROVED', 'DENIED', 'COMPLETED', 'REJECTED')),
    reason TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_cb_pending_alias_transfers_status ON central_bank_pending_alias_transfers(status);

DROP TRIGGER IF EXISTS trg_cb_pending_alias_transfers_updated_at ON central_bank_pending_alias_transfers;
CREATE TRIGGER trg_cb_pending_alias_transfers_updated_at
BEFORE UPDATE ON central_bank_pending_alias_transfers
FOR EACH ROW
EXECUTE FUNCTION set_updated_at();
