const API_URL = import.meta.env.VITE_API_URL || '/api';
document.getElementById('api-url').textContent = API_URL;
const $ = id => document.getElementById(id);
const esc = value => String(value ?? '').replace(/[&<>"']/g, c => ({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;'}[c]));
async function api(path, options) { const res=await fetch(API_URL+path, options); const data=await res.json(); if(!res.ok) throw new Error(data.error || 'Request failed'); return data; }
function setOnline(online){ $('network-status').textContent=online?'Network online':'API offline'; document.querySelector('.network-pill').classList.toggle('offline',!online); }
async function loadStatus(){
  try { const d=await api('/status'); $('block-height').textContent=d.latest_height.toLocaleString(); $('validator-count').textContent=d.validators; $('peer-count').textContent=d.peer_count; $('block-time').textContent=d.block_time_ms+'ms'; $('chain-id').textContent=d.chain_id; $('last-refresh').textContent='Updated '+new Date().toLocaleTimeString(); setOnline(true); }
  catch(e){ setOnline(false); $('last-refresh').textContent=e.message; }
}
async function loadBlock(){
  try { const b=await api('/block/latest'); $('latest-height').textContent=b.height ?? b.index ?? '--'; $('latest-hash').textContent=b.hash || '--'; $('latest-proposer').textContent=b.proposer || '--'; $('latest-tx-count').textContent=b.tx_count ?? b.transactions?.length ?? 0; $('latest-gas-used').textContent=(b.gas_used ?? 0).toLocaleString(); }
  catch(e){ $('latest-hash').textContent=e.message; }
}
async function loadValidators(){
  try { const d=await api('/validators'); const list=Array.isArray(d)?d:(d.validators||[]); $('validator-list').innerHTML=list.map((v,i)=>'<div class="table-row"><span><b>#'+(i+1)+' '+esc(v.id)+'</b><small>'+esc(v.public_key||'').slice(0,22)+'…</small></span><span>'+Number(v.stake||0).toLocaleString()+'</span><span>'+((v.commission||0)/100)+'%</span><span><em class="'+(v.active?'ok':'bad')+'">'+(v.active?'Active':'Inactive')+'</em></span></div>').join('') || '<div class="empty">No validators returned.</div>'; }
  catch(e){ $('validator-list').innerHTML='<div class="empty">'+esc(e.message)+'</div>'; }
}
async function lookup(value){
  const q=value.trim(); if(!q) return;
  const out=$('lookup-result'); out.textContent='Searching…';
  try { let data; if(/^\d+$/.test(q)) data=await api('/block/'+q); else if(q.startsWith('0x')) data=await api('/account/'+encodeURIComponent(q)); else data=await api('/tx/'+encodeURIComponent(q)); out.textContent=JSON.stringify(data,null,2); }
  catch(e){ out.textContent=e.message; }
}
$('lookup-form').addEventListener('submit',e=>{e.preventDefault();lookup($('lookup-input').value);});
$('refresh').addEventListener('click',()=>Promise.all([loadStatus(),loadBlock(),loadValidators()]));
loadStatus(); loadBlock(); loadValidators(); setInterval(()=>{loadStatus();loadBlock();},10000);