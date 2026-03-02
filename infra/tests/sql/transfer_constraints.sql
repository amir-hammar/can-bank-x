\set ON_ERROR_STOP on

BEGIN;

INSERT INTO transfers (
  id,
  customer_id,
  from_account_id,
  to_account_id,
  amount,
  status,
  idempotency_key
) VALUES (
  '66666666-6666-6666-6666-666666666661',
  '11111111-1111-1111-1111-111111111111',
  '44444444-4444-4444-4444-444444444441',
  '44444444-4444-4444-4444-444444444442',
  50.00,
  'PENDING',
  'idem-1'
);

DO $$
BEGIN
  BEGIN
    INSERT INTO transfers (
      id,
      customer_id,
      from_account_id,
      to_account_id,
      amount,
      status,
      idempotency_key
    ) VALUES (
      '66666666-6666-6666-6666-666666666662',
      '11111111-1111-1111-1111-111111111111',
      '44444444-4444-4444-4444-444444444443',
      '44444444-4444-4444-4444-444444444444',
      75.00,
      'PENDING',
      'idem-1'
    );
    RAISE EXCEPTION 'ASSERT_FAIL: transfer idempotency unique constraint unexpectedly succeeded';
  EXCEPTION WHEN unique_violation THEN
    NULL;
  END;
END $$;

INSERT INTO audit_log (id, actor_type, actor_id, action, entity_type, entity_id, metadata, trace_id)
VALUES (
  '77777777-7777-7777-7777-777777777771',
  'SYSTEM',
  'seed',
  'TRANSFER_CREATED',
  'TRANSFER',
  '66666666-6666-6666-6666-666666666661',
  '{}'::jsonb,
  'trace-transfer-1'
);

DO $$
BEGIN
  BEGIN
    UPDATE audit_log
    SET action = 'SHOULD_NOT_UPDATE'
    WHERE id = '77777777-7777-7777-7777-777777777771';
    RAISE EXCEPTION 'ASSERT_FAIL: transfer audit_log update unexpectedly succeeded';
  EXCEPTION WHEN OTHERS THEN
    NULL;
  END;
END $$;

DO $$
BEGIN
  BEGIN
    DELETE FROM audit_log
    WHERE id = '77777777-7777-7777-7777-777777777771';
    RAISE EXCEPTION 'ASSERT_FAIL: transfer audit_log delete unexpectedly succeeded';
  EXCEPTION WHEN OTHERS THEN
    NULL;
  END;
END $$;

SELECT 'transfer_constraints.sql passed' AS result;

ROLLBACK;
