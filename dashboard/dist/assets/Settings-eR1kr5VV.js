import{c as E,r as a,j as e,a as c}from"./index-DmZi9o4B.js";import{C as L,K as D,W as H}from"./webhook-CqOc0qL5.js";import{S as J}from"./shield-check-By1IlAFg.js";import{C as V}from"./copy-CrMUemfa.js";import{R as Z}from"./refresh-cw-CLAn8pyW.js";/**
 * @license lucide-react v0.469.0 - ISC
 *
 * This source code is licensed under the ISC license.
 * See the LICENSE file in the root directory of this source tree.
 */const W=E("TriangleAlert",[["path",{d:"m21.73 18-8-14a2 2 0 0 0-3.48 0l-8 14A2 2 0 0 0 4 21h16a2 2 0 0 0 1.73-3",key:"wmoenq"}],["path",{d:"M12 9v4",key:"juzpu7"}],["path",{d:"M12 17h.01",key:"p32p05"}]]);/**
 * @license lucide-react v0.469.0 - ISC
 *
 * This source code is licensed under the ISC license.
 * See the LICENSE file in the root directory of this source tree.
 */const X=E("Zap",[["path",{d:"M4 14a1 1 0 0 1-.78-1.63l9.9-10.2a.5.5 0 0 1 .86.46l-1.92 6.02A1 1 0 0 0 13 10h7a1 1 0 0 1 .78 1.63l-9.9 10.2a.5.5 0 0 1-.86-.46l1.92-6.02A1 1 0 0 0 11 14z",key:"1xq2db"}]]),$=["payment.success","payment.failed","refund.initiated","settlement.processed","dispute.created"];function ae(){var A,P,I;const[M,u]=a.useState(!0),[p,g]=a.useState(!1),[b,y]=a.useState(""),[j,v]=a.useState(""),[f,k]=a.useState(""),[w,N]=a.useState("individual"),[h,S]=a.useState(""),[n,O]=a.useState(null),[t,U]=a.useState(null),[B,_]=a.useState(null),[C,r]=a.useState(""),[z,i]=a.useState(""),R=a.useMemo(()=>{const s=localStorage.getItem("feelpay_merchant");if(!s)return"";try{return JSON.parse(s).id||""}catch{return""}},[]),m=async()=>{u(!0);try{const[s,o,l]=await Promise.all([c("GET","/v1/merchants/profile",void 0,void 0,{cacheMs:3e4}),c("GET","/v1/merchants/keys",void 0,void 0,{cacheMs:3e4}),c("GET","/v1/merchants/settings/security",void 0,void 0,{cacheMs:3e4}).then(d=>({ok:!0,value:d})).catch(d=>({ok:!1,err:d}))]);let x;if(l.ok)x=l.value,i("");else{if(!(l.err instanceof Error?l.err.message:"").includes("(404)"))throw l.err;x={mfa_enabled:!1,settlement_cycle:"T+2",per_transaction_limit:0,daily_limit:s.daily_limit||0,monthly_limit:s.monthly_limit||0,allowed_payment_methods:{},webhook_secret_hint:"unsupported_on_current_backend",updated_at:new Date().toISOString()},i("Security endpoint unavailable on running backend. Restart/update API server to enable full controls.")}O(s),U(x),y(o.api_key||""),v(s.name||""),k(s.business_name||""),N((s.business_type||"individual").toLowerCase()),S(s.webhook_url||"")}finally{u(!1)}};a.useEffect(()=>{m()},[]);const K=(s,o)=>{navigator.clipboard.writeText(s),_(o),setTimeout(()=>_(null),1600)},F=async()=>{r(""),await c("POST","/v1/merchants/keys/roll",{});const s=await c("GET","/v1/merchants/keys");y(s.api_key),r("API keys rotated. Update server env immediately.")},q=async()=>{r(""),i("");try{await c("POST","/v1/merchants/webhook-secret/roll",{}),await m(),r("Webhook secret rotated. Update signature verification key.")}catch(s){const o=s instanceof Error?s.message:"";if(o.includes("(404)")){i("Webhook secret rotation route unavailable on running backend. Deploy/restart latest API build.");return}i(o||"Failed to rotate webhook secret.")}},T=async()=>{g(!0),r("");try{await c("PUT","/v1/merchants/profile",{name:j.trim()||null,business_name:f.trim()||null,business_type:w,webhook_url:h.trim()||null}),await m(),r("Settings saved.")}finally{g(!1)}},G=async()=>{if(!R){r("Merchant session missing. Re-login required.");return}const s=h.trim();if(!s){i("Set and save a webhook URL before sending a test event.");return}if(!s.startsWith("https://")){i("Webhook URL must start with https:// for test delivery.");return}r("");try{await c("POST",`/v1/merchants/${R}/webhooks/test`,{event_type:"payment.success",webhook_url:s}),r("Test webhook sent for payment.success."),i("")}catch(o){const l=o instanceof Error?o.message:"";if(l.toLowerCase().includes("not implemented")||l.includes("(501)")){i("Test webhook route exists but server still running old/stub implementation. Restart API with latest code.");return}i(l||"Failed to send test webhook.")}};return M?e.jsx("div",{className:"loading",children:"Loading settings..."}):e.jsxs("div",{className:"page",children:[e.jsx("div",{className:"page-header",children:e.jsxs("div",{children:[e.jsx("h1",{children:"Gateway Settings"}),e.jsx("p",{children:"Production controls for identity, API auth, webhooks, and operational safety."})]})}),C&&e.jsx("div",{className:"card",style:{marginBottom:16,borderColor:"rgba(88, 174, 116, 0.42)"},children:e.jsxs("div",{style:{display:"flex",alignItems:"center",gap:8,color:"#97efb2"},children:[e.jsx(L,{size:16}),e.jsx("span",{children:C})]})}),z&&e.jsx("div",{className:"card",style:{marginBottom:16,borderColor:"rgba(245, 158, 11, 0.42)"},children:e.jsxs("div",{style:{display:"flex",alignItems:"center",gap:8,color:"#f8c786"},children:[e.jsx(W,{size:16}),e.jsx("span",{children:z})]})}),e.jsxs("div",{className:"settings-grid",children:[e.jsxs("section",{className:"card",children:[e.jsxs("h3",{className:"section-title",children:[e.jsx(J,{size:17})," Business Identity"]}),e.jsxs("div",{className:"input-group",children:[e.jsx("label",{children:"Owner / Account Name"}),e.jsx("input",{value:j,onChange:s=>v(s.target.value),placeholder:"Merchant account name"})]}),e.jsxs("div",{className:"input-group",children:[e.jsx("label",{children:"Business Name"}),e.jsx("input",{value:f,onChange:s=>k(s.target.value),placeholder:"Legal business name"})]}),e.jsxs("div",{className:"input-group",children:[e.jsx("label",{children:"Business Type"}),e.jsxs("select",{value:w,onChange:s=>N(s.target.value),children:[e.jsx("option",{value:"individual",children:"Individual"}),e.jsx("option",{value:"sole_proprietorship",children:"Sole Proprietorship"}),e.jsx("option",{value:"partnership",children:"Partnership"}),e.jsx("option",{value:"corporation",children:"Corporation"}),e.jsx("option",{value:"llc",children:"LLC"})]})]}),e.jsxs("div",{className:"input-group",children:[e.jsx("label",{children:"Email"}),e.jsx("input",{value:(n==null?void 0:n.email)||"",readOnly:!0})]}),e.jsx("button",{className:"btn-primary",onClick:()=>void T(),disabled:p,children:p?"Saving...":"Save Business Settings"})]}),e.jsxs("section",{className:"card",children:[e.jsxs("h3",{className:"section-title",children:[e.jsx(D,{size:17})," API Authentication"]}),e.jsxs("div",{className:"input-group",children:[e.jsx("label",{children:"API Key"}),e.jsxs("div",{className:"key-row",children:[e.jsx("input",{value:b,readOnly:!0,style:{fontFamily:"Fira Code, monospace",fontSize:12}}),e.jsx("button",{className:"btn-secondary",onClick:()=>K(b,"api"),children:B==="api"?e.jsx(L,{size:14}):e.jsx(V,{size:14})})]})]}),e.jsxs("div",{className:"security-list",children:[e.jsxs("div",{className:"security-row",children:[e.jsx("span",{children:"MFA status"}),e.jsx("strong",{children:t!=null&&t.mfa_enabled?"Enabled":"Disabled"})]}),e.jsxs("div",{className:"security-row",children:[e.jsx("span",{children:"Merchant status"}),e.jsx("strong",{children:(n==null?void 0:n.status)||"unknown"})]}),e.jsxs("div",{className:"security-row",children:[e.jsx("span",{children:"Onboarding"}),e.jsx("strong",{children:(n==null?void 0:n.onboarding_status)||"unknown"})]})]}),e.jsxs("button",{className:"btn-secondary",onClick:()=>void F(),children:[e.jsx(Z,{size:16})," Rotate API Keys"]}),e.jsx("p",{className:"mini-help",children:"Rotation invalidates prior secret. Roll one deployment window, not random time."})]})]}),e.jsxs("div",{className:"settings-grid",style:{marginTop:16},children:[e.jsxs("section",{className:"card",children:[e.jsxs("h3",{className:"section-title",children:[e.jsx(H,{size:17})," Webhook Security"]}),e.jsxs("div",{className:"input-group",children:[e.jsx("label",{children:"Webhook URL"}),e.jsx("input",{type:"url",value:h,onChange:s=>S(s.target.value),placeholder:"https://merchant.example/webhooks/feelpay"})]}),e.jsxs("div",{className:"input-group",children:[e.jsx("label",{children:"Webhook Secret Hint"}),e.jsx("input",{value:(t==null?void 0:t.webhook_secret_hint)||"not_set",readOnly:!0})]}),e.jsx("div",{className:"webhook-events",children:$.map(s=>e.jsx("div",{className:"event-chip",children:s},s))}),e.jsxs("div",{className:"settings-actions",children:[e.jsx("button",{className:"btn-primary",onClick:()=>void T(),disabled:p,children:"Save Webhook URL"}),e.jsx("button",{className:"btn-secondary",onClick:()=>void q(),children:"Rotate Webhook Secret"}),e.jsx("button",{className:"btn-secondary",onClick:()=>void G(),children:"Send Test Webhook"})]}),e.jsx("p",{className:"mini-help",children:"Enforce `https`. Verify `X-FeelPay-Signature` against raw body with HMAC-SHA256."})]}),e.jsxs("section",{className:"card",children:[e.jsxs("h3",{className:"section-title",children:[e.jsx(X,{size:17})," Operational Limits"]}),e.jsxs("div",{className:"ops-grid",children:[e.jsxs("div",{className:"ops-tile",children:[e.jsx("span",{children:"Per transaction"}),e.jsxs("strong",{children:["INR ",((A=t==null?void 0:t.per_transaction_limit)==null?void 0:A.toLocaleString())||0]})]}),e.jsxs("div",{className:"ops-tile",children:[e.jsx("span",{children:"Daily limit"}),e.jsxs("strong",{children:["INR ",((P=t==null?void 0:t.daily_limit)==null?void 0:P.toLocaleString())||0]})]}),e.jsxs("div",{className:"ops-tile",children:[e.jsx("span",{children:"Monthly limit"}),e.jsxs("strong",{children:["INR ",((I=t==null?void 0:t.monthly_limit)==null?void 0:I.toLocaleString())||0]})]}),e.jsxs("div",{className:"ops-tile",children:[e.jsx("span",{children:"Settlement cycle"}),e.jsx("strong",{children:(t==null?void 0:t.settlement_cycle)||"T+2"})]})]}),e.jsxs("div",{className:"method-box",children:[e.jsx("label",{children:"Allowed Payment Methods JSON"}),e.jsx("pre",{children:JSON.stringify((t==null?void 0:t.allowed_payment_methods)||{},null,2)})]}),e.jsxs("div",{className:"risk-note",children:[e.jsx(W,{size:14}),e.jsx("span",{children:"Limit updates require compliance approval. Use onboarding/ops workflow for changes."})]})]})]}),e.jsx("style",{children:`
        .settings-grid {
          display: grid;
          grid-template-columns: repeat(auto-fit, minmax(360px, 1fr));
          gap: 16px;
        }
        .section-title {
          font-size: 16px;
          margin-bottom: 14px;
          display: inline-flex;
          align-items: center;
          gap: 8px;
        }
        .key-row {
          display: flex;
          gap: 8px;
        }
        .key-row .btn-secondary {
          min-width: 42px;
          display: inline-flex;
          justify-content: center;
          align-items: center;
        }
        .security-list {
          border: 1px solid rgba(103, 166, 255, 0.22);
          border-radius: 10px;
          padding: 10px;
          margin: 12px 0;
          background: rgba(14, 34, 63, 0.56);
        }
        .security-row {
          display: flex;
          justify-content: space-between;
          padding: 7px 0;
          font-size: 13px;
          border-bottom: 1px solid rgba(100, 158, 245, 0.14);
        }
        .security-row:last-child {
          border-bottom: none;
        }
        .webhook-events {
          display: flex;
          flex-wrap: wrap;
          gap: 8px;
          margin: 10px 0 12px;
        }
        .event-chip {
          font-size: 12px;
          border: 1px solid rgba(99, 164, 253, 0.28);
          background: rgba(14, 33, 60, 0.72);
          border-radius: 999px;
          padding: 5px 9px;
          color: #c8ddfa;
        }
        .settings-actions {
          display: flex;
          gap: 10px;
          flex-wrap: wrap;
        }
        .ops-grid {
          display: grid;
          grid-template-columns: repeat(2, minmax(0, 1fr));
          gap: 10px;
          margin-bottom: 12px;
        }
        .ops-tile {
          border: 1px solid rgba(98, 163, 253, 0.26);
          border-radius: 10px;
          padding: 10px;
          background: rgba(13, 34, 62, 0.6);
          display: flex;
          flex-direction: column;
          gap: 4px;
          font-size: 12px;
          color: #aac7ea;
        }
        .ops-tile strong {
          font-size: 14px;
          color: #eef5ff;
        }
        .method-box label {
          font-size: 12px;
          color: #b6cff0;
          display: block;
          margin-bottom: 6px;
        }
        .method-box pre {
          margin: 0;
          background: rgba(9, 22, 42, 0.84);
          border: 1px solid rgba(102, 164, 255, 0.22);
          border-radius: 10px;
          padding: 10px;
          max-height: 170px;
          overflow: auto;
          color: #d2e6ff;
          font-size: 12px;
        }
        .risk-note {
          margin-top: 10px;
          display: inline-flex;
          align-items: center;
          gap: 8px;
          color: #f7c98c;
          font-size: 12px;
          border: 1px solid rgba(245, 158, 11, 0.28);
          background: rgba(245, 158, 11, 0.1);
          border-radius: 8px;
          padding: 8px 10px;
        }
        .mini-help {
          margin-top: 8px;
          font-size: 12px;
          color: #9ebbe0;
        }
      `})]})}export{ae as default};
