import { addLog } from './log.js';

const { open } = window.__TAURI__.dialog;
const { listen } = window.__TAURI__.event;

const dropZone = document.getElementById('dropZone');
const fileInfo = document.getElementById('fileInfo');
const fileName = document.getElementById('fileName');
const clearFile = document.getElementById('clearFile');

const VIDEO_EXTS = ['mp4', 'mov', 'avi', 'mkv', 'webm', 'flv'];

let selectedFilePath = null;
let isProcessing = false;
let _onFileChange = null; // callback

export function onFileChange(callback) {
  _onFileChange = callback;
}

export function getSelectedFile() {
  return selectedFilePath;
}

export function setProcessing(val) {
  isProcessing = val;
}

function setFile(path) {
  selectedFilePath = path;
  const name = path.split('/').pop().split('\\').pop();
  fileName.textContent = name;
  fileInfo.style.display = 'block';
  dropZone.style.display = 'none';
  if (_onFileChange) _onFileChange(path);
}

function clearSelectedFile() {
  if (isProcessing) return;
  selectedFilePath = null;
  fileInfo.style.display = 'none';
  dropZone.style.display = 'block';
  if (_onFileChange) _onFileChange(null);
}

// Click to open file dialog
dropZone.addEventListener('click', async () => {
  if (isProcessing) return;
  try {
    const selected = await open({
      multiple: false,
      filters: [{ name: '影片檔案', extensions: VIDEO_EXTS }]
    });
    if (selected) setFile(selected);
  } catch (e) {
    console.error('File dialog error:', e);
  }
});

// Drag & Drop visual feedback
dropZone.addEventListener('dragover', (e) => { e.preventDefault(); e.stopPropagation(); dropZone.classList.add('drag-over'); });
dropZone.addEventListener('dragleave', (e) => { e.preventDefault(); e.stopPropagation(); dropZone.classList.remove('drag-over'); });
dropZone.addEventListener('drop', (e) => { e.preventDefault(); e.stopPropagation(); dropZone.classList.remove('drag-over'); });

// Tauri native file drop
listen('tauri://drag-drop', (event) => {
  if (isProcessing) return;
  const paths = event.payload.paths;
  if (paths && paths.length > 0) {
    const path = paths[0];
    const ext = path.split('.').pop().toLowerCase();
    if (VIDEO_EXTS.includes(ext)) {
      setFile(path);
    } else {
      addLog('不支援的檔案格式：' + path, 'error');
    }
  }
});

clearFile.addEventListener('click', clearSelectedFile);
