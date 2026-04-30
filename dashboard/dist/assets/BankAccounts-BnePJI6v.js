import{r as n,j as e,X as N,a as g}from"./index-DmZi9o4B.js";import{C as y}from"./circle-alert-BT9_AK1r.js";import{C as _}from"./circle-check-big-3eDQ9hXq.js";import{P as S}from"./plus-Bj7MaHiu.js";import{L as z}from"./landmark-Clv75ufG.js";import{S as k}from"./shield-check-By1IlAFg.js";import{T as A}from"./trash-2-CwydIfLd.js";function C({onClose:c}){const[t,o]=n.useState({account_holder_name:"",account_number:"",ifsc_code:"",account_type:"savings"}),[i,l]=n.useState({}),[p,m]=n.useState(!1),x=(()=>{const a=localStorage.getItem("feelpay_merchant");if(!a)return"";try{return JSON.parse(a).id||""}catch{return""}})(),u=()=>{const a={};return t.account_holder_name.trim()||(a.account_holder_name="Account holder name is required"),t.account_number.trim()?/^\d{9,18}$/.test(t.account_number)||(a.account_number="Invalid account number format"):a.account_number="Account number is required",t.ifsc_code.trim()?/^[A-Z]{4}0[A-Z0-9]{6}$/.test(t.ifsc_code)||(a.ifsc_code="Invalid IFSC code format"):a.ifsc_code="IFSC code is required",l(a),Object.keys(a).length===0},f=async a=>{if(a.preventDefault(),!!u()){if(!x){l({submit:"Merchant session missing. Login again."});return}m(!0);try{await g("POST",`/v1/merchants/${x}/bank-accounts`,t),c()}catch(d){l({submit:d instanceof Error?d.message:"Failed to add bank account"})}finally{m(!1)}}};return e.jsxs("div",{className:"modal-overlay",onClick:c,children:[e.jsxs("div",{className:"modal-content",onClick:a=>a.stopPropagation(),children:[e.jsxs("div",{className:"modal-header",children:[e.jsxs("div",{children:[e.jsx("div",{className:"modal-kicker",children:"Settlement Setup"}),e.jsx("h2",{children:"Add Bank Account"})]}),e.jsx("button",{className:"btn-close",onClick:c,children:e.jsx(N,{size:20})})]}),e.jsxs("form",{onSubmit:f,className:"form",children:[i.submit&&e.jsxs("div",{className:"alert alert-error",children:[e.jsx(y,{size:16}),i.submit]}),e.jsxs("div",{className:"form-group",children:[e.jsx("label",{children:"Account Holder Name *"}),e.jsx("input",{type:"text",value:t.account_holder_name,onChange:a=>o({...t,account_holder_name:a.target.value}),placeholder:"John Doe"}),i.account_holder_name&&e.jsx("span",{className:"error-text",children:i.account_holder_name})]}),e.jsxs("div",{className:"form-group",children:[e.jsx("label",{children:"Account Number *"}),e.jsx("input",{type:"text",value:t.account_number,onChange:a=>o({...t,account_number:a.target.value}),placeholder:"1234567890123456"}),i.account_number&&e.jsx("span",{className:"error-text",children:i.account_number})]}),e.jsxs("div",{className:"form-group",children:[e.jsx("label",{children:"IFSC Code *"}),e.jsx("input",{type:"text",value:t.ifsc_code,onChange:a=>o({...t,ifsc_code:a.target.value.toUpperCase()}),placeholder:"HDFC0001234"}),i.ifsc_code&&e.jsx("span",{className:"error-text",children:i.ifsc_code})]}),e.jsxs("div",{className:"form-group",children:[e.jsx("label",{children:"Account Type *"}),e.jsxs("div",{className:"account-type-toggle",role:"group","aria-label":"Account type",children:[e.jsx("button",{type:"button",className:`type-btn ${t.account_type==="savings"?"active":""}`,onClick:()=>o({...t,account_type:"savings"}),children:"Savings"}),e.jsx("button",{type:"button",className:`type-btn ${t.account_type==="current"?"active":""}`,onClick:()=>o({...t,account_type:"current"}),children:"Current"})]})]}),e.jsxs("div",{className:"form-actions",children:[e.jsx("button",{type:"button",className:"btn-secondary",onClick:c,children:"Cancel"}),e.jsx("button",{type:"submit",className:"btn-primary",disabled:p,children:p?"Adding...":"Add Account"})]})]})]}),e.jsx("style",{children:`
        .modal-overlay {
          position: fixed;
          top: 0;
          left: 0;
          right: 0;
          bottom: 0;
          background: rgba(4, 10, 24, 0.78);
          backdrop-filter: blur(5px);
          display: flex;
          align-items: center;
          justify-content: center;
          z-index: 2200;
          padding: 16px;
        }
        .modal-content {
          background:
            radial-gradient(140% 170% at 8% -2%, rgba(91, 154, 247, 0.22), transparent 48%),
            radial-gradient(130% 160% at 100% 0%, rgba(245, 158, 11, 0.14), transparent 46%),
            linear-gradient(180deg, rgba(13, 31, 61, 0.98), rgba(10, 24, 46, 0.98));
          border: 1px solid rgba(107, 168, 255, 0.34);
          border-radius: 18px;
          box-shadow: 0 18px 64px rgba(0, 0, 0, 0.46);
          max-width: 620px;
          width: 90%;
          max-height: 90vh;
          overflow-y: auto;
        }
        .modal-header {
          display: flex;
          justify-content: space-between;
          align-items: center;
          padding: 20px 22px;
          border-bottom: 1px solid rgba(107, 168, 255, 0.24);
        }
        .modal-kicker {
          font-size: 11px;
          text-transform: uppercase;
          letter-spacing: 0.16em;
          font-weight: 700;
          color: #9bc3ff;
          margin-bottom: 4px;
        }
        .modal-header h2 {
          margin: 0;
          font-size: 26px;
          line-height: 1.05;
          font-weight: 800;
          color: #f0f6ff;
        }
        .btn-close {
          background: rgba(14, 34, 63, 0.9);
          border: 1px solid rgba(91, 154, 247, 0.35);
          border-radius: 10px;
          cursor: pointer;
          padding: 8px;
          display: flex;
          align-items: center;
          justify-content: center;
          color: #c4dcff;
        }
        .btn-close:hover {
          border-color: rgba(127, 186, 255, 0.72);
          color: #eef5ff;
        }
        .form {
          padding: 20px 22px 24px;
          display: flex;
          flex-direction: column;
          gap: 14px;
        }
        .form-group label {
          display: block;
          margin-bottom: 7px;
          font-weight: 700;
          font-size: 13px;
          color: #c6dcfb;
          letter-spacing: 0.02em;
        }
        .form-group input {
          width: 100%;
          padding: 11px 13px;
          border: 1px solid rgba(91, 154, 247, 0.34);
          border-radius: 12px;
          font-size: 14px;
          font-family: inherit;
          color: #eaf3ff;
          background: linear-gradient(180deg, rgba(17, 41, 77, 0.94), rgba(12, 29, 56, 0.94));
          outline: none;
          transition: border-color 0.2s ease, box-shadow 0.2s ease;
        }
        .form-group input:focus {
          border-color: rgba(122, 182, 255, 0.95);
          box-shadow: 0 0 0 3px rgba(77, 141, 235, 0.2);
        }
        .form-group input::placeholder {
          color: #93b2de;
        }
        .account-type-toggle {
          display: grid;
          grid-template-columns: 1fr 1fr;
          gap: 10px;
          padding: 5px;
          border: 1px solid rgba(91, 154, 247, 0.34);
          border-radius: 12px;
          background: rgba(12, 29, 56, 0.7);
        }
        .type-btn {
          border: 1px solid rgba(97, 162, 255, 0.24);
          background: rgba(18, 46, 84, 0.6);
          color: #b8d3f6;
          border-radius: 9px;
          min-height: 42px;
          font-size: 14px;
          font-weight: 700;
          cursor: pointer;
          transition: all 0.2s ease;
        }
        .type-btn:hover {
          border-color: rgba(127, 186, 255, 0.52);
          color: #e8f2ff;
        }
        .type-btn.active {
          background: linear-gradient(135deg, #f8a429, #9f5cff);
          color: #fff;
          box-shadow: 0 4px 18px rgba(120, 74, 203, 0.38);
          border-color: rgba(255, 215, 155, 0.24);
        }
        .error-text {
          display: block;
          color: #ffabb4;
          font-size: 12px;
          margin-top: 6px;
        }
        .form-actions {
          display: flex;
          gap: 12px;
          margin-top: 10px;
          padding-top: 2px;
        }
        .form-actions button {
          flex: 1;
          min-height: 42px;
        }
        .alert.alert-error {
          display: flex;
          align-items: center;
          gap: 8px;
          border: 1px solid rgba(247, 83, 96, 0.42);
          background: rgba(247, 83, 96, 0.14);
          color: #ffb2ba;
          border-radius: 10px;
          padding: 10px 12px;
          margin-bottom: 12px;
          font-size: 13px;
          font-weight: 600;
        }
      `})]})}function P({accountId:c,onClose:t}){const[o,i]=n.useState(null),[l,p]=n.useState(""),[m,x]=n.useState(!0),[u,f]=n.useState(!1),[a,d]=n.useState(null),[b,j]=n.useState(!1),h=(()=>{const s=localStorage.getItem("feelpay_merchant");if(!s)return"";try{return JSON.parse(s).id||""}catch{return""}})();n.useEffect(()=>{v()},[]);const v=async()=>{try{const s=await g("GET",`/v1/merchants/${h}/bank-accounts/${c}/verification-data`);i(s)}catch(s){d(s instanceof Error?s.message:"Failed to load verification data")}finally{x(!1)}},r=async s=>{if(s.preventDefault(),!l){d("Please enter the penny drop amount");return}f(!0),d(null);try{await g("POST",`/v1/merchants/${h}/bank-accounts/${c}/verify`,{penny_drop_amount_paise:parseInt(l)}),j(!0),setTimeout(t,2e3)}catch(w){d(w instanceof Error?w.message:"Verification failed")}finally{f(!1)}};return m?e.jsx("div",{className:"modal-overlay",onClick:t,children:e.jsx("div",{className:"modal-content",onClick:s=>s.stopPropagation(),children:e.jsx("div",{className:"loading",children:"Loading verification data..."})})}):e.jsxs("div",{className:"modal-overlay",onClick:t,children:[e.jsxs("div",{className:"modal-content",onClick:s=>s.stopPropagation(),children:[e.jsxs("div",{className:"modal-header",children:[e.jsx("h2",{children:"Verify Bank Account"}),e.jsx("button",{className:"btn-close",onClick:t,children:e.jsx(N,{size:20})})]}),e.jsxs("form",{onSubmit:r,className:"form",children:[a&&e.jsxs("div",{className:"alert alert-error",children:[e.jsx(y,{size:16}),a]}),b&&e.jsxs("div",{className:"alert alert-success",children:[e.jsx(_,{size:16}),"Account verified successfully!"]}),e.jsxs("div",{className:"verification-info",children:[e.jsx("h3",{children:"Penny Drop Verification"}),e.jsx("p",{children:"A small amount (₹1-2) has been deposited to your bank account. Enter the exact amount you received to verify the account."}),o&&e.jsxs("div",{className:"info-box",children:[e.jsxs("div",{className:"info-row",children:[e.jsx("span",{children:"Reference Code:"}),e.jsx("strong",{className:"mono",children:o.penny_drop_reference})]}),e.jsxs("div",{className:"info-row",children:[e.jsx("span",{children:"Expected Amount:"}),e.jsxs("strong",{children:["₹",(o.penny_drop_amount_paise/100).toFixed(2)]})]})]})]}),e.jsxs("div",{className:"form-group",children:[e.jsx("label",{children:"Enter Penny Drop Amount (in paise) *"}),e.jsx("input",{type:"number",value:l,onChange:s=>p(s.target.value),placeholder:"e.g., 150 for ₹1.50",disabled:b}),e.jsx("small",{children:"Enter the amount in paise (e.g., 150 for ₹1.50)"})]}),e.jsxs("div",{className:"form-actions",children:[e.jsx("button",{type:"button",className:"btn-secondary",onClick:t,disabled:b,children:"Cancel"}),e.jsx("button",{type:"submit",className:"btn-primary",disabled:u||b,children:u?"Verifying...":"Verify Account"})]})]})]}),e.jsx("style",{children:`
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
          max-width: 500px;
          width: 90%;
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
          font-size: 18px;
        }
        .btn-close {
          background: none;
          border: none;
          cursor: pointer;
          padding: 4px;
          display: flex;
          align-items: center;
          justify-content: center;
          color: var(--text-secondary);
        }
        .form {
          padding: 20px;
        }
        .verification-info {
          margin-bottom: 20px;
          padding: 16px;
          background: var(--bg-secondary);
          border-radius: 8px;
        }
        .verification-info h3 {
          margin: 0 0 8px 0;
          font-size: 14px;
        }
        .verification-info p {
          margin: 0 0 12px 0;
          font-size: 13px;
          color: var(--text-secondary);
        }
        .info-box {
          display: flex;
          flex-direction: column;
          gap: 8px;
        }
        .info-row {
          display: flex;
          justify-content: space-between;
          font-size: 13px;
        }
        .form-group {
          margin-bottom: 16px;
        }
        .form-group label {
          display: block;
          margin-bottom: 6px;
          font-weight: 500;
          font-size: 14px;
        }
        .form-group input {
          width: 100%;
          padding: 8px 12px;
          border: 1px solid var(--border-color);
          border-radius: 4px;
          font-size: 14px;
          font-family: inherit;
        }
        .form-group small {
          display: block;
          margin-top: 4px;
          font-size: 12px;
          color: var(--text-secondary);
        }
        .form-actions {
          display: flex;
          gap: 12px;
          margin-top: 24px;
        }
        .form-actions button {
          flex: 1;
        }
      `})]})}function E({accountId:c,onClose:t}){const[o,i]=n.useState(!1),[l,p]=n.useState(null),[m,x]=n.useState(!1),u=(()=>{const a=localStorage.getItem("feelpay_merchant");if(!a)return"";try{return JSON.parse(a).id||""}catch{return""}})(),f=async()=>{i(!0),p(null);try{await g("PUT",`/v1/merchants/${u}/bank-accounts/${c}/set-primary`,{}),x(!0),setTimeout(t,2e3)}catch(a){p(a instanceof Error?a.message:"Failed to set primary account")}finally{i(!1)}};return e.jsxs("div",{className:"modal-overlay",onClick:t,children:[e.jsxs("div",{className:"modal-content",onClick:a=>a.stopPropagation(),children:[e.jsxs("div",{className:"modal-header",children:[e.jsx("h2",{children:"Set as Primary Account"}),e.jsx("button",{className:"btn-close",onClick:t,children:e.jsx(N,{size:20})})]}),e.jsxs("div",{className:"modal-body",children:[l&&e.jsxs("div",{className:"alert alert-error",children:[e.jsx(y,{size:16}),l]}),m&&e.jsxs("div",{className:"alert alert-success",children:[e.jsx(_,{size:16}),"Account set as primary successfully!"]}),!m&&e.jsxs(e.Fragment,{children:[e.jsx("p",{children:"This account will be used as the default settlement destination for all future payouts."}),e.jsx("p",{className:"warning-text",children:"This action cannot be undone immediately. You can change it later if needed."}),e.jsxs("div",{className:"form-actions",children:[e.jsx("button",{type:"button",className:"btn-secondary",onClick:t,disabled:o,children:"Cancel"}),e.jsx("button",{type:"button",className:"btn-primary",onClick:f,disabled:o,children:o?"Setting...":"Set as Primary"})]})]})]})]}),e.jsx("style",{children:`
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
          max-width: 400px;
          width: 90%;
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
          font-size: 18px;
        }
        .btn-close {
          background: none;
          border: none;
          cursor: pointer;
          padding: 4px;
          display: flex;
          align-items: center;
          justify-content: center;
          color: var(--text-secondary);
        }
        .modal-body {
          padding: 20px;
        }
        .modal-body p {
          margin: 0 0 12px 0;
          font-size: 14px;
          line-height: 1.5;
        }
        .warning-text {
          color: var(--text-secondary);
          font-size: 13px;
        }
        .form-actions {
          display: flex;
          gap: 12px;
          margin-top: 24px;
        }
        .form-actions button {
          flex: 1;
        }
      `})]})}function B(){const[c,t]=n.useState([]),[o,i]=n.useState(!0),[l,p]=n.useState(!1),[m,x]=n.useState(null),[u,f]=n.useState(null),a=n.useMemo(()=>{const r=localStorage.getItem("feelpay_merchant");if(!r)return"";try{return JSON.parse(r).id||""}catch{return""}},[]);n.useEffect(()=>{d()},[]);const d=async()=>{if(!a){i(!1);return}try{const r=await g("GET",`/v1/merchants/${a}/bank-accounts`,void 0,void 0,{cacheMs:45e3});t(r.bank_accounts||r.accounts||[])}catch(r){console.error("Failed to fetch bank accounts:",r)}finally{i(!1)}},b=async r=>{if(confirm("Delete this bank account?"))try{await g("DELETE",`/v1/merchants/${a}/bank-accounts/${r}`),t(c.filter(s=>s.id!==r))}catch(s){console.error("Failed to delete account:",s)}},j=()=>{p(!1),d()},h=()=>{x(null),d()},v=()=>{f(null),d()};return o?e.jsx("div",{className:"page-container",children:e.jsx("div",{className:"loading",children:"Loading bank accounts..."})}):e.jsxs("div",{className:"page-container",children:[e.jsxs("div",{className:"page-header",children:[e.jsx("h1",{children:"Bank Account Management"}),e.jsx("p",{children:"Manage settlement bank accounts and verification"})]}),l&&e.jsx(C,{onClose:j}),m&&e.jsx(P,{accountId:m,onClose:h}),u&&e.jsx(E,{accountId:u,onClose:v}),e.jsxs("div",{className:"card",children:[e.jsxs("div",{className:"card-header",children:[e.jsx("h2",{children:"Bank Accounts"}),e.jsxs("button",{className:"btn-primary",onClick:()=>p(!0),children:[e.jsx(S,{size:16}),"Add Bank Account"]})]}),c.length===0?e.jsxs("div",{className:"bank-empty",children:[e.jsxs("div",{className:"bank-empty-left",children:[e.jsx("div",{className:"bank-empty-icon",children:e.jsx(z,{size:28})}),e.jsx("h3",{children:"No bank accounts configured"}),e.jsx("p",{children:"Add payout account to unlock settlements, automated transfers, and faster reconciliation."}),e.jsx("div",{className:"bank-empty-cta",children:e.jsxs("button",{className:"btn-primary",onClick:()=>p(!0),children:[e.jsx(S,{size:16}),"Add First Account"]})})]}),e.jsxs("div",{className:"bank-empty-right",children:[e.jsxs("div",{className:"bank-empty-check",children:[e.jsx(k,{size:16}),"Verification with penny-drop validation"]}),e.jsxs("div",{className:"bank-empty-check",children:[e.jsx(k,{size:16}),"Primary account failover controls"]}),e.jsxs("div",{className:"bank-empty-check",children:[e.jsx(k,{size:16}),"Full payout audit trail support"]})]})]}):e.jsx("div",{className:"accounts-grid",children:c.map(r=>e.jsxs("div",{className:"account-card",children:[e.jsxs("div",{className:"account-header",children:[e.jsxs("div",{children:[e.jsx("h3",{children:r.account_holder_name}),e.jsx("p",{className:"account-number",children:r.account_number_masked})]}),r.is_primary&&e.jsx("span",{className:"badge badge-primary",children:"Primary"})]}),e.jsxs("div",{className:"account-details",children:[e.jsxs("div",{className:"detail-row",children:[e.jsx("span",{children:"IFSC:"}),e.jsx("strong",{className:"mono",children:r.ifsc_code})]}),e.jsxs("div",{className:"detail-row",children:[e.jsx("span",{children:"Type:"}),e.jsx("strong",{children:r.account_type})]}),e.jsxs("div",{className:"detail-row",children:[e.jsx("span",{children:"Status:"}),e.jsxs("span",{className:`badge badge-${r.verification_status}`,children:[r.verification_status==="verified"&&e.jsx(_,{size:12}),r.verification_status==="failed"&&e.jsx(y,{size:12}),r.verification_status]})]}),r.verified_at&&e.jsxs("div",{className:"detail-row",children:[e.jsx("span",{children:"Verified:"}),e.jsx("span",{children:new Date(r.verified_at).toLocaleDateString()})]})]}),e.jsxs("div",{className:"account-actions",children:[r.verification_status==="pending"&&e.jsx("button",{className:"btn-secondary",onClick:()=>x(r.id),children:"Verify"}),!r.is_primary&&r.verification_status==="verified"&&e.jsx("button",{className:"btn-secondary",onClick:()=>f(r.id),children:"Set as Primary"}),e.jsx("button",{className:"btn-danger",onClick:()=>b(r.id),children:e.jsx(A,{size:14})})]})]},r.id))})]}),e.jsx("style",{children:`
        .card {
          border-radius: 16px;
          border: 1px solid rgba(98, 160, 250, 0.3);
          background:
            radial-gradient(150% 190% at 0% 0%, rgba(91, 154, 247, 0.15), transparent 42%),
            linear-gradient(180deg, rgba(17, 39, 72, 0.96), rgba(10, 25, 47, 0.96));
          overflow: hidden;
        }
        .card-header h2 {
          font-size: 32px;
          line-height: 1.06;
          letter-spacing: -0.02em;
        }
        .bank-empty {
          min-height: 260px;
          display: grid;
          grid-template-columns: 1.2fr 1fr;
          gap: 24px;
          padding: 8px 4px 2px;
        }
        .bank-empty-left {
          display: flex;
          flex-direction: column;
          align-items: flex-start;
          justify-content: center;
          gap: 10px;
        }
        .bank-empty-icon {
          width: 54px;
          height: 54px;
          border-radius: 14px;
          display: grid;
          place-items: center;
          color: #e8f2ff;
          border: 1px solid rgba(114, 174, 255, 0.44);
          background: linear-gradient(135deg, rgba(245, 158, 11, 0.32), rgba(139, 92, 246, 0.34));
        }
        .bank-empty h3 {
          font-size: 24px;
          line-height: 1.05;
          margin: 4px 0 0;
          letter-spacing: -0.02em;
        }
        .bank-empty p {
          color: #a4bfdf;
          font-size: 14px;
          max-width: 440px;
        }
        .bank-empty-cta {
          margin-top: 4px;
        }
        .bank-empty-right {
          border: 1px solid rgba(103, 165, 255, 0.24);
          background: rgba(14, 34, 63, 0.62);
          border-radius: 14px;
          padding: 14px;
          display: flex;
          flex-direction: column;
          justify-content: center;
          gap: 10px;
        }
        .bank-empty-check {
          display: flex;
          align-items: center;
          gap: 8px;
          color: #c3daf9;
          font-size: 13px;
          line-height: 1.35;
        }
        .bank-empty-check svg {
          flex: 0 0 auto;
          color: #7fd6a6;
        }
        .accounts-grid {
          display: grid;
          grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
          gap: 16px;
        }
        .account-card {
          border: 1px solid rgba(95, 158, 250, 0.28);
          border-radius: 12px;
          padding: 16px;
          background: linear-gradient(180deg, rgba(16, 38, 70, 0.86), rgba(10, 24, 45, 0.86));
          display: flex;
          flex-direction: column;
          gap: 12px;
        }
        .account-header {
          display: flex;
          justify-content: space-between;
          align-items: flex-start;
          gap: 12px;
        }
        .account-header h3 {
          margin: 0;
          font-size: 16px;
        }
        .account-number {
          margin: 4px 0 0 0;
          font-size: 13px;
          color: #9fbcdf;
          font-family: monospace;
        }
        .account-details {
          display: flex;
          flex-direction: column;
          gap: 8px;
          padding: 12px 0;
          border-top: 1px solid var(--border-color);
          border-bottom: 1px solid var(--border-color);
        }
        .detail-row {
          display: flex;
          justify-content: space-between;
          font-size: 13px;
        }
        .detail-row span:first-child {
          color: #9fbcdf;
        }
        .account-actions {
          display: flex;
          gap: 8px;
        }
        .account-actions button {
          flex: 1;
          padding: 8px 12px;
          font-size: 13px;
        }
        .btn-danger {
          background: rgba(247, 83, 96, 0.16);
          border: 1px solid rgba(247, 83, 96, 0.45);
          color: #ffb1b8;
          border-radius: 10px;
          cursor: pointer;
          padding: 8px 12px;
          font-size: 13px;
          transition: all 0.2s;
        }
        .btn-danger:hover {
          background: rgba(247, 83, 96, 0.24);
        }
        @media (max-width: 900px) {
          .bank-empty {
            grid-template-columns: 1fr;
          }
        }
      `})]})}export{B as default};
