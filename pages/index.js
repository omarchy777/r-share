const API_URL = 'http://140.245.17.34:8080/api/relay/status';
const REFRESH_INTERVAL = 3000;
const FETCH_TIMEOUT = 5000;
const MAX_RETRIES = 3;
const RETRY_DELAY = 2000;

let intervalId = null;
let isPageVisible = true;
let retryCount = 0;
let lastSuccessfulFetch = null;

// Notification
function showToast(message, type = 'error', duration = 4000) {
    const container = document.getElementById('toastContainer');
    if (!container) return;

    const toast = document.createElement('div');
    toast.className = `toast ${type}`;

    const messageEl = document.createElement('div');
    messageEl.className = 'toast-message';
    messageEl.textContent = message;
    toast.appendChild(messageEl);

    container.appendChild(toast);

    requestAnimationFrame(() => {
        toast.classList.add('show');
    });

    setTimeout(() => {
        toast.classList.remove('show');
        setTimeout(() => {
            if (toast.parentNode) {
                container.removeChild(toast);
            }
        }, 300);
    }, duration);
}

// Utility functions
function formatUptime(seconds) {
    if (!Number.isFinite(seconds)) return '—';

    const days = Math.floor(seconds / 86400);
    const hours = Math.floor((seconds % 86400) / 3600);
    const minutes = Math.floor((seconds % 3600) / 60);
    const secs = Math.floor(seconds % 60);

    if (days > 0) {
        return `${days}d ${hours}h ${minutes}m`;
    } else if (hours > 0) {
        return `${hours}h ${minutes}m ${secs}s`;
    } else if (minutes > 0) {
        return `${minutes}m ${secs}s`;
    } else {
        return `${secs}s`;
    }
}

function formatBytes(gb) {
    if (!Number.isFinite(gb)) return '—';

    if (gb >= 1) {
        return `${gb.toFixed(2)} GB`;
    } else {
        return `${(gb * 1024).toFixed(2)} MB`;
    }
}

function formatSpeed(mbps) {
    if (!Number.isFinite(mbps)) return '—';
    return `${mbps.toFixed(2)} MB/s`;
}

function formatTimestamp(isoString) {
    if (!isoString) return '—';

    try {
        const date = new Date(isoString);
        if (isNaN(date.getTime())) return '—';
        return date.toLocaleTimeString();
    } catch {
        return '—';
    }
}

function formatMemory(usedMB, maxMB) {
    if (!Number.isFinite(usedMB) || !Number.isFinite(maxMB) || maxMB === 0) return '—';

    const usedGB = (usedMB / 1024).toFixed(2);
    const maxGB = (maxMB / 1024).toFixed(2);
    return `${usedGB} / ${maxGB} GB`;
}

function safeSetText(id, value) {
    const el = document.getElementById(id);
    if (el) el.textContent = value;
}

function safeSetWidth(id, percent) {
    const el = document.getElementById(id);
    if (el && Number.isFinite(percent)) {
        el.style.width = `${Math.min(100, Math.max(0, percent))}%`;
    }
}

function setStatus(state, text) {
    const statusDot = document.getElementById('statusDot');
    const statusText = document.getElementById('statusText');

    if (statusDot) statusDot.className = `status-dot ${state}`;
    if (statusText) statusText.textContent = text;
}

function updateDashboard(data) {
    safeSetText('serverVersion', data.serverVersion || '—');
    safeSetText('uptime', formatUptime(data.uptimeSeconds));
    safeSetText('totalBandwidth', formatBytes(data.totalBandwidthGB));
    safeSetText('activeSessions', data.activeSessions ?? '—');
    safeSetText('pendingSessions', data.pendingSessions ?? '—');
    safeSetText('completedSessions', data.totalSessionsCompleted ?? '—');
    safeSetText('failedSessions', data.totalSessionsFailed ?? '—');
    safeSetText('avgSpeed', formatSpeed(data.averageTransferSpeedMBps));
    safeSetText('peakBandwidth', formatSpeed(data.peakBandwidthMBps));
    safeSetText('currentTransfers', data.currentTransferCount ?? '—');
    safeSetText('threadCount', data.threadCount ?? '—');
    safeSetText('timestamp', formatTimestamp(data.timestamp));

    const memoryPercent = (data.memoryUsedMB / data.memoryMaxMB) * 100;
    safeSetText('memoryUsage', formatMemory(data.memoryUsedMB, data.memoryMaxMB));
    safeSetWidth('memoryBar', memoryPercent);

    const cpuPercent = data.cpuUsagePercent;
    safeSetText('cpuUsage', Number.isFinite(cpuPercent) ? `${cpuPercent.toFixed(1)}%` : '—');
    safeSetWidth('cpuBar', cpuPercent);

    setStatus('online', 'Online');
    retryCount = 0;
    lastSuccessfulFetch = Date.now();
}

async function fetchStatus() {
    const controller = new AbortController();
    const timeoutId = setTimeout(() => controller.abort(), FETCH_TIMEOUT);

    try {
        const response = await fetch(API_URL, {
            signal: controller.signal,
            cache: 'no-store'
        });

        clearTimeout(timeoutId);

        if (!response.ok) {
            const error = new Error(`HTTP ${response.status}: ${response.statusText}`);
            console.error('Fetch error:', error.message);
            handleFetchError(error.message);
            return;
        }

        const contentType = response.headers.get('content-type') || '';
        if (!contentType.includes('application/json')) {
            const error = new Error('Server returned non-JSON response');
            console.error('Fetch error:', error.message);
            handleFetchError(error.message);
            return;
        }

        let data;
        try {
            data = await response.json();
        } catch (parseError) {
            const error = new Error('Invalid JSON response from server');
            console.error('Fetch error:', error.message);
            handleFetchError(error.message);
            return;
        }

        updateDashboard(data);

    } catch (error) {
        clearTimeout(timeoutId);

        const errorMessage = error.name === 'AbortError'
            ? 'Request timeout'
            : error.message || 'Connection failed';

        console.error('Fetch error:', errorMessage);
        handleFetchError(errorMessage);
    }
}

function handleFetchError(errorMessage) {
    if (retryCount < MAX_RETRIES) {
        retryCount++;
        setStatus('reconnecting', `Reconnecting (${retryCount}/${MAX_RETRIES})...`);

        if (retryCount === 1) {
            showToast(`Connection lost: ${errorMessage}`, 'warning');
        }

        setTimeout(() => {
            void fetchStatus();
        }, RETRY_DELAY);
    } else {
        setStatus('offline', 'Offline');
        showToast(`Connection failed after ${MAX_RETRIES} attempts`, 'error');
    }
}

function startPolling() {
    stopPolling();
    void fetchStatus();
    intervalId = setInterval(() => {
        if (isPageVisible) {
            void fetchStatus();
        }
    }, REFRESH_INTERVAL);
}

function stopPolling() {
    if (intervalId !== null) {
        clearInterval(intervalId);
        intervalId = null;
    }
}

document.addEventListener('visibilitychange', () => {
    const wasHidden = !isPageVisible;
    isPageVisible = !document.hidden;

    if (isPageVisible && wasHidden) {
        const timeSinceLastFetch = lastSuccessfulFetch
            ? Date.now() - lastSuccessfulFetch
            : Infinity;

        if (timeSinceLastFetch > REFRESH_INTERVAL) {
            void fetchStatus();
        }
    }
});

window.addEventListener('beforeunload', () => {
    stopPolling();
});

window.addEventListener('online', () => {
    showToast('Network connection restored', 'success', 3000);
    retryCount = 0;
    void fetchStatus();
});

window.addEventListener('offline', () => {
    setStatus('offline', 'No Internet');
    showToast('Network connection lost', 'error');
});

startPolling();
