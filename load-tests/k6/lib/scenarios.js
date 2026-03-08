import { sleep } from "k6";
import { apiErrorEvents } from "./metrics.js";
import { requestWithTags, parseJsonSafe } from "./http_client.js";
import { authHeaders } from "./config.js";

function randomIntBetween(min, max) {
  return Math.floor(Math.random() * (max - min + 1)) + min;
}

function idempotencyKey() {
  return `k6-${Date.now()}-${__VU}-${__ITER}`;
}

function getCustomerIdFromPayload(payload) {
  if (!payload) {
    return "";
  }
  if (payload.customer && typeof payload.customer === "object") {
    return payload.customer.id || payload.customer.customer_id || "";
  }
  return payload.id || payload.customer_id || "";
}

function getFirstAccountId(payload) {
  const hasObjectPayload = payload && typeof payload === "object";
  const payloadItems = hasObjectPayload && Array.isArray(payload.items) ? payload.items : [];
  const payloadAccounts = hasObjectPayload && Array.isArray(payload.accounts) ? payload.accounts : [];

  const items = Array.isArray(payload)
    ? payload
    : payloadItems.length > 0
      ? payloadItems
      : payloadAccounts.length > 0
        ? payloadAccounts
        : [];

  if (items.length === 0) {
    if (!hasObjectPayload) {
      return "";
    }
    return payload.account_id || payload.accountId || payload.id || "";
  }

  return items[0].account_id || items[0].accountId || items[0].id || "";
}

function getTransferId(payload) {
  if (!payload) {
    return "";
  }
  return payload.transfer_id || payload.id || "";
}

function recordMissingData(endpoint, method, code) {
  apiErrorEvents.add(1, {
    service: "test-driver",
    endpoint,
    method,
    status: "0",
    expected_response: "false",
    error_code: code,
  });
}

export function setupSession(config) {
  const session = {
    token: config.bearerToken,
    customerId: config.staticCustomerId,
    fromAccountId: config.staticFromAccountId,
    beneficiaryUsername: config.transferBeneficiaryUsername,
  };

  if (!session.token) {
    return session;
  }

  if (!session.customerId) {
    const meResponse = requestWithTags("GET", config.baseUrl, "/api/v1/customers/me", {
      headers: authHeaders(session.token),
    });
    const mePayload = parseJsonSafe(meResponse);
    session.customerId = getCustomerIdFromPayload(mePayload);

    // If profile is not created yet for this JWT subject, register once then retry profile lookup.
    if (!session.customerId) {
      requestWithTags("POST", config.baseUrl, "/api/v1/customers/register", {
        headers: authHeaders(session.token),
      });

      const meRetryResponse = requestWithTags("GET", config.baseUrl, "/api/v1/customers/me", {
        headers: authHeaders(session.token),
      });
      const meRetryPayload = parseJsonSafe(meRetryResponse);
      session.customerId = getCustomerIdFromPayload(meRetryPayload);
    }
  }

  if (!session.fromAccountId && session.customerId) {
    const accountsResponse = requestWithTags(
      "GET",
      config.baseUrl,
      `/api/v1/accounts?customer_id=${encodeURIComponent(session.customerId)}`,
      { headers: authHeaders(session.token) }
    );
    const accountsPayload = parseJsonSafe(accountsResponse);
    session.fromAccountId = getFirstAccountId(accountsPayload);

    if (!session.fromAccountId) {
      const defaultAccountResponse = requestWithTags(
        "GET",
        config.baseUrl,
        `/api/v1/accounts/default?customer_id=${encodeURIComponent(session.customerId)}`,
        { headers: authHeaders(session.token) }
      );
      const defaultAccountPayload = parseJsonSafe(defaultAccountResponse);
      session.fromAccountId = getFirstAccountId(defaultAccountPayload);
    }
  }

  return session;
}

export function accountConsultationFlow(config, session) {
  if (!session.token) {
    recordMissingData("/api/v1/auth/me", "GET", "MISSING_BEARER_TOKEN");
    sleep(randomIntBetween(1, 2));
    return;
  }

  requestWithTags("GET", config.baseUrl, "/api/v1/auth/me", {
    headers: authHeaders(session.token),
  });

  if (!session.customerId) {
    const customerMeResponse = requestWithTags("GET", config.baseUrl, "/api/v1/customers/me", {
      headers: authHeaders(session.token),
    });
    const customerPayload = parseJsonSafe(customerMeResponse);
    session.customerId = getCustomerIdFromPayload(customerPayload);
  }

  if (!session.customerId) {
    recordMissingData("/api/v1/accounts", "GET", "MISSING_CUSTOMER_ID");
    sleep(randomIntBetween(1, 2));
    return;
  }

  const accountsResponse = requestWithTags(
    "GET",
    config.baseUrl,
    `/api/v1/accounts?customer_id=${encodeURIComponent(session.customerId)}`,
    {
      headers: authHeaders(session.token),
    }
  );

  if (!session.fromAccountId) {
    const accountsPayload = parseJsonSafe(accountsResponse);
    session.fromAccountId = getFirstAccountId(accountsPayload);

    if (!session.fromAccountId) {
      const defaultAccountResponse = requestWithTags(
        "GET",
        config.baseUrl,
        `/api/v1/accounts/default?customer_id=${encodeURIComponent(session.customerId)}`,
        {
          headers: authHeaders(session.token),
        }
      );
      const defaultAccountPayload = parseJsonSafe(defaultAccountResponse);
      session.fromAccountId = getFirstAccountId(defaultAccountPayload);
    }
  }

  if (session.fromAccountId) {
    requestWithTags(
      "GET",
      config.baseUrl,
      `/api/v1/accounts/balance?account_id=${encodeURIComponent(session.fromAccountId)}`,
      {
        headers: authHeaders(session.token),
      }
    );
  } else {
    recordMissingData("/api/v1/accounts/balance", "GET", "MISSING_FROM_ACCOUNT_ID");
  }

  requestWithTags(
    "GET",
    config.baseUrl,
    `/api/v1/transfers?customer_id=${encodeURIComponent(session.customerId)}&limit=20`,
    {
      headers: authHeaders(session.token),
    }
  );

  sleep(randomIntBetween(1, 2));
}

export function transferFlow(config, session) {
  if (!session.token) {
    recordMissingData("/api/v1/transfers", "POST", "MISSING_BEARER_TOKEN");
    sleep(randomIntBetween(3, 5));
    return;
  }

  if (!session.customerId || !session.fromAccountId || !session.beneficiaryUsername) {
    recordMissingData("/api/v1/transfers", "POST", "MISSING_TRANSFER_INPUT");
    sleep(randomIntBetween(3, 5));
    return;
  }

  const payload = JSON.stringify({
    customer_id: session.customerId,
    from_account_id: session.fromAccountId,
    beneficiary_username: session.beneficiaryUsername,
    amount: config.transferAmount,
    idempotency_key: idempotencyKey(),
  });

  const transferResponse = requestWithTags("POST", config.baseUrl, "/api/v1/transfers", {
    headers: authHeaders(session.token),
    body: payload,
    expectedStatus: (status) => status === 201,
  });

  const transferPayload = parseJsonSafe(transferResponse);
  const transferId = getTransferId(transferPayload);
  if (transferId) {
    requestWithTags("GET", config.baseUrl, `/api/v1/transfers/${encodeURIComponent(transferId)}`, {
      headers: authHeaders(session.token),
      metricEndpoint: "/api/v1/transfers/{id}",
    });
  } else {
    recordMissingData("/api/v1/transfers/{id}", "GET", "MISSING_TRANSFER_ID");
  }

  sleep(randomIntBetween(3, 5));
}
