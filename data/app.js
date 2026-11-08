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
let appConfig = null;

// --- Tab Management ---
function switchTab(tabName) {
  const tabs = ['broadcast', 'network', 'ota'];
  tabs.forEach(t => {
    const btn = document.getElementById('tabBtn' + capitalize(t));
    const content = document.getElementById('tabContent' + capitalize(t));
    if (btn) btn.classList.toggle('active', t === tabName);
    if (content) content.style.display = (t === tabName) ? 'block' : 'none';
  });
  if (tabName === 'network' || tabName === 'broadcast') fetchConfig();
  if (tabName === 'ota') fetchOtaStatus();
}

async function fetchConfig() {
  try {
    const res = await fetch('/api/config');
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
    if (stEl) stEl.value = String(cfg.station);
    activeStationIdx = cfg.station;
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
async function pollStatus() {
  try {
    const res = await fetch('/api/status');
    if (!res.ok) throw new Error('Status fetch failed');
    const data = await res.json();

    isTransmitting = data.transmitting;
    activeStationIdx = data.activeStation || 0;
    userOffsetHours = data.offsetHours || 0;
    carouselMode = data.carouselEnabled || false;
    serverEpochSec = data.epochSec || Math.floor(Date.now() / 1000);
    clientEpochBaseMs = Date.now();

    // Update UI elements
    updateStatusDisplay(data);
  } catch (err) {
    console.warn('Status poll offline:', err);
  }
}

function updateStatusDisplay(data) {
  const statusBadge = document.getElementById('statusBadge');
  const statusText = document.getElementById('statusText');
  const btnTransmit = document.getElementById('btnTransmit');
  const btnStop = document.getElementById('btnStop');

  if (data.transmitting) {
    statusBadge.className = 'status-indicator status-live';
    statusText.innerText = 'TRANSMITTING';
    btnTransmit.disabled = true;
    btnStop.disabled = false;
  } else {
    statusBadge.className = 'status-indicator status-standby';
    statusText.innerText = 'STANDBY';
    btnTransmit.disabled = false;
    btnStop.disabled = true;
  }

  const st = STATIONS[data.activeStation] || STATIONS[0];
  document.getElementById('activeStation').innerText = data.carouselEnabled
    ? `🔄 ${st.name}`
    : st.name;
  document.getElementById('carrierFreq').innerText = (data.carrierFreq || st.freq).toLocaleString() + ' Hz';

  // Carousel progress
  const progressBox = document.getElementById('carouselProgressBox');
  if (data.transmitting && data.carouselEnabled && data.carouselTotalStages > 1) {
    progressBox.style.display = 'block';
    document.getElementById('carouselStageLabel').innerText =
      `Stage ${data.carouselStage}/${data.carouselTotalStages}: ${st.name}`;
    const min = Math.floor(data.stageRemainSec / 60);
    const sec = data.stageRemainSec % 60;
    document.getElementById('carouselTimeRemain').innerText =
      `${String(min).padStart(2, '0')}:${String(sec).padStart(2, '0')} remain`;
  } else {
    progressBox.style.display = 'none';
  }

  // Network stats
  if (data.ip) document.getElementById('netIp').innerText = data.ip;
  if (data.wifiSsid) document.getElementById('netSsid').innerText = data.wifiSsid;
  if (data.wifiRssi !== undefined) {
    document.getElementById('netRssi').innerText = `${data.wifiRssi} dBm`;
  }
  if (data.hostname) {
    document.getElementById('netMdns').innerText = `${data.hostname}.local`;
  }
  if (data.version) {
    document.getElementById('versionBadge').innerText = data.version;
    document.getElementById('otaFirmwareVersion').innerText = data.version;
  }
}

// Tick clock every second locally based on synced server epoch
function tickClock() {
  if (serverEpochSec === 0) return;
  const elapsedSec = Math.floor((Date.now() - clientEpochBaseMs) / 1000);
  const currentSec = serverEpochSec + elapsedSec;

  const utcDate = new Date(currentSec * 1000);
  document.getElementById('utcClock').innerText =
    utcDate.toISOString().substring(11, 19) + ' UTC';

  const st = STATIONS[activeStationIdx] || STATIONS[0];
  const stationSec = currentSec + st.tzOffset + (userOffsetHours * 3600);
  const stationDate = new Date(stationSec * 1000);
  document.getElementById('localClock').innerText =
    stationDate.toISOString().substring(11, 19);
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
async function fetchOtaStatus() {
  try {
    const res = await fetch('/api/ota/status');
    if (!res.ok) throw new Error('OTA status fetch failed');
    const data = await res.json();
    document.getElementById('otaFirmwareVersion').innerText = data.firmware_version || '--';
    document.getElementById('otaRunningSlot').innerText = data.running_slot || '--';
    document.getElementById('otaNextSlot').innerText = data.next_update_slot || '--';
    if (data.default_url) document.getElementById('otaUrl').value = data.default_url;
  } catch (e) {
    console.warn('OTA status error:', e);
  }
}

async function triggerOtaFromUrl() {
  const url = document.getElementById('otaUrl').value.trim();
  if (!url) {
    alert('Please enter a firmware binary URL.');
    return;
  }
  if (!confirm(`Download and flash firmware from:\n${url}\n\nThe ESP32 will reboot automatically after completion.`)) {
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
  } catch (err) {
    btn.disabled = false;
    bar.style.display = 'none';
    alertBox.className = 'alert error';
    alertBox.innerText = '✗ OTA update failed: ' + err.message;
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

// --- Initial Startup ---
window.onload = () => {
  // Register Service Worker for PWA Standalone App support
  if ('serviceWorker' in navigator) {
    navigator.serviceWorker.register('/sw.js').catch((err) => {
      console.warn('[PWA] ServiceWorker registration failed:', err);
    });
  }

  fetchConfig();
  pollStatus();
  setInterval(pollStatus, 4000);
  setInterval(tickClock, 1000);
};
