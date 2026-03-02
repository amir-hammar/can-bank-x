\set ON_ERROR_STOP on

BEGIN;

INSERT INTO customers (id, keycloak_sub, email, status)
VALUES ('11111111-1111-1111-1111-111111111111', 'kc-sub-1', 'user1@example.com', 'PENDING');

DO $$
BEGIN
  BEGIN
    INSERT INTO customers (id, keycloak_sub, email, status)
    VALUES ('11111111-1111-1111-1111-111111111112', 'kc-sub-2', 'user1@example.com', 'PENDING');
    RAISE EXCEPTION 'ASSERT_FAIL: duplicate email insert unexpectedly succeeded';
  EXCEPTION WHEN unique_violation THEN
    NULL;
  END;
END $$;

DO $$
BEGIN
  BEGIN
    INSERT INTO customers (id, keycloak_sub, email, status)
    VALUES ('11111111-1111-1111-1111-111111111113', 'kc-sub-1', 'user2@example.com', 'PENDING');
    RAISE EXCEPTION 'ASSERT_FAIL: duplicate keycloak_sub insert unexpectedly succeeded';
  EXCEPTION WHEN unique_violation THEN
    NULL;
  END;
END $$;

INSERT INTO kyc_cases (id, customer_id, status)
VALUES ('22222222-2222-2222-2222-222222222221', '11111111-1111-1111-1111-111111111111', 'PENDING');

DO $$
BEGIN
  BEGIN
    INSERT INTO kyc_cases (id, customer_id, status)
    VALUES ('22222222-2222-2222-2222-222222222222', '11111111-1111-1111-1111-111111111111', 'ACTIVE');
    RAISE EXCEPTION 'ASSERT_FAIL: duplicate customer_id in kyc_cases unexpectedly succeeded';
  EXCEPTION WHEN unique_violation THEN
    NULL;
  END;
END $$;

INSERT INTO audit_log (id, actor_type, actor_id, action, entity_type, entity_id, metadata, trace_id)
VALUES (
  '33333333-3333-3333-3333-333333333331',
  'SYSTEM',
  'seed',
  'CUSTOMER_REGISTERED',
  'CUSTOMER',
  '11111111-1111-1111-1111-111111111111',
  '{}'::jsonb,
  'trace-user-1'
);

DO $$
BEGIN
  BEGIN
    UPDATE audit_log
    SET action = 'SHOULD_NOT_UPDATE'
    WHERE id = '33333333-3333-3333-3333-333333333331';
    RAISE EXCEPTION 'ASSERT_FAIL: audit_log update unexpectedly succeeded';
  EXCEPTION WHEN OTHERS THEN
    NULL;
  END;
END $$;

DO $$
BEGIN
  BEGIN
    DELETE FROM audit_log
    WHERE id = '33333333-3333-3333-3333-333333333331';
    RAISE EXCEPTION 'ASSERT_FAIL: audit_log delete unexpectedly succeeded';
  EXCEPTION WHEN OTHERS THEN
    NULL;
  END;
END $$;

SELECT 'user_constraints.sql passed' AS result;

ROLLBACK;
