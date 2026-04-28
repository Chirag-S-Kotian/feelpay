const http = require("http");

const PORT = process.env.MOCK_PROVIDER_PORT || 9090;

function json(res, code, body) {
  res.writeHead(code, { "Content-Type": "application/json" });
  res.end(JSON.stringify(body));
}

const server = http.createServer((req, res) => {
  if (req.method === "POST" && req.url === "/authorize") {
    return json(res, 200, {
      status: "authorized",
      provider_reference: `mock_auth_${Date.now()}`,
    });
  }

  if (req.method === "POST" && req.url === "/capture") {
    return json(res, 200, {
      status: "captured",
      provider_reference: `mock_cap_${Date.now()}`,
    });
  }

  if (req.method === "POST" && req.url === "/refund") {
    return json(res, 200, {
      status: "refunded",
      provider_reference: `mock_ref_${Date.now()}`,
    });
  }

  if (req.method === "POST" && req.url === "/webhook") {
    return json(res, 200, { status: "received" });
  }

  if (req.method === "GET" && req.url === "/health") {
    return json(res, 200, { ok: true });
  }

  return json(res, 404, { error: "not_found" });
});

server.listen(PORT, () => {
  console.log(`mock provider server listening on :${PORT}`);
});

