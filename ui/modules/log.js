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

// Store logs in memory for export (capped to prevent memory leaks)
const MAX_LOG_ENTRIES = 5000;
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

  // Cap DOM nodes to match memory cap — prevent unbounded DOM growth
  while (logContent.children.length > MAX_LOG_ENTRIES) {
    logContent.removeChild(logContent.firstChild);
  }

  logContent.scrollTop = logContent.scrollHeight;

  logEntries.push({ time, level, message });
  if (logEntries.length > MAX_LOG_ENTRIES) {
    logEntries = logEntries.slice(-MAX_LOG_ENTRIES);
  }
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
