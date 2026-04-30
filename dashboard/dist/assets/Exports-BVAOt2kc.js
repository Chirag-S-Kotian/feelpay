import{c as S,r,a as g,j as e,S as A}from"./index-DmZi9o4B.js";import{C as P}from"./circle-alert-BT9_AK1r.js";import{C as B}from"./circle-check-big-3eDQ9hXq.js";import{C as G}from"./copy-CrMUemfa.js";import{C as H}from"./clock-D8-t-m5_.js";import{D as O}from"./download-BcZMe9fw.js";/**
 * @license lucide-react v0.469.0 - ISC
 *
 * This source code is licensed under the ISC license.
 * See the LICENSE file in the root directory of this source tree.
 */const q=S("Calendar",[["path",{d:"M8 2v4",key:"1cmpym"}],["path",{d:"M16 2v4",key:"4m81vk"}],["rect",{width:"18",height:"18",x:"3",y:"4",rx:"2",key:"1hopcy"}],["path",{d:"M3 10h18",key:"8toen8"}]]);/**
 * @license lucide-react v0.469.0 - ISC
 *
 * This source code is licensed under the ISC license.
 * See the LICENSE file in the root directory of this source tree.
 */const K=S("Filter",[["polygon",{points:"22 3 2 3 10 12.46 10 19 14 21 14 12.46 22 3",key:"1yg77f"}]]);function Z(){const[c,f]=r.useState("export"),[d,E]=r.useState(""),[p,z]=r.useState(""),[b,D]=r.useState("csv"),[l,j]=r.useState(["transaction_id","amount_paise","payment_method","status","created_at"]),[o,n]=r.useState({status:[],payment_method:[],amount_min:0,amount_max:1e6}),[y,F]=r.useState(!1),[v,I]=r.useState(!1),[k,_]=r.useState(!1),[N,T]=r.useState([]),[J,M]=r.useState(!0),[x,C]=r.useState(null),[w,i]=r.useState(null),m=(()=>{const t=localStorage.getItem("feelpay_merchant");if(!t)return"";try{return JSON.parse(t).id||""}catch{return""}})();r.useEffect(()=>{h();const t=setInterval(h,5e3);return()=>clearInterval(t)},[]);const h=async()=>{try{const t=await g("GET",`/v1/merchants/${m}/exports?limit=20&offset=0`);T(t.exports||[])}catch(t){console.error("Failed to fetch export jobs:",t),i(t instanceof Error?t.message:"Failed to fetch export jobs. Check backend is running.")}finally{M(!1)}},$=async()=>{if(!d||!p){i("Please select both start and end dates");return}_(!0),i(null),C(null);try{const t={date_range:{start:d,end:p},format:b,columns:l,filters:{status:o.status.length>0?o.status:void 0,payment_method:o.payment_method.length>0?o.payment_method:void 0,amount_range:o.amount_min>0||o.amount_max<1e6?{min:o.amount_min,max:o.amount_max}:void 0}},a=await g("POST",`/v1/merchants/${m}/exports/transactions`,t);C(a.job_id),await h()}catch(t){i(t instanceof Error?t.message:"Export failed")}finally{_(!1)}},L=async t=>{try{const a=await g("GET",`/v1/merchants/${m}/exports/${t}/download`);window.location.href=a.download_url}catch(a){i(a instanceof Error?a.message:"Download failed")}},R=["transaction_id","order_id","customer_email","customer_phone","amount_paise","currency","payment_method","status","created_at","settled_at","refund_amount","refund_status"];return e.jsxs("div",{className:"page-container exports-page",children:[e.jsxs("div",{className:"page-header",children:[e.jsx("h1",{children:"Transaction Exports"}),e.jsx("p",{children:"Export transactions for reconciliation and analysis"})]}),e.jsxs("div",{className:"tabs",children:[e.jsx("button",{className:`tab ${c==="export"?"active":""}`,onClick:()=>f("export"),children:"New Export"}),e.jsx("button",{className:`tab ${c==="history"?"active":""}`,onClick:()=>f("history"),children:"Export History"})]}),c==="export"&&e.jsx("div",{className:"card",children:e.jsxs("div",{className:"form-section",children:[e.jsx("h2",{children:"Export Transactions"}),w&&e.jsxs("div",{className:"alert alert-error",children:[e.jsx(P,{size:16}),w]}),x&&e.jsxs("div",{className:"alert alert-success",children:[e.jsx(B,{size:16}),"Export job created! Job ID: ",x,e.jsx("button",{className:"btn-copy",onClick:()=>{navigator.clipboard.writeText(x)},children:e.jsx(G,{size:14})})]}),e.jsxs("div",{className:"form-group",children:[e.jsx("label",{children:"Date Range"}),e.jsxs("div",{className:"date-range",children:[e.jsx("input",{type:"date",value:d,onChange:t=>E(t.target.value),placeholder:"Start date"}),e.jsx("span",{children:"to"}),e.jsx("input",{type:"date",value:p,onChange:t=>z(t.target.value),placeholder:"End date"})]})]}),e.jsxs("div",{className:"form-group",children:[e.jsx("label",{children:"Format"}),e.jsxs("select",{value:b,onChange:t=>D(t.target.value),children:[e.jsx("option",{value:"csv",children:"CSV"}),e.jsx("option",{value:"excel",children:"Excel (.xlsx)"})]})]}),e.jsxs("div",{className:"form-actions",children:[e.jsxs("button",{className:"btn-secondary",onClick:()=>F(!y),children:[e.jsx(K,{size:16}),"Filters"]}),e.jsxs("button",{className:"btn-secondary",onClick:()=>I(!v),children:[e.jsx(A,{size:16}),"Columns"]})]}),y&&e.jsxs("div",{className:"filter-panel",children:[e.jsx("h3",{children:"Filters"}),e.jsxs("div",{className:"filter-group",children:[e.jsx("label",{children:"Status"}),e.jsx("div",{className:"checkbox-group",children:["success","failed","pending"].map(t=>e.jsxs("label",{className:"checkbox",children:[e.jsx("input",{type:"checkbox",checked:o.status.includes(t),onChange:a=>{a.target.checked?n(s=>({...s,status:[...s.status,t]})):n(s=>({...s,status:s.status.filter(u=>u!==t)}))}}),t]},t))})]}),e.jsxs("div",{className:"filter-group",children:[e.jsx("label",{children:"Payment Method"}),e.jsx("div",{className:"checkbox-group",children:["card","upi","netbanking"].map(t=>e.jsxs("label",{className:"checkbox",children:[e.jsx("input",{type:"checkbox",checked:o.payment_method.includes(t),onChange:a=>{a.target.checked?n(s=>({...s,payment_method:[...s.payment_method,t]})):n(s=>({...s,payment_method:s.payment_method.filter(u=>u!==t)}))}}),t]},t))})]}),e.jsxs("div",{className:"filter-group",children:[e.jsx("label",{children:"Amount Range (₹)"}),e.jsxs("div",{className:"amount-range",children:[e.jsx("input",{type:"number",value:o.amount_min,onChange:t=>n(a=>({...a,amount_min:parseInt(t.target.value)})),placeholder:"Min"}),e.jsx("input",{type:"number",value:o.amount_max,onChange:t=>n(a=>({...a,amount_max:parseInt(t.target.value)})),placeholder:"Max"})]})]})]}),v&&e.jsxs("div",{className:"columns-panel",children:[e.jsx("h3",{children:"Select Columns"}),e.jsx("div",{className:"checkbox-group",children:R.map(t=>e.jsxs("label",{className:"checkbox",children:[e.jsx("input",{type:"checkbox",checked:l.includes(t),onChange:a=>{a.target.checked?j([...l,t]):j(l.filter(s=>s!==t))}}),t]},t))})]}),e.jsx("button",{className:"btn-primary",onClick:$,disabled:k,children:k?"Creating export...":"Export Transactions"})]})}),c==="history"&&e.jsxs("div",{className:"card",children:[e.jsx("h2",{children:"Export History"}),J?e.jsx("div",{className:"loading",children:"Loading export history..."}):N.length===0?e.jsxs("div",{className:"empty-state",children:[e.jsx(q,{size:32}),e.jsx("p",{children:"No exports yet"})]}):e.jsx("div",{className:"table-container",children:e.jsxs("table",{className:"data-table",children:[e.jsx("thead",{children:e.jsxs("tr",{children:[e.jsx("th",{children:"Job ID"}),e.jsx("th",{children:"Type"}),e.jsx("th",{children:"Status"}),e.jsx("th",{children:"Records"}),e.jsx("th",{children:"Size"}),e.jsx("th",{children:"Created"}),e.jsx("th",{children:"Action"})]})}),e.jsx("tbody",{children:N.map(t=>e.jsxs("tr",{children:[e.jsxs("td",{className:"mono",children:[t.job_id.slice(0,8),"..."]}),e.jsx("td",{children:t.job_type}),e.jsx("td",{children:e.jsxs("span",{className:`badge badge-${t.status}`,children:[t.status==="processing"&&e.jsx(H,{size:12}),t.status]})}),e.jsx("td",{children:t.record_count||"-"}),e.jsx("td",{children:t.file_size_bytes?`${(t.file_size_bytes/1024).toFixed(2)} KB`:"-"}),e.jsx("td",{children:new Date(t.created_at).toLocaleDateString()}),e.jsxs("td",{children:[t.status==="completed"&&e.jsx("button",{className:"btn-small",onClick:()=>L(t.job_id),children:e.jsx(O,{size:14})}),t.status==="failed"&&e.jsx("span",{className:"error-text",title:t.error_message,children:"Error"})]})]},t.job_id))})]})})]}),e.jsx("style",{children:`
        .exports-page .tabs {
          display: inline-flex;
          gap: 8px;
          padding: 6px;
          border-radius: 16px;
          border: 1px solid var(--fp-border);
          background: #e7decb;
          margin-bottom: 18px;
        }

        .exports-page .tab {
          border: 0;
          border-radius: 12px;
          padding: 9px 14px;
          font-size: 13px;
          font-weight: 600;
          color: var(--fp-text-muted) !important;
          background: transparent;
          cursor: pointer;
        }

        .exports-page .tab.active {
          color: #fff9ec !important;
          background: var(--fp-cta);
          border: 1px solid #d74a58;
          box-shadow: 0 8px 14px rgba(235, 0, 27, 0.24);
        }

        .exports-page .card {
          border: 1px solid var(--fp-border);
          background: #fff9ec;
        }

        .exports-page h2 {
          margin: 0 0 14px;
          font-size: 21px;
        }

        .exports-page .form-group {
          margin-bottom: 14px;
        }

        .exports-page .form-group > label {
          display: block;
          margin-bottom: 8px;
          font-size: 13px;
          font-weight: 600;
          color: var(--fp-text-muted);
        }

        .exports-page .date-range {
          display: grid;
          grid-template-columns: 1fr auto 1fr;
          gap: 10px;
          align-items: center;
        }

        .exports-page .date-range span {
          color: var(--fp-text-muted);
          font-size: 13px;
        }

        .exports-page input,
        .exports-page select {
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

        .exports-page input:focus,
        .exports-page select:focus {
          border-color: var(--fp-cta);
          box-shadow: 0 0 0 3px rgba(235, 0, 27, 0.14);
        }

        .exports-page select {
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

        .exports-page .form-actions {
          display: flex;
          gap: 10px;
          justify-content: flex-end;
          margin: 12px 0;
        }

        .exports-page .btn-primary,
        .exports-page .btn-secondary {
          display: inline-flex;
          align-items: center;
          justify-content: center;
          gap: 8px;
          border-radius: 10px;
          font-weight: 600;
          border: 1px solid var(--fp-border);
          cursor: pointer;
        }

        .exports-page .btn-primary {
          height: 44px;
          padding: 0 16px;
          border: 0;
          color: #fff9ec;
          background: var(--fp-cta);
        }

        .exports-page .btn-secondary {
          height: 40px;
          padding: 0 12px;
          color: var(--fp-text);
          background: #f2ead8;
        }

        .exports-page .filter-panel,
        .exports-page .columns-panel {
          margin: 12px 0;
          padding: 14px;
          border-radius: 12px;
          border: 1px solid var(--fp-border);
          background: #f7f0e1;
        }

        .exports-page .checkbox-group {
          display: flex;
          flex-wrap: wrap;
          gap: 10px;
        }

        .exports-page .checkbox {
          display: inline-flex;
          align-items: center;
          gap: 6px;
          font-size: 13px;
          color: var(--fp-text-muted);
          padding: 6px 10px;
          border: 1px solid var(--fp-border);
          border-radius: 8px;
          background: #f2ead8;
        }

        .exports-page .checkbox input {
          width: 14px;
          height: 14px;
        }

        .exports-page .amount-range {
          display: grid;
          grid-template-columns: repeat(2, minmax(0, 1fr));
          gap: 10px;
        }

        .exports-page .alert {
          margin-bottom: 12px;
          border-radius: 10px;
          padding: 10px 12px;
          display: flex;
          align-items: center;
          gap: 8px;
          font-size: 13px;
        }

        .exports-page .alert-error {
          border: 1px solid rgba(239, 68, 68, 0.45);
          color: #991627;
          background: rgba(200, 31, 47, 0.1);
        }

        .exports-page .alert-success {
          border: 1px solid rgba(34, 197, 94, 0.4);
          color: #2f6b40;
          background: rgba(52, 122, 71, 0.1);
        }

        .exports-page .btn-copy {
          margin-left: auto;
          border: 1px solid rgba(148, 163, 184, 0.3);
          background: #f2ead8;
          color: var(--fp-text);
          width: 28px;
          height: 28px;
          border-radius: 8px;
          display: grid;
          place-items: center;
          cursor: pointer;
        }

        .exports-page .mono {
          font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
          color: var(--fp-primary-soft);
        }

        .exports-page .btn-small {
          border: 1px solid var(--fp-border);
          background: #f2ead8;
          color: var(--fp-text);
          width: 30px;
          height: 30px;
          border-radius: 8px;
          display: grid;
          place-items: center;
          cursor: pointer;
        }

        .exports-page .error-text {
          color: #fca5a5;
          font-size: 12px;
        }

        @media (max-width: 900px) {
          .exports-page .date-range {
            grid-template-columns: 1fr;
          }

          .exports-page .form-actions {
            justify-content: flex-start;
            flex-wrap: wrap;
          }

          .exports-page .amount-range {
            grid-template-columns: 1fr;
          }
        }
      `})]})}export{Z as default};
