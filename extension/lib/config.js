const CONFIG_KEY = 'serverConfig';
const DEFAULT_URL = 'http://localhost:44380';

export async function getConfig() {
  const result = await chrome.storage.local.get(CONFIG_KEY);
  return result[CONFIG_KEY] || { serverUrl: DEFAULT_URL, apiToken: '' };
}

export function authHeaders(apiToken) {
  const headers = { 'Content-Type': 'application/json' };
  if (apiToken) headers['Authorization'] = `Bearer ${apiToken}`;
  return headers;
}
