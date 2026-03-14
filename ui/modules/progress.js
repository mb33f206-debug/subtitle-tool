const { listen } = window.__TAURI__.event;

const progressSection = document.getElementById('progressSection');
const progressBar = document.getElementById('progressBar');
const progressText = document.getElementById('progressText');
const pacmanLoader = document.getElementById('pacmanLoader');

let _onComplete = null;
let _errorTimer = null;

export function onComplete(callback) {
  _onComplete = callback;
}

export function showProgress() {
  if (_errorTimer) { clearTimeout(_errorTimer); _errorTimer = null; }
  progressSection.style.display = 'block';
  progressBar.style.width = '0%';
  progressBar.style.background = '';
  progressText.textContent = '準備中...';
  pacmanLoader.classList.remove('done');
}

export function showError() {
  if (_errorTimer) { clearTimeout(_errorTimer); _errorTimer = null; }
  pacmanLoader.classList.add('done');
  progressBar.style.width = '100%';
  progressBar.style.background = 'var(--danger)';
  progressText.textContent = '處理失敗';
  _errorTimer = setTimeout(() => { progressBar.style.background = ''; _errorTimer = null; }, 3000);
}

listen('progress', (event) => {
  const { stage, percent, message } = event.payload;
  progressBar.style.width = percent + '%';
  progressText.textContent = message;
  if (percent >= 100) {
    pacmanLoader.classList.add('done');
    if (_onComplete) setTimeout(_onComplete, 500);
  }
});
