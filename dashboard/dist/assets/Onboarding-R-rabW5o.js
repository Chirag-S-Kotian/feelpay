import{c as P,r,j as e,B as z,a as c}from"./index-DmZi9o4B.js";import{S as V}from"./shield-check-By1IlAFg.js";import{L as q}from"./landmark-Clv75ufG.js";import{R as M}from"./rocket-CrF7_YKj.js";/**
 * @license lucide-react v0.469.0 - ISC
 *
 * This source code is licensed under the ISC license.
 * See the LICENSE file in the root directory of this source tree.
 */const w=P("CircleCheck",[["circle",{cx:"12",cy:"12",r:"10",key:"1mglay"}],["path",{d:"m9 12 2 2 4-4",key:"dzmm74"}]]);/**
 * @license lucide-react v0.469.0 - ISC
 *
 * This source code is licensed under the ISC license.
 * See the LICENSE file in the root directory of this source tree.
 */const L=P("Circle",[["circle",{cx:"12",cy:"12",r:"10",key:"1mglay"}]]),H=[{id:1,name:"Business Profile",icon:z},{id:2,name:"KYC Verification",icon:V},{id:3,name:"Settlement Account",icon:q},{id:4,name:"Go-Live Review",icon:M}],$=[{value:"pvt_ltd",label:"Private Limited"},{value:"llp",label:"LLP"},{value:"sole_proprietor",label:"Sole Proprietor"},{value:"partnership",label:"Partnership"},{value:"trust",label:"Trust"},{value:"ngo",label:"NGO"}];function Q(){var T;const[n,f]=r.useState(1),[B,k]=r.useState(!0),[h,C]=r.useState(!1),[S,u]=r.useState(""),[_,b]=r.useState(""),[a,E]=r.useState(null),[p,D]=r.useState(null),[j,v]=r.useState("not_started"),[y,A]=r.useState(!0),[s,t]=r.useState({businessName:"",businessType:"",pan:"",gstin:"",addressLine1:"",city:"",state:"",postalCode:"",bankName:"",accountHolderName:"",accountNumber:"",ifscCode:""}),x=r.useMemo(()=>{const i=(a==null?void 0:a.verifications)||[],o=i.some(d=>d.verification_type==="pan"&&d.status==="verified"),m=s.gstin.trim().length===0||i.some(d=>d.verification_type==="gstin"&&d.status==="verified"),g=i.some(d=>d.verification_type==="bank")||(a==null?void 0:a.onboarding_status)!=="pending";return{business:s.businessName.trim().length>0&&s.businessType.trim().length>0,kyc:o&&m,bank:g,review:(a==null?void 0:a.onboarding_status)==="approved"||(a==null?void 0:a.onboarding_status)==="in_progress"}},[a,s]),N=async()=>{k(!0),u("");try{const[i,o,m,g]=await Promise.all([c("GET","/v1/onboarding/status",void 0,void 0,{cacheMs:2e4}),c("GET","/v1/merchants/profile",void 0,void 0,{cacheMs:3e4}),c("GET","/v1/onboarding/limits",void 0,void 0,{cacheMs:2e4}).then(l=>({ok:!0,value:l})).catch(()=>({ok:!1})),c("GET","/v1/onboarding/digilocker/status",void 0,void 0,{cacheMs:2e4}).then(l=>({ok:!0,value:l})).catch(l=>({ok:!1,err:l}))]),d=m.ok?m.value:{per_transaction_limit:0,daily_limit:i.daily_limit||0,monthly_limit:i.monthly_limit||0,settlement_cycle:"T+2"};g.ok?(v(g.value.status||"not_started"),A(!0)):(g.err instanceof Error?g.err.message:"").includes("disabled")?(A(!1),v("disabled")):v("not_started"),E(i),D(d),t(l=>({...l,businessName:o.business_name||l.businessName,businessType:o.business_type||l.businessType}))}catch(i){u(i instanceof Error?i.message:"Failed to load onboarding data")}finally{k(!1)}};r.useEffect(()=>{N()},[]);const R=i=>{if(i===2){if(!s.businessName.trim())return"Business name required";if(!s.businessType.trim())return"Business type required"}if(i===3){if(!/^[A-Z]{5}[0-9]{4}[A-Z]$/.test(s.pan.trim().toUpperCase()))return"Valid PAN required";if(s.gstin.trim()&&!/^[0-9]{2}[A-Z]{5}[0-9]{4}[A-Z]{1}[A-Z0-9]{1}Z[A-Z0-9]{1}$/.test(s.gstin.trim().toUpperCase()))return"GSTIN format invalid"}if(i===4){if(!s.accountHolderName.trim())return"Account holder name required";if(!/^\d{8,20}$/.test(s.accountNumber.trim()))return"Valid account number required";if(!/^[A-Z]{4}0[A-Z0-9]{6}$/.test(s.ifscCode.trim().toUpperCase()))return"Valid IFSC required"}return null},O=async()=>{await c("POST","/v1/onboarding/business-info",{business_name:s.businessName.trim(),business_type:s.businessType,pan:s.pan.trim().toUpperCase()||"AAAAA0000A",gstin:s.gstin.trim().toUpperCase()||null,address:{line1:s.addressLine1,city:s.city,state:s.state,postal_code:s.postalCode}})},F=async()=>{await c("POST","/v1/onboarding/kyc",{verification_type:"pan",document_number:s.pan.trim().toUpperCase()}),s.gstin.trim()&&await c("POST","/v1/onboarding/kyc",{verification_type:"gstin",document_number:s.gstin.trim().toUpperCase()})},G=async()=>{await c("POST","/v1/onboarding/bank-account",{account_holder_name:s.accountHolderName.trim(),account_number:s.accountNumber.trim(),ifsc_code:s.ifscCode.trim().toUpperCase(),bank_name:s.bankName.trim()||void 0})},I=async()=>{u(""),b("");const i=R(n+1);if(i){u(i);return}C(!0);try{n===1&&await O(),n===2&&await F(),n===3&&await G(),await c("POST","/v1/onboarding/registration/step",{step:n+1,data:{ui_step:n+1,updated_at:new Date().toISOString()}}),await N(),f(o=>Math.min(4,o+1)),b(n===3?"Onboarding submitted. Verification in progress.":"Step saved.")}catch(o){u(o instanceof Error?o.message:"Failed to save step")}finally{C(!1)}},U=[{label:"Business profile",ok:x.business},{label:"PAN / GST verification",ok:x.kyc},{label:"DigiLocker verification",ok:j==="verified"||!y},{label:"Settlement account",ok:x.bank},{label:"Activation state",ok:x.review}];if(B)return e.jsx("div",{className:"loading",children:"Loading onboarding..."});const Z=async()=>{u(""),b("");try{const i=await c("POST","/v1/onboarding/digilocker/initiate",{purpose:"merchant_kyc"});i.redirect_url&&window.open(i.redirect_url,"_blank","noopener,noreferrer"),b("DigiLocker flow initiated. Complete auth, then click Refresh Verification Status."),v("pending")}catch(i){u(i instanceof Error?i.message:"Failed to initiate DigiLocker")}};return e.jsxs("div",{className:"page",children:[e.jsx("div",{className:"page-header",children:e.jsxs("div",{children:[e.jsx("h1",{children:"Merchant Onboarding"}),e.jsx("p",{children:"Progressive onboarding like payment gateways: collect minimal info first, verify, then activate."})]})}),S&&e.jsx("div",{className:"alert alert-error",children:S}),_&&e.jsx("div",{className:"env-badge",children:_}),e.jsx("div",{className:"card",style:{marginBottom:16},children:e.jsx("div",{className:"onboarding-steps",children:H.map(i=>{const o=i.icon,m=n===i.id;return e.jsxs("button",{type:"button",onClick:()=>i.id<=n&&f(i.id),className:`onboarding-step ${m?"active":""}`,disabled:i.id>n,children:[e.jsx(o,{size:14}),i.name]},i.id)})})}),e.jsxs("div",{className:"card",children:[n===1&&e.jsxs(e.Fragment,{children:[e.jsx("h3",{children:"Business Profile"}),e.jsxs("div",{className:"input-grid",children:[e.jsxs("div",{className:"input-group",children:[e.jsx("label",{children:"Business Name"}),e.jsx("input",{value:s.businessName,onChange:i=>t({...s,businessName:i.target.value})})]}),e.jsxs("div",{className:"input-group",children:[e.jsx("label",{children:"Business Type"}),e.jsxs("select",{value:s.businessType,onChange:i=>t({...s,businessType:i.target.value}),children:[e.jsx("option",{value:"",children:"Select type"}),$.map(i=>e.jsx("option",{value:i.value,children:i.label},i.value))]})]}),e.jsxs("div",{className:"input-group",children:[e.jsx("label",{children:"Address Line 1"}),e.jsx("input",{value:s.addressLine1,onChange:i=>t({...s,addressLine1:i.target.value})})]}),e.jsxs("div",{className:"input-group",children:[e.jsx("label",{children:"City"}),e.jsx("input",{value:s.city,onChange:i=>t({...s,city:i.target.value})})]}),e.jsxs("div",{className:"input-group",children:[e.jsx("label",{children:"State"}),e.jsx("input",{value:s.state,onChange:i=>t({...s,state:i.target.value})})]}),e.jsxs("div",{className:"input-group",children:[e.jsx("label",{children:"Postal Code"}),e.jsx("input",{value:s.postalCode,onChange:i=>t({...s,postalCode:i.target.value})})]})]})]}),n===2&&e.jsxs(e.Fragment,{children:[e.jsx("h3",{children:"KYC Verification"}),e.jsxs("div",{className:"input-grid",children:[e.jsxs("div",{className:"input-group",children:[e.jsx("label",{children:"PAN"}),e.jsx("input",{value:s.pan,maxLength:10,onChange:i=>t({...s,pan:i.target.value.toUpperCase()}),placeholder:"ABCDE1234F"})]}),e.jsxs("div",{className:"input-group",children:[e.jsx("label",{children:"GSTIN (optional)"}),e.jsx("input",{value:s.gstin,maxLength:15,onChange:i=>t({...s,gstin:i.target.value.toUpperCase()}),placeholder:"22AAAAA0000A1Z5"})]})]}),e.jsxs("div",{className:"card",style:{marginTop:12},children:[e.jsx("h4",{style:{marginBottom:6},children:"DigiLocker Auto-KYC"}),e.jsx("p",{className:"mini-help",style:{marginTop:0},children:"Enterprise path: fetch verified docs via DigiLocker to accelerate approval."}),e.jsxs("div",{className:"review-row",style:{marginBottom:8},children:[j==="verified"?e.jsx(w,{size:16,color:"#5ad189"}):e.jsx(L,{size:16,color:"#9fb8da"}),e.jsxs("span",{children:["Status: ",j]})]}),e.jsx("button",{type:"button",className:"btn-secondary",onClick:()=>void Z(),disabled:!y||h,children:y?"Connect DigiLocker":"DigiLocker Disabled by Infra"})]}),e.jsx("p",{className:"mini-help",children:"Pattern from Stripe/Razorpay: collect mandatory first (PAN), optional next (GSTIN), then verify async."})]}),n===3&&e.jsxs(e.Fragment,{children:[e.jsx("h3",{children:"Settlement Bank Account"}),e.jsxs("div",{className:"input-grid",children:[e.jsxs("div",{className:"input-group",children:[e.jsx("label",{children:"Account Holder Name"}),e.jsx("input",{value:s.accountHolderName,onChange:i=>t({...s,accountHolderName:i.target.value})})]}),e.jsxs("div",{className:"input-group",children:[e.jsx("label",{children:"Bank Name"}),e.jsx("input",{value:s.bankName,onChange:i=>t({...s,bankName:i.target.value})})]}),e.jsxs("div",{className:"input-group",children:[e.jsx("label",{children:"Account Number"}),e.jsx("input",{value:s.accountNumber,onChange:i=>t({...s,accountNumber:i.target.value})})]}),e.jsxs("div",{className:"input-group",children:[e.jsx("label",{children:"IFSC Code"}),e.jsx("input",{value:s.ifscCode,maxLength:11,onChange:i=>t({...s,ifscCode:i.target.value.toUpperCase()})})]})]})]}),n===4&&e.jsxs(e.Fragment,{children:[e.jsx("h3",{children:"Go-Live Review"}),e.jsx("div",{className:"review-list",children:U.map(i=>e.jsxs("div",{className:"review-row",children:[i.ok?e.jsx(w,{size:16,color:"#5ad189"}):e.jsx(L,{size:16,color:"#9fb8da"}),e.jsx("span",{children:i.label})]},i.label))}),e.jsxs("div",{className:"ops-grid",style:{marginTop:10},children:[e.jsxs("div",{className:"ops-tile",children:[e.jsx("span",{children:"Tier"}),e.jsx("strong",{children:(a==null?void 0:a.tier)||"N/A"})]}),e.jsxs("div",{className:"ops-tile",children:[e.jsx("span",{children:"Onboarding Status"}),e.jsx("strong",{children:(a==null?void 0:a.onboarding_status)||"pending"})]}),e.jsxs("div",{className:"ops-tile",children:[e.jsx("span",{children:"Per Txn Limit"}),e.jsxs("strong",{children:["INR ",((T=p==null?void 0:p.per_transaction_limit)==null?void 0:T.toLocaleString())||0]})]}),e.jsxs("div",{className:"ops-tile",children:[e.jsx("span",{children:"Settlement"}),e.jsx("strong",{children:(p==null?void 0:p.settlement_cycle)||"T+2"})]})]})]}),e.jsxs("div",{className:"form-actions",style:{marginTop:16},children:[n>1&&e.jsx("button",{type:"button",className:"btn-secondary",onClick:()=>f(n-1),disabled:h,children:"Back"}),n<4&&e.jsx("button",{type:"button",className:"btn-primary",onClick:()=>void I(),disabled:h,children:h?"Saving...":"Continue"}),n===4&&e.jsx("button",{type:"button",className:"btn-primary",onClick:()=>void N(),disabled:h,children:"Refresh Verification Status"})]})]}),e.jsx("style",{children:`
        .alert.alert-error {
          border: 1px solid rgba(248, 113, 113, 0.4);
          background: rgba(239, 68, 68, 0.15);
          color: #fecaca;
          padding: 10px 12px;
          border-radius: 10px;
          margin-bottom: 12px;
        }
        .onboarding-steps {
          display: flex;
          flex-wrap: wrap;
          gap: 8px;
        }
        .onboarding-step {
          border: 1px solid rgba(102, 163, 252, 0.24);
          background: rgba(13, 33, 60, 0.72);
          color: #cbe1ff;
          border-radius: 999px;
          padding: 8px 12px;
          display: inline-flex;
          align-items: center;
          gap: 6px;
          cursor: pointer;
          font-size: 12px;
          font-weight: 600;
        }
        .onboarding-step.active {
          background: linear-gradient(135deg, #f59e0b, #8b5cf6);
          border-color: rgba(255, 205, 130, 0.4);
          color: #fff;
        }
        .onboarding-step:disabled {
          opacity: 0.55;
          cursor: not-allowed;
        }
        .input-grid {
          display: grid;
          grid-template-columns: repeat(auto-fit, minmax(240px, 1fr));
          gap: 12px;
        }
        .mini-help {
          margin-top: 10px;
          color: #9db7da;
          font-size: 13px;
        }
        .review-list {
          display: grid;
          gap: 8px;
        }
        .review-row {
          display: flex;
          align-items: center;
          gap: 8px;
          color: #d9eaff;
        }
      `})]})}export{Q as default};
