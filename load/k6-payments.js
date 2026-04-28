import http from "k6/http";
import { check, sleep } from "k6";

export const options = {
  scenarios: {
    sustained_500_tps: {
      executor: "constant-arrival-rate",
      rate: 500,
      timeUnit: "1s",
      duration: "5m",
      preAllocatedVUs: 300,
      maxVUs: 1000,
    },
  },
  thresholds: {
    http_req_failed: ["rate<0.01"],
    http_req_duration: ["p(95)<500"],
  },
};

const BASE = __ENV.BASE_URL || "http://127.0.0.1:8080";

export default function () {
  const res = http.get(`${BASE}/v1/health`);
  check(res, {
    "status is 200": (r) => r.status === 200,
  });
  sleep(1);
}
