import http from "k6/http";
import { getCommonConfig, buildK6Options } from "../lib/config.js";
import { setupSession, transferFlow } from "../lib/scenarios.js";

http.setResponseCallback(http.expectedStatuses({ min: 200, max: 399 }));

const config = getCommonConfig(15, "2m");

export const options = buildK6Options(config);

export function setup() {
  return setupSession(config);
}

export default function (session) {
  transferFlow(config, session);
}
