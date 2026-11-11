/*
 * SPDX-License-Identifier: GPL-3.0-only
 * Copyright (c) 2026 Robert Longbottom (@thomaslr)
 * Commercial licensing available — see LICENSE for details.
 */

// TimeStation MultiBand6 Atomic Beacon Frontend Logic

const STATIONS = [
  { name: "BPC (China 68.5 kHz)", freq: 68500, tzOffset: 28800 },
  { name: "WWVB (USA 60.0 kHz)", freq: 60000, tzOffset: 0 },
  { name: "MSF (UK 60.0 kHz)", freq: 60000, tzOffset: 0 },
  { name: "DCF77 (Germany 77.5 kHz)", freq: 77500, tzOffset: 3600 },
  { name: "JJY40 (Japan 40.0 kHz)", freq: 40000, tzOffset: 32400 },
  { name: "JJY60 (Japan 60.0 kHz)", freq: 60000, tzOffset: 32400 }
];

let isTransmitting = false;
let carouselMode = false;
let userOffsetHours = 0;
let activeStationIdx = 0;
let serverEpochSec = 0;
let clientEpochBaseMs = Date.now();
let transmitRemainSec = 0;
let transmitRemainBaseMs = Date.now();
let appConfig = null;
let appStatus = null;
let currentFirmwareVersion = '0.1.12';
let latestReleaseData = null;
let logAutoRefreshInterval = null;
let statusPollTimer = null;
let dashboardInitialized = false;

// Fetch helper with AbortController timeout so a dropped/stalled socket never hangs polling
async function fetchWithTimeout(url, options = {}, timeoutMs = 6000) {
  const controller = new AbortController();
  const timer = setTimeout(() => controller.abort(), timeoutMs);
  try {
    return await fetch(url, { ...options, signal: controller.signal });
  } finally {
    clearTimeout(timer);
  }
}

function formatDuration(totalSec) {
  const sec = Math.max(0, Math.floor(Number(totalSec) || 0));
  const days = Math.floor(sec / 86400);
  const hours = Math.floor((sec % 86400) / 3600);
  const mins = Math.floor((sec % 3600) / 60);
  const secs = sec % 60;
  if (days > 0) return `${days}d ${hours}h ${mins}m`;
  if (hours > 0) return `${hours}h ${mins}m ${secs}s`;
  if (mins > 0) return `${mins}m ${secs}s`;
  return `${secs}s`;
}

// --- Tab Management ---
function switchTab(tabName) {
  const tabs = ['broadcast', 'network', 'ota', 'logs'];
  tabs.forEach(t => {
    const btn = document.getElementById('tabBtn' + capitalize(t));
    const content = document.getElementById('tabContent' + capitalize(t));
    if (btn) btn.classList.toggle('active', t === tabName);
    if (content) content.style.display = (t === tabName) ? 'block' : 'none';
  });
  if (tabName === 'network' || tabName === 'broadcast') fetchConfig();
  if (tabName === 'ota') fetchOtaStatus();
  if (tabName === 'logs') {
    fetchLogs();
    if (!logAutoRefreshInterval) {
      logAutoRefreshInterval = setInterval(fetchLogs, 3000);
    }
  } else {
    if (logAutoRefreshInterval) {
      clearInterval(logAutoRefreshInterval);
      logAutoRefreshInterval = null;
    }
  }
}

async function fetchConfig() {
  try {
    const res = await fetchWithTimeout('/api/config', {}, 6000);
    if (!res.ok) throw new Error('HTTP ' + res.status);
    const cfg = await res.json();
    appConfig = cfg;
    loadConfigToForm(cfg);
  } catch (err) {
    console.warn('Config fetch error:', err);
  }
}

function loadConfigToForm(cfg) {
  if (!cfg) return;

  // Broadcast parameters
  if (cfg.station !== undefined) {
    const stEl = document.getElementById('station');
    let stIdx = 0;
    if (typeof cfg.station === 'number') {
      stIdx = cfg.station;
    } else if (typeof cfg.station === 'string') {
      const map = { bpc: 0, wwvb: 1, msf: 2, dcf77: 3, jjy40: 4, jjy60: 5 };
      stIdx = map[cfg.station.toLowerCase()] !== undefined ? map[cfg.station.toLowerCase()] : 0;
    }
    if (stEl) stEl.value = String(stIdx);
    activeStationIdx = stIdx;
  }
  if (cfg.offset_hours !== undefined) {
    const offEl = document.getElementById('offsetHours');
    if (offEl) offEl.value = String(cfg.offset_hours);
    userOffsetHours = cfg.offset_hours;
  }
  if (cfg.schedule_enabled !== undefined) {
    const schEl = document.getElementById('scheduleEnabled');
    if (schEl) schEl.checked = cfg.schedule_enabled;
  }
  if (cfg.broadcast_hour !== undefined && cfg.broadcast_minute !== undefined) {
    const btEl = document.getElementById('broadcastTime');
    if (btEl) {
      btEl.value = `${String(cfg.broadcast_hour).padStart(2, '0')}:${String(cfg.broadcast_minute).padStart(2, '0')}`;
    }
  }
  if (cfg.broadcast_duration_min !== undefined) {
    const bdEl = document.getElementById('broadcastDuration');
    if (bdEl) bdEl.value = String(cfg.broadcast_duration_min);
  }
  if (cfg.carousel_enabled !== undefined) {
    setBroadcastMode(cfg.carousel_enabled);
  }
  if (cfg.carousel_mask !== undefined) {
    const ids = ['car_bpc', 'car_wwvb', 'car_msf', 'car_dcf77', 'car_jjy40', 'car_jjy60'];
    for (let i = 0; i < 6; i++) {
      const el = document.getElementById(ids[i]);
      if (el) el.checked = !!(cfg.carousel_mask & (1 << i));
    }
  }
  if (cfg.carousel_duration_min !== undefined) {
    const cdEl = document.getElementById('carouselDuration');
    if (cdEl) cdEl.value = String(cfg.carousel_duration_min);
  }
  updateCarouselCalc();

  // NTP parameters
  if (cfg.system && cfg.system.ntp_server) {
    const ntpEl = document.getElementById('ntpServer');
    if (ntpEl) ntpEl.value = cfg.system.ntp_server;
  }

  // Wi-Fi Radio parameters
  if (cfg.wifi) {
    const txEl = document.getElementById('cfgWifiTxPower');
    if (txEl && cfg.wifi.tx_power !== undefined) {
      txEl.value = String(cfg.wifi.tx_power);
    }
    const msEl = document.getElementById('cfgWifiModemSleep');
    if (msEl && cfg.wifi.modem_sleep !== undefined) {
      msEl.checked = cfg.wifi.modem_sleep;
    }
    const bwEl = document.getElementById('cfgWifiForce20mhz');
    if (bwEl && cfg.wifi.force_20mhz !== undefined) {
      bwEl.checked = cfg.wifi.force_20mhz;
    }
  }
}

function capitalize(s) {
  return s.charAt(0).toUpperCase() + s.slice(1);
}

// --- Status Polling & Clock Ticking ---
function scheduleAdaptivePoll() {
  if (statusPollTimer) clearTimeout(statusPollTimer);
  const interval = isTransmitting ? 5000 : 15000;
  statusPollTimer = setTimeout(async () => {
    await pollStatus();
  }, interval);
}

async function pollStatus() {
  try {
    const res = await fetchWithTimeout('/api/status', {}, 6000);
    if (!res.ok) throw new Error('Status fetch failed');
    const data = await res.json();
    appStatus = data;

    const activeSt = data.active_station !== undefined ? data.active_station : (data.activeStation || 0);
    const offsetH = data.offset_hours !== undefined ? data.offset_hours : (data.offsetHours || 0);
    const carEn = data.carousel_enabled !== undefined ? data.carousel_enabled : (data.carouselEnabled || false);
    const epSec = data.epoch_sec !== undefined ? data.epoch_sec : (data.epochSec || Math.floor(Date.now() / 1000));
    const totRemain = data.total_remain_sec !== undefined ? data.total_remain_sec : (data.totalRemainSec || 0);

    isTransmitting = !!data.transmitting;
    activeStationIdx = activeSt;
    userOffsetHours = offsetH;
    carouselMode = carEn;
    serverEpochSec = epSec;
    clientEpochBaseMs = Date.now();
    transmitRemainSec = totRemain;
    transmitRemainBaseMs = Date.now();

    // Update UI elements
    updateStatusDisplay(data);
    scheduleAdaptivePoll();
  } catch (err) {
    console.warn('Status poll offline:', err);
    scheduleAdaptivePoll();
  }
}

function updateStopButtonCountdown() {
  const btnStop = document.getElementById('btnStop');
  if (!btnStop) return;
  if (isTransmitting && transmitRemainSec > 0) {
    const elapsedSec = Math.floor((Date.now() - transmitRemainBaseMs) / 1000);
    const remain = Math.max(0, transmitRemainSec - elapsedSec);
    const min = Math.floor(remain / 60);
    const sec = remain % 60;
    btnStop.innerText = `⏹ Stop (${String(min).padStart(2, '0')}:${String(sec).padStart(2, '0')})`;
    if (remain === 0 && isTransmitting) {
      setTimeout(pollStatus, 500);
    }
  } else {
    btnStop.innerText = '⏹ Stop';
  }
}

function updateStatusDisplay(data) {
  const statusBadge = document.getElementById('statusBadge');
  const statusText = document.getElementById('statusText');
  const btnTransmit = document.getElementById('btnTransmit');
  const btnStop = document.getElementById('btnStop');
  const manualSyncBanner = document.getElementById('manualSyncBanner');
  const manualSyncStation = document.getElementById('manualSyncStation');
  const triggerSource = data.trigger_source || 'standby';
  const isButtonTriggered = data.transmitting && triggerSource === 'button';

  const activeSt = data.active_station !== undefined ? data.active_station : (data.activeStation || 0);
  const st = STATIONS[activeSt] || STATIONS[0];

  if (data.transmitting) {
    statusBadge.className = 'status-indicator status-live';
    if (isButtonTriggered) {
      statusText.innerText = 'TRANSMITTING (Bedside Button)';
      if (manualSyncBanner) {
        if (manualSyncStation) manualSyncStation.innerText = st.name;
        manualSyncBanner.style.display = 'flex';
      }
    } else {
      statusText.innerText = 'TRANSMITTING';
      if (manualSyncBanner) manualSyncBanner.style.display = 'none';
    }
    btnTransmit.disabled = true;
    btnStop.disabled = false;
  } else {
    statusBadge.className = 'status-indicator status-standby';
    statusText.innerText = 'STANDBY';
    btnTransmit.disabled = false;
    btnStop.disabled = true;
    if (manualSyncBanner) manualSyncBanner.style.display = 'none';
  }
  updateStopButtonCountdown();

  // Update GPIO button diagnostics
  const diagGpio3 = document.getElementById('diagGpio3');
  if (diagGpio3 && data.gpio3_level !== undefined) {
    diagGpio3.textContent = data.gpio3_level === 0 ? 'LOW (Pressed)' : 'HIGH (Released)';
    diagGpio3.style.color = data.gpio3_level === 0 ? 'var(--accent)' : 'inherit';
  }
  const diagGpio9 = document.getElementById('diagGpio9');
  if (diagGpio9 && data.gpio9_level !== undefined) {
    diagGpio9.textContent = data.gpio9_level === 0 ? 'LOW (Pressed)' : 'HIGH (Released)';
    diagGpio9.style.color = data.gpio9_level === 0 ? 'var(--accent)' : 'inherit';
  }

  const carEn = data.carousel_enabled !== undefined ? data.carousel_enabled : (data.carouselEnabled || false);
  document.getElementById('activeStation').innerText = carEn
    ? `🔄 ${st.name}`
    : st.name;

  const freq = data.carrier_freq || data.carrierFreq || st.freq;
  document.getElementById('carrierFreq').innerText = freq.toLocaleString() + ' Hz';

  // Carousel progress
  const progressBox = document.getElementById('carouselProgressBox');
  const carStages = data.carousel_total_stages !== undefined ? data.carousel_total_stages : (data.carouselTotalStages || 1);
  const carStage = data.carousel_stage !== undefined ? data.carousel_stage : (data.carouselStage || 1);
  const stageRemain = data.stage_remain_sec !== undefined ? data.stage_remain_sec : (data.stageRemainSec || 0);

  if (data.transmitting && carEn && carStages > 1) {
    progressBox.style.display = 'block';
    document.getElementById('carouselStageLabel').innerText =
      `Stage ${carStage}/${carStages}: ${st.name}`;
    const min = Math.floor(stageRemain / 60);
    const sec = stageRemain % 60;
    document.getElementById('carouselTimeRemain').innerText =
      `${String(min).padStart(2, '0')}:${String(sec).padStart(2, '0')} remain`;
  } else {
    progressBox.style.display = 'none';
  }

  // Network stats
  if (data.ip) document.getElementById('netIp').innerText = data.ip;
  const ssid = data.wifi_ssid || data.wifiSsid;
  if (ssid) document.getElementById('netSsid').innerText = ssid;
  const rssi = data.wifi_rssi !== undefined ? data.wifi_rssi : data.wifiRssi;
  if (rssi !== undefined && rssi !== 0) {
    document.getElementById('netRssi').innerText = `${rssi} dBm`;
    const diagRssi = document.getElementById('diagRssi');
    if (diagRssi) diagRssi.textContent = `${rssi} dBm`;
  }
  if (data.uptime_sec !== undefined) {
    const diagUptime = document.getElementById('diagUptime');
    if (diagUptime) diagUptime.textContent = formatDuration(data.uptime_sec);
  }
  const hostname = data.hostname;
  if (hostname) {
    document.getElementById('netMdns').innerText = `${hostname}.local`;
  }
  if (data.version) {
    currentFirmwareVersion = data.version;
    const cleanVer = data.version.startsWith('v') ? data.version : ('v' + data.version);
    document.getElementById('versionBadge').innerText = cleanVer;
    document.getElementById('otaFirmwareVersion').innerText = cleanVer;
  }
}

// Tick clock every second locally based on synced server epoch
function tickClock() {
  if (serverEpochSec === 0) return;
  const elapsedSec = Math.floor((Date.now() - clientEpochBaseMs) / 1000);
  const currentSec = serverEpochSec + elapsedSec;

  const utcEl = document.getElementById('utcClock');
  const localEl = document.getElementById('localClock');

  if (currentSec < 1700000000) {
    if (utcEl) utcEl.innerText = 'Syncing NTP...';
    if (localEl) localEl.innerText = 'Syncing NTP...';
  } else {
    const utcDate = new Date(currentSec * 1000);
    if (utcEl) {
      utcEl.innerText = utcDate.toISOString().substring(11, 19) + ' UTC';
    }

    const st = STATIONS[activeStationIdx] || STATIONS[0];
    const stationSec = currentSec + st.tzOffset + (userOffsetHours * 3600);
    const stationDate = new Date(stationSec * 1000);
    if (localEl) {
      localEl.innerText = stationDate.toISOString().substring(11, 19);
    }
  }

  updateStopButtonCountdown();
}

// --- Broadcast Control Actions ---
async function triggerTransmit() {
  try {
    const res = await fetch('/api/transmit', { method: 'POST' });
    if (!res.ok) throw new Error('Start failed');
    pollStatus();
  } catch (e) {
    alert('Failed to start broadcast: ' + e.message);
  }
}

async function triggerStop() {
  try {
    const res = await fetch('/api/stop', { method: 'POST' });
    if (!res.ok) throw new Error('Stop failed');
    pollStatus();
  } catch (e) {
    alert('Failed to stop broadcast: ' + e.message);
  }
}

function setBroadcastMode(isCarousel) {
  carouselMode = isCarousel;
  document.getElementById('btnModeSingle').classList.toggle('active', !isCarousel);
  document.getElementById('btnModeCarousel').classList.toggle('active', isCarousel);
  document.getElementById('singleStationSection').style.display = isCarousel ? 'none' : 'block';
  document.getElementById('carouselSection').style.display = isCarousel ? 'block' : 'none';
  updateCarouselCalc();
}

function onStationSelectChange() {
  const idx = parseInt(document.getElementById('station').value);
  activeStationIdx = idx;
  const st = STATIONS[idx];
  document.getElementById('activeStation').innerText = st.name;
  document.getElementById('carrierFreq').innerText = st.freq.toLocaleString() + ' Hz';
}

function updateCarouselCalc() {
  let count = 0;
  for (let i = 0; i < 6; i++) {
    const ids = ['car_bpc', 'car_wwvb', 'car_msf', 'car_dcf77', 'car_jjy40', 'car_jjy60'];
    const el = document.getElementById(ids[i]);
    if (el && el.checked) count++;
  }
  const dur = parseInt(document.getElementById('carouselDuration').value) || 15;
  const total = count * dur;
  document.getElementById('carouselTotalCalc').innerText =
    `${total} minutes (${count} stations × ${dur} min)`;
}

async function saveBroadcastConfig(e) {
  e.preventDefault();
  const alertBox = document.getElementById('broadcastAlert');
  alertBox.style.display = 'none';

  let mask = 0;
  const ids = ['car_bpc', 'car_wwvb', 'car_msf', 'car_dcf77', 'car_jjy40', 'car_jjy60'];
  for (let i = 0; i < 6; i++) {
    const el = document.getElementById(ids[i]);
    if (el && el.checked) mask |= (1 << i);
  }

  const payload = {
    station: parseInt(document.getElementById('station').value),
    offset_hours: parseInt(document.getElementById('offsetHours').value),
    schedule_enabled: document.getElementById('scheduleEnabled').checked,
    broadcast_time: document.getElementById('broadcastTime').value,
    broadcast_duration_min: parseInt(document.getElementById('broadcastDuration').value),
    carousel_enabled: carouselMode,
    carousel_mask: mask,
    carousel_duration_min: parseInt(document.getElementById('carouselDuration').value)
  };

  try {
    const res = await fetch('/api/config', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(payload)
    });
    if (!res.ok) throw new Error('Save failed');
    alertBox.className = 'alert success';
    alertBox.innerText = '✓ Broadcast configuration saved successfully!';
    pollStatus();
  } catch (err) {
    alertBox.className = 'alert error';
    alertBox.innerText = '✗ Error saving configuration: ' + err.message;
  }
}

// --- Network & Wi-Fi Actions ---
async function scanWifiNetworks(btn) {
  const select = document.getElementById('scanSelect');
  btn.disabled = true;
  btn.innerText = 'Scanning...';
  select.innerHTML = '<option value="" disabled selected>Scanning 2.4 GHz networks...</option>';

  try {
    const res = await fetch('/api/scan');
    if (!res.ok) throw new Error('Scan failed');
    const networks = await res.json();

    select.innerHTML = '<option value="" disabled selected>Select a discovered network</option>';
    if (networks && networks.length > 0) {
      networks.forEach(net => {
        const opt = document.createElement('option');
        opt.value = net;
        opt.textContent = net;
        select.appendChild(opt);
      });
    }
    const custom = document.createElement('option');
    custom.value = '__custom__';
    custom.textContent = '✎ Enter custom/hidden SSID...';
    select.appendChild(custom);
  } catch (e) {
    select.innerHTML = '<option value="" disabled selected>Scan failed or timed out</option>';
  } finally {
    btn.disabled = false;
    btn.innerText = '🔍 Scan Networks';
  }
}

function onScanSelect(val) {
  if (val !== '__custom__') {
    document.getElementById('wifiSsid').value = val;
    document.getElementById('wifiPassword').focus();
  } else {
    document.getElementById('wifiSsid').value = '';
    document.getElementById('wifiSsid').focus();
  }
}

function togglePasswordVisibility(fieldId, btn) {
  const f = document.getElementById(fieldId);
  if (f.type === 'password') {
    f.type = 'text';
    btn.innerText = 'Hide';
  } else {
    f.type = 'password';
    btn.innerText = 'Show';
  }
}

function setNtpPreset(server) {
  document.getElementById('ntpServer').value = server;
}

async function saveWifiCredentials() {
  const ssid = document.getElementById('wifiSsid').value.trim();
  const password = document.getElementById('wifiPassword').value;
  if (!ssid) {
    alert('Please choose or enter a Wi-Fi network SSID.');
    return;
  }
  if (!confirm(`Save Wi-Fi credentials for "${ssid}" and connect? The beacon will restart.`)) {
    return;
  }

  try {
    const res = await fetch('/api/provision', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ ssid, password })
    });
    if (!res.ok) throw new Error('Save failed');
    alert('Credentials saved! The device is restarting into station mode.');
  } catch (e) {
    alert('Failed to save Wi-Fi: ' + e.message);
  }
}

async function saveWifiRadioConfig(e) {
  e.preventDefault();
  const alertBox = document.getElementById('radioAlert');
  alertBox.style.display = 'none';

  const txPower = parseInt(document.getElementById('cfgWifiTxPower').value, 10);
  const modemSleep = document.getElementById('cfgWifiModemSleep').checked;
  const force20mhz = document.getElementById('cfgWifiForce20mhz').checked;

  const payload = {
    wifi: {
      tx_power: txPower,
      modem_sleep: modemSleep,
      force_20mhz: force20mhz
    }
  };

  try {
    const res = await fetch('/api/config', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(payload)
    });
    if (!res.ok) throw new Error('Failed to commit radio settings');
    if (!appConfig) appConfig = {};
    if (!appConfig.wifi) appConfig.wifi = {};
    Object.assign(appConfig.wifi, payload.wifi);

    alertBox.className = 'alert success';
    alertBox.innerText = '✓ Wi-Fi radio parameters saved and applied immediately!';
    alertBox.style.display = 'block';
    setTimeout(() => { alertBox.style.display = 'none'; }, 6000);
  } catch (err) {
    alertBox.className = 'alert error';
    alertBox.innerText = '✗ Error saving radio settings: ' + err.message;
    alertBox.style.display = 'block';
  }
}

async function saveNtpConfig(e) {
  e.preventDefault();
  const alertBox = document.getElementById('ntpAlert');
  alertBox.style.display = 'none';

  const server = document.getElementById('ntpServer').value.trim();
  if (!server) {
    alert('Please enter a valid NTP server.');
    return;
  }

  const payload = {
    system: {
      ntp_server: server
    }
  };

  try {
    const res = await fetch('/api/config', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(payload)
    });
    if (!res.ok) throw new Error('Failed to commit NTP server');
    if (!appConfig) appConfig = {};
    if (!appConfig.system) appConfig.system = {};
    Object.assign(appConfig.system, payload.system);

    alertBox.className = 'alert success';
    alertBox.innerText = '✓ NTP server updated and saved!';
    alertBox.style.display = 'block';
    setTimeout(() => { alertBox.style.display = 'none'; }, 6000);
  } catch (err) {
    alertBox.className = 'alert error';
    alertBox.innerText = '✗ Error saving NTP settings: ' + err.message;
    alertBox.style.display = 'block';
  }
}

async function forceNtpSync() {
  const alertBox = document.getElementById('ntpAlert');
  if (alertBox) {
    alertBox.className = 'alert';
    alertBox.innerText = 'Requesting immediate NTP synchronization...';
    alertBox.style.display = 'block';
  }
  try {
    const res = await fetch('/api/sync-ntp', { method: 'POST' });
    if (!res.ok) throw new Error('Sync request failed');
    setTimeout(pollStatus, 1500);
    if (alertBox) {
      alertBox.className = 'alert success';
      alertBox.innerText = '✓ NTP synchronization requested from server.';
      setTimeout(() => { alertBox.style.display = 'none'; }, 6000);
    }
  } catch (err) {
    if (alertBox) {
      alertBox.className = 'alert error';
      alertBox.innerText = '✗ Error: ' + err.message;
    }
  }
}

async function resetWifiProvisioning() {
  if (!confirm('Are you sure you want to erase Wi-Fi credentials?\nThe beacon will reboot into SoftAP Provisioning Mode (TimeStation-Setup-XXXX) at 192.168.4.1.')) {
    return;
  }
  try {
    const res = await fetch('/api/wifi/reset', { method: 'POST' });
    if (!res.ok) throw new Error('Reset failed');
    alert('Wi-Fi credentials erased! Rebooting into setup mode...');
  } catch (e) {
    alert('Failed to reset Wi-Fi: ' + e.message);
  }
}

async function rebootEsp32() {
  if (!confirm('Reboot the ESP32 microcontroller?')) return;
  try {
    await fetch('/api/system/reboot', { method: 'POST' });
    alert('Reboot signal sent. The device will restart in 2 seconds.');
  } catch (e) {
    alert('Reboot command sent.');
  }
}

// --- OTA Firmware Actions ---

let latestOtaStatus = null;

// SemVer comparison and Markdown formatting helpers
function parseSemVer(v) {
  if (!v) return [0, 0, 0];
  const cleaned = v.replace(/^v/i, '').trim();
  const parts = cleaned.split('.').map(n => parseInt(n, 10) || 0);
  while (parts.length < 3) parts.push(0);
  return parts;
}

function isNewerVersion(remote, local) {
  const [rMaj, rMin, rPat] = parseSemVer(remote);
  const [lMaj, lMin, lPat] = parseSemVer(local);
  if (rMaj !== lMaj) return rMaj > lMaj;
  if (rMin !== lMin) return rMin > lMin;
  return rPat > lPat;
}

function formatMarkdown(text) {
  if (!text) return '<p>No release notes provided for this version.</p>';
  let html = text
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/^### (.*$)/gim, '<h4 style="color: var(--accent); margin: 12px 0 6px 0; font-size: 14px;">$1</h4>')
    .replace(/^## (.*$)/gim, '<h3 style="color: var(--accent); margin: 14px 0 8px 0; font-size: 15px;">$1</h3>')
    .replace(/\*\*(.*?)\*\*/g, '<strong>$1</strong>')
    .replace(/`([^`]+)`/g, '<code style="background: rgba(192, 132, 252, 0.15); padding: 2px 5px; border-radius: 4px; font-size: 12px;">$1</code>')
    .replace(/^\s*[-•]\s+(.*$)/gim, '<li style="margin-bottom: 4px;">$1</li>')
    .replace(/\[([^\]]+)\]\(([^)]+)\)/g, '<a href="$2" target="_blank" rel="noopener noreferrer" style="color: var(--accent); text-decoration: underline;">$1</a>');

  html = html.replace(/((?:<li>.*?<\/li>\s*)+)/gims, '<ul style="padding-left: 20px; margin: 6px 0 12px;">$1</ul>');
  html = html.replace(/\n\n+/g, '<p style="margin: 6px 0;"></p>');
  return html;
}

function formatTimeAgo(ts) {
  if (!ts) return 'never';
  const diffSec = Math.floor((Date.now() - ts) / 1000);
  if (diffSec < 60) return 'just now';
  const diffMin = Math.floor(diffSec / 60);
  if (diffMin < 60) return `${diffMin}m ago`;
  const diffHours = Math.floor(diffMin / 60);
  if (diffHours < 24) return `${diffHours}h ago`;
  const diffDays = Math.floor(diffHours / 24);
  return `${diffDays}d ago`;
}

function renderUpdateBadge(status) {
  const badge = document.getElementById('otaUpdateBadge');
  const meta = document.getElementById('otaCheckMeta');
  const btn = document.getElementById('btnOtaUrl');
  if (!badge) return;

  if (!status) {
    badge.innerText = 'Check unavailable';
    badge.className = 'badge';
    return;
  }

  const currentVer = currentFirmwareVersion.startsWith('v') ? currentFirmwareVersion : ('v' + currentFirmwareVersion);
  const remoteTag = status.latest_version || currentVer;
  const isNewer = Boolean(status.update_available);

  if (isNewer) {
    badge.innerText = `✨ Update Available: ${remoteTag}`;
    badge.className = 'badge badge-update';
    if (btn) btn.innerText = `⬇ Review & Apply ${remoteTag}`;
  } else if (status.last_check_status === 'ok') {
    badge.innerText = `✓ Up to date (${currentVer})`;
    badge.className = 'badge badge-green';
    if (btn) btn.innerText = `⬇ Re-flash ${currentVer}`;
  } else if (status.last_check_status === 'pending_initial_check') {
    badge.innerText = '⏳ Initial check pending...';
    badge.className = 'badge';
  } else {
    badge.innerText = 'Check unavailable';
    badge.className = 'badge';
  }

  // Update binary URL input if provided
  if (status.binary_url) {
    const urlEl = document.getElementById('otaUrl');
    if (urlEl) urlEl.value = status.binary_url;
  }

  if (meta) {
    if (status.last_check_epoch && status.last_check_epoch > 0) {
      meta.innerText = `Daily check: checked ${formatTimeAgo(status.last_check_epoch * 1000)} (auto-checks after boot & at 00:05 midnight)`;
    } else {
      meta.innerText = 'Daily check: auto-checks after boot and daily at 00:05 midnight';
    }
  }
}

async function triggerDeviceOtaCheck() {
  const badge = document.getElementById('otaUpdateBadge');
  const meta = document.getElementById('otaCheckMeta');
  if (badge) {
    badge.innerText = 'Checking ESP32...';
    badge.className = 'badge';
  }
  if (meta) {
    meta.innerText = 'ESP32 querying latest release from GitHub...';
  }
  try {
    const res = await fetch('/api/ota/check', { method: 'POST' });
    if (!res.ok) throw new Error('HTTP ' + res.status);

    let attempts = 0;
    const pollInterval = setInterval(async () => {
      attempts++;
      await fetchOtaStatus();
      if (attempts >= 5 || (latestOtaStatus && latestOtaStatus.update_available)) {
        clearInterval(pollInterval);
      }
    }, 1500);
  } catch (err) {
    if (meta) meta.innerText = 'Check request failed: ' + err.message;
  }
}

async function fetchOtaStatus() {
  try {
    const res = await fetchWithTimeout('/api/ota/status', {}, 6000);
    if (!res.ok) throw new Error('OTA status fetch failed');
    const data = await res.json();
    latestOtaStatus = data;

    if (data.firmware_version) {
      currentFirmwareVersion = data.firmware_version;
      const cleanVer = data.firmware_version.startsWith('v') ? data.firmware_version : ('v' + data.firmware_version);
      document.getElementById('otaFirmwareVersion').innerText = cleanVer;
    }
    document.getElementById('otaRunningSlot').innerText = data.running_slot || '--';
    document.getElementById('otaNextSlot').innerText = data.next_update_slot || '--';
    if (data.default_url) document.getElementById('otaUrl').value = data.default_url;

    renderUpdateBadge(data);
  } catch (e) {
    console.warn('OTA status error:', e);
  }
}

// Reports release details and changelog before asking user to confirm
async function showOtaReportBeforeApply() {
  const currentVer = currentFirmwareVersion.startsWith('v') ? currentFirmwareVersion : ('v' + currentFirmwareVersion);
  const targetVer = (latestOtaStatus && latestOtaStatus.latest_version) ? latestOtaStatus.latest_version : currentVer;
  const isNewer = latestOtaStatus ? Boolean(latestOtaStatus.update_available) : false;

  const notesPlaceholder = '<p style="color: var(--text-muted);">Fetching release notes from GitHub...</p>';
  const title = isNewer
    ? `🚀 New Firmware Update Available: ${targetVer}`
    : `📦 Firmware Release Details: ${targetVer}`;

  const renderModal = (contentHtml) => {
    const html = `
      <div style="margin-bottom: 14px; padding: 12px; background: rgba(0, 0, 0, 0.35); border-radius: 10px; border: 1px solid var(--border);">
        <div style="display: flex; justify-content: space-between; font-size: 13px; margin-bottom: 6px;">
          <span style="color: var(--text-muted);">Current Installed Version:</span>
          <strong>${currentVer}</strong>
        </div>
        <div style="display: flex; justify-content: space-between; font-size: 13px;">
          <span style="color: var(--text-muted);">Target Release Version:</span>
          <strong style="color: ${isNewer ? '#fef08a' : '#10b981'}; font-weight: 700;">${targetVer} ${isNewer ? '(Newer Version)' : '(Currently Installed)'}</strong>
        </div>
      </div>

      <div style="margin-bottom: 14px;">
        <div style="font-weight: 700; color: var(--accent); margin-bottom: 8px; font-size: 13px;">
          📋 What's Included in This Release:
        </div>
        <div id="otaModalNotes" style="max-height: 220px; overflow-y: auto; padding: 10px 14px; background: rgba(0, 0, 0, 0.2); border-radius: 8px; border: 1px solid rgba(255, 255, 255, 0.1);">
          ${contentHtml}
        </div>
      </div>

      <div style="font-size: 12px; color: var(--text-muted); margin-bottom: 16px; line-height: 1.45;">
        🛡️ <strong>Dual-OTA Rollback Protected:</strong> The firmware will stream directly into your standby partition over TLS. The ESP32 bootloader automatically rolls back to your working slot if verification fails.
      </div>

      <div style="display: flex; gap: 10px; justify-content: flex-end;">
        <button type="button" class="btn btn-secondary" onclick="hideInfo()">Cancel</button>
        <button type="button" class="btn btn-primary" onclick="confirmAndApplyOta()">
          ⬇ Flash &amp; Apply ${targetVer}
        </button>
      </div>
    `;
    showInfo(title, html);
  };

  renderModal(notesPlaceholder);

  // Asynchronously fetch GitHub release notes to populate details without blocking modal
  try {
    const res = await fetch(`https://api.github.com/repos/thomaslr/MultiBand6-ESP32-AtomicBeacon/releases/tags/${targetVer}`);
    if (res.ok) {
      const data = await res.json();
      const el = document.getElementById('otaModalNotes');
      if (el) el.innerHTML = formatMarkdown(data.body);
    } else {
      const el = document.getElementById('otaModalNotes');
      if (el) el.innerHTML = `<p>Release ${targetVer} is verified and ready to install. <a href="https://github.com/thomaslr/MultiBand6-ESP32-AtomicBeacon/releases/tag/${targetVer}" target="_blank" rel="noopener noreferrer">View changelog on GitHub</a>.</p>`;
    }
  } catch (_) {
    const el = document.getElementById('otaModalNotes');
    if (el) el.innerHTML = `<p>Release ${targetVer} is ready to flash directly from GitHub Releases. <a href="https://github.com/thomaslr/MultiBand6-ESP32-AtomicBeacon/releases" target="_blank" rel="noopener noreferrer">View changelog on GitHub</a>.</p>`;
  }
}

function confirmAndApplyOta() {
  hideInfo();
  triggerOtaFromUrl(true);
}

async function triggerOtaFromUrl(confirmed = false) {
  const url = document.getElementById('otaUrl').value.trim();
  if (!url) {
    alert('Please enter a firmware binary URL.');
    return;
  }
  if (!confirmed && !confirm(`Download and flash firmware from:\n${url}\n\nThe ESP32 will reboot automatically after completion.`)) {
    return;
  }

  const btn = document.getElementById('btnOtaUrl');
  const bar = document.getElementById('otaUrlProgress');
  const alertBox = document.getElementById('otaAlert');
  btn.disabled = true;
  bar.style.display = 'block';
  alertBox.style.display = 'none';

  try {
    const res = await fetch('/api/ota/update', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ url })
    });
    const result = await res.json();
    if (!res.ok) throw new Error(result.error || 'OTA failed');
    alertBox.className = 'alert success';
    alertBox.innerText = '✓ OTA flash successful! Rebooting into updated slot in 2 seconds...';
    alertBox.style.display = 'block';
  } catch (err) {
    btn.disabled = false;
    bar.style.display = 'none';
    alertBox.className = 'alert error';
    alertBox.innerText = '✗ OTA update failed: ' + err.message;
    alertBox.style.display = 'block';
  }
}

function uploadOtaBinary() {
  const fileInput = document.getElementById('otaFileInput');
  if (!fileInput.files || fileInput.files.length === 0) {
    alert('Please select a .bin firmware binary file first.');
    return;
  }
  const file = fileInput.files[0];
  if (!file.name.endsWith('.bin')) {
    alert('Invalid file: only .bin files are supported.');
    return;
  }
  if (file.size > 2031616) { // 1984 KB
    alert(`File is too large (${(file.size / 1024).toFixed(0)} KB). Dual-OTA partition limit is 1984 KB!`);
    return;
  }

  if (!confirm(`Flash ${file.name} (${(file.size / 1024).toFixed(0)} KB) to the standby partition?`)) {
    return;
  }

  const btn = document.getElementById('btnOtaUpload');
  const bar = document.getElementById('otaUploadProgress');
  const fill = document.getElementById('otaUploadFill');
  const alertBox = document.getElementById('otaAlert');

  btn.disabled = true;
  bar.style.display = 'block';
  fill.style.width = '0%';
  alertBox.style.display = 'none';

  const xhr = new XMLHttpRequest();
  xhr.open('POST', '/api/ota/upload', true);
  xhr.setRequestHeader('Content-Type', 'application/octet-stream');

  xhr.upload.onprogress = (e) => {
    if (e.lengthComputable) {
      const pct = Math.round((e.loaded / e.total) * 100);
      fill.style.width = pct + '%';
    }
  };

  xhr.onload = () => {
    if (xhr.status >= 200 && xhr.status < 300) {
      alertBox.className = 'alert success';
      alertBox.innerText = '✓ Upload complete and flashed! Device is restarting...';
    } else {
      btn.disabled = false;
      bar.style.display = 'none';
      alertBox.className = 'alert error';
      alertBox.innerText = '✗ Upload error: ' + xhr.responseText;
    }
  };

  xhr.onerror = () => {
    btn.disabled = false;
    bar.style.display = 'none';
    alertBox.className = 'alert error';
    alertBox.innerText = '✗ Upload failed due to network error.';
  };

  xhr.send(file);
}

// --- Popover Information Modal ---
function showInfo(title, htmlContent) {
  document.getElementById('popTitle').innerText = title;
  document.getElementById('popBody').innerHTML = htmlContent;
  document.getElementById('popoverBackdrop').style.display = 'block';
  document.getElementById('globalPopover').style.display = 'block';
}

function hideInfo() {
  document.getElementById('popoverBackdrop').style.display = 'none';
  document.getElementById('globalPopover').style.display = 'none';
}

// --- Diagnostics & Live Log Streaming ---
async function fetchLogs() {
  try {
    const res = await fetchWithTimeout('/api/logs', {}, 6000);
    if (!res.ok) return;
    const data = await res.json();

    // Update Telemetry Metrics
    if (data.free_heap !== undefined) {
      document.getElementById('diagFreeHeap').textContent = (data.free_heap / 1024).toFixed(1) + ' KB';
    }
    if (data.min_free_heap !== undefined) {
      document.getElementById('diagMinHeap').textContent = (data.min_free_heap / 1024).toFixed(1) + ' KB';
    }
    if (data.uptime_sec !== undefined) {
      document.getElementById('diagUptime').textContent = formatDuration(data.uptime_sec);
    }
    if (appStatus && appStatus.wifi_rssi !== undefined) {
      document.getElementById('diagRssi').textContent = appStatus.wifi_rssi + ' dBm';
    }

    // Update Terminal Console
    const consoleElem = document.getElementById('logConsole');
    if (consoleElem && data.logs) {
      consoleElem.textContent = data.logs.join('\n') || '[Log buffer is currently empty]';
      const autoScroll = document.getElementById('autoScrollLogs');
      if (autoScroll && autoScroll.checked) {
        consoleElem.scrollTop = consoleElem.scrollHeight;
      }
    }
  } catch (err) {
    console.warn('Failed to fetch diagnostics:', err);
  }
}

async function clearLogs() {
  try {
    await fetch('/api/logs/clear', { method: 'POST' });
    fetchLogs();
  } catch (e) {
    console.warn('Failed to clear logs:', e);
  }
}

function copyLogs() {
  const consoleElem = document.getElementById('logConsole');
  if (consoleElem) {
    navigator.clipboard.writeText(consoleElem.textContent).then(() => {
      alert('Diagnostics logs copied to clipboard!');
    }).catch(() => {
      alert('Unable to copy logs to clipboard.');
    });
  }
}

// --- Initial Startup ---
function initDashboard() {
  if (dashboardInitialized) return;
  dashboardInitialized = true;

  // Register Service Worker for PWA Standalone App support
  if ('serviceWorker' in navigator) {
    navigator.serviceWorker.register('/sw.js').then((reg) => {
      reg.update().catch(() => {});
    }).catch((err) => {
      console.warn('[PWA] ServiceWorker registration failed:', err);
    });
  }

  // Prioritize real-time status & clock first so single-threaded ESP32 httpd responds immediately
  pollStatus(); // Initializes state and starts adaptive polling loop (5s active, 15s standby)
  setInterval(tickClock, 1000);   // 1-second local clock tick

  // Stagger secondary initial fetches slightly to avoid socket contention on initial page load
  setTimeout(fetchConfig, 150);
  setTimeout(fetchOtaStatus, 400);
  setInterval(fetchOtaStatus, 300000); // 5-minute OTA status refresh from ESP32

  // Immediate refresh when tab becomes visible after being backgrounded
  document.addEventListener('visibilitychange', () => {
    if (!document.hidden) {
      pollStatus();
      fetchOtaStatus();
    }
  });
}

if (document.readyState === 'loading') {
  document.addEventListener('DOMContentLoaded', initDashboard);
} else {
  initDashboard();
}
window.addEventListener('load', initDashboard);
