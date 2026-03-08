import http from "k6/http";
import { getCommonConfig, buildK6Options } from "../lib/config.js";
import { setupSession, accountConsultationFlow, transferFlow } from "../lib/scenarios.js";

http.setResponseCallback(http.expectedStatuses({ min: 200, max: 399 }));

const config = getCommonConfig(50, "3m");

export const options = buildK6Options(config);

export function setup() {
  return setupSession(config);
}

export default function (session) {
  if (Math.random() < config.readRatio) {
    accountConsultationFlow(config, session);
  } else {
    transferFlow(config, session);
  }
}
