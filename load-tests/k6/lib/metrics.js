import { Counter } from "k6/metrics";

// Exported by k6 Prometheus RW as k6_api_error_events_total.
export const apiErrorEvents = new Counter("api_error_events");
