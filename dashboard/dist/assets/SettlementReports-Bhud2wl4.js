import{c as N,r as i,a as x,j as e}from"./index-DmZi9o4B.js";import{R as A}from"./refresh-cw-CLAn8pyW.js";import{D as L}from"./download-BcZMe9fw.js";/**
 * @license lucide-react v0.469.0 - ISC
 *
 * This source code is licensed under the ISC license.
 * See the LICENSE file in the root directory of this source tree.
 */const w=N("FileText",[["path",{d:"M15 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7Z",key:"1rqfz7"}],["path",{d:"M14 2v4a2 2 0 0 0 2 2h4",key:"tnqrlb"}],["path",{d:"M10 9H8",key:"b1mrlr"}],["path",{d:"M16 13H8",key:"t4e002"}],["path",{d:"M16 17H8",key:"z1uh3a"}]]);/**
 * @license lucide-react v0.469.0 - ISC
 *
 * This source code is licensed under the ISC license.
 * See the LICENSE file in the root directory of this source tree.
 */const C=N("Loader",[["path",{d:"M12 2v4",key:"3427ic"}],["path",{d:"m16.2 7.8 2.9-2.9",key:"r700ao"}],["path",{d:"M18 12h4",key:"wj9ykh"}],["path",{d:"m16.2 16.2 2.9 2.9",key:"1bxg5t"}],["path",{d:"M12 18v4",key:"jadmvz"}],["path",{d:"m4.9 19.1 2.9-2.9",key:"bwix9q"}],["path",{d:"M2 12h4",key:"j09sii"}],["path",{d:"m4.9 4.9 2.9 2.9",key:"giyufr"}]]);function M(){var v;const[h,y]=i.useState([]),[k,S]=i.useState(!0),[m,F]=i.useState("pdf"),[g,u]=i.useState(null),[c,D]=i.useState(null),[r,f]=i.useState(null),[j,d]=i.useState("");i.useEffect(()=>{b()},[]);const b=async()=>{try{d("");const t=await x("GET","/v1/settlements?page=1&per_page=50");y(t.settlements||[])}catch(t){console.error("Failed to fetch settlements:",t);const a=t instanceof Error?t.message:"Failed to load settlements";d(a)}finally{S(!1)}},R=async t=>{D(t);try{d("");const a=await x("GET",`/v1/settlements/${t.id}`);f(a)}catch(a){console.error("Failed to fetch report preview:",a);const s=a instanceof Error?a.message:"Failed to load settlement detail";d(s)}},_=t=>{var s;return[["Settlement ID",t.id],["Settlement Date",t.settlement_date||"N/A"],["Status",t.status],["UTR",t.utr||"N/A"],["Gross Amount (INR)",(t.amount/100).toFixed(2)],["Fee Amount (INR)",(t.fee_amount/100).toFixed(2)],["Tax Amount (INR)",(t.tax_amount/100).toFixed(2)],["Net Amount (INR)",(t.net_amount/100).toFixed(2)],["Transaction Count",`${((s=t.items)==null?void 0:s.length)||0}`],[],["Transaction ID","Amount (INR)","Fee (INR)"],...(t.items||[]).map(n=>[n.transaction_id,(n.amount/100).toFixed(2),(n.fee/100).toFixed(2)])].map(n=>n.map(l=>`"${String(l??"").replace(/"/g,'""')}"`).join(",")).join(`
`)},p=(t,a,s)=>{const n=new Blob([a],{type:s}),l=URL.createObjectURL(n),o=document.createElement("a");o.href=l,o.download=t,document.body.appendChild(o),o.click(),document.body.removeChild(o),URL.revokeObjectURL(l)},T=async(t,a)=>{u(t);try{d("");let s=r;if((!s||s.id!==t)&&(s=await x("GET",`/v1/settlements/${t}`),f(s)),!s)throw new Error("Settlement detail unavailable");const n=_(s),l=new Date().toISOString().slice(0,10),o=`settlement-${s.id}-${l}`;if(a==="csv")p(`${o}.csv`,n,"text/csv;charset=utf-8");else if(a==="excel")p(`${o}.xls`,n,"application/vnd.ms-excel;charset=utf-8");else{const z=`Settlement Summary

${n.replace(/,/g," | ")}`;p(`${o}.txt`,z,"text/plain;charset=utf-8")}}catch(s){console.error("Download failed:",s);const n=s instanceof Error?s.message:"Download failed";d(n)}finally{u(null)}};return k?e.jsx("div",{className:"page-container",children:e.jsx("div",{className:"loading",children:"Loading settlements..."})}):e.jsxs("div",{className:"page-container",children:[e.jsxs("div",{className:"page-header",children:[e.jsx("h1",{children:"Settlement Reports"}),e.jsx("p",{children:"Download detailed settlement reports"})]}),j&&e.jsx("div",{className:"error-banner",role:"alert",children:j}),e.jsxs("div",{className:"grid grid-2",children:[e.jsxs("div",{className:"card",children:[e.jsxs("div",{className:"card-title-row",children:[e.jsx("h2",{children:"Settlements"}),e.jsxs("button",{className:"refresh-btn",onClick:b,type:"button",children:[e.jsx(A,{size:14}),"Refresh"]})]}),h.length===0?e.jsxs("div",{className:"empty-state",children:[e.jsx(w,{size:32}),e.jsx("p",{children:"No settlements found"})]}):e.jsx("div",{className:"table-container",children:e.jsxs("table",{className:"data-table",children:[e.jsx("thead",{children:e.jsxs("tr",{children:[e.jsx("th",{children:"Date"}),e.jsx("th",{children:"Status"}),e.jsx("th",{children:"Transactions"}),e.jsx("th",{children:"Gross Amount"}),e.jsx("th",{children:"Fee + Tax"}),e.jsx("th",{children:"Net Amount"}),e.jsx("th",{children:"Action"})]})}),e.jsx("tbody",{children:h.map(t=>e.jsxs("tr",{children:[e.jsx("td",{children:new Date(t.settlement_date||t.created_at).toLocaleDateString()}),e.jsx("td",{children:e.jsx("span",{className:`status-pill status-${t.status.toLowerCase()}`,children:t.status})}),e.jsx("td",{children:"-"}),e.jsxs("td",{children:["₹",(t.amount/100).toFixed(2)]}),e.jsxs("td",{children:["₹",((t.fee_amount+t.tax_amount)/100).toFixed(2)]}),e.jsxs("td",{className:"font-bold",children:["₹",(t.net_amount/100).toFixed(2)]}),e.jsx("td",{children:e.jsx("button",{className:"btn-small",onClick:()=>R(t),children:e.jsx(w,{size:14})})})]},t.id))})]})})]}),c&&r&&e.jsxs("div",{className:"card",children:[e.jsx("h2",{children:"Report Preview"}),e.jsxs("div",{className:"report-preview",children:[e.jsxs("div",{className:"preview-section",children:[e.jsx("h3",{children:"Settlement Summary"}),e.jsxs("div",{className:"preview-row",children:[e.jsx("span",{children:"Settlement Date:"}),e.jsx("strong",{children:new Date(r.settlement_date||r.created_at).toLocaleDateString()})]}),e.jsxs("div",{className:"preview-row",children:[e.jsx("span",{children:"Transactions:"}),e.jsx("strong",{children:((v=r.items)==null?void 0:v.length)||0})]})]}),e.jsxs("div",{className:"preview-section",children:[e.jsx("h3",{children:"Financial Breakdown"}),e.jsxs("div",{className:"preview-row",children:[e.jsx("span",{children:"Gross Amount:"}),e.jsxs("strong",{children:["₹",(r.amount/100).toFixed(2)]})]}),e.jsxs("div",{className:"preview-row",children:[e.jsx("span",{children:"Fee:"}),e.jsxs("strong",{children:["₹",(r.fee_amount/100).toFixed(2)]})]}),e.jsxs("div",{className:"preview-row",children:[e.jsx("span",{children:"Tax on Fee:"}),e.jsxs("strong",{children:["₹",(r.tax_amount/100).toFixed(2)]})]}),e.jsxs("div",{className:"preview-row",children:[e.jsx("span",{children:"Net Amount:"}),e.jsxs("strong",{children:["₹",(r.net_amount/100).toFixed(2)]})]})]}),e.jsxs("div",{className:"preview-section",children:[e.jsx("h3",{children:"Bank Transfer"}),e.jsxs("div",{className:"preview-row",children:[e.jsx("span",{children:"Status:"}),e.jsx("strong",{children:r.status})]}),e.jsxs("div",{className:"preview-row",children:[e.jsx("span",{children:"Transfer Date:"}),e.jsx("strong",{children:r.processed_at?new Date(r.processed_at).toLocaleDateString():"N/A"})]}),e.jsxs("div",{className:"preview-row",children:[e.jsx("span",{children:"UTR:"}),e.jsx("strong",{className:"mono",children:r.utr||"N/A"})]})]}),e.jsxs("div",{className:"format-selector",children:[e.jsx("label",{children:"Download Format:"}),e.jsx("div",{className:"format-buttons",children:["pdf","csv","excel"].map(t=>e.jsx("button",{className:`btn-format ${m===t?"active":""}`,onClick:()=>F(t),children:t.toUpperCase()},t))})]}),e.jsx("button",{className:"btn-primary",onClick:()=>T(c.id,m),disabled:g===c.id,children:g===c.id?e.jsxs(e.Fragment,{children:[e.jsx(C,{size:16,className:"spin"}),"Downloading..."]}):e.jsxs(e.Fragment,{children:[e.jsx(L,{size:16}),"Download Report"]})})]})]})]}),e.jsx("style",{children:`
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
        .card-title-row {
          display: flex;
          justify-content: space-between;
          align-items: center;
          gap: 12px;
          margin-bottom: 6px;
        }
        .refresh-btn {
          display: inline-flex;
          align-items: center;
          gap: 6px;
          border: 1px solid rgba(102, 172, 255, 0.35);
          color: #bcdfff;
          background: linear-gradient(135deg, rgba(26, 58, 103, 0.85), rgba(22, 41, 79, 0.85));
          border-radius: 10px;
          padding: 8px 12px;
          font-size: 12px;
          font-weight: 700;
          cursor: pointer;
        }
        .refresh-btn:hover {
          border-color: rgba(102, 172, 255, 0.6);
          color: #d7ebff;
        }
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
        .report-preview {
          display: flex;
          flex-direction: column;
          gap: 16px;
        }
        .preview-section {
          border: 1px solid var(--border-color);
          border-radius: 8px;
          padding: 12px;
          background: var(--bg-secondary);
        }
        .preview-section h3 {
          margin: 0 0 12px 0;
          font-size: 14px;
          font-weight: 600;
          color: var(--text-secondary);
        }
        .preview-row {
          display: flex;
          justify-content: space-between;
          padding: 8px 0;
          border-bottom: 1px solid var(--border-color);
          font-size: 14px;
        }
        .preview-row:last-child {
          border-bottom: none;
        }
        .preview-row.highlight {
          background: var(--bg-tertiary);
          padding: 8px;
          border-radius: 4px;
          font-weight: 600;
        }
        .status-pill {
          display: inline-flex;
          border-radius: 999px;
          padding: 3px 8px;
          font-size: 11px;
          font-weight: 700;
          text-transform: uppercase;
          letter-spacing: 0.04em;
        }
        .status-processed,
        .status-completed,
        .status-success {
          background: rgba(36, 201, 132, 0.22);
          color: #89f7c0;
        }
        .status-pending {
          background: rgba(255, 183, 3, 0.2);
          color: #ffe199;
        }
        .status-failed {
          background: rgba(255, 107, 107, 0.2);
          color: #ffb3b3;
        }
        .format-selector {
          margin-top: 12px;
        }
        .format-buttons {
          display: flex;
          gap: 8px;
          margin-top: 8px;
        }
        .btn-format {
          flex: 1;
          padding: 10px 12px;
          border: 1px solid rgba(102, 172, 255, 0.35);
          background: linear-gradient(180deg, rgba(20, 42, 78, 0.9), rgba(14, 31, 60, 0.9));
          border-radius: 10px;
          cursor: pointer;
          font-size: 12px;
          font-weight: 600;
          color: #c2d8ff;
          transition: all 0.2s ease;
        }
        .btn-format.active {
          background: linear-gradient(135deg, #f8a429, #9f5cff);
          color: #fff;
          border-color: transparent;
        }
        .spin {
          animation: spin 1s linear infinite;
        }
        @keyframes spin {
          from { transform: rotate(0deg); }
          to { transform: rotate(360deg); }
        }
      `})]})}export{M as default};
