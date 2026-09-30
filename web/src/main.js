const API_URL = import.meta.env.VITE_API_URL || 'http://localhost:3000';

async function loadStatus() {
  const res = await fetch(`${API_URL}/status`);
  const data = await res.json();
  document.getElementById('latest-height').textContent = data.latest_height;
  document.getElementById('validator-count').textContent = data.validators;
  document.getElementById('peer-count').textContent = data.peer_count;
  document.getElementById('chain-id').textContent = data.chain_id;
}

async function loadLatestBlock() {
  const res = await fetch(`${API_URL}/block/latest`);
  const data = await res.json();
  document.getElementById('block-data').textContent = JSON.stringify(data, null, 2);
}

loadStatus();
loadLatestBlock();
