import{c as j,r as n,g as S,s as w,j as e,a as u}from"./index-DmZi9o4B.js";import{K as _,C as v,W as R}from"./webhook-CqOc0qL5.js";import{C as g}from"./copy-CrMUemfa.js";import{R as z}from"./refresh-cw-CLAn8pyW.js";import{S as o}from"./shield-check-By1IlAFg.js";import{R as P}from"./rocket-CrF7_YKj.js";/**
 * @license lucide-react v0.469.0 - ISC
 *
 * This source code is licensed under the ISC license.
 * See the LICENSE file in the root directory of this source tree.
 */const C=j("BookOpen",[["path",{d:"M12 7v14",key:"1akyts"}],["path",{d:"M3 18a1 1 0 0 1-1-1V4a1 1 0 0 1 1-1h5a4 4 0 0 1 4 4 4 4 0 0 1 4-4h5a1 1 0 0 1 1 1v13a1 1 0 0 1-1 1h-6a3 3 0 0 0-3 3 3 3 0 0 0-3-3z",key:"ruj8y"}]]);/**
 * @license lucide-react v0.469.0 - ISC
 *
 * This source code is licensed under the ISC license.
 * See the LICENSE file in the root directory of this source tree.
 */const T=j("SquareTerminal",[["path",{d:"m7 11 2-2-2-2",key:"1lz0vl"}],["path",{d:"M11 13h4",key:"1p7l4v"}],["rect",{width:"18",height:"18",x:"3",y:"3",rx:"2",ry:"2",key:"1m3agn"}]]),O={test:"https://sandbox-api.feelpay.in",live:"https://api.feelpay.in"},E=[{title:"One-time payments",desc:"Create payment, track status, capture/refund with idempotent retries.",endpoints:["POST /v1/payments","GET /v1/payments/{id}","POST /v1/payments/{id}/refund"]},{title:"Subscriptions and mandates",desc:"Recurring billing with pause/resume/cancel lifecycle APIs.",endpoints:["POST /v1/subscriptions","POST /v1/mandates","POST /v1/subscriptions/{id}/pause"]},{title:"Settlement and reconciliation",desc:"Fetch settlement list, balance, exports, and automate reporting.",endpoints:["GET /v1/settlements","GET /v1/settlements/balance","GET /v1/merchants/{id}/exports"]}],I=[{code:"400",meaning:"Invalid payload, state, or validation failed",action:"Fix request fields, retry with corrected body"},{code:"401",meaning:"Missing/invalid API key or auth context",action:"Verify `x-api-key`, auth token, and env mode"},{code:"404",meaning:"Resource/route not found",action:"Check path params, merchant scope, and endpoint version"},{code:"409",meaning:"Idempotency key reused with different payload",action:"Reuse key only for exact same body"},{code:"500",meaning:"Server/database failure",action:"Retry with same idempotency key; alert support if persistent"}];function H(){const[i,f]=n.useState("Loading..."),[a,l]=n.useState(S()),[p,m]=n.useState(null),[x,b]=n.useState("curl"),r=O[a],c="pay_01HV7M5YVQX2S2AZ8WMK6QF7SZ",h=async()=>{const s=await u("GET","/v1/merchants/keys");f(s.api_key);const t=localStorage.getItem("feelpay_merchant");if(t){const d=JSON.parse(t);d[a==="live"?"api_key_live":"api_key_test"]=s.api_key,d.api_key=s.api_key,localStorage.setItem("feelpay_merchant",JSON.stringify(d))}};n.useEffect(()=>{w(a),h()},[a]);const y=(s,t)=>{navigator.clipboard.writeText(s),m(t),setTimeout(()=>m(null),1600)},k=async()=>{await u("POST","/v1/merchants/keys/roll",{}),await h()},N=n.useMemo(()=>({curl:`curl -X POST ${r}/v1/payments \\
  -H "x-api-key: ${i}" \\
  -H "x-feelpay-environment: ${a}" \\
  -H "idempotency-key: ${c}" \\
  -H "Content-Type: application/json" \\
  -d '{
    "amount": 50000,
    "currency": "INR",
    "order_id": "ORD-2026-0001",
    "customer_email": "customer@example.com",
    "customer_phone": "+919999999999",
    "description": "Premium plan payment",
    "return_url": "https://merchant.example/return",
    "method_data": {
      "method": "card",
      "data": {
        "card_number": "4111111111111111",
        "expiry_month": 12,
        "expiry_year": 2028,
        "cvv": "123"
      }
    }
  }'`,node:`const payload = {
  amount: 50000,
  currency: 'INR',
  order_id: 'ORD-2026-0001',
  customer_email: 'customer@example.com',
  customer_phone: '+919999999999',
  description: 'Premium plan payment',
  return_url: 'https://merchant.example/return',
  method_data: {
    method: 'card',
    data: {
      card_number: '4111111111111111',
      expiry_month: 12,
      expiry_year: 2028,
      cvv: '123',
    },
  },
};

const res = await fetch('${r}/v1/payments', {
  method: 'POST',
  headers: {
    'x-api-key': '${i}',
    'x-feelpay-environment': '${a}',
    'idempotency-key': '${c}',
    'content-type': 'application/json',
  },
  body: JSON.stringify(payload),
});

console.log(res.status, await res.json());`,python:`import requests

url = "${r}/v1/payments"
headers = {
    "x-api-key": "${i}",
    "x-feelpay-environment": "${a}",
    "idempotency-key": "${c}",
    "content-type": "application/json",
}
payload = {
    "amount": 50000,
    "currency": "INR",
    "order_id": "ORD-2026-0001",
    "customer_email": "customer@example.com",
    "customer_phone": "+919999999999",
    "description": "Premium plan payment",
    "return_url": "https://merchant.example/return",
    "method_data": {
        "method": "card",
        "data": {
            "card_number": "4111111111111111",
            "expiry_month": 12,
            "expiry_year": 2028,
            "cvv": "123"
        }
    }
}

r = requests.post(url, headers=headers, json=payload, timeout=20)
print(r.status_code, r.json())`,webhook:`import crypto from 'crypto';

function verifyFeelPayWebhook(rawBody, signature, webhookSecret) {
  const expected = crypto
    .createHmac('sha256', webhookSecret)
    .update(rawBody)
    .digest('hex');

  return crypto.timingSafeEqual(
    Buffer.from(expected, 'hex'),
    Buffer.from(signature, 'hex'),
  );
}

// Verify header: X-FeelPay-Signature
// Use raw request body. Do not parse before signature check.`}),[i,r,a]);return e.jsxs("div",{className:"page",children:[e.jsxs("div",{className:"dev-hero",children:[e.jsxs("div",{children:[e.jsx("div",{className:"dev-kicker",children:"Developer Platform"}),e.jsx("h1",{children:"Build, test, ship payments fast."}),e.jsx("p",{children:"Stripe/Razorpay-style integration hub: keys, examples, use-cases, reliability and go-live controls."})]}),e.jsxs("div",{className:"mode-toggle",children:[e.jsx("button",{className:a==="test"?"active":"",onClick:()=>l("test"),children:"Test Mode"}),e.jsx("button",{className:a==="live"?"active":"",onClick:()=>l("live"),children:"Live Mode"})]})]}),e.jsxs("section",{className:"api-section",children:[e.jsxs("div",{className:"dev-section-title",children:[e.jsx(_,{size:18})," Credentials & Base URL"]}),e.jsxs("div",{className:"dev-grid",children:[e.jsxs("div",{className:"dev-card",children:[e.jsx("label",{children:"Base URL"}),e.jsxs("div",{className:"key-row",children:[e.jsx("code",{children:r}),e.jsx("button",{onClick:()=>y(r,"base"),children:p==="base"?e.jsx(v,{size:16}):e.jsx(g,{size:16})})]})]}),e.jsxs("div",{className:"dev-card",children:[e.jsx("label",{children:"API Key"}),e.jsxs("div",{className:"key-row",children:[e.jsx("code",{children:i}),e.jsx("button",{onClick:()=>y(i,"key"),children:p==="key"?e.jsx(v,{size:16}):e.jsx(g,{size:16})})]})]})]}),e.jsxs("div",{className:"dev-actions",children:[e.jsxs("button",{className:"btn-secondary",onClick:k,children:[e.jsx(z,{size:16})," Rotate API Key"]}),e.jsx("p",{className:"help-text",children:"Secret keys not re-displayed. Rotate immediately if leaked."})]})]}),e.jsxs("section",{className:"docs-section",children:[e.jsxs("div",{className:"dev-section-title",children:[e.jsx(T,{size:18})," Quickstart"]}),e.jsxs("ul",{className:"dev-bullets",children:[e.jsxs("li",{children:["Auth via ",e.jsx("code",{children:"x-api-key"}),"."]}),e.jsxs("li",{children:["POST retries must include stable ",e.jsx("code",{children:"idempotency-key"}),"."]}),e.jsxs("li",{children:["Amount in paise (",e.jsx("code",{children:"50000 = INR 500.00"}),")."]}),e.jsxs("li",{children:["Set ",e.jsx("code",{children:"x-feelpay-environment"})," = ",e.jsx("code",{children:"test"})," or ",e.jsx("code",{children:"live"}),"."]})]}),e.jsx("div",{className:"lang-tabs",children:[["curl","cURL"],["node","Node.js"],["python","Python"],["webhook","Webhook Verify"]].map(([s,t])=>e.jsx("button",{className:x===s?"active":"",onClick:()=>b(s),children:t},s))}),e.jsx("pre",{className:"code-block",children:e.jsx("code",{children:N[x]})})]}),e.jsxs("section",{className:"card",children:[e.jsxs("div",{className:"dev-section-title",children:[e.jsx(C,{size:18})," Use Cases"]}),e.jsx("div",{className:"usecase-grid",children:E.map(s=>e.jsxs("div",{className:"usecase-card",children:[e.jsx("h3",{children:s.title}),e.jsx("p",{children:s.desc}),e.jsx("div",{className:"endpoint-list",children:s.endpoints.map(t=>e.jsx("code",{children:t},t))})]},s.title))})]}),e.jsxs("section",{className:"webhooks-section",children:[e.jsxs("div",{className:"dev-section-title",children:[e.jsx(R,{size:18})," Webhooks & Security"]}),e.jsxs("div",{className:"webhook-meta",children:[e.jsxs("div",{className:"meta-item",children:[e.jsx(o,{size:16})," Verify ",e.jsx("code",{children:"X-FeelPay-Signature"})," with HMAC-SHA256"]}),e.jsxs("div",{className:"meta-item",children:[e.jsx(o,{size:16})," Store dedupe key/event id; ignore duplicate deliveries"]}),e.jsxs("div",{className:"meta-item",children:[e.jsx(o,{size:16})," Do not rely on event ordering; make handlers idempotent"]})]}),e.jsxs("div",{className:"webhook-events",children:[e.jsx("div",{className:"event",children:"payment.success"}),e.jsx("div",{className:"event",children:"payment.failed"}),e.jsx("div",{className:"event",children:"refund.initiated"}),e.jsx("div",{className:"event",children:"settlement.processed"}),e.jsx("div",{className:"event",children:"dispute.created"})]})]}),e.jsxs("section",{className:"card",children:[e.jsxs("div",{className:"dev-section-title",children:[e.jsx(P,{size:18})," Go-Live Checklist"]}),e.jsxs("div",{className:"checklist-grid",children:[e.jsx("div",{className:"check-item",children:"✅ Test mode flow validated end-to-end"}),e.jsx("div",{className:"check-item",children:"✅ Webhook signature verification implemented"}),e.jsx("div",{className:"check-item",children:"✅ Retry-safe idempotency keys wired on POSTs"}),e.jsx("div",{className:"check-item",children:"✅ Alerting configured for 4xx/5xx + webhook failures"}),e.jsx("div",{className:"check-item",children:"✅ API key rotation runbook documented"}),e.jsx("div",{className:"check-item",children:"✅ Refund/settlement reconciliation cron in place"})]})]}),e.jsxs("section",{className:"card",children:[e.jsx("div",{className:"dev-section-title",children:"Error Handling Matrix"}),e.jsx("div",{className:"error-table-wrap",children:e.jsxs("table",{className:"data-table",children:[e.jsx("thead",{children:e.jsxs("tr",{children:[e.jsx("th",{children:"Code"}),e.jsx("th",{children:"Meaning"}),e.jsx("th",{children:"Action"})]})}),e.jsx("tbody",{children:I.map(s=>e.jsxs("tr",{children:[e.jsx("td",{children:e.jsx("code",{children:s.code})}),e.jsx("td",{children:s.meaning}),e.jsx("td",{children:s.action})]},s.code))})]})})]}),e.jsx("style",{children:`
        .dev-hero {
          display: flex;
          justify-content: space-between;
          align-items: flex-start;
          gap: 20px;
          border: 1px solid rgba(108, 170, 255, 0.3);
          border-radius: 16px;
          padding: 20px;
          margin-bottom: 18px;
          background:
            radial-gradient(140% 140% at 10% 0%, rgba(89, 152, 246, 0.2), transparent 42%),
            radial-gradient(120% 130% at 100% 0%, rgba(245, 158, 11, 0.14), transparent 46%),
            linear-gradient(180deg, rgba(16, 38, 72, 0.98), rgba(10, 24, 46, 0.98));
        }
        .dev-kicker {
          font-size: 11px;
          text-transform: uppercase;
          letter-spacing: 0.15em;
          color: #9ec5ff;
          margin-bottom: 6px;
          font-weight: 700;
        }
        .dev-hero h1 {
          margin: 0;
          font-size: 32px;
          letter-spacing: -0.02em;
          line-height: 1.03;
        }
        .dev-hero p {
          margin-top: 8px;
          color: #a9c4e7;
          max-width: 680px;
        }
        .dev-section-title {
          display: inline-flex;
          align-items: center;
          gap: 8px;
          font-weight: 700;
          margin-bottom: 12px;
        }
        .dev-grid {
          display: grid;
          grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
          gap: 12px;
        }
        .dev-card {
          border: 1px solid rgba(97, 160, 252, 0.25);
          border-radius: 12px;
          padding: 12px;
          background: rgba(17, 39, 72, 0.7);
        }
        .dev-card label {
          display: block;
          margin-bottom: 8px;
          color: #b8cff0;
          font-size: 12px;
          font-weight: 700;
          text-transform: uppercase;
          letter-spacing: 0.05em;
        }
        .dev-actions {
          margin-top: 12px;
          display: flex;
          gap: 12px;
          align-items: center;
          flex-wrap: wrap;
        }
        .dev-bullets {
          margin: 8px 0 14px 18px;
          color: #c4d9f5;
          display: grid;
          gap: 6px;
        }
        .usecase-grid {
          display: grid;
          grid-template-columns: repeat(auto-fit, minmax(260px, 1fr));
          gap: 12px;
        }
        .usecase-card {
          border: 1px solid rgba(99, 161, 252, 0.26);
          background: rgba(16, 38, 70, 0.65);
          border-radius: 12px;
          padding: 14px;
        }
        .usecase-card h3 {
          margin: 0 0 6px;
          font-size: 18px;
        }
        .usecase-card p {
          margin: 0 0 10px;
          color: #acc7e9;
          font-size: 14px;
        }
        .endpoint-list {
          display: flex;
          flex-direction: column;
          gap: 7px;
        }
        .endpoint-list code {
          background: rgba(7, 19, 38, 0.75);
          border: 1px solid rgba(93, 155, 248, 0.28);
          border-radius: 8px;
          padding: 6px 8px;
          color: #d0e4ff;
          font-size: 12px;
        }
        .webhook-meta {
          display: grid;
          gap: 8px;
          margin-bottom: 12px;
        }
        .meta-item {
          display: inline-flex;
          align-items: center;
          gap: 8px;
          color: #b9d3f4;
          font-size: 14px;
        }
        .checklist-grid {
          display: grid;
          grid-template-columns: repeat(auto-fit, minmax(260px, 1fr));
          gap: 10px;
        }
        .check-item {
          border: 1px solid rgba(102, 166, 255, 0.25);
          background: rgba(13, 34, 62, 0.72);
          border-radius: 10px;
          padding: 10px 12px;
          color: #cde2ff;
          font-size: 14px;
        }
        .error-table-wrap {
          overflow-x: auto;
        }
        @media (max-width: 860px) {
          .dev-hero {
            flex-direction: column;
          }
        }
      `})]})}export{H as default};
