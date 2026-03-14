import { addLog, clearLogs, exportLogs, exportLogsJson } from './modules/log.js';
import { getSelectedFile, setProcessing, onFileChange } from './modules/file.js';
import { initSettings, getOutputDir, setOutputDir, getLogSettings, displayDir } from './modules/settings.js';
import { showProgress, showError, onComplete } from './modules/progress.js';

const { invoke } = window.__TAURI__.core;
const dialog = window.__TAURI__?.dialog || {};
const { open, save } = dialog;

// --- DOM ---
const btnStart = document.getElementById('btnStart');
const btnStartText = btnStart.querySelector('.text_button');
const useGemini = document.getElementById('useGemini');
const geminiLabel = document.getElementById('geminiLabel');
const languageSelect = document.getElementById('language');
const outputFormatSelect = document.getElementById('outputFormat');
const translateToSelect = document.getElementById('translateTo');
const btnExportLog = document.getElementById('btnExportLog');
const btnSaveLog = document.getElementById('btnSaveLog');
const outputDirDisplay = document.getElementById('outputDirDisplay');
const pickOutputDir = document.getElementById('pickOutputDir');
const clearOutputDir = document.getElementById('clearOutputDir');

let isProcessing = false;

// --- Init settings & output dir ---
await initSettings();
updateOutputDirDisplay();

function updateOutputDirDisplay() {
  displayDir(outputDirDisplay, getOutputDir(), '與影片同目錄');
}

// --- Output dir helpers ---
async function persistOutputDir(dir) {
  setOutputDir(dir);
  updateOutputDirDisplay();
  try {
    const settings = await invoke('get_settings');
    if (!settings.output) settings.output = {};
    settings.output.default_dir = dir;
    await invoke('save_settings', { settings });
  } catch (e) {
    console.error('Failed to save output dir:', e);
  }
}

// --- Output dir picker (main page) ---
pickOutputDir.addEventListener('click', async () => {
  if (isProcessing) return;
  const dir = await open({ directory: true });
  if (dir) await persistOutputDir(dir);
});

clearOutputDir.addEventListener('click', async () => {
  if (isProcessing) return;
  await persistOutputDir('');
});

// --- File change callback ---
onFileChange((path) => {
  btnStart.disabled = !path;
});

// --- Gemini toggle ---
useGemini.addEventListener('change', () => {
  geminiLabel.textContent = useGemini.checked ? '已啟用' : '已停用';
});

// --- Reset state ---
function resetProcessingState() {
  isProcessing = false;
  setProcessing(false);
  btnStart.disabled = false;
  btnStart.classList.remove('processing');
  if (btnStartText) btnStartText.textContent = '開始提取字幕';
}

// --- Progress complete callback ---
onComplete(resetProcessingState);

// --- Export log (clipboard) ---
if (btnExportLog) {
  btnExportLog.addEventListener('click', async () => {
    const text = exportLogs();
    try {
      await navigator.clipboard.writeText(text);
      addLog('日誌已複製到剪貼簿', 'success');
    } catch (e) {
      addLog('複製失敗：' + e, 'error');
    }
  });
}

// --- Log format helper ---
function getLogContent(format) {
  return format === 'json' ? exportLogsJson() : exportLogs();
}

// --- Save log to file ---
if (btnSaveLog) {
  btnSaveLog.addEventListener('click', async () => {
    const logSettings = getLogSettings();
    const ext = logSettings.export_format === 'json' ? 'json' : 'txt';
    const filePath = await save({
      filters: [{ name: '日誌檔案', extensions: [ext] }],
      defaultPath: 'subtitle_log.' + ext,
    });
    if (!filePath) return;
    const content = getLogContent(ext);
    try {
      await invoke('save_log_file', { path: filePath, content });
      addLog('日誌已儲存: ' + filePath, 'success');
    } catch (e) {
      addLog('儲存失敗：' + e, 'error');
    }
  });
}

// --- Auto-export logs ---
async function autoExportLogs(videoPath) {
  const logSettings = getLogSettings();
  if (!logSettings.auto_export) return;

  try {
    const outputDir = logSettings.export_dir || getOutputDir();
    // Determine base directory: log dir > output dir > video's directory
    const videoDir = videoPath.substring(0, Math.max(videoPath.lastIndexOf('/'), videoPath.lastIndexOf('\\')));
    const dir = outputDir || videoDir;

    const videoName = videoPath.split(/[/\\]/).pop().replace(/\.[^.]+$/, '');
    const timestamp = new Date().toISOString().replace(/[:.]/g, '-').slice(0, 19);
    const ext = logSettings.export_format === 'json' ? 'json' : 'txt';
    const content = getLogContent(ext);
    const sep = videoPath.includes('\\') ? '\\' : '/';
    const logPath = dir + sep + videoName + '_log_' + timestamp + '.' + ext;

    await invoke('save_log_file', { path: logPath, content });
    addLog('日誌已自動匯出: ' + logPath, 'success');
  } catch (e) {
    addLog('日誌自動匯出失敗: ' + e, 'error');
  }
}

// --- Start processing ---
btnStart.addEventListener('click', async () => {
  const filePath = getSelectedFile();
  if (!filePath || isProcessing) return;

  isProcessing = true;
  setProcessing(true);
  btnStart.disabled = true;
  btnStart.classList.add('processing');
  if (btnStartText) btnStartText.textContent = '處理中...';

  showProgress();
  clearLogs();

  try {
    const result = await invoke('process_video', {
      req: {
        video_path: filePath,
        language: languageSelect.value,
        use_gemini: useGemini.checked,
        translate_to: translateToSelect.value,
        output_format: outputFormatSelect.value,
        output_dir: getOutputDir(),
      }
    });
    addLog('輸出檔案：', 'success');
    result.split('\n').forEach(f => { if (f.trim()) addLog('  ' + f, 'success'); });

    // Auto-export logs if enabled
    await autoExportLogs(filePath);
  } catch (error) {
    addLog('錯誤：' + error, 'error');
    resetProcessingState();
    showError();
  }
});
