import { getConfig, authHeaders } from '../lib/config.js';

const CONFIG_KEY = 'serverConfig';

const urlInput = document.getElementById('server-url');
const tokenInput = document.getElementById('api-token');
const saveBtn = document.getElementById('save');
const testBtn = document.getElementById('test');
const messageEl = document.getElementById('message');

// Load saved config
async function loadConfig() {
  const config = await getConfig();
  urlInput.value = config.serverUrl;
  tokenInput.value = config.apiToken;
}

// Show status message
function showMessage(text, type) {
  messageEl.textContent = text;
  messageEl.className = `message ${type}`;
  messageEl.style.display = 'block';
  setTimeout(() => { messageEl.style.display = 'none'; }, 3000);
}

// Save config
saveBtn.addEventListener('click', async () => {
  const serverUrl = urlInput.value.trim().replace(/\/+$/, '');
  const apiToken = tokenInput.value.trim();

  if (!serverUrl) {
    showMessage('Server URL is required', 'error');
    return;
  }

  await chrome.storage.local.set({
    [CONFIG_KEY]: { serverUrl, apiToken }
  });

  showMessage('Settings saved', 'success');
});

// Test connection
testBtn.addEventListener('click', async () => {
  const serverUrl = urlInput.value.trim().replace(/\/+$/, '');
  const apiToken = tokenInput.value.trim();

  if (!serverUrl) {
    showMessage('Server URL is required', 'error');
    return;
  }

  testBtn.disabled = true;
  testBtn.textContent = 'Testing...';

  try {
    const response = await fetch(`${serverUrl}/health`, {
      headers: authHeaders(apiToken)
    });

    if (response.ok) {
      showMessage('Connection successful', 'success');
    } else {
      showMessage(`Connection failed: HTTP ${response.status}`, 'error');
    }
  } catch (error) {
    showMessage(`Connection failed: ${error.message}`, 'error');
  } finally {
    testBtn.disabled = false;
    testBtn.textContent = 'Test Connection';
  }
});

loadConfig();
