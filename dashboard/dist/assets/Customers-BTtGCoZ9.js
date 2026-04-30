import{c as T,r as t,a as n,j as e,U as X,C as L}from"./index-DmZi9o4B.js";import{D as B}from"./download-BcZMe9fw.js";import{P as F}from"./plus-Bj7MaHiu.js";import{S as H}from"./search-CPXQdOII.js";/**
 * @license lucide-react v0.469.0 - ISC
 *
 * This source code is licensed under the ISC license.
 * See the LICENSE file in the root directory of this source tree.
 */const K=T("Mail",[["rect",{width:"20",height:"16",x:"2",y:"4",rx:"2",key:"18n3k1"}],["path",{d:"m22 7-8.97 5.7a1.94 1.94 0 0 1-2.06 0L2 7",key:"1ocrg3"}]]);/**
 * @license lucide-react v0.469.0 - ISC
 *
 * This source code is licensed under the ISC license.
 * See the LICENSE file in the root directory of this source tree.
 */const Q=T("Phone",[["path",{d:"M22 16.92v3a2 2 0 0 1-2.18 2 19.79 19.79 0 0 1-8.63-3.07 19.5 19.5 0 0 1-6-6 19.79 19.79 0 0 1-3.07-8.67A2 2 0 0 1 4.11 2h3a2 2 0 0 1 2 1.72 12.84 12.84 0 0 0 .7 2.81 2 2 0 0 1-.45 2.11L8.09 9.91a16 16 0 0 0 6 6l1.27-1.27a2 2 0 0 1 2.11-.45 12.84 12.84 0 0 0 2.81.7A2 2 0 0 1 22 16.92z",key:"foiqr5"}]]);function te(){const[h,A]=t.useState([]),[D,M]=t.useState(!0),[l,R]=t.useState(""),[I,i]=t.useState(!1),[m,p]=t.useState(null),[f,$]=t.useState([]),[g,j]=t.useState(!1),[b,U]=t.useState("csv"),[v,a]=t.useState(""),d=(()=>{const s=localStorage.getItem("feelpay_merchant");if(!s)return"";try{return JSON.parse(s).id||""}catch{return""}})(),[o,V]=t.useState("email"),[c,y]=t.useState(""),[N,w]=t.useState(""),[C,k]=t.useState(""),[S,_]=t.useState("");t.useEffect(()=>{u()},[]);const u=async()=>{try{const s=await n("GET","/v1/customers");A(s.data||[])}catch(s){console.error("Failed to fetch customers:",s)}finally{M(!1)}},q=async s=>{try{const r=await n("GET",`/v1/customers/${s}/payment-methods`);$(r||[])}catch(r){console.error("Failed to fetch payment methods:",r)}},O=async()=>{if(a(""),!c.trim()){a("Reference value required");return}if(o==="email"&&!/^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(c.trim())){a("Enter valid email reference");return}if(o==="phone"&&!/^\+?[0-9\s-]{8,20}$/.test(c.trim())){a("Enter valid phone reference");return}try{await n("POST","/v1/customers",{customer_reference:c.trim(),customer_reference_type:o,name:N||void 0,email:C||void 0,phone:S||void 0}),i(!1),y(""),w(""),k(""),_(""),a(""),u()}catch(s){console.error("Failed to create customer:",s),a(s instanceof Error?s.message:"Failed to create customer")}},G=async s=>{if(confirm("Are you sure you want to delete this customer?"))try{await n("DELETE",`/v1/customers/${s}`),u()}catch(r){console.error("Failed to delete customer:",r)}},Y=async()=>{if(d){j(!0);try{const s=await n("POST",`/v1/merchants/${d}/exports/customers`,{format:b});window.location.href=`/v1/merchants/${d}/exports/${s.job_id}/file`}catch(s){console.error("Failed to export customers:",s)}finally{j(!1)}}},J=s=>new Date(s).toLocaleDateString("en-IN",{day:"numeric",month:"short",year:"numeric"}),x=h.filter(s=>{var E,z,P;if(!l)return!0;const r=l.toLowerCase();return s.customer_reference.toLowerCase().includes(r)||((E=s.name)==null?void 0:E.toLowerCase().includes(r))||((z=s.email)==null?void 0:z.toLowerCase().includes(r))||((P=s.phone)==null?void 0:P.includes(l))});return e.jsxs("div",{className:"fade-in",children:[e.jsxs("div",{className:"page-header",children:[e.jsxs("div",{children:[e.jsx("h1",{children:"Customers"}),e.jsx("p",{children:"Manage your customer database and saved payment methods"})]}),e.jsxs("div",{className:"customers-toolbar",children:[e.jsxs("select",{className:"form-input customers-toolbar-select",value:b,onChange:s=>U(s.target.value),"aria-label":"Export format",children:[e.jsx("option",{value:"csv",children:"CSV"}),e.jsx("option",{value:"excel",children:"Excel"})]}),e.jsxs("button",{className:"btn btn-secondary",onClick:Y,disabled:g||!d,children:[e.jsx(B,{size:18})," ",g?"Exporting...":"Export"]}),e.jsxs("button",{className:"btn btn-primary",onClick:()=>i(!0),children:[e.jsx(F,{size:18})," Add Customer"]})]})]}),I&&e.jsxs("div",{className:"card customers-create-card",children:[e.jsxs("div",{className:"customers-create-header",children:[e.jsx("h3",{children:"Add Customer"}),e.jsx("p",{children:"Use strong reference values to dedupe and track returning buyers."})]}),v&&e.jsx("div",{className:"customers-create-error",children:v}),e.jsxs("div",{className:"customers-form-grid",children:[e.jsxs("div",{className:"customers-field",children:[e.jsx("label",{className:"form-label",children:"Reference Type"}),e.jsxs("select",{className:"form-input",value:o,onChange:s=>V(s.target.value),children:[e.jsx("option",{value:"email",children:"Email"}),e.jsx("option",{value:"phone",children:"Phone"}),e.jsx("option",{value:"external_id",children:"External ID"})]})]}),e.jsxs("div",{className:"customers-field",children:[e.jsx("label",{className:"form-label",children:"Reference Value"}),e.jsx("input",{type:"text",className:"form-input",value:c,onChange:s=>y(s.target.value),placeholder:o==="email"?"customer@example.com":o==="phone"?"+91 98765 43210":"EXT-1001"})]}),e.jsxs("div",{className:"customers-field",children:[e.jsx("label",{className:"form-label",children:"Name (optional)"}),e.jsx("input",{type:"text",className:"form-input",value:N,onChange:s=>w(s.target.value),placeholder:"Customer name"})]}),e.jsxs("div",{className:"customers-field",children:[e.jsx("label",{className:"form-label",children:"Email (optional)"}),e.jsx("input",{type:"email",className:"form-input",value:C,onChange:s=>k(s.target.value),placeholder:"customer@example.com"})]}),e.jsxs("div",{className:"customers-field customers-span-2",children:[e.jsx("label",{className:"form-label",children:"Phone (optional)"}),e.jsx("input",{type:"tel",className:"form-input",value:S,onChange:s=>_(s.target.value),placeholder:"+91 98765 43210"})]})]}),e.jsxs("div",{className:"customers-form-actions",children:[e.jsx("button",{className:"btn btn-primary",onClick:O,children:"Add Customer"}),e.jsx("button",{className:"btn btn-secondary",onClick:()=>i(!1),children:"Cancel"})]})]}),e.jsx("div",{className:"card customers-search-card",children:e.jsxs("div",{className:"customers-search-wrap",children:[e.jsx(H,{size:18,className:"customers-search-icon"}),e.jsx("input",{type:"text",className:"form-input",placeholder:"Search customers by reference, name, email, phone",value:l,onChange:s=>R(s.target.value)})]})}),D?e.jsx("div",{className:"card",children:"Loading..."}):x.length===0?e.jsxs("div",{className:"card customers-empty-state",children:[e.jsx("div",{className:"customers-empty-icon",children:e.jsx(X,{size:42,style:{color:"var(--fp-primary-soft)"}})}),e.jsx("h3",{children:"No Customers Yet"}),e.jsx("p",{children:"Add your first customer to start tracking payment methods and repeat payments."}),e.jsxs("button",{className:"btn btn-primary",onClick:()=>i(!0),children:[e.jsx(F,{size:18})," Add Customer"]})]}):e.jsxs("div",{className:"card customers-list-card",children:[e.jsxs("div",{className:"customers-list-header",children:[e.jsx("h3",{children:"Customer Records"}),e.jsxs("div",{className:"customers-list-stats",children:[e.jsxs("span",{children:[x.length," shown"]}),e.jsxs("span",{children:[h.length," total"]})]})]}),e.jsxs("table",{className:"data-table",children:[e.jsx("thead",{children:e.jsxs("tr",{children:[e.jsx("th",{children:"Customer"}),e.jsx("th",{children:"Contact"}),e.jsx("th",{children:"Reference"}),e.jsx("th",{children:"Created"}),e.jsx("th",{children:"Actions"})]})}),e.jsx("tbody",{children:x.map(s=>e.jsxs("tr",{children:[e.jsxs("td",{children:[e.jsx("div",{className:"customers-name",children:s.name||"N/A"}),e.jsxs("div",{className:"customers-sub",children:["ID: ",s.id.slice(0,8),"..."]})]}),e.jsxs("td",{children:[s.email&&e.jsxs("div",{className:"customers-contact-row",children:[e.jsx(K,{size:14,className:"customers-contact-ico"}),e.jsx("span",{children:s.email})]}),s.phone&&e.jsxs("div",{className:"customers-contact-row",children:[e.jsx(Q,{size:14,className:"customers-contact-ico"}),e.jsx("span",{children:s.phone})]}),!s.email&&!s.phone&&"-"]}),e.jsxs("td",{children:[e.jsx("span",{className:"badge",children:s.customer_reference_type}),e.jsx("div",{className:"customers-sub",style:{marginTop:4},children:s.customer_reference})]}),e.jsx("td",{className:"customers-date",children:J(s.created_at)}),e.jsx("td",{children:e.jsxs("div",{className:"customers-action-row",children:[e.jsxs("button",{className:"btn btn-sm btn-secondary",onClick:()=>{p(s),q(s.id)},children:[e.jsx(L,{size:14})," Payment Methods"]}),e.jsx("button",{className:"btn btn-sm btn-danger",onClick:()=>G(s.id),children:"Delete"})]})})]},s.id))})]})]}),m&&e.jsx("div",{className:"customers-modal-overlay",onClick:()=>p(null),children:e.jsxs("div",{className:"card customers-modal",onClick:s=>s.stopPropagation(),children:[e.jsxs("h3",{className:"customers-modal-title",children:["Payment Methods - ",m.name||m.customer_reference]}),f.length===0?e.jsx("p",{className:"customers-modal-empty",children:"No saved payment methods"}):e.jsx("div",{className:"customers-pm-list",children:f.map(s=>e.jsxs("div",{className:"customers-pm-item",children:[e.jsx(L,{size:24,style:{color:"var(--fp-primary)"}}),e.jsxs("div",{style:{flex:1},children:[s.method_type==="card"&&e.jsxs(e.Fragment,{children:[e.jsxs("div",{className:"customers-name",children:[s.card_brand," •••• ",s.card_last4]}),e.jsxs("div",{className:"customers-sub",children:["Expires ",s.card_expiry_month,"/",s.card_expiry_year]})]}),s.method_type==="upi"&&e.jsxs(e.Fragment,{children:[e.jsx("div",{className:"customers-name",children:"UPI"}),e.jsx("div",{className:"customers-sub",children:s.vpa})]}),s.method_type==="bank_account"&&e.jsxs(e.Fragment,{children:[e.jsx("div",{className:"customers-name",children:s.bank_name}),e.jsxs("div",{className:"customers-sub",children:["•••• ",s.bank_account_last4]})]})]}),s.is_default&&e.jsx("span",{className:"badge success",children:"Default"})]},s.id))}),e.jsx("button",{className:"btn btn-secondary",style:{marginTop:16,width:"100%"},onClick:()=>p(null),children:"Close"})]})}),e.jsx("style",{children:`
        .customers-toolbar {
          display: flex;
          gap: 12px;
          align-items: center;
          flex-wrap: wrap;
        }

        .customers-toolbar-select {
          width: 140px;
        }

        .customers-create-card {
          margin-bottom: 24px;
          border: 1px solid var(--fp-border);
          background: #fff9ec;
        }

        .customers-create-header h3 {
          font-size: 18px;
          margin-bottom: 6px;
        }

        .customers-create-header p {
          color: var(--fp-text-muted);
          margin-bottom: 16px;
          font-size: 14px;
        }

        .customers-create-error {
          margin: 0 0 14px;
          padding: 10px 12px;
          border-radius: 10px;
          font-size: 13px;
          border: 1px solid color-mix(in srgb, #ef4444 55%, transparent);
          background: rgba(200, 31, 47, 0.1);
          color: #991627;
        }

        .customers-form-grid {
          display: grid;
          grid-template-columns: repeat(2, minmax(0, 1fr));
          gap: 16px;
        }

        .customers-field {
          display: flex;
          flex-direction: column;
          gap: 8px;
          min-width: 0;
        }

        .customers-field .form-label {
          margin: 0;
          font-size: 13px;
          font-weight: 600;
          color: var(--fp-text-muted);
        }

        .customers-field .form-input,
        .customers-toolbar-select,
        .customers-search-wrap .form-input {
          height: 42px;
          width: 100%;
          min-width: 0;
          padding: 0 12px;
          font-size: 14px;
          border-radius: 999px;
          border: 1px solid var(--fp-border);
          background: #fff9ec;
          color: var(--fp-text);
        }

        .customers-field .form-input::placeholder,
        .customers-search-wrap .form-input::placeholder {
          color: color-mix(in srgb, var(--fp-text-muted) 88%, #ffffff 12%);
        }

        .customers-field .form-input:focus,
        .customers-toolbar-select:focus,
        .customers-search-wrap .form-input:focus {
          border-color: var(--fp-cta);
          box-shadow: 0 0 0 3px rgba(235, 0, 27, 0.14);
        }

        .customers-field select.form-input,
        .customers-toolbar-select {
          appearance: none;
          background-image:
            linear-gradient(45deg, transparent 50%, #8a857d 50%),
            linear-gradient(135deg, #8a857d 50%, transparent 50%);
          background-position:
            calc(100% - 16px) calc(50% + 1px),
            calc(100% - 11px) calc(50% + 1px);
          background-size: 5px 5px, 5px 5px;
          background-repeat: no-repeat;
          padding-right: 30px;
        }

        .customers-span-2 {
          grid-column: span 2;
        }

        .customers-form-actions {
          display: flex;
          gap: 12px;
          margin-top: 20px;
        }

        .customers-search-card {
          margin-bottom: 24px;
          padding: 14px;
        }

        .customers-search-wrap {
          position: relative;
        }

        .customers-search-icon {
          position: absolute;
          left: 12px;
          top: 50%;
          transform: translateY(-50%);
          color: var(--fp-text-dim);
          pointer-events: none;
        }

        .customers-search-wrap .form-input {
          padding-left: 40px;
        }

        .customers-empty-state {
          text-align: center;
          padding: 56px 20px;
          background: #fff9ec;
        }

        .customers-empty-icon {
          width: 74px;
          height: 74px;
          margin: 0 auto 16px;
          border-radius: 20px;
          display: grid;
          place-items: center;
          border: 1px solid var(--fp-border);
          background: #f2ead8;
        }

        .customers-empty-state h3 {
          margin-bottom: 8px;
        }

        .customers-empty-state p {
          color: var(--fp-text-muted);
          margin-bottom: 22px;
        }

        .customers-list-card {
          overflow: hidden;
        }

        .customers-list-header {
          display: flex;
          justify-content: space-between;
          align-items: center;
          margin-bottom: 16px;
          gap: 10px;
        }

        .customers-list-header h3 {
          margin: 0;
          font-size: 16px;
        }

        .customers-list-stats {
          display: flex;
          gap: 8px;
        }

        .customers-list-stats span {
          padding: 5px 10px;
          border-radius: 999px;
          font-size: 12px;
          color: var(--fp-text-muted);
          border: 1px solid var(--fp-border);
          background: #f2ead8;
        }

        .customers-name {
          font-weight: 600;
          font-size: 14px;
        }

        .customers-sub,
        .customers-date {
          font-size: 12px;
          color: var(--fp-text-muted);
        }

        .customers-contact-row {
          display: flex;
          align-items: center;
          gap: 6px;
          margin-bottom: 4px;
          font-size: 13px;
        }

        .customers-contact-ico {
          color: var(--fp-text-dim);
        }

        .customers-action-row {
          display: flex;
          gap: 8px;
          flex-wrap: wrap;
        }

        .customers-modal-overlay {
          position: fixed;
          inset: 0;
          background: rgba(0, 0, 0, 0.5);
          display: flex;
          align-items: center;
          justify-content: center;
          z-index: 1000;
        }

        .customers-modal {
          width: 520px;
          max-height: 80vh;
          overflow: auto;
        }

        .customers-modal-title {
          font-size: 16px;
          font-weight: 600;
          margin-bottom: 20px;
        }

        .customers-modal-empty {
          color: var(--fp-text-muted);
          text-align: center;
          padding: 24px;
        }

        .customers-pm-list {
          display: flex;
          flex-direction: column;
          gap: 12px;
        }

        .customers-pm-item {
          padding: 12px;
          border: 1px solid var(--fp-border);
          border-radius: 8px;
          display: flex;
          align-items: center;
          gap: 12px;
        }

        @media (max-width: 960px) {
          .customers-form-grid {
            grid-template-columns: 1fr;
          }

          .customers-span-2 {
            grid-column: span 1;
          }

          .customers-list-header {
            flex-direction: column;
            align-items: flex-start;
          }

          .customers-modal {
            width: min(92vw, 520px);
          }
        }
      `})]})}export{te as default};
