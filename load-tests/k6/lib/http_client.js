import http from "k6/http";
import { check } from "k6";
import { apiErrorEvents } from "./metrics.js";

function inferServiceFromPath(path) {
  if (path.startsWith("/api/v1/accounts")) {
    return "account-service";
  }
  if (path.startsWith("/api/v1/transfers")) {
    return "transfer-service";
  }
  if (path.startsWith("/api/v1/auth") || path.startsWith("/api/v1/customers") || path.startsWith("/api/v1/kyc")) {
    return "user-service";
  }
  return "api-gateway";
}

function safeStatus(response) {
  if (!response || typeof response.status === "undefined") {
    return "0";
  }
  return String(response.status);
}

export function requestWithTags(
  method,
  baseUrl,
  path,
  { headers = {}, body = null, expectedStatus = (s) => s >= 200 && s < 400, metricEndpoint = null } = {}
) {
  const endpoint = metricEndpoint || path.split("?")[0];
  const requestTags = {
    service: inferServiceFromPath(endpoint),
    endpoint,
    method: method.toUpperCase(),
  };

  let response = null;
  try {
    response = http.request(method, `${baseUrl}${path}`, body, {
      headers,
      tags: requestTags,
    });
  } catch (error) {
    apiErrorEvents.add(1, {
      service: requestTags.service,
      endpoint: requestTags.endpoint,
      method: requestTags.method,
      status: "0",
      expected_response: "false",
      error_code: "NETWORK_ERROR",
    });
    return null;
  }

  const ok = expectedStatus(response.status);
  check(response, {
    expected_response: () => ok,
  });

  if (!ok) {
    apiErrorEvents.add(1, {
      service: requestTags.service,
      endpoint: requestTags.endpoint,
      method: requestTags.method,
      status: safeStatus(response),
      expected_response: "false",
      error_code: `HTTP_${safeStatus(response)}`,
    });
  }

  return response;
}

export function parseJsonSafe(response) {
  if (!response || !response.body) {
    return null;
  }

  try {
    return response.json();
  } catch (_) {
    return null;
  }
}
