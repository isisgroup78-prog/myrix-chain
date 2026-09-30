const API_URL = import.meta.env.VITE_API_URL || 'http://localhost:3000';

async function loadStatus() {
  try {
    const res = await fetch(`${API_URL}/status`);
    const data = await res.json();
    document.getElementById('block-height').textContent = data.latest_height.toLocaleString();
    document.getElementById('validator-count').textContent = data.validators;
    document.getElementById('peer-count').textContent = data.peer_count;
    document.getElementById('block-time').textContent = `${data.block_time_ms}ms`;
  } catch (error) {
    console.error('Failed to fetch status', error);
  }
}

async function loadLatestBlock() {
  try {
    const res = await fetch(`${API_URL}/block/latest`);
    const block = await res.json();
    document.getElementById('latest-height').textContent = block.height.toLocaleString();
    document.getElementById('latest-hash').textContent = block.hash.slice(0, 24) + '...';
    document.getElementById('latest-proposer').textContent = block.proposer;
    document.getElementById('latest-tx-count').textContent = block.tx_count;
    document.getElementById('latest-gas-used').textContent = `${(block.gas_used / 1_000_000).toFixed(2)}M`;
  } catch (error) {
    console.error('Failed to fetch latest block', error);
  }
}

async function loadValidators() {
  try {
    const res = await fetch(`${API_URL}/validators`);
    const validators = await res.json();
    const html = validators.map((v, i) => `
      <div class="validator-item">
        <span class="rank">#${i + 1}</span>
        <span class="validator-id">${v.id}</span>
        <span class="stake">${(v.stake / 1_000_000).toFixed(0)}M</span>
        <span class="commission">${v.commission / 100}%</span>
        <span class="status ${v.active ? 'active' : 'inactive'}">${v.active ? 'Active' : 'Inactive'}</span>
      </div>
    `).join('');
    document.getElementById('validator-list').innerHTML = html;
  } catch (error) {
    console.error('Failed to fetch validators', error);
  }
}

loadStatus();
loadLatestBlock();
loadValidators();
setInterval(() => {
  loadStatus();
  loadLatestBlock();
}, 12000);
