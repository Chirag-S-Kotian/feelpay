use actix_web::{HttpResponse, Responder};

pub async fn admin_docs() -> impl Responder {
    let html = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <title>FeelPay Admin API - Swagger UI</title>
    <link rel="stylesheet" type="text/css" href="https://cdn.jsdelivr.net/npm/swagger-ui-dist@5/swagger-ui.css">
    <link rel="icon" type="image/png" href="https://feelpay.in/favicon.ico">
    <style>
        html { box-sizing: border-box; overflow: -moz-scrollbars-vertical; overflow-y: scroll; }
        *, *:before, *:after { box-sizing: inherit; }
        body { margin:0; background: #fafafa; }
        .topbar { display: none; }
    </style>
</head>
<body>
    <div id="swagger-ui"></div>
    <script src="https://cdn.jsdelivr.net/npm/swagger-ui-dist@5/swagger-ui-bundle.js"></script>
    <script src="https://cdn.jsdelivr.net/npm/swagger-ui-dist@5/swagger-ui-standalone-preset.js"></script>
    <script>
    window.onload = function() {
        window.ui = SwaggerUIBundle({
            url: "/openapi.json",
            dom_id: '#swagger-ui',
            deepLinking: true,
            presets: [
                SwaggerUIBundle.presets.apis,
                SwaggerUIStandalonePreset
            ],
            plugins: [
                SwaggerUIBundle.plugins.DownloadUrl
            ],
            layout: "StandaloneLayout"
        });
    };
    </script>
</body>
</html>"#;

    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html)
}

const SPEC: &str = r#"{
  "openapi": "3.0.3",
  "info": {
    "title": "FeelPay Admin API",
    "version": "1.0.0",
    "description": "Administrative management API for the FeelPay Payment Gateway platform. Supports merchant operations, provider credentials, platform statistics, disputes, settlements, bank accounts, and bulk exports.",
    "contact": {
      "name": "FeelPay Operations",
      "email": "admin@feelpay.in"
    }
  },
  "servers": [
    { "url": "https://feelpay-admin-backend.onrender.com", "description": "Production Admin Backend" },
    { "url": "http://localhost:8081", "description": "Local Development Server" }
  ],
  "tags": [
    { "name": "System", "description": "Health and system readiness checks" },
    { "name": "Auth", "description": "Admin authentication and token generation" },
    { "name": "Platform", "description": "Platform statistics and overview metrics" },
    { "name": "Merchants", "description": "Merchant accounts, onboarding status, and impersonation" },
    { "name": "Credentials", "description": "Payment provider credentials and vault keys" },
    { "name": "Banking", "description": "Merchant bank accounts and penny-drop verification" },
    { "name": "Exports", "description": "Bulk data export jobs and downloads" },
    { "name": "Operations", "description": "Transactions, settlements, disputes, and revenue" }
  ],
  "paths": {
    "/health": {
      "get": {
        "tags": ["System"],
        "summary": "Health Check",
        "description": "Returns status of the admin backend service.",
        "responses": {
          "200": { "description": "Service is healthy" }
        }
      }
    },
    "/ready": {
      "get": {
        "tags": ["System"],
        "summary": "Readiness Check",
        "description": "Verifies database connectivity.",
        "responses": {
          "200": { "description": "Service and database ready" },
          "503": { "description": "Database unavailable" }
        }
      }
    },
    "/admin/login": {
      "post": {
        "tags": ["Auth"],
        "summary": "Admin Login",
        "description": "Authenticate an admin merchant user with email and password.",
        "requestBody": {
          "required": true,
          "content": {
            "application/json": {
              "schema": {
                "type": "object",
                "required": ["email", "password"],
                "properties": {
                  "email": { "type": "string", "example": "admin@feelpay.in" },
                  "password": { "type": "string", "example": "SuperSecretPass123!" }
                }
              }
            }
          }
        },
        "responses": {
          "200": { "description": "JWT token issued" },
          "401": { "description": "Invalid credentials" },
          "403": { "description": "Admin access required" }
        }
      }
    },
    "/admin/bootstrap-token": {
      "get": {
        "tags": ["Auth"],
        "summary": "Bootstrap Admin Token",
        "description": "Generate an initial token for primary admin merchant in dev / initial setup.",
        "responses": {
          "200": { "description": "Bootstrap token returned" },
          "401": { "description": "No admin merchant configured" }
        }
      }
    },
    "/admin/stats": {
      "get": {
        "tags": ["Platform"],
        "summary": "Platform Overview Stats",
        "security": [{ "BearerAuth": [] }],
        "responses": {
          "200": { "description": "Platform volume, active merchants, and health status" }
        }
      }
    },
    "/admin/operations/summary": {
      "get": {
        "tags": ["Platform"],
        "summary": "Operations Summary",
        "security": [{ "BearerAuth": [] }],
        "responses": {
          "200": { "description": "24h volume, failed count, pending disputes/settlements, capture rate" }
        }
      }
    },
    "/admin/revenue/summary": {
      "get": {
        "tags": ["Platform"],
        "summary": "Revenue Summary",
        "security": [{ "BearerAuth": [] }],
        "responses": {
          "200": { "description": "Fee earnings, MDR breakdown, net platform profit" }
        }
      }
    },
    "/admin/merchants": {
      "get": {
        "tags": ["Merchants"],
        "summary": "List Merchants",
        "security": [{ "BearerAuth": [] }],
        "parameters": [
          { "name": "limit", "in": "query", "schema": { "type": "integer", "default": 50 } },
          { "name": "offset", "in": "query", "schema": { "type": "integer", "default": 0 } },
          { "name": "status", "in": "query", "schema": { "type": "string" } },
          { "name": "q", "in": "query", "schema": { "type": "string" } }
        ],
        "responses": {
          "200": { "description": "List of merchants" }
        }
      }
    },
    "/admin/merchants/{id}/impersonate": {
      "post": {
        "tags": ["Merchants"],
        "summary": "Impersonate Merchant",
        "description": "Generate a scoped merchant session token for debugging merchant issues.",
        "security": [{ "BearerAuth": [] }],
        "parameters": [
          { "name": "id", "in": "path", "required": true, "schema": { "type": "string", "format": "uuid" } }
        ],
        "responses": {
          "200": { "description": "Impersonation session JWT token" }
        }
      }
    },
    "/admin/provider-credentials": {
      "get": {
        "tags": ["Credentials"],
        "summary": "List Provider Credentials",
        "security": [{ "BearerAuth": [] }],
        "responses": {
          "200": { "description": "List of configured credentials (masked)" }
        }
      },
      "post": {
        "tags": ["Credentials"],
        "summary": "Upsert Provider Credential",
        "security": [{ "BearerAuth": [] }],
        "requestBody": {
          "required": true,
          "content": {
            "application/json": {
              "schema": {
                "type": "object",
                "required": ["provider", "key_name", "value"],
                "properties": {
                  "provider": { "type": "string", "example": "razorpay" },
                  "key_name": { "type": "string", "example": "key_secret" },
                  "value": { "type": "string", "example": "rzp_live_abc123" }
                }
              }
            }
          }
        },
        "responses": {
          "200": { "description": "Credential saved into encrypted vault" }
        }
      }
    },
    "/admin/provider-credentials/{id}/validate": {
      "post": {
        "tags": ["Credentials"],
        "summary": "Validate Provider Credential",
        "security": [{ "BearerAuth": [] }],
        "parameters": [
          { "name": "id", "in": "path", "required": true, "schema": { "type": "string", "format": "uuid" } }
        ],
        "responses": {
          "200": { "description": "Validation result against downstream gateway" }
        }
      }
    },
    "/admin/bank-accounts": {
      "get": {
        "tags": ["Banking"],
        "summary": "List Merchant Bank Accounts",
        "security": [{ "BearerAuth": [] }],
        "responses": {
          "200": { "description": "List of bank accounts and penny-drop verification status" }
        }
      }
    },
    "/admin/transactions": {
      "get": {
        "tags": ["Operations"],
        "summary": "List Platform Transactions",
        "security": [{ "BearerAuth": [] }],
        "parameters": [
          { "name": "limit", "in": "query", "schema": { "type": "integer", "default": 50 } },
          { "name": "offset", "in": "query", "schema": { "type": "integer", "default": 0 } }
        ],
        "responses": {
          "200": { "description": "Global transaction records" }
        }
      }
    },
    "/admin/settlements": {
      "get": {
        "tags": ["Operations"],
        "summary": "List Platform Settlements",
        "security": [{ "BearerAuth": [] }],
        "responses": {
          "200": { "description": "Settlement batches and status" }
        }
      }
    },
    "/admin/disputes": {
      "get": {
        "tags": ["Operations"],
        "summary": "List Disputes & Chargebacks",
        "security": [{ "BearerAuth": [] }],
        "responses": {
          "200": { "description": "Chargebacks, evidence submission status, and deadlines" }
        }
      }
    },
    "/admin/exports": {
      "get": {
        "tags": ["Exports"],
        "summary": "List Export Jobs",
        "security": [{ "BearerAuth": [] }],
        "responses": {
          "200": { "description": "History of generated export jobs" }
        }
      }
    },
    "/admin/exports/bulk": {
      "post": {
        "tags": ["Exports"],
        "summary": "Create Bulk Export Job",
        "security": [{ "BearerAuth": [] }],
        "requestBody": {
          "required": true,
          "content": {
            "application/json": {
              "schema": {
                "type": "object",
                "required": ["entity_types"],
                "properties": {
                  "entity_types": {
                    "type": "array",
                    "items": { "type": "string" },
                    "example": ["transactions", "settlements"]
                  },
                  "format": { "type": "string", "example": "csv" },
                  "merchant_filter": { "type": "string" }
                }
              }
            }
          }
        },
        "responses": {
          "200": { "description": "Job queued with job_id" }
        }
      }
    },
    "/admin/exports/{id}/download": {
      "get": {
        "tags": ["Exports"],
        "summary": "Get Export Download URL",
        "security": [{ "BearerAuth": [] }],
        "parameters": [
          { "name": "id", "in": "path", "required": true, "schema": { "type": "string", "format": "uuid" } }
        ],
        "responses": {
          "200": { "description": "Pre-signed or local download URL" },
          "400": { "description": "Export file not ready yet" },
          "404": { "description": "Export job not found" }
        }
      }
    }
  },
  "components": {
    "securitySchemes": {
      "BearerAuth": {
        "type": "http",
        "scheme": "bearer",
        "bearerFormat": "JWT"
      }
    }
  }
}"#;

pub async fn admin_openapi_json() -> impl Responder {
    HttpResponse::Ok()
        .content_type("application/json")
        .body(SPEC)
}
