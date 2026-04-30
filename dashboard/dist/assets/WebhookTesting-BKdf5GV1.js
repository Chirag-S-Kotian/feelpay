import{c as T,r as n,a as u,j as e}from"./index-DmZi9o4B.js";import{C as g}from"./circle-alert-BT9_AK1r.js";import{E}from"./eye-t5k_4cSB.js";import{C}from"./clock-D8-t-m5_.js";import{C as z}from"./circle-check-big-3eDQ9hXq.js";/**
 * @license lucide-react v0.469.0 - ISC
 *
 * This source code is licensed under the ISC license.
 * See the LICENSE file in the root directory of this source tree.
 */const P=T("Send",[["path",{d:"M14.536 21.686a.5.5 0 0 0 .937-.024l6.5-19a.496.496 0 0 0-.635-.635l-19 6.5a.5.5 0 0 0-.024.937l7.93 3.18a2 2 0 0 1 1.112 1.11z",key:"1ffxy3"}],["path",{d:"m21.854 2.147-10.94 10.939",key:"12cjpa"}]]),R=["payment.success","payment.failed","refund.initiated","settlement.processed","dispute.created"];function $(){const[i,f]=n.useState([]),[t,p]=n.useState(null),[r,j]=n.useState("payment.success"),[b,y]=n.useState(!0),[h,x]=n.useState(!1),[a,d]=n.useState(null),[v,l]=n.useState(!1),[c,w]=n.useState(null),m=(()=>{const s=localStorage.getItem("feelpay_merchant");if(!s)return"";try{return JSON.parse(s).id||""}catch{return""}})();n.useEffect(()=>{N()},[]);const N=async()=>{try{const o=(await u("GET","/v1/developers/webhooks")).webhooks||[];f(o),p(o[0]??null)}catch(s){console.error("Failed to fetch webhook endpoints:",s)}finally{y(!1)}},k=async()=>{if(t){x(!0),d(null);try{const s=await u("POST",`/v1/merchants/${m}/webhooks/test`,{event_type:r,webhook_endpoint_id:t.id});d(s)}catch(s){d({test_id:"error",status:"failed",error_message:s instanceof Error?s.message:"Test failed"})}finally{x(!1)}}},S=async()=>{try{w({event_type:r,payload:{id:`evt_${Date.now()}`,created_at:new Date().toISOString(),merchant_id:m,event_type:r}}),l(!0)}catch(s){console.error("Failed to fetch sample payload:",s)}};return b?e.jsx("div",{className:"page-container",children:e.jsx("div",{className:"loading",children:"Loading webhook configuration..."})}):i.length===0?e.jsxs("div",{className:"page-container",children:[e.jsxs("div",{className:"page-header",children:[e.jsx("h1",{children:"Webhook Testing"}),e.jsx("p",{children:"Test your webhook configuration"})]}),e.jsx("div",{className:"card",children:e.jsxs("div",{className:"empty-state",children:[e.jsx(g,{size:32}),e.jsx("p",{children:"No webhook endpoints configured"}),e.jsx("p",{className:"text-secondary",children:"Configure a webhook endpoint in Developers settings first"})]})})]}):e.jsxs("div",{className:"page-container",children:[e.jsxs("div",{className:"page-header",children:[e.jsx("h1",{children:"Webhook Testing"}),e.jsx("p",{children:"Send test events to verify your webhook configuration"})]}),e.jsxs("div",{className:"grid grid-2",children:[e.jsxs("div",{className:"card",children:[e.jsx("h2",{children:"Test Configuration"}),e.jsxs("div",{className:"form-group",children:[e.jsx("label",{children:"Webhook Endpoint"}),e.jsx("select",{value:(t==null?void 0:t.id)||"",onChange:s=>{const o=i.find(_=>_.id===s.target.value);p(o||null)},children:i.map(s=>e.jsx("option",{value:s.id,children:s.url},s.id))}),t&&e.jsxs("div",{className:"endpoint-info",children:[e.jsxs("div",{className:"info-row",children:[e.jsx("span",{children:"Status:"}),e.jsx("span",{className:`badge badge-${t.status}`,children:t.status})]}),e.jsxs("div",{className:"info-row",children:[e.jsx("span",{children:"Subscribed Events:"}),e.jsx("span",{children:t.event_subscriptions.length})]})]})]}),e.jsxs("div",{className:"form-group",children:[e.jsx("label",{children:"Event Type"}),e.jsx("select",{value:r,onChange:s=>j(s.target.value),children:R.map(s=>e.jsx("option",{value:s,children:s},s))})]}),e.jsxs("div",{className:"form-actions",children:[e.jsxs("button",{className:"btn-secondary",onClick:S,children:[e.jsx(E,{size:16}),"View Sample Payload"]}),e.jsx("button",{className:"btn-primary",onClick:k,disabled:h,children:h?e.jsxs(e.Fragment,{children:[e.jsx(C,{size:16,className:"spin"}),"Sending..."]}):e.jsxs(e.Fragment,{children:[e.jsx(P,{size:16}),"Send Test Event"]})})]})]}),a&&e.jsxs("div",{className:"card",children:[e.jsx("h2",{children:"Test Result"}),e.jsxs("div",{className:`result-container result-${a.status}`,children:[a.status==="sent"?e.jsx(z,{size:32}):e.jsx(g,{size:32}),e.jsx("div",{className:"result-status",children:a.status==="sent"?"Success":"Failed"}),a.http_status&&e.jsxs("div",{className:"result-row",children:[e.jsx("span",{children:"HTTP Status:"}),e.jsx("strong",{children:a.http_status})]}),a.response_time_ms&&e.jsxs("div",{className:"result-row",children:[e.jsx("span",{children:"Response Time:"}),e.jsxs("strong",{children:[a.response_time_ms,"ms"]})]}),a.response_body&&e.jsxs("div",{className:"result-row",children:[e.jsx("span",{children:"Response Body:"}),e.jsx("pre",{className:"response-body",children:a.response_body})]}),a.error_message&&e.jsxs("div",{className:"result-row",children:[e.jsx("span",{children:"Error:"}),e.jsx("span",{className:"error-text",children:a.error_message})]}),e.jsxs("div",{className:"result-row",children:[e.jsx("span",{children:"Test ID:"}),e.jsx("span",{className:"mono",children:a.test_id})]})]})]})]}),v&&c&&e.jsx("div",{className:"modal-overlay",onClick:()=>l(!1),children:e.jsxs("div",{className:"modal-content",onClick:s=>s.stopPropagation(),children:[e.jsxs("div",{className:"modal-header",children:[e.jsxs("h2",{children:["Sample Payload: ",c.event_type]}),e.jsx("button",{className:"btn-close",onClick:()=>l(!1),children:"✕"})]}),e.jsx("div",{className:"modal-body",children:e.jsx("pre",{className:"payload-json",children:JSON.stringify(c.payload,null,2)})})]})}),e.jsx("style",{children:`
        .grid-2 {
          display: grid;
          grid-template-columns: 1fr 1fr;
          gap: 24px;
        }
        @media (max-width: 1024px) {
          .grid-2 {
            grid-template-columns: 1fr;
          }
        }
        .endpoint-info {
          margin-top: 12px;
          padding: 12px;
          background: var(--bg-secondary);
          border-radius: 4px;
          font-size: 13px;
        }
        .info-row {
          display: flex;
          justify-content: space-between;
          padding: 4px 0;
        }
        .result-container {
          display: flex;
          flex-direction: column;
          align-items: center;
          gap: 16px;
          padding: 20px;
          border-radius: 8px;
          text-align: center;
        }
        .result-container.result-sent {
          background: #f0fdf4;
          color: #166534;
        }
        .result-container.result-failed {
          background: #fef2f2;
          color: #991b1b;
        }
        .result-status {
          font-size: 18px;
          font-weight: 600;
        }
        .result-row {
          width: 100%;
          text-align: left;
          padding: 8px 0;
          border-bottom: 1px solid rgba(0, 0, 0, 0.1);
          display: flex;
          justify-content: space-between;
          font-size: 13px;
        }
        .result-row:last-child {
          border-bottom: none;
        }
        .response-body {
          background: var(--bg-secondary);
          padding: 8px;
          border-radius: 4px;
          font-size: 11px;
          overflow-x: auto;
          max-height: 150px;
          margin: 0;
        }
        .modal-overlay {
          position: fixed;
          top: 0;
          left: 0;
          right: 0;
          bottom: 0;
          background: rgba(0, 0, 0, 0.5);
          display: flex;
          align-items: center;
          justify-content: center;
          z-index: 1000;
        }
        .modal-content {
          background: var(--bg-primary);
          border-radius: 8px;
          box-shadow: 0 10px 40px rgba(0, 0, 0, 0.2);
          max-width: 600px;
          width: 90%;
          max-height: 80vh;
          overflow-y: auto;
        }
        .modal-header {
          display: flex;
          justify-content: space-between;
          align-items: center;
          padding: 20px;
          border-bottom: 1px solid var(--border-color);
        }
        .modal-header h2 {
          margin: 0;
          font-size: 16px;
        }
        .btn-close {
          background: none;
          border: none;
          cursor: pointer;
          font-size: 20px;
          color: var(--text-secondary);
        }
        .modal-body {
          padding: 20px;
        }
        .payload-json {
          background: var(--bg-secondary);
          padding: 12px;
          border-radius: 4px;
          font-size: 12px;
          overflow-x: auto;
          margin: 0;
        }
        .spin {
          animation: spin 1s linear infinite;
        }
        @keyframes spin {
          from { transform: rotate(0deg); }
          to { transform: rotate(360deg); }
        }
      `})]})}export{$ as default};
