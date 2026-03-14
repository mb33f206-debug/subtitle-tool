const { listen } = window.__TAURI__.event;

const logSection = document.getElementById('logSection');
const logContent = document.getElementById('logContent');
const toggleLog = document.getElementById('toggleLog');

// Log level styling
const LOG_LEVELS = {
  info: '',
  success: 'success',
  error: 'error',
  warn: 'warn'
};

// Store logs in memory for export
let logEntries = [];

export function addLog(message, level = 'info') {
  logSection.style.display = 'block';
  const line = document.createElement('div');
  line.className = 'log-line';
  if (LOG_LEVELS[level]) {
    line.classList.add(LOG_LEVELS[level]);
  }
  const time = new Date().toLocaleTimeString('zh-TW', { hour12: false });
  const text = '[' + time + '] ' + message;
  line.textContent = text;
  logContent.appendChild(line);
  logContent.scrollTop = logContent.scrollHeight;

  // Store for export
  logEntries.push({ time, level, message });
}

export function clearLogs() {
  logContent.replaceChildren();
  logEntries = [];
}

export function exportLogs() {
  return logEntries.map(e => '[' + e.time + '] [' + e.level.toUpperCase() + '] ' + e.message).join('\n');
}

export function exportLogsJson() {
  return JSON.stringify(logEntries, null, 2);
}

// Listen for backend log events (structured with level)
listen('log', (event) => {
  const payload = event.payload;
  if (typeof payload === 'string') {
    addLog(payload);
  } else {
    addLog(payload.message, payload.level || 'info');
  }
});

// Toggle log visibility
toggleLog.addEventListener('click', () => {
  logContent.classList.toggle('collapsed');
  const svg = toggleLog.querySelector('svg');
  svg.style.transform = logContent.classList.contains('collapsed') ? 'rotate(-90deg)' : '';
});
