import{c as F,r as t,a as j,j as e,U as I}from"./index-DmZi9o4B.js";import{C as U}from"./circle-alert-BT9_AK1r.js";/**
 * @license lucide-react v0.469.0 - ISC
 *
 * This source code is licensed under the ISC license.
 * See the LICENSE file in the root directory of this source tree.
 */const z=F("Save",[["path",{d:"M15.2 3a2 2 0 0 1 1.4.6l3.8 3.8a2 2 0 0 1 .6 1.4V19a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2z",key:"1c8476"}],["path",{d:"M17 21v-7a1 1 0 0 0-1-1H8a1 1 0 0 0-1 1v7",key:"1ydtos"}],["path",{d:"M7 3v4a1 1 0 0 0 1 1h7",key:"t51u73"}]]),E=["payment.success","payment.failed","refund.initiated","refund.success","dispute.created","settlement.processed"],P=["email","sms","in_app"];function L(){const[d,y]=t.useState([]),[c,b]=t.useState(null),[p,h]=t.useState({}),[N,m]=t.useState(!0),[_,f]=t.useState(!1),[u,l]=t.useState(null),[g,x]=t.useState(!1),w=(()=>{const s=localStorage.getItem("feelpay_merchant");if(!s)return"";try{return JSON.parse(s).id||""}catch{return""}})();t.useEffect(()=>{k()},[]),t.useEffect(()=>{c&&r(c)},[c]);const k=async()=>{try{const s=await j("GET","/v1/merchants/team"),a=Array.isArray(s)?s:s.members||[];y(a);const i=a.at(0);i&&b(i.id)}catch(s){l(s instanceof Error?s.message:"Failed to load users")}finally{m(!1)}},r=async s=>{try{const a=await j("GET",`/v1/merchants/${w}/notification-preferences/user/${s}`),i={};E.forEach(v=>{var S;const M=(S=a.preferences)==null?void 0:S.find(A=>A.event_type===v);i[v]=M||{event_type:v,email_enabled:!0,sms_enabled:!1,in_app_enabled:!0}}),h(i),x(!1)}catch(a){l(a instanceof Error?a.message:"Failed to load preferences")}},n=(s,a)=>{const i=p[s];i&&(h(v=>({...v,[s]:{...i,[`${a}_enabled`]:!i[`${a}_enabled`]}})),x(!1))},o=async()=>{if(c){f(!0),l(null),x(!1);try{const s=Object.values(p);await Promise.all(s.map(a=>j("PUT",`/v1/merchants/${w}/notification-preferences/user/${c}/${a.event_type}`,{email_enabled:a.email_enabled,sms_enabled:a.sms_enabled,in_app_enabled:a.in_app_enabled}))),x(!0)}catch(s){l(s instanceof Error?s.message:"Failed to save preferences")}finally{f(!1)}}};return N?e.jsx("div",{className:"loading",children:"Loading team members..."}):d.length===0?e.jsx("div",{className:"card",children:e.jsxs("div",{className:"empty-state",children:[e.jsx(I,{size:32}),e.jsx("p",{children:"No team members found"})]})}):e.jsxs("div",{className:"card",children:[u&&e.jsxs("div",{className:"alert alert-error",children:[e.jsx(U,{size:16}),u]}),g&&e.jsx("div",{className:"alert alert-success",children:"Preferences saved successfully!"}),e.jsxs("div",{className:"user-selector",children:[e.jsx("label",{children:"Select Team Member:"}),e.jsx("select",{value:c||"",onChange:s=>b(s.target.value),children:d.map(s=>e.jsxs("option",{value:s.id,children:[s.full_name||s.name||s.email," (",s.email,")"]},s.id))})]}),c&&e.jsxs(e.Fragment,{children:[e.jsxs("div",{className:"preferences-matrix",children:[e.jsxs("div",{className:"matrix-header",children:[e.jsx("div",{className:"event-column",children:"Event Type"}),P.map(s=>e.jsx("div",{className:"channel-column",children:s==="in_app"?"In-App":s.charAt(0).toUpperCase()+s.slice(1)},s))]}),E.map(s=>e.jsxs("div",{className:"matrix-row",children:[e.jsx("div",{className:"event-column",children:e.jsx("span",{className:"event-label",children:s})}),P.map(a=>{var i;return e.jsx("div",{className:"channel-column",children:e.jsxs("label",{className:"toggle-switch",children:[e.jsx("input",{type:"checkbox",checked:!!((i=p[s])!=null&&i[`${a}_enabled`]),onChange:()=>n(s,a)}),e.jsx("span",{className:"toggle-slider"})]})},`${s}-${a}`)})]},s))]}),e.jsx("div",{className:"form-actions",children:e.jsx("button",{className:"btn-primary",onClick:o,disabled:_,children:_?"Saving...":e.jsxs(e.Fragment,{children:[e.jsx(z,{size:16}),"Save Preferences"]})})})]}),e.jsx("style",{children:`
        .user-selector {
          margin-bottom: 20px;
        }
        .user-selector label {
          display: block;
          margin-bottom: 8px;
          font-weight: 500;
        }
        .user-selector select {
          width: 100%;
          padding: 8px 12px;
          border: 1px solid var(--border-color);
          border-radius: 4px;
          font-size: 14px;
        }
        .preferences-matrix {
          border: 1px solid var(--border-color);
          border-radius: 8px;
          overflow: hidden;
          margin-bottom: 20px;
        }
        .matrix-header {
          display: grid;
          grid-template-columns: 200px repeat(3, 1fr);
          background: var(--bg-secondary);
          border-bottom: 2px solid var(--border-color);
          font-weight: 600;
          font-size: 13px;
        }
        .matrix-row {
          display: grid;
          grid-template-columns: 200px repeat(3, 1fr);
          border-bottom: 1px solid var(--border-color);
          align-items: center;
        }
        .matrix-row:last-child {
          border-bottom: none;
        }
        .event-column,
        .channel-column {
          padding: 12px 16px;
        }
        .event-label {
          font-size: 13px;
          font-family: monospace;
        }
        .channel-column {
          display: flex;
          justify-content: center;
        }
        .toggle-switch {
          position: relative;
          display: inline-block;
          width: 40px;
          height: 24px;
        }
        .toggle-switch input {
          opacity: 0;
          width: 0;
          height: 0;
        }
        .toggle-slider {
          position: absolute;
          cursor: pointer;
          top: 0;
          left: 0;
          right: 0;
          bottom: 0;
          background-color: #ccc;
          transition: 0.3s;
          border-radius: 24px;
        }
        .toggle-slider:before {
          position: absolute;
          content: "";
          height: 18px;
          width: 18px;
          left: 3px;
          bottom: 3px;
          background-color: white;
          transition: 0.3s;
          border-radius: 50%;
        }
        input:checked + .toggle-slider {
          background-color: var(--primary);
        }
        input:checked + .toggle-slider:before {
          transform: translateX(16px);
        }
      `})]})}const $=["payment.success","payment.failed","refund.initiated","refund.success","dispute.created","settlement.processed","subscription.charged","subscription.failed","kyc.approved","kyc.rejected","kyc.resubmit_required"],C=["email","sms","in_app"];function V(){const[d,y]=t.useState({}),[c,b]=t.useState(!0),[p,h]=t.useState(!1),[N,m]=t.useState(null),[_,f]=t.useState(!1),[u,l]=t.useState(!1),g=t.useMemo(()=>{const r=localStorage.getItem("feelpay_merchant");if(!r)return"";try{return JSON.parse(r).id||""}catch{return""}},[]);t.useEffect(()=>{x()},[]);const x=async()=>{if(!g){m("Merchant session missing. Login again."),b(!1);return}try{const r=await j("GET",`/v1/merchants/${g}/notification-preferences`,void 0,void 0,{cacheMs:45e3}),n={};$.forEach(o=>{var a;const s=(a=r.preferences)==null?void 0:a.find(i=>i.event_type===o);n[o]=s||{event_type:o,email_enabled:!0,sms_enabled:!1,in_app_enabled:!0}}),y(n)}catch(r){m(r instanceof Error?r.message:"Failed to load preferences")}finally{b(!1)}},w=(r,n)=>{const o=d[r];o&&(y(s=>({...s,[r]:{...o,[`${n}_enabled`]:!o[`${n}_enabled`]}})),f(!1))},k=async()=>{h(!0),m(null),f(!1);try{const r=Object.values(d);await Promise.all(r.map(n=>j("PUT",`/v1/merchants/${g}/notification-preferences/${n.event_type}`,{email_enabled:n.email_enabled,sms_enabled:n.sms_enabled,in_app_enabled:n.in_app_enabled}))),f(!0)}catch(r){m(r instanceof Error?r.message:"Failed to save preferences")}finally{h(!1)}};return c?e.jsx("div",{className:"page-container",children:e.jsx("div",{className:"loading",children:"Loading preferences..."})}):e.jsxs("div",{className:"page-container",children:[e.jsxs("div",{className:"page-header",children:[e.jsx("h1",{children:"Notification Preferences"}),e.jsx("p",{children:"Configure how you receive notifications for different events"})]}),e.jsxs("div",{className:"tabs",children:[e.jsx("button",{className:`tab ${u?"":"active"}`,onClick:()=>l(!1),children:"Merchant Preferences"}),e.jsx("button",{className:`tab ${u?"active":""}`,onClick:()=>l(!0),children:"Team Member Preferences"})]}),u?e.jsx(L,{}):e.jsxs("div",{className:"card",children:[N&&e.jsxs("div",{className:"alert alert-error",children:[e.jsx(U,{size:16}),N]}),_&&e.jsx("div",{className:"alert alert-success",children:"Preferences saved successfully!"}),e.jsxs("div",{className:"preferences-matrix",children:[e.jsxs("div",{className:"matrix-header",children:[e.jsx("div",{className:"event-column",children:"Event Type"}),C.map(r=>e.jsx("div",{className:"channel-column",children:r==="in_app"?"In-App":r.charAt(0).toUpperCase()+r.slice(1)},r))]}),$.map(r=>e.jsxs("div",{className:"matrix-row",children:[e.jsx("div",{className:"event-column",children:e.jsx("span",{className:"event-label",children:r})}),C.map(n=>{var o;return e.jsx("div",{className:"channel-column",children:e.jsxs("label",{className:"toggle-switch",children:[e.jsx("input",{type:"checkbox",checked:!!((o=d[r])!=null&&o[`${n}_enabled`]),onChange:()=>w(r,n)}),e.jsx("span",{className:"toggle-slider"})]})},`${r}-${n}`)})]},r))]}),e.jsx("div",{className:"form-actions",children:e.jsx("button",{className:"btn-primary",onClick:k,disabled:p,children:p?"Saving...":e.jsxs(e.Fragment,{children:[e.jsx(z,{size:16}),"Save Preferences"]})})})]}),e.jsx("style",{children:`
        .tabs {
          display: inline-flex;
          gap: 8px;
          margin-bottom: 14px;
          padding: 4px;
          border: 1px solid var(--fp-border);
          background: #e7decb;
          border-radius: 999px;
        }
        .tab {
          border: 1px solid transparent;
          background: transparent;
          color: var(--fp-text-muted) !important;
          border-radius: 999px;
          padding: 8px 14px;
          font-size: 12px;
          font-weight: 700;
          cursor: pointer;
        }
        .tab.active {
          border-color: var(--fp-border);
          background: #fff9ec;
          color: var(--fp-text) !important;
        }
        .card {
          border-radius: 28px;
          border: 1px solid var(--fp-border);
          background: #fff9ec;
        }
        .alert {
          display: flex;
          align-items: center;
          gap: 8px;
          border-radius: 10px;
          padding: 10px 12px;
          margin-bottom: 12px;
          font-size: 13px;
          font-weight: 600;
        }
        .alert-error {
          border: 1px solid rgba(200, 31, 47, 0.35);
          background: rgba(200, 31, 47, 0.1);
          color: #991627;
        }
        .alert-success {
          border: 1px solid rgba(52, 122, 71, 0.34);
          background: rgba(52, 122, 71, 0.1);
          color: #2f6b40;
        }
        .preferences-matrix {
          border: 1px solid var(--fp-border);
          border-radius: 20px;
          overflow: hidden;
          margin-bottom: 20px;
          background: #fffdf5;
        }
        .matrix-header {
          display: grid;
          grid-template-columns: 200px repeat(3, 1fr);
          background: #f2ead8;
          border-bottom: 1px solid var(--fp-border);
          font-weight: 600;
          font-size: 13px;
          color: var(--fp-text-muted);
        }
        .matrix-row {
          display: grid;
          grid-template-columns: 200px repeat(3, 1fr);
          border-bottom: 1px solid #ece2cc;
          align-items: center;
          background: #fff9ec;
        }
        .matrix-row:last-child {
          border-bottom: none;
        }
        .event-column,
        .channel-column {
          padding: 12px 16px;
        }
        .event-label {
          font-size: 13px;
          font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono", "Courier New", monospace;
          color: var(--fp-text);
        }
        .channel-column {
          display: flex;
          justify-content: center;
        }
        .toggle-switch {
          position: relative;
          display: inline-block;
          width: 40px;
          height: 24px;
        }
        .toggle-switch input {
          opacity: 0;
          width: 0;
          height: 0;
        }
        .toggle-slider {
          position: absolute;
          cursor: pointer;
          top: 0;
          left: 0;
          right: 0;
          bottom: 0;
          background-color: #c7baa0;
          transition: 0.3s;
          border-radius: 24px;
        }
        .toggle-slider:before {
          position: absolute;
          content: "";
          height: 18px;
          width: 18px;
          left: 3px;
          bottom: 3px;
          background-color: white;
          transition: 0.3s;
          border-radius: 50%;
        }
        input:checked + .toggle-slider {
          background: linear-gradient(135deg, #eb001b, #f57b88);
        }
        input:checked + .toggle-slider:before {
          transform: translateX(16px);
        }
        @media (max-width: 768px) {
          .matrix-header,
          .matrix-row {
            grid-template-columns: 120px repeat(3, 1fr);
          }
          .event-column,
          .channel-column {
            padding: 8px 12px;
            font-size: 12px;
          }
        }
      `})]})}export{V as default};
