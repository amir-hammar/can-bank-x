export function getCommonConfig(defaultVus = 20, defaultDuration = "2m") {
  const baseUrl = __ENV.BASE_URL || "http://localhost:8080";
  const vus = Number(__ENV.VUS || defaultVus);
  const duration = __ENV.DURATION || defaultDuration;

  const config = {
    baseUrl,
    vus,
    duration,
    readRatio: Number(__ENV.READ_RATIO || 0.7),
    transferAmount: Number(__ENV.TRANSFER_AMOUNT || 5.0),
    transferBeneficiaryUsername: __ENV.BENEFICIARY_USERNAME || "",
    staticCustomerId: __ENV.CUSTOMER_ID || "",
    staticFromAccountId: __ENV.FROM_ACCOUNT_ID || "",
    bearerToken: __ENV.BEARER_TOKEN || "",
    architecture: (__ENV.ARCHITECTURE || "microservices").toLowerCase(),
    targetThroughput: Number(__ENV.TARGET_THROUGHPUT || 0),
    targetAvailability: Number(__ENV.TARGET_AVAILABILITY || 0),
    targetP95Ms: Number(__ENV.TARGET_P95_MS || 0),
    rpsTarget: Number(__ENV.RPS_TARGET || 660),
    preAllocatedVUs: Number(__ENV.PRE_ALLOCATED_VUS || 400),
    maxVUs: Number(__ENV.MAX_VUS || 1200),
  };

  if (__ENV.STAGES) {
    config.stages = JSON.parse(__ENV.STAGES);
  }

  return config;
}

export function buildThresholds(config) {
  const defaultsByArchitecture = {
    microservices: {
      p95Ms: 500,
      throughput: 600,
      availability: 0.95,
    },
    "event-driven": {
      p95Ms: 250,
      throughput: 1000,
      availability: 0.99,
    },
  };

  const selected = defaultsByArchitecture[config.architecture] || defaultsByArchitecture.microservices;
  const targetP95Ms = config.targetP95Ms > 0 ? config.targetP95Ms : selected.p95Ms;
  const targetThroughput = config.targetThroughput > 0 ? config.targetThroughput : selected.throughput;
  const targetAvailability = config.targetAvailability > 0 ? config.targetAvailability : selected.availability;

  const thresholds = {
    http_req_failed: [`rate<${Math.max(0, 1 - targetAvailability)}`],
    http_req_duration: [`p(95)<${targetP95Ms}`],
  };

  if (targetThroughput > 0) {
    thresholds.http_reqs = [`rate>=${targetThroughput}`];
  }

  return thresholds;
}

export function authHeaders(token) {
  if (!token) {
    return { "Content-Type": "application/json" };
  }

  return {
    Authorization: `Bearer ${token}`,
    "Content-Type": "application/json",
  };
}

export function buildK6Options(config) {
  const options = {
    thresholds: buildThresholds(config),
  }

  if (config.stages) {
    options.stages = config.stages;
  } else {
    options.vus = config.vus;
    options.duration = config.duration;
  }

  return options;
}
