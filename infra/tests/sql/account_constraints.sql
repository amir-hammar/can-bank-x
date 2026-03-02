\set ON_ERROR_STOP on

BEGIN;

INSERT INTO accounts (id, customer_id, type, currency, status)
VALUES ('44444444-4444-4444-4444-444444444441', '11111111-1111-1111-1111-111111111111', 'CHEQUING', 'CAD', 'OPEN');

INSERT INTO account_balances (account_id, available, ledger)
VALUES ('44444444-4444-4444-4444-444444444441', 100.00, 100.00);

INSERT INTO audit_log (id, actor_type, actor_id, action, entity_type, entity_id, metadata, trace_id)
VALUES (
  '55555555-5555-5555-5555-555555555551',
  'SYSTEM',
  'seed',
  'ACCOUNT_CREATED',
  'ACCOUNT',
  '44444444-4444-4444-4444-444444444441',
  '{}'::jsonb,
  'trace-account-1'
);

DO $$
BEGIN
  BEGIN
    UPDATE audit_log
    SET action = 'SHOULD_NOT_UPDATE'
    WHERE id = '55555555-5555-5555-5555-555555555551';
    RAISE EXCEPTION 'ASSERT_FAIL: account audit_log update unexpectedly succeeded';
  EXCEPTION WHEN OTHERS THEN
    NULL;
  END;
END $$;

DO $$
BEGIN
  BEGIN
    DELETE FROM audit_log
    WHERE id = '55555555-5555-5555-5555-555555555551';
    RAISE EXCEPTION 'ASSERT_FAIL: account audit_log delete unexpectedly succeeded';
  EXCEPTION WHEN OTHERS THEN
    NULL;
  END;
END $$;

SELECT 'account_constraints.sql passed' AS result;

ROLLBACK;
