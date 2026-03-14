const { listen } = window.__TAURI__.event;

const progressSection = document.getElementById('progressSection');
const progressBar = document.getElementById('progressBar');
const progressText = document.getElementById('progressText');
const pacmanLoader = document.getElementById('pacmanLoader');

let _onComplete = null;

export function onComplete(callback) {
  _onComplete = callback;
}

export function showProgress() {
  progressSection.style.display = 'block';
  progressBar.style.width = '0%';
  progressBar.style.background = '';
  progressText.textContent = '準備中...';
  pacmanLoader.classList.remove('done');
}

export function showError() {
  pacmanLoader.classList.add('done');
  progressBar.style.width = '100%';
  progressBar.style.background = 'var(--danger)';
  progressText.textContent = '處理失敗';
  setTimeout(() => { progressBar.style.background = ''; }, 3000);
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
