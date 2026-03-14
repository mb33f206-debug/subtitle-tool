const { invoke } = window.__TAURI__.core;
const dialog = window.__TAURI__?.dialog || {};
const { open } = dialog;
import { addLog } from './log.js';

// --- DOM Elements ---
const btnSettings = document.getElementById('btnSettings');
const settingsModal = document.getElementById('settingsModal');
const closeSettings = document.getElementById('closeSettings');
const cancelSettings = document.getElementById('cancelSettings');
const saveSettingsBtn = document.getElementById('saveSettings');

// Appearance
const fontSizeValue = document.getElementById('fontSizeValue');

// API
const apiMode = document.getElementById('apiMode');
const apiBaseUrl = document.getElementById('apiBaseUrl');
const apiKey = document.getElementById('apiKey');
const apiModel = document.getElementById('apiModel');
const detectModelsBtn = document.getElementById('detectModels');
const apiHint = document.getElementById('apiHint');

// Output (settings modal)
const settingsOutputDir = document.getElementById('settingsOutputDir');
const pickSettingsOutputDir = document.getElementById('pickSettingsOutputDir');
const clearSettingsOutputDir = document.getElementById('clearSettingsOutputDir');

// Log (settings modal)
const logAutoExportCheckbox = document.getElementById('logAutoExport');
const logAutoExportLabel = document.getElementById('logAutoExportLabel');
const logExportFormat = document.getElementById('logExportFormat');
const settingsLogDir = document.getElementById('settingsLogDir');
const pickSettingsLogDir = document.getElementById('pickSettingsLogDir');
const clearSettingsLogDir = document.getElementById('clearSettingsLogDir');

// --- Module state ---
let currentFontSize = 14;
let currentTheme = 'dark';
let currentOutputDir = '';
let currentLogAutoExport = false;
let currentLogFormat = 'txt';
let currentLogDir = '';

// --- Exported getters ---
export function getOutputDir() { return currentOutputDir; }
export function setOutputDir(dir) { currentOutputDir = dir; }
export function getLogSettings() {
  return {
    auto_export: currentLogAutoExport,
    export_format: currentLogFormat,
    export_dir: currentLogDir,
  };
}

// --- Theme & Font (internal) ---
function applyTheme(theme) {
  if (theme === 'light') {
    document.documentElement.setAttribute('data-theme', 'light');
  } else {
    document.documentElement.removeAttribute('data-theme');
  }
}

function applyFontSize(size) {
  document.documentElement.style.fontSize = size + 'px';
}

function applyUiSettings(ui) {
  currentTheme = ui.theme || 'dark';
  currentFontSize = ui.font_size || 14;
  applyTheme(currentTheme);
  applyFontSize(currentFontSize);
}

// --- Dir display helpers ---
export function displayDir(el, dir, fallback) {
  el.textContent = dir || fallback;
  if (dir) {
    el.style.color = '';
  } else {
    el.style.color = 'var(--text-muted)';
  }
}

// --- Tab switching ---
const allTabs = document.querySelectorAll('.tab');
const allTabContents = document.querySelectorAll('.tab-content');
allTabs.forEach(tab => {
  tab.addEventListener('click', () => {
    allTabs.forEach(t => t.classList.remove('active'));
    allTabContents.forEach(c => c.classList.remove('active'));
    tab.classList.add('active');
    document.getElementById('tab-' + tab.dataset.tab).classList.add('active');
  });
});

// --- Theme buttons ---
const allThemeBtns = document.querySelectorAll('.theme-btn');
allThemeBtns.forEach(btn => {
  btn.addEventListener('click', () => {
    allThemeBtns.forEach(b => b.classList.remove('active'));
    btn.classList.add('active');
    currentTheme = btn.dataset.theme;
    applyTheme(currentTheme);
  });
});

// --- Font size ---
document.getElementById('fontDecrease').addEventListener('click', () => {
  if (currentFontSize > 10) { currentFontSize--; fontSizeValue.textContent = currentFontSize; applyFontSize(currentFontSize); }
});
document.getElementById('fontIncrease').addEventListener('click', () => {
  if (currentFontSize < 32) { currentFontSize++; fontSizeValue.textContent = currentFontSize; applyFontSize(currentFontSize); }
});

// --- API mode ---
apiMode.addEventListener('change', () => {
  if (apiMode.value === 'official') {
    apiBaseUrl.value = 'https://generativelanguage.googleapis.com/v1beta/openai/';
    apiBaseUrl.disabled = true;
    apiHint.textContent = 'Official 模式：直接使用 Google Gemini API。API Key 請到 Google AI Studio 取得。';
  } else {
    apiBaseUrl.disabled = false;
    apiHint.textContent = 'Proxy 模式：使用 OpenAI 相容的代理伺服器。需要填入 Base URL、API Key 和模型名稱。';
  }
});

// --- Detect models ---
detectModelsBtn.addEventListener('click', async () => {
  const url = apiBaseUrl.value.trim();
  const key = apiKey.value.trim();
  if (!url || !key) { apiHint.textContent = '請先填入 Base URL 和 API Key'; return; }
  detectModelsBtn.disabled = true;
  detectModelsBtn.textContent = '偵測中...';
  try {
    const models = await invoke('fetch_models', { baseUrl: url, apiKey: key });
    const currentVal = apiModel.value;
    apiModel.replaceChildren();
    models.forEach(m => { const opt = document.createElement('option'); opt.value = m; opt.textContent = m; apiModel.appendChild(opt); });
    if (currentVal && models.includes(currentVal)) apiModel.value = currentVal;
    apiHint.textContent = '偵測到 ' + models.length + ' 個模型';
  } catch (e) {
    apiHint.textContent = '偵測失敗：' + e;
  }
  detectModelsBtn.disabled = false;
  detectModelsBtn.textContent = '偵測';
});

// --- Model value helper ---
function setModelValue(model) {
  if (!model) return;
  if (!Array.from(apiModel.options).some(o => o.value === model)) {
    const opt = document.createElement('option'); opt.value = model; opt.textContent = model; apiModel.appendChild(opt);
  }
  apiModel.value = model;
}

// --- Output dir pickers (settings modal) ---
pickSettingsOutputDir.addEventListener('click', async () => {
  const dir = await open({ directory: true });
  if (dir) {
    currentOutputDir = dir;
    displayDir(settingsOutputDir, dir, '未設定（與影片同目錄）');
  }
});
clearSettingsOutputDir.addEventListener('click', () => {
  currentOutputDir = '';
  displayDir(settingsOutputDir, '', '未設定（與影片同目錄）');
});

// --- Log settings (settings modal) ---
logAutoExportCheckbox.addEventListener('change', () => {
  currentLogAutoExport = logAutoExportCheckbox.checked;
  logAutoExportLabel.textContent = currentLogAutoExport ? '已啟用' : '已停用';
});

pickSettingsLogDir.addEventListener('click', async () => {
  const dir = await open({ directory: true });
  if (dir) {
    currentLogDir = dir;
    displayDir(settingsLogDir, dir, '未設定（與輸出目錄相同）');
  }
});
clearSettingsLogDir.addEventListener('click', () => {
  currentLogDir = '';
  displayDir(settingsLogDir, '', '未設定（與輸出目錄相同）');
});

// --- Apply all settings to module state ---
function applyAllSettings(settings) {
  if (settings.ui) applyUiSettings(settings.ui);
  currentOutputDir = settings.output?.default_dir || '';
  currentLogAutoExport = settings.log?.auto_export || false;
  currentLogFormat = settings.log?.export_format || 'txt';
  currentLogDir = settings.log?.export_dir || '';
}

// --- Open settings modal ---
btnSettings.addEventListener('click', async () => {
  try {
    const settings = await invoke('get_settings');
    // Gemini
    if (settings.gemini) {
      apiMode.value = settings.gemini.mode || 'proxy';
      apiBaseUrl.value = settings.gemini.base_url || '';
      apiKey.value = settings.gemini.api_key || '';
      setModelValue(settings.gemini.model || '');
    }
    applyAllSettings(settings);
    fontSizeValue.textContent = currentFontSize;
    allThemeBtns.forEach(b => b.classList.toggle('active', b.dataset.theme === currentTheme));

    displayDir(settingsOutputDir, currentOutputDir, '未設定（與影片同目錄）');
    logAutoExportCheckbox.checked = currentLogAutoExport;
    logAutoExportLabel.textContent = currentLogAutoExport ? '已啟用' : '已停用';
    logExportFormat.value = currentLogFormat;
    displayDir(settingsLogDir, currentLogDir, '未設定（與輸出目錄相同）');

    apiBaseUrl.disabled = (apiMode.value === 'official');
    apiMode.dispatchEvent(new Event('change'));
  } catch (e) { console.error('Load settings error:', e); }
  settingsModal.classList.add('active');
});

// --- Close/Save ---
function closeModal() { settingsModal.classList.remove('active'); }
closeSettings.addEventListener('click', closeModal);
cancelSettings.addEventListener('click', closeModal);
settingsModal.addEventListener('click', (e) => { if (e.target === settingsModal) closeModal(); });

saveSettingsBtn.addEventListener('click', async () => {
  currentLogFormat = logExportFormat.value;
  const settings = {
    gemini: { mode: apiMode.value, base_url: apiBaseUrl.value, api_key: apiKey.value, model: apiModel.value },
    ui: { font_size: currentFontSize, theme: currentTheme },
    output: { default_dir: currentOutputDir },
    log: { auto_export: currentLogAutoExport, export_format: currentLogFormat, export_dir: currentLogDir },
  };
  try {
    await invoke('save_settings', { settings });
    addLog('設定已儲存', 'success');
    closeModal();
  } catch (e) {
    addLog('設定儲存失敗：' + e, 'error');
  }
});

// --- Load on startup ---
export async function initSettings() {
  try {
    const settings = await invoke('get_settings');
    applyAllSettings(settings);
    return settings;
  } catch (e) {
    console.log('Settings not found, using defaults');
    return null;
  }
}
