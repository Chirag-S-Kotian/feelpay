import{c as v,r as o,j as e,X as _,a as b}from"./index-DmZi9o4B.js";import{C as w}from"./circle-alert-BT9_AK1r.js";import{P as k}from"./plus-Bj7MaHiu.js";import{C as N}from"./clock-D8-t-m5_.js";import{T as S}from"./trash-2-CwydIfLd.js";/**
 * @license lucide-react v0.469.0 - ISC
 *
 * This source code is licensed under the ISC license.
 * See the LICENSE file in the root directory of this source tree.
 */const C=v("Pen",[["path",{d:"M21.174 6.812a1 1 0 0 0-3.986-3.987L3.842 16.174a2 2 0 0 0-.5.83l-1.321 4.352a.5.5 0 0 0 .623.622l4.353-1.32a2 2 0 0 0 .83-.497z",key:"1a8usu"}]]);function R({report:s,onClose:d}){const[a,n]=o.useState({report_type:(s==null?void 0:s.report_type)||"transaction_summary",frequency:(s==null?void 0:s.frequency)||"daily",day_of_week:(s==null?void 0:s.day_of_week)||1,day_of_month:(s==null?void 0:s.day_of_month)||1,time_of_day:(s==null?void 0:s.time_of_day)||"09:00",recipient_emails:(s==null?void 0:s.recipient_emails.join(", "))||"",format:(s==null?void 0:s.format)||"pdf"}),[i,l]=o.useState({}),[y,m]=o.useState(!1),p=(()=>{const r=localStorage.getItem("feelpay_merchant");if(!r)return"";try{return JSON.parse(r).id||""}catch{return""}})(),u=()=>{const r={};return a.recipient_emails.trim()?a.recipient_emails.split(",").map(f=>f.trim()).filter(f=>!f.match(/^[^\s@]+@[^\s@]+\.[^\s@]+$/)).length>0&&(r.recipient_emails="Invalid email format"):r.recipient_emails="At least one email is required",a.frequency==="weekly"&&(a.day_of_week<0||a.day_of_week>6)&&(r.day_of_week="Invalid day of week"),a.frequency==="monthly"&&(a.day_of_month<1||a.day_of_month>31)&&(r.day_of_month="Day must be between 1 and 31"),l(r),Object.keys(r).length===0},x=async r=>{if(r.preventDefault(),!!u()){if(!p){l({submit:"Merchant session missing. Login again."});return}m(!0);try{const c={report_type:a.report_type,frequency:a.frequency,day_of_week:a.frequency==="weekly"?a.day_of_week:void 0,day_of_month:a.frequency==="monthly"?a.day_of_month:void 0,time_of_day:a.time_of_day,recipient_emails:a.recipient_emails.split(",").map(g=>g.trim()),format:a.format};s?await b("PUT",`/v1/merchants/${p}/scheduled-reports/${s.id}`,c):await b("POST",`/v1/merchants/${p}/scheduled-reports`,c),d()}catch(c){l({submit:c instanceof Error?c.message:"Failed to save schedule"})}finally{m(!1)}}};return e.jsxs("div",{className:"modal-overlay",onClick:d,children:[e.jsxs("div",{className:"modal-content",onClick:r=>r.stopPropagation(),children:[e.jsxs("div",{className:"modal-header",children:[e.jsx("h2",{children:s?"Edit Schedule":"Schedule Report"}),e.jsx("button",{className:"btn-close",onClick:d,children:e.jsx(_,{size:20})})]}),e.jsxs("form",{onSubmit:x,className:"form",children:[i.submit&&e.jsxs("div",{className:"alert alert-error",children:[e.jsx(w,{size:16}),i.submit]}),e.jsxs("div",{className:"form-group",children:[e.jsx("label",{children:"Report Type *"}),e.jsxs("select",{value:a.report_type,onChange:r=>n({...a,report_type:r.target.value}),children:[e.jsx("option",{value:"transaction_summary",children:"Transaction Summary"}),e.jsx("option",{value:"settlement_reconciliation",children:"Settlement Reconciliation"}),e.jsx("option",{value:"refund_report",children:"Refund Report"}),e.jsx("option",{value:"dispute_report",children:"Dispute Report"}),e.jsx("option",{value:"gst_report",children:"GST Report"}),e.jsx("option",{value:"custom",children:"Custom Date Range"})]})]}),e.jsxs("div",{className:"form-group",children:[e.jsx("label",{children:"Frequency *"}),e.jsxs("select",{value:a.frequency,onChange:r=>n({...a,frequency:r.target.value}),children:[e.jsx("option",{value:"daily",children:"Daily"}),e.jsx("option",{value:"weekly",children:"Weekly"}),e.jsx("option",{value:"monthly",children:"Monthly"})]})]}),a.frequency==="weekly"&&e.jsxs("div",{className:"form-group",children:[e.jsx("label",{children:"Day of Week *"}),e.jsxs("select",{value:a.day_of_week,onChange:r=>n({...a,day_of_week:parseInt(r.target.value)}),children:[e.jsx("option",{value:0,children:"Sunday"}),e.jsx("option",{value:1,children:"Monday"}),e.jsx("option",{value:2,children:"Tuesday"}),e.jsx("option",{value:3,children:"Wednesday"}),e.jsx("option",{value:4,children:"Thursday"}),e.jsx("option",{value:5,children:"Friday"}),e.jsx("option",{value:6,children:"Saturday"})]}),i.day_of_week&&e.jsx("span",{className:"error-text",children:i.day_of_week})]}),a.frequency==="monthly"&&e.jsxs("div",{className:"form-group",children:[e.jsx("label",{children:"Day of Month *"}),e.jsx("input",{type:"number",min:"1",max:"31",value:a.day_of_month,onChange:r=>n({...a,day_of_month:parseInt(r.target.value)})}),i.day_of_month&&e.jsx("span",{className:"error-text",children:i.day_of_month})]}),e.jsxs("div",{className:"form-group",children:[e.jsx("label",{children:"Time of Day (HH:MM) *"}),e.jsx("input",{type:"time",value:a.time_of_day,onChange:r=>n({...a,time_of_day:r.target.value})})]}),e.jsxs("div",{className:"form-group",children:[e.jsx("label",{children:"Recipient Emails * (comma-separated)"}),e.jsx("textarea",{value:a.recipient_emails,onChange:r=>n({...a,recipient_emails:r.target.value}),placeholder:"finance@company.com, accounting@company.com",rows:3}),i.recipient_emails&&e.jsx("span",{className:"error-text",children:i.recipient_emails})]}),e.jsxs("div",{className:"form-group",children:[e.jsx("label",{children:"Format *"}),e.jsxs("select",{value:a.format,onChange:r=>n({...a,format:r.target.value}),children:[e.jsx("option",{value:"csv",children:"CSV"}),e.jsx("option",{value:"pdf",children:"PDF"}),e.jsx("option",{value:"excel",children:"Excel"})]})]}),e.jsxs("div",{className:"form-actions",children:[e.jsx("button",{type:"button",className:"btn-secondary",onClick:d,children:"Cancel"}),e.jsx("button",{type:"submit",className:"btn-primary",disabled:y,children:y?"Saving...":"Save Schedule"})]})]})]}),e.jsx("style",{children:`
        .modal-overlay {
          position: fixed;
          top: 0;
          left: 0;
          right: 0;
          bottom: 0;
          background: rgba(20, 20, 19, 0.38);
          backdrop-filter: blur(5px);
          display: flex;
          align-items: center;
          justify-content: center;
          z-index: 2000;
          padding: 16px;
        }
        .modal-content {
          background: #fff9ec;
          border-radius: 28px;
          border: 1px solid var(--fp-border);
          box-shadow: 0 18px 70px rgba(20, 20, 19, 0.2);
          max-width: 620px;
          width: 90%;
          max-height: 90vh;
          overflow-y: auto;
          isolation: isolate;
        }
        .modal-header {
          display: flex;
          justify-content: space-between;
          align-items: center;
          padding: 20px 22px;
          border-bottom: 1px solid var(--fp-border);
        }
        .modal-header h2 {
          margin: 0;
          font-size: 20px;
          font-weight: 800;
          color: var(--fp-text);
        }
        .btn-close {
          background: #f2ead8;
          border: 1px solid var(--fp-border);
          border-radius: 999px;
          cursor: pointer;
          padding: 8px;
          display: flex;
          align-items: center;
          justify-content: center;
          color: var(--fp-text-muted);
        }
        .btn-close:hover {
          border-color: var(--fp-cta);
          color: var(--fp-text);
        }
        .form {
          padding: 20px 22px 24px;
        }
        .form-group {
          margin-bottom: 18px;
        }
        .form-group label {
          display: block;
          margin-bottom: 7px;
          font-weight: 700;
          font-size: 13px;
          letter-spacing: 0.02em;
          color: var(--fp-text-muted);
        }
        .form-group input,
        .form-group select,
        .form-group textarea {
          width: 100%;
          padding: 11px 13px;
          border: 1px solid var(--fp-border);
          border-radius: 999px;
          font-size: 14px;
          font-family: inherit;
          color: var(--fp-text);
          background: #fff9ec;
          outline: none;
          transition: border-color 0.2s ease, box-shadow 0.2s ease;
        }
        .form-group input:focus,
        .form-group select:focus,
        .form-group textarea:focus {
          border-color: var(--fp-cta);
          box-shadow: 0 0 0 3px rgba(235, 0, 27, 0.14);
        }
        .form-group textarea {
          min-height: 92px;
          resize: vertical;
          border-radius: 20px;
        }
        .form-group input::placeholder,
        .form-group textarea::placeholder {
          color: var(--fp-text-dim);
        }
        .error-text {
          display: block;
          color: #991627;
          font-size: 12px;
          margin-top: 6px;
        }
        .form-actions {
          display: flex;
          gap: 12px;
          margin-top: 26px;
        }
        .form-actions button {
          flex: 1;
          min-height: 40px;
        }
        .alert.alert-error {
          display: flex;
          align-items: center;
          gap: 8px;
          border: 1px solid rgba(200, 31, 47, 0.35);
          background: rgba(200, 31, 47, 0.1);
          color: #991627;
          border-radius: 16px;
          padding: 10px 12px;
          margin-bottom: 12px;
          font-size: 13px;
          font-weight: 600;
        }
      `})]})}function T(){const[s,d]=o.useState([]),[a,n]=o.useState(!0),[i,l]=o.useState(!1),[y,m]=o.useState(null),[p,u]=o.useState(""),x=(()=>{const t=localStorage.getItem("feelpay_merchant");if(!t)return"";try{return JSON.parse(t).id||""}catch{return""}})();o.useEffect(()=>{r()},[]);const r=async()=>{if(!x){u("Merchant session missing. Login again."),n(!1);return}try{u("");const t=await b("GET",`/v1/merchants/${x}/scheduled-reports`);d(t.reports||t.scheduled_reports||[])}catch(t){console.error("Failed to fetch scheduled reports:",t),u(t instanceof Error?t.message:"Failed to fetch scheduled reports")}finally{n(!1)}},c=async t=>{if(confirm("Delete this scheduled report?"))try{await b("DELETE",`/v1/merchants/${x}/scheduled-reports/${t}`),d(s.filter(h=>h.id!==t))}catch(h){console.error("Failed to delete report:",h)}},g=()=>{l(!1),m(null),r()},f=t=>t.frequency==="daily"?"Daily":t.frequency==="weekly"?`Weekly (${["Sun","Mon","Tue","Wed","Thu","Fri","Sat"][t.day_of_week||0]})`:`Monthly (Day ${t.day_of_month})`;return a?e.jsx("div",{className:"page-container",children:e.jsx("div",{className:"loading",children:"Loading scheduled reports..."})}):e.jsxs("div",{className:"page-container",children:[e.jsxs("div",{className:"page-header",children:[e.jsx("h1",{children:"Scheduled Reports"}),e.jsx("p",{children:"Manage recurring report generation and delivery"})]}),p&&e.jsx("div",{className:"error-banner",children:p}),i&&e.jsx(R,{report:y,onClose:g}),e.jsxs("div",{className:"card",children:[e.jsxs("div",{className:"card-header",children:[e.jsx("h2",{children:"Active Schedules"}),e.jsxs("button",{className:"btn-primary",onClick:()=>{m(null),l(!0)},children:[e.jsx(k,{size:16}),"Schedule Report"]})]}),s.length===0?e.jsxs("div",{className:"empty-state",children:[e.jsx(N,{size:32}),e.jsx("p",{children:"No scheduled reports yet"}),e.jsx("span",{className:"empty-sub",children:"Create recurring report for finance, settlement, GST."}),e.jsx("button",{className:"btn-primary",onClick:()=>{m(null),l(!0)},children:"Create First Schedule"})]}):e.jsx("div",{className:"table-container",children:e.jsxs("table",{className:"data-table",children:[e.jsx("thead",{children:e.jsxs("tr",{children:[e.jsx("th",{children:"Report Type"}),e.jsx("th",{children:"Frequency"}),e.jsx("th",{children:"Time"}),e.jsx("th",{children:"Recipients"}),e.jsx("th",{children:"Format"}),e.jsx("th",{children:"Next Run"}),e.jsx("th",{children:"Status"}),e.jsx("th",{children:"Actions"})]})}),e.jsx("tbody",{children:s.map(t=>e.jsxs("tr",{children:[e.jsx("td",{className:"font-semibold",children:t.report_type}),e.jsx("td",{children:f(t)}),e.jsx("td",{className:"mono",children:t.time_of_day}),e.jsx("td",{children:e.jsxs("div",{className:"recipients-list",children:[t.recipient_emails.slice(0,2).map((h,j)=>e.jsx("span",{className:"recipient-badge",children:h},j)),t.recipient_emails.length>2&&e.jsxs("span",{className:"recipient-badge",children:["+",t.recipient_emails.length-2]})]})}),e.jsx("td",{children:t.format.toUpperCase()}),e.jsx("td",{children:new Date(t.next_run_at).toLocaleString()}),e.jsx("td",{children:e.jsx("span",{className:`badge badge-${t.is_active?"success":"secondary"}`,children:t.is_active?"Active":"Inactive"})}),e.jsx("td",{children:e.jsxs("div",{className:"action-buttons",children:[e.jsx("button",{className:"btn-icon",onClick:()=>{m(t),l(!0)},title:"Edit",children:e.jsx(C,{size:14})}),e.jsx("button",{className:"btn-icon btn-danger",onClick:()=>c(t.id),title:"Delete",children:e.jsx(S,{size:14})})]})})]},t.id))})]})})]}),e.jsx("style",{children:`
        .error-banner {
          margin-bottom: 16px;
          border: 1px solid rgba(247, 83, 96, 0.45);
          background: rgba(247, 83, 96, 0.14);
          color: #ffadb6;
          border-radius: 12px;
          padding: 10px 14px;
          font-size: 13px;
          font-weight: 600;
        }
        .recipients-list {
          display: flex;
          flex-wrap: wrap;
          gap: 4px;
        }
        .empty-state {
          min-height: 220px;
          display: flex;
          flex-direction: column;
          justify-content: center;
          align-items: flex-start;
          gap: 10px;
          padding: 10px 2px;
        }
        .empty-sub {
          color: #9eb9df;
          font-size: 13px;
          max-width: 480px;
        }
        .recipient-badge {
          display: inline-block;
          padding: 2px 8px;
          background: var(--bg-secondary);
          border-radius: 4px;
          font-size: 12px;
          white-space: nowrap;
          overflow: hidden;
          text-overflow: ellipsis;
          max-width: 120px;
        }
        .action-buttons {
          display: flex;
          gap: 8px;
        }
        .btn-icon {
          padding: 6px;
          background: var(--bg-secondary);
          border: 1px solid var(--border-color);
          border-radius: 4px;
          cursor: pointer;
          display: flex;
          align-items: center;
          justify-content: center;
          transition: all 0.2s;
        }
        .btn-icon:hover {
          background: var(--bg-tertiary);
        }
        .btn-icon.btn-danger:hover {
          background: #fee;
          border-color: #f00;
          color: #f00;
        }
      `})]})}export{T as default};
