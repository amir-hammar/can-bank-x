import http from "k6/http";
import { sleep } from "k6";

export const options = {
  vus: Number(__ENV.VUS || 10),
  duration: __ENV.DURATION || "30s",
};

const baseUrl = __ENV.BASE_URL || "http://localhost:8080";

export default function () {
  http.get(`${baseUrl}/health`, {
    tags: {
      service: "api-gateway",
      endpoint: "/health",
      method: "GET",
    },
  });
  sleep(1);
}
