import{c as P,r as a,a as c,j as e,b as T}from"./index-DmZi9o4B.js";import{P as v}from"./plus-Bj7MaHiu.js";import{C as F}from"./copy-CrMUemfa.js";import{T as H}from"./trash-2-CwydIfLd.js";/**
 * @license lucide-react v0.469.0 - ISC
 *
 * This source code is licensed under the ISC license.
 * See the LICENSE file in the root directory of this source tree.
 */const q=P("ExternalLink",[["path",{d:"M15 3h6v6",key:"1q9fwt"}],["path",{d:"M10 14 21 3",key:"gplh6r"}],["path",{d:"M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6",key:"a6xqqp"}]]);function V(){const[s,N]=a.useState([]),[C,w]=a.useState(!0),[L,r]=a.useState(!1),[p,d]=a.useState(null),[i,m]=a.useState(""),[x,l]=a.useState(""),[u,h]=a.useState(""),[f,y]=a.useState(""),[g,j]=a.useState(""),[b,S]=a.useState("24");a.useEffect(()=>{o()},[]);const o=async()=>{try{const t=await c("GET","/v1/payment-links");N(t.data||[])}catch(t){console.error("Failed to fetch payment links:",t)}finally{w(!1)}},z=async()=>{if(l(""),!i||Number(i)<=0){l("Enter valid amount greater than 0");return}try{const t=new Date;t.setHours(t.getHours()+parseInt(b)),await c("POST","/v1/payment-links",{amount:parseInt(i)*100,description:u,customer_email:f||void 0,customer_name:g||void 0,expires_at:t.toISOString()}),r(!1),m(""),h(""),y(""),j(""),l(""),o()}catch(t){console.error("Failed to create payment link:",t),l(t instanceof Error?t.message:"Failed to create payment link")}},E=async t=>{if(confirm("Are you sure you want to cancel this payment link?"))try{await c("POST",`/v1/payment-links/${t}/cancel`),o()}catch(n){console.error("Failed to cancel payment link:",n)}},_=(t,n)=>{navigator.clipboard.writeText(t),d(n),setTimeout(()=>d(k=>k===n?null:k),1200)},I=t=>new Intl.NumberFormat("en-IN",{style:"currency",currency:"INR"}).format(t/100),A=t=>new Date(t).toLocaleDateString("en-IN",{day:"numeric",month:"short",year:"numeric",hour:"2-digit",minute:"2-digit"}),D=t=>({active:"badge success",paid:"badge captured",expired:"badge failed",cancelled:"badge refunded"})[t]||"badge";return e.jsxs("div",{className:"fade-in",children:[e.jsxs("div",{className:"page-header",children:[e.jsxs("div",{children:[e.jsx("h1",{children:"Payment Links"}),e.jsx("p",{children:"Create and manage shareable payment links"})]}),e.jsxs("button",{className:"btn btn-primary",onClick:()=>r(!0),children:[e.jsx(v,{size:18})," Create Link"]})]}),L&&e.jsxs("div",{className:"card payment-link-create-card",children:[e.jsxs("div",{className:"payment-link-create-header",children:[e.jsx("h3",{children:"Create Payment Link"}),e.jsx("p",{children:"Generate branded checkout URL for quick payment collection."})]}),x&&e.jsx("div",{className:"payment-link-create-error",children:x}),e.jsxs("div",{className:"payment-link-form-grid",children:[e.jsxs("div",{className:"payment-link-field",children:[e.jsx("label",{className:"form-label",children:"Amount (₹)"}),e.jsxs("div",{className:"payment-link-input-wrap",children:[e.jsx("span",{className:"payment-link-input-prefix",children:"₹"}),e.jsx("input",{type:"number",className:"form-input payment-link-input-with-prefix",value:i,onChange:t=>m(t.target.value),placeholder:"Enter amount"})]})]}),e.jsxs("div",{className:"payment-link-field",children:[e.jsx("label",{className:"form-label",children:"Expiry (hours)"}),e.jsxs("select",{className:"form-input",value:b,onChange:t=>S(t.target.value),children:[e.jsx("option",{value:"1",children:"1 hour"}),e.jsx("option",{value:"6",children:"6 hours"}),e.jsx("option",{value:"12",children:"12 hours"}),e.jsx("option",{value:"24",children:"24 hours"}),e.jsx("option",{value:"48",children:"48 hours"}),e.jsx("option",{value:"168",children:"7 days"})]})]}),e.jsxs("div",{className:"payment-link-field payment-link-span-2",children:[e.jsx("label",{className:"form-label",children:"Description"}),e.jsx("input",{type:"text",className:"form-input",value:u,onChange:t=>h(t.target.value),placeholder:"What is this payment for?"})]}),e.jsxs("div",{className:"payment-link-field",children:[e.jsx("label",{className:"form-label",children:"Customer Name (optional)"}),e.jsx("input",{type:"text",className:"form-input",value:g,onChange:t=>j(t.target.value),placeholder:"Customer name"})]}),e.jsxs("div",{className:"payment-link-field",children:[e.jsx("label",{className:"form-label",children:"Customer Email (optional)"}),e.jsx("input",{type:"email",className:"form-input",value:f,onChange:t=>y(t.target.value),placeholder:"customer@example.com"})]})]}),e.jsxs("div",{className:"payment-link-form-actions",children:[e.jsx("button",{className:"btn btn-primary",onClick:z,children:"Create Link"}),e.jsx("button",{className:"btn btn-secondary",onClick:()=>r(!1),children:"Cancel"})]})]}),C?e.jsx("div",{className:"card",children:"Loading..."}):s.length===0?e.jsxs("div",{className:"card payment-link-empty-state",children:[e.jsx("div",{className:"payment-link-empty-icon-wrap",children:e.jsx(T,{size:40,style:{color:"var(--fp-primary-soft)"}})}),e.jsx("h3",{children:"No Payment Links Yet"}),e.jsx("p",{children:"Create your first payment link to start accepting payments"}),e.jsxs("button",{className:"btn btn-primary",onClick:()=>r(!0),children:[e.jsx(v,{size:18})," Create Link"]})]}):e.jsxs("div",{className:"card payment-link-list-card",children:[e.jsxs("div",{className:"payment-link-list-header",children:[e.jsx("h3",{children:"Active + Historical Links"}),e.jsxs("div",{className:"payment-link-list-stats",children:[e.jsxs("span",{children:[s.length," total"]}),e.jsxs("span",{children:[s.filter(t=>t.status==="active").length," active"]})]})]}),e.jsxs("table",{className:"data-table",children:[e.jsx("thead",{children:e.jsxs("tr",{children:[e.jsx("th",{children:"Link ID"}),e.jsx("th",{children:"Amount"}),e.jsx("th",{children:"Description"}),e.jsx("th",{children:"Customer"}),e.jsx("th",{children:"Status"}),e.jsx("th",{children:"Views / Attempts"}),e.jsx("th",{children:"Created"}),e.jsx("th",{children:"Actions"})]})}),e.jsx("tbody",{children:s.map(t=>e.jsxs("tr",{children:[e.jsx("td",{children:e.jsxs("div",{className:"payment-link-id-cell",children:[e.jsx("code",{children:t.link_id}),t.short_url&&e.jsx("small",{children:t.short_url})]})}),e.jsx("td",{children:e.jsx("span",{className:"payment-link-amount",children:I(t.amount)})}),e.jsx("td",{children:t.description||"-"}),e.jsx("td",{children:t.customer_name||t.customer_email?e.jsxs("div",{className:"payment-link-customer-cell",children:[e.jsx("div",{children:t.customer_name}),e.jsx("small",{children:t.customer_email})]}):"-"}),e.jsx("td",{children:e.jsx("span",{className:D(t.status),children:t.status})}),e.jsxs("td",{children:[t.view_count," / ",t.attempt_count]}),e.jsx("td",{style:{fontSize:13,color:"var(--fp-text-muted)"},children:A(t.created_at)}),e.jsx("td",{children:e.jsxs("div",{style:{display:"flex",gap:8},children:[e.jsx("button",{className:"btn btn-sm btn-secondary",title:p===t.id?"Copied":"Copy link",onClick:()=>_(t.full_url,t.id),children:p===t.id?"Copied":e.jsx(F,{size:14})}),e.jsx("a",{href:t.full_url,target:"_blank",rel:"noopener noreferrer",className:"btn btn-sm btn-secondary",title:"Open link",children:e.jsx(q,{size:14})}),t.status==="active"&&e.jsx("button",{className:"btn btn-sm btn-danger",title:"Cancel link",onClick:()=>E(t.id),children:e.jsx(H,{size:14})})]})})]},t.id))})]})]}),e.jsx("style",{children:`
        .payment-link-create-card {
          margin-bottom: 24px;
          border: 1px solid var(--fp-border);
          background: #fff9ec;
        }

        .payment-link-create-header h3 {
          font-size: 18px;
          font-weight: 700;
          margin-bottom: 6px;
          letter-spacing: 0.01em;
        }

        .payment-link-create-header p {
          color: var(--fp-text-muted);
          font-size: 14px;
          margin-bottom: 18px;
        }

        .payment-link-form-grid {
          display: grid;
          grid-template-columns: repeat(2, minmax(0, 1fr));
          gap: 16px;
        }

        .payment-link-create-error {
          margin: 0 0 14px;
          padding: 10px 12px;
          border-radius: 10px;
          font-size: 13px;
          border: 1px solid color-mix(in srgb, #ef4444 55%, transparent);
          background: rgba(200, 31, 47, 0.1);
          color: #991627;
        }

        .payment-link-field {
          display: flex;
          flex-direction: column;
          gap: 8px;
          min-width: 0;
        }

        .payment-link-field .form-label {
          display: block;
          font-size: 13px;
          font-weight: 600;
          color: var(--fp-text-muted);
          margin: 0;
          letter-spacing: 0.01em;
        }

        .payment-link-field .form-input {
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

        .payment-link-field .form-input:focus {
          border-color: var(--fp-cta);
          box-shadow: 0 0 0 3px rgba(235, 0, 27, 0.14);
        }

        .payment-link-field .form-input::placeholder {
          color: color-mix(in srgb, var(--fp-text-muted) 88%, #ffffff 12%);
        }

        .payment-link-input-wrap {
          position: relative;
        }

        .payment-link-input-prefix {
          position: absolute;
          left: 12px;
          top: 50%;
          transform: translateY(-50%);
          font-size: 13px;
          color: var(--fp-text-muted);
          pointer-events: none;
        }

        .payment-link-input-with-prefix {
          padding-left: 28px !important;
        }

        .payment-link-field select.form-input {
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

        .payment-link-span-2 {
          grid-column: span 2;
        }

        .payment-link-form-actions {
          display: flex;
          gap: 12px;
          margin-top: 20px;
        }

        .payment-link-empty-state {
          text-align: center;
          padding: 56px 20px;
          background: #fff9ec;
        }

        .payment-link-empty-icon-wrap {
          width: 74px;
          height: 74px;
          margin: 0 auto 16px;
          border-radius: 20px;
          display: grid;
          place-items: center;
          border: 1px solid var(--fp-border);
          background: #f2ead8;
        }

        .payment-link-empty-state h3 {
          margin-bottom: 8px;
          font-size: 22px;
          letter-spacing: 0.01em;
        }

        .payment-link-empty-state p {
          color: var(--fp-text-muted);
          margin-bottom: 22px;
        }

        .payment-link-list-card {
          overflow: hidden;
        }

        .payment-link-list-header {
          display: flex;
          justify-content: space-between;
          align-items: center;
          margin-bottom: 16px;
        }

        .payment-link-list-header h3 {
          margin: 0;
          font-size: 16px;
          letter-spacing: 0.01em;
        }

        .payment-link-list-stats {
          display: flex;
          gap: 8px;
        }

        .payment-link-list-stats span {
          padding: 5px 10px;
          border-radius: 999px;
          font-size: 12px;
          color: var(--fp-text-muted);
          border: 1px solid var(--fp-border);
          background: #f2ead8;
        }

        .payment-link-id-cell code {
          display: block;
          font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
          font-size: 12px;
          color: var(--fp-primary-soft);
          margin-bottom: 2px;
        }

        .payment-link-id-cell small,
        .payment-link-customer-cell small {
          color: var(--fp-text-muted);
          font-size: 12px;
        }

        .payment-link-amount {
          font-weight: 700;
        }

        @media (max-width: 960px) {
          .payment-link-form-grid {
            grid-template-columns: 1fr;
          }

          .payment-link-span-2 {
            grid-column: span 1;
          }

          .payment-link-list-header {
            flex-direction: column;
            align-items: flex-start;
            gap: 10px;
          }
        }
      `})]})}export{V as default};
