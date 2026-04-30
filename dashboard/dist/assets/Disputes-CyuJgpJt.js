import{r as t,j as e,a as u}from"./index-DmZi9o4B.js";function z(){const[c,N]=t.useState([]),[x,h]=t.useState(!0),[_,r]=t.useState(!1),[d,m]=t.useState(null),[f,g]=t.useState(""),[b,v]=t.useState([]),[j,w]=t.useState(!1),[o,a]=t.useState(null),i=c.filter(s=>["open","needs_response","under_review"].includes(s.status)),p=async()=>{h(!0),a(null);try{const s=await u("GET","/v1/disputes");N(s.disputes||[])}catch(s){a(s instanceof Error?s.message:"Failed to load disputes")}finally{h(!1)}};t.useEffect(()=>{p()},[]);const E=async s=>{try{a(null),await u("POST",`/v1/disputes/${s}/accept`,{}),await p()}catch(n){a(n instanceof Error?n.message:"Failed to accept dispute")}},y=s=>{var n;m(s||((n=i[0])==null?void 0:n.id)||null),g(""),v([]),a(null),r(!0)},k=s=>new Promise((n,C)=>{const l=new FileReader;l.onload=()=>n(String(l.result||"")),l.onerror=()=>C(new Error("Failed to read file")),l.readAsDataURL(s)}),S=async()=>{if(d){w(!0),a(null);try{const s=await Promise.all(b.map(k));await u("POST",`/v1/disputes/${d}/evidence`,{text:f||"Evidence submitted from merchant dashboard.",documents:s}),r(!1),await p()}catch(s){a(s instanceof Error?s.message:"Evidence upload failed")}finally{w(!1)}}};return e.jsxs("div",{className:"fade-in",children:[e.jsxs("div",{className:"page-header",children:[e.jsxs("div",{children:[e.jsx("h1",{children:"Disputes"}),e.jsx("p",{children:"Chargeback defense command center"})]}),e.jsx("button",{className:"btn btn-primary",onClick:()=>y(),children:"Upload Evidence"})]}),e.jsxs("div",{className:"dispute-banner",children:[e.jsx("div",{className:"dispute-banner__kicker",children:"RISK SURFACE"}),e.jsx("div",{className:"dispute-banner__title",children:"Action window is short. Reply with evidence before due-by cut."})]}),e.jsxs("div",{className:"dispute-grid",children:[o&&e.jsx("div",{className:"modal-error",children:o}),!x&&c.map(s=>e.jsxs("div",{className:"dispute-card",children:[e.jsxs("div",{className:"dispute-card__head",children:[e.jsx("span",{className:`badge ${s.status}`,children:s.status}),e.jsx("span",{className:"dispute-card__id",children:s.id})]}),e.jsxs("div",{className:"dispute-card__amount",children:["₹",s.amount.toLocaleString("en-IN")]}),e.jsxs("div",{className:"dispute-card__row",children:[e.jsx("span",{children:"Payment"}),e.jsx("code",{children:s.payment_order_id})]}),e.jsxs("div",{className:"dispute-card__row",children:[e.jsx("span",{children:"Reason"}),e.jsx("strong",{children:(s.reason_message||s.reason_code||"unknown").replaceAll("_"," ")})]}),e.jsxs("div",{className:"dispute-card__row",children:[e.jsx("span",{children:"Evidence due"}),e.jsx("strong",{children:s.evidence_due_by?new Date(s.evidence_due_by).toLocaleDateString("en-IN"):"-"})]}),e.jsxs("div",{className:"dispute-card__actions",children:[e.jsx("button",{className:"btn btn-secondary",onClick:()=>void E(s.id),children:"Accept"}),e.jsx("button",{className:"btn btn-primary",onClick:()=>y(s.id),disabled:!["open","needs_response","under_review"].includes(s.status),children:"Respond"})]})]},s.id)),!x&&c.length===0&&e.jsx("div",{children:"No disputes found."})]}),_&&e.jsx("div",{className:"modal-overlay",onClick:()=>r(!1),children:e.jsxs("div",{className:"modal-content",onClick:s=>s.stopPropagation(),children:[e.jsx("div",{className:"modal-kicker",children:"Chargeback Defense"}),e.jsx("h3",{children:"Upload Evidence"}),e.jsx("p",{className:"modal-subtitle",children:"Send supporting docs and context to strengthen your dispute response."}),o&&e.jsx("div",{className:"modal-error",children:o}),i.length===0&&e.jsx("div",{className:"modal-hint",children:"No open disputes right now. Add dispute via webhook/test flow first."}),i.length>0&&e.jsxs(e.Fragment,{children:[e.jsx("label",{children:"Select dispute"}),e.jsx("select",{value:d||"",onChange:s=>m(s.target.value),children:i.map(s=>e.jsxs("option",{value:s.id,children:[s.id," (",s.status,")"]},s.id))})]}),e.jsx("label",{children:"Evidence note"}),e.jsx("textarea",{value:f,onChange:s=>g(s.target.value),placeholder:"Explain why charge valid. Add order timeline, delivery proof, customer communication.",rows:4}),e.jsx("label",{children:"Evidence files"}),e.jsxs("label",{className:"file-drop",children:[e.jsx("input",{type:"file",multiple:!0,accept:".pdf,.png,.jpg,.jpeg,.txt",onChange:s=>v(Array.from(s.target.files||[]))}),e.jsx("span",{className:"file-drop-title",children:"Choose files or drag into picker"}),e.jsx("span",{className:"file-drop-sub",children:"PDF, PNG, JPG, TXT"})]}),e.jsxs("div",{className:"file-count",children:[b.length," file(s) selected"]}),e.jsxs("div",{className:"modal-actions",children:[e.jsx("button",{className:"btn btn-secondary",onClick:()=>r(!1),children:"Cancel"}),e.jsx("button",{className:"btn btn-primary",onClick:()=>void S(),disabled:j||i.length===0||!d,children:j?"Submitting...":"Submit Evidence"})]})]})}),e.jsx("style",{children:`
        .modal-overlay {
          position: fixed;
          inset: 0;
          background: rgba(3, 8, 20, 0.74);
          display: flex;
          align-items: center;
          justify-content: center;
          z-index: 2200;
          padding: 16px;
        }
        .modal-content {
          width: min(620px, 100%);
          background: radial-gradient(140% 160% at 20% 0%, rgba(34, 85, 152, 0.36), rgba(16, 38, 74, 0.98));
          border: 1px solid rgba(108, 168, 255, 0.35);
          border-radius: 18px;
          padding: 20px;
          color: #e8f2ff;
          display: flex;
          flex-direction: column;
          gap: 11px;
          box-shadow: 0 18px 60px rgba(0, 0, 0, 0.42);
        }
        .modal-kicker {
          font-size: 11px;
          letter-spacing: 0.16em;
          text-transform: uppercase;
          color: #9cc3ff;
          font-weight: 700;
        }
        .modal-content h3 {
          margin: 0;
          font-size: 28px;
          font-weight: 800;
          line-height: 1.05;
          color: #f0f6ff;
        }
        .modal-subtitle {
          margin: 0 0 2px;
          font-size: 13px;
          color: #a8c4ec;
          line-height: 1.5;
        }
        .modal-content label {
          font-size: 13px;
          font-weight: 700;
          color: #c8dbfb;
        }
        .modal-content textarea,
        .modal-content input[type="file"],
        .modal-content select {
          width: 100%;
          padding: 10px 12px;
          border-radius: 10px;
          border: 1px solid rgba(91, 154, 247, 0.34);
          background: linear-gradient(180deg, rgba(17, 41, 77, 0.94), rgba(12, 29, 56, 0.94));
          color: #eaf3ff;
          outline: none;
          transition: border-color 0.2s ease, box-shadow 0.2s ease;
        }
        .modal-content textarea:focus,
        .modal-content select:focus {
          border-color: rgba(122, 182, 255, 0.95);
          box-shadow: 0 0 0 3px rgba(77, 141, 235, 0.2);
        }
        .modal-content textarea {
          resize: vertical;
          min-height: 100px;
        }
        .modal-content textarea::placeholder {
          color: #93b2de;
        }
        .file-drop {
          display: flex;
          flex-direction: column;
          align-items: center;
          justify-content: center;
          gap: 6px;
          border: 1px dashed rgba(122, 182, 255, 0.55);
          border-radius: 12px;
          min-height: 84px;
          padding: 12px;
          background: linear-gradient(180deg, rgba(22, 51, 92, 0.9), rgba(16, 38, 70, 0.9));
          cursor: pointer;
        }
        .file-drop input[type="file"] {
          position: absolute;
          opacity: 0;
          width: 1px;
          height: 1px;
          pointer-events: none;
        }
        .file-drop-title {
          font-size: 13px;
          font-weight: 700;
          color: #dbe9ff;
        }
        .file-drop-sub {
          font-size: 12px;
          color: #92b4e2;
        }
        .file-count {
          font-size: 12px;
          color: #b5ccef;
        }
        .modal-hint {
          border: 1px solid rgba(255, 180, 85, 0.35);
          background: rgba(255, 180, 85, 0.12);
          color: #ffd79d;
          border-radius: 10px;
          padding: 9px 11px;
          font-size: 12px;
          font-weight: 600;
        }
        .modal-actions {
          display: flex;
          gap: 10px;
          justify-content: flex-end;
          margin-top: 10px;
        }
        .modal-error {
          border: 1px solid rgba(247, 83, 96, 0.45);
          background: rgba(247, 83, 96, 0.14);
          color: #ffadb6;
          border-radius: 10px;
          padding: 8px 10px;
          font-size: 13px;
          font-weight: 600;
        }
      `})]})}export{z as default};
