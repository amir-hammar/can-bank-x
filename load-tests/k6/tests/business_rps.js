import http from "k6/http";
import { getCommonConfig, buildThresholds, authHeaders } from "../lib/config.js";
import { setupSession } from "../lib/scenarios.js";
import { requestWithTags, parseJsonSafe } from "../lib/http_client.js";
import { apiErrorEvents } from "../lib/metrics.js";

http.setResponseCallback(http.expectedStatuses({ min: 200, max: 399 }));

const config = getCommonConfig(200, "2m");

const totalRps = Math.max(1, config.rpsTarget);
const profileRps = Math.max(1, Math.floor(totalRps * 0.3));
const accountsRps = Math.max(1, Math.floor(totalRps * 0.35));
const balanceRps = Math.max(1, totalRps - profileRps - accountsRps);

function scenario(rate, execName) {
  return {
    executor: "constant-arrival-rate",
    rate,
    timeUnit: "1s",
    duration: config.duration,
    preAllocatedVUs: Math.max(20, Math.floor(config.preAllocatedVUs / 4)),
    maxVUs: Math.max(50, Math.floor(config.maxVUs / 4)),
    exec: execName,
  };
}

const scenarioOptions = {
  profile: scenario(profileRps, "profileRequest"),
  accounts: scenario(accountsRps, "accountsRequest"),
  balance: scenario(balanceRps, "balanceRequest"),
};

export const options = {
  thresholds: buildThresholds(config),
  scenarios: scenarioOptions,
};

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

function ensureCustomerId(session) {
  if (session.customerId) {
    return true;
  }

  const customerMeResponse = requestWithTags("GET", config.baseUrl, "/api/v1/customers/me", {
    headers: authHeaders(session.token),
  });
  const payload = parseJsonSafe(customerMeResponse);
  session.customerId = getCustomerIdFromPayload(payload);

  if (!session.customerId) {
    requestWithTags("POST", config.baseUrl, "/api/v1/customers/register", {
      headers: authHeaders(session.token),
    });

    const customerMeRetryResponse = requestWithTags("GET", config.baseUrl, "/api/v1/customers/me", {
      headers: authHeaders(session.token),
    });
    const retryPayload = parseJsonSafe(customerMeRetryResponse);
    session.customerId = getCustomerIdFromPayload(retryPayload);
  }

  return Boolean(session.customerId);
}

function ensureAccountId(session) {
  if (session.fromAccountId) {
    return true;
  }

  if (!ensureCustomerId(session)) {
    return false;
  }

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
    const defaultPayload = parseJsonSafe(defaultAccountResponse);
    session.fromAccountId = getFirstAccountId(defaultPayload);
  }

  return Boolean(session.fromAccountId);
}

function recordMissing(endpoint, code) {
  apiErrorEvents.add(1, {
    service: "test-driver",
    endpoint,
    method: "GET",
    status: "0",
    expected_response: "false",
    error_code: code,
  });
}

export function setup() {
  return setupSession(config);
}

export function profileRequest(session) {
  requestWithTags("GET", config.baseUrl, "/api/v1/auth/me", {
    headers: authHeaders(session.token),
  });
}

export function accountsRequest(session) {
  if (!ensureCustomerId(session)) {
    recordMissing("/api/v1/accounts", "MISSING_CUSTOMER_ID");
    return;
  }

  requestWithTags("GET", config.baseUrl, `/api/v1/accounts?customer_id=${encodeURIComponent(session.customerId)}`, {
    headers: authHeaders(session.token),
  });
}

export function balanceRequest(session) {
  if (!ensureAccountId(session)) {
    recordMissing("/api/v1/accounts/balance", "MISSING_FROM_ACCOUNT_ID");
    return;
  }

  requestWithTags(
    "GET",
    config.baseUrl,
    `/api/v1/accounts/balance?account_id=${encodeURIComponent(session.fromAccountId)}`,
    {
      headers: authHeaders(session.token),
    }
  );
}

