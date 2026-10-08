#pragma once

#include <pgmspace.h>

const char INDEX_HTML[] PROGMEM = R"rawliteral(<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>TimeStation — ESP32 Radio Atomic Clock</title>
  <style>
    :root {
      --bg: #0f172a;
      --card-bg: #1e293b;
      --border: #334155;
      --accent: #38bdf8;
      --accent-hover: #0284c7;
      --text: #f8fafc;
      --text-muted: #94a3b8;
      --green: #10b981;
      --red: #ef4444;
      --amber: #f59e0b;
    }
    * { box-sizing: border-box; margin: 0; padding: 0; }
    body {
      background: var(--bg);
      color: var(--text);
      font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
      line-height: 1.5;
      padding: 16px;
      display: flex;
      justify-content: center;
    }
    .container {
      width: 100%;
      max-width: 600px;
      display: flex;
      flex-direction: column;
      gap: 16px;
    }
    .header {
      text-align: center;
      padding: 12px 0;
    }
    .header h1 {
      font-size: 24px;
      font-weight: 800;
      color: var(--accent);
      letter-spacing: -0.5px;
    }
    .header p {
      font-size: 13px;
      color: var(--text-muted);
    }
    .card {
      background: var(--card-bg);
      border: 1px solid var(--border);
      border-radius: 12px;
      padding: 20px;
      box-shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.3);
    }
    .card-title {
      font-size: 15px;
      font-weight: 700;
      color: var(--accent);
      margin-bottom: 14px;
      display: flex;
      align-items: center;
      justify-content: space-between;
    }
    .status-badge {
      display: inline-flex;
      align-items: center;
      gap: 6px;
      padding: 4px 10px;
      border-radius: 9999px;
      font-size: 12px;
      font-weight: 600;
      background: #064e3b;
      color: #34d399;
    }
    .status-badge.transmitting {
      background: #78350f;
      color: #fde047;
      animation: pulse 1.5s infinite;
    }
    .status-badge.idle {
      background: #334155;
      color: #cbd5e1;
    }
    @keyframes pulse {
      0%, 100% { opacity: 1; }
      50% { opacity: 0.6; }
    }
    .clock-display {
      font-size: 32px;
      font-weight: 800;
      font-family: monospace;
      color: #f8fafc;
      text-align: center;
      letter-spacing: 2px;
      margin: 8px 0;
    }
    .stat-row {
      display: flex;
      justify-content: space-between;
      font-size: 13px;
      padding: 6px 0;
      border-bottom: 1px solid rgba(255, 255, 255, 0.05);
    }
    .stat-row:last-child { border-bottom: none; }
    .stat-label { color: var(--text-muted); }
    .stat-value { font-weight: 600; }
    .form-group {
      margin-bottom: 14px;
    }
    label {
      display: block;
      font-size: 13px;
      font-weight: 600;
      color: var(--text-muted);
      margin-bottom: 6px;
    }
    select, input[type="text"], input[type="password"], input[type="time"], input[type="number"] {
      width: 100%;
      padding: 10px 12px;
      border-radius: 8px;
      border: 1px solid var(--border);
      background: #0f172a;
      color: var(--text);
      font-size: 14px;
      outline: none;
      transition: border-color 0.2s;
    }
    select:focus, input:focus {
      border-color: var(--accent);
    }
    .mode-toggle {
      display: flex;
      gap: 6px;
      margin-bottom: 14px;
      background: #0f172a;
      padding: 4px;
      border-radius: 8px;
      border: 1px solid var(--border);
    }
    .mode-btn {
      flex: 1;
      padding: 8px 10px;
      font-size: 13px;
      font-weight: 700;
      color: var(--text-muted);
      border: none;
      background: transparent;
      border-radius: 6px;
      cursor: pointer;
      text-align: center;
      transition: all 0.2s;
    }
    .mode-btn.active {
      background: var(--accent);
      color: #0f172a;
    }
    .carousel-checklist {
      display: flex;
      flex-direction: column;
      gap: 6px;
    }
    .carousel-item {
      display: flex;
      align-items: center;
      gap: 10px;
      padding: 8px 12px;
      background: #0f172a;
      border: 1px solid var(--border);
      border-radius: 8px;
      cursor: pointer;
      transition: border-color 0.2s;
    }
    .carousel-item:hover {
      border-color: var(--accent);
    }
    .carousel-item input {
      width: 16px;
      height: 16px;
      accent-color: var(--accent);
    }
    .carousel-item span {
      font-size: 13px;
      color: var(--text);
      font-weight: 500;
    }
    .carousel-calc {
      font-size: 12px;
      font-weight: 600;
      color: #38bdf8;
      background: rgba(56, 189, 248, 0.1);
      border: 1px solid rgba(56, 189, 248, 0.25);
      padding: 8px 12px;
      border-radius: 8px;
      margin-top: 10px;
    }
    .btn {
      display: inline-block;
      width: 100%;
      padding: 12px;
      border-radius: 8px;
      border: none;
      font-size: 14px;
      font-weight: 700;
      cursor: pointer;
      text-align: center;
      transition: background-color 0.2s;
    }
    .btn-primary {
      background: var(--accent);
      color: #0f172a;
    }
    .btn-primary:hover { background: var(--accent-hover); }
    .btn-transmit {
      background: var(--green);
      color: #0f172a;
      margin-top: 8px;
    }
    .btn-stop {
      background: var(--red);
      color: #f8fafc;
      margin-top: 8px;
    }
    .checkbox-group {
      display: flex;
      align-items: center;
      gap: 10px;
      margin: 10px 0;
    }
    .checkbox-group input {
      width: 18px;
      height: 18px;
      accent-color: var(--accent);
    }
    .checkbox-group label {
      margin-bottom: 0;
      color: var(--text);
      cursor: pointer;
    }
    .alert {
      padding: 10px 14px;
      border-radius: 8px;
      font-size: 13px;
      margin-top: 10px;
      display: none;
    }
    .alert-success { background: #064e3b; color: #6ee7b7; border: 1px solid #059669; }
    .alert-error { background: #7f1d1d; color: #fca5a5; border: 1px solid #b91c1c; }
    .info-btn {
      display: inline-flex;
      align-items: center;
      justify-content: center;
      width: 17px;
      height: 17px;
      border-radius: 50%;
      background: rgba(56, 189, 248, 0.15);
      color: var(--accent);
      font-size: 11px;
      font-weight: 700;
      font-family: serif;
      font-style: italic;
      border: 1px solid rgba(56, 189, 248, 0.4);
      cursor: pointer;
      margin-left: 6px;
      vertical-align: middle;
      padding: 0;
      line-height: 1;
      transition: all 0.2s ease;
      user-select: none;
      -webkit-tap-highlight-color: transparent;
    }
    .info-btn:hover, .info-btn:focus-visible, .info-btn.active {
      background: var(--accent);
      color: #0f172a;
      border-color: var(--accent);
      outline: none;
      box-shadow: 0 0 8px rgba(56, 189, 248, 0.4);
    }
    .popover-box {
      position: fixed;
      z-index: 9999;
      max-width: 320px;
      width: calc(100vw - 32px);
      background: #1e293b;
      border: 1px solid #475569;
      border-radius: 10px;
      padding: 12px 14px;
      box-shadow: 0 10px 25px -5px rgba(0, 0, 0, 0.6), 0 0 0 1px rgba(255, 255, 255, 0.05);
      font-size: 12px;
      line-height: 1.45;
      color: #e2e8f0;
      pointer-events: auto;
      opacity: 0;
      transform: translateY(4px);
      transition: opacity 0.15s ease, transform 0.15s ease;
      visibility: hidden;
    }
    .popover-box.visible {
      opacity: 1;
      transform: translateY(0);
      visibility: visible;
    }
    .popover-title {
      display: flex;
      align-items: center;
      justify-content: space-between;
      font-weight: 700;
      font-size: 13px;
      color: var(--accent);
      margin-bottom: 6px;
      border-bottom: 1px solid rgba(255, 255, 255, 0.08);
      padding-bottom: 4px;
    }
    .popover-close {
      background: none;
      border: none;
      color: var(--text-muted);
      font-size: 18px;
      line-height: 1;
      cursor: pointer;
      padding: 0 4px;
    }
    .popover-close:hover { color: var(--text); }
    .popover-content p {
      margin-bottom: 6px;
    }
    .popover-content p:last-child {
      margin-bottom: 0;
    }
    .popover-warn {
      color: #fde047;
      background: rgba(245, 158, 11, 0.12);
      border: 1px solid rgba(245, 158, 11, 0.35);
      border-radius: 6px;
      padding: 6px 8px;
      margin-top: 6px;
      font-size: 11.5px;
      line-height: 1.4;
    }
    .offset-warning-banner {
      display: block;
      margin-top: 8px;
      padding: 8px 12px;
      border-radius: 8px;
      font-size: 12px;
      line-height: 1.4;
      border: 1px solid;
    }
    .offset-warning-banner.warn {
      background: rgba(245, 158, 11, 0.1);
      border-color: rgba(245, 158, 11, 0.4);
      color: #fde047;
    }
    .offset-warning-banner.neutral {
      background: rgba(16, 185, 129, 0.1);
      border-color: rgba(16, 185, 129, 0.3);
      color: #6ee7b7;
    }
  </style>
</head>
<body>
  <div class="container">
    <div class="header">
      <h1>📡 TimeStation</h1>
      <p>ESP32 Worldwide Radio Atomic Clock Simulator</p>
    </div>

    <!-- Live Status Card -->
    <div class="card">
      <div class="card-title">
        <span>CURRENT STATUS</span>
        <span id="statusBadge" class="status-badge idle">● IDLE</span>
      </div>
      <div id="liveClock" class="clock-display">--:--:--</div>
      <div class="stat-row">
        <span class="stat-label">Operating Mode <button type="button" class="info-btn" data-title="Operating Mode" data-content="<p><strong>Single Station:</strong> Broadcasts continuously on one designated frequency.</p><p><strong>Multi-Station Carousel:</strong> Cycles through selected stations sequentially at clean minute marks to automatically calibrate watches from different worldwide markets.</p>">i</button></span>
        <span id="modeDisplay" class="stat-value">Single Station</span>
      </div>
      <div class="stat-row">
        <span class="stat-label">Active Station</span>
        <span id="activeStation" class="stat-value">BPC (China 68.5 kHz)</span>
      </div>
      <div class="stat-row">
        <span class="stat-label">Carrier Frequency</span>
        <span id="carrierFreq" class="stat-value">68,500 Hz</span>
      </div>
      <div id="stageRow" class="stat-row" style="display: none;">
        <span class="stat-label">Carousel Stage</span>
        <span id="stageDisplay" class="stat-value">Stage 1 of 3</span>
      </div>
      <div id="countdownRow" class="stat-row" style="display: none;">
        <span class="stat-label">Time Remaining</span>
        <span id="countdownDisplay" class="stat-value">--:--</span>
      </div>
      <div class="stat-row">
        <span class="stat-label">Next Scheduled Sync</span>
        <span id="nextSync" class="stat-value">02:00 AM (Daily)</span>
      </div>
      <div class="stat-row">
        <span class="stat-label">Antenna Pin <button type="button" class="info-btn" id="antennaPinInfoBtn" data-title="Antenna Pin & Driver" data-content="<p>GPIO 2 outputs the LF square-wave carrier generated by the ESP32 hardware LEDC PWM timer.</p><p>Drives the C1815 NPN transistor through a 1k&Omega; base resistor to resonate the 16&times;18 mm 3.5 mH I-type inductor tank circuit.</p>">i</button></span>
        <span id="antennaPin" class="stat-value">GPIO 2 (LEDC PWM)</span>
      </div>

      <button id="btnTransmitNow" class="btn btn-transmit" onclick="toggleTransmit()">⚡ Broadcast Now (Test Mode)</button>
    </div>

    <!-- Configuration Form -->
    <form id="configForm" class="card" onsubmit="saveConfig(event)">
      <div class="card-title">
        <span>TRANSMISSION MODE</span>
        <button type="button" class="info-btn" data-title="Transmission Mode Guide" data-content="<p>Choose <strong>Single Station</strong> for a dedicated watch type, or <strong>Carousel</strong> if you own multiple MultiBand 6 watches from different markets (e.g. US, Japan, Europe).</p>">i</button>
      </div>
      <div class="mode-toggle">
        <button type="button" id="btnModeSingle" class="mode-btn active" onclick="setMode(false)">Single Station</button>
        <button type="button" id="btnModeCarousel" class="mode-btn" onclick="setMode(true)">🔄 Multi-Station Carousel</button>
      </div>

      <!-- Single Station Section -->
      <div id="singleStationSection">
        <div class="form-group">
          <label for="station">Time Signal Station <button type="button" class="info-btn" data-title="Atomic Time Stations" data-content="<p><strong>BPC (68.5 kHz):</strong> China (Casio HKG, BJS)</p><p><strong>WWVB (60.0 kHz):</strong> North America (NYC, CHI, DEN, LAX)</p><p><strong>MSF (60.0 kHz):</strong> United Kingdom (LON)</p><p><strong>DCF77 (77.5 kHz):</strong> Europe (BER, PAR, ROM, ATH)</p><p><strong>JJY (40/60 kHz):</strong> Japan East / West (TYO)</p>">i</button></label>
          <select id="station" name="station">
            <option value="0">🇨🇳 BPC — China (68.5 kHz)</option>
            <option value="1">🇺🇸 WWVB — USA (60.0 kHz)</option>
            <option value="2">🇬🇧 MSF — United Kingdom (60.0 kHz)</option>
            <option value="3">🇩🇪 DCF77 — Germany (77.5 kHz)</option>
            <option value="4">🇯🇵 JJY40 — Japan East (40.0 kHz)</option>
            <option value="5">🇯🇵 JJY60 — Japan West (60.0 kHz)</option>
          </select>
        </div>

        <div class="form-group">
          <label for="broadcastDuration">Broadcast Duration (Minutes) <button type="button" class="info-btn" data-title="Broadcast Duration" data-content="<p>Casio MultiBand 6 watches generally require 2 to 14 minutes of uninterrupted carrier to correlate the timecode frames.</p><p><strong>Recommendation:</strong> 20&ndash;25 minutes provides ample time for multiple sync retries.</p>">i</button></label>
          <input type="number" id="broadcastDuration" name="broadcastDuration" value="25" min="5" max="120">
        </div>
      </div>

      <!-- Carousel Mode Section -->
      <div id="carouselSection" style="display: none;">
        <div class="form-group">
          <label>Carousel Sequence (Select Stations) <button type="button" class="info-btn" data-title="Carousel Sequence" data-content="<p>The ESP32 rotates through each checked station in order, retuning the carrier frequency and modulation timing at exact :00 minute boundaries.</p>">i</button></label>
          <div class="carousel-checklist">
            <label class="carousel-item">
              <input type="checkbox" id="car_0" value="0" onchange="updateCarouselCalc()">
              <span>🇨🇳 BPC — China (68.5 kHz)</span>
            </label>
            <label class="carousel-item">
              <input type="checkbox" id="car_5" value="5" onchange="updateCarouselCalc()">
              <span>🇯🇵 JJY60 — Japan West (60.0 kHz)</span>
            </label>
            <label class="carousel-item">
              <input type="checkbox" id="car_1" value="1" onchange="updateCarouselCalc()">
              <span>🇺🇸 WWVB — USA (60.0 kHz)</span>
            </label>
            <label class="carousel-item">
              <input type="checkbox" id="car_4" value="4" onchange="updateCarouselCalc()">
              <span>🇯🇵 JJY40 — Japan East (40.0 kHz)</span>
            </label>
            <label class="carousel-item">
              <input type="checkbox" id="car_2" value="2" onchange="updateCarouselCalc()">
              <span>🇬🇧 MSF — United Kingdom (60.0 kHz)</span>
            </label>
            <label class="carousel-item">
              <input type="checkbox" id="car_3" value="3" onchange="updateCarouselCalc()">
              <span>🇩🇪 DCF77 — Germany (77.5 kHz)</span>
            </label>
          </div>
        </div>

        <div class="form-group">
          <label for="carouselDurationMin">Duration Per Station (Minutes) <button type="button" class="info-btn" data-title="Stage Duration" data-content="<p>Time spent broadcasting each station before switching. 15 minutes is recommended to allow watches adequate lock time.</p>">i</button></label>
          <input type="number" id="carouselDurationMin" name="carouselDurationMin" value="15" min="5" max="60" oninput="updateCarouselCalc()">
        </div>

        <div id="carouselCalcNotice" class="carousel-calc">
          ⏱ Total Carousel Duration: 45 minutes (3 stations × 15 min)
        </div>
      </div>

      <div class="card-title" style="margin-top: 20px;">SCHEDULE & TIME SETTINGS</div>

      <div class="form-group">
        <label for="offsetHours">Custom Time Offset (Hours) <button type="button" class="info-btn" data-title="Timezone Offset & World Time Limits" data-content="<p>Adjusts the transmitted time by &plusmn;N hours relative to true UTC/NTP time.</p><p><strong>Best for:</strong> Simple atomic wall/desk clocks with fixed transmitter tuners and no local timezone setting.</p><div class='popover-warn'><strong>⚠️ World Time Clocks Warning:</strong> MultiBand 6 watches calculate internal UTC as: <em>(Received Time &minus; Home City Offset)</em>. Shifting the broadcast moves the watch's internal UTC baseline, causing <strong>all World Time cities on your watch to be offset by the same amount</strong>.<br><br><em>Tip:</em> For Casio watches, keep offset at 0, set World Time to your local city, and use the watch's built-in Home/World Time quick-swap shortcut.</div>">i</button></label>
        <input type="number" id="offsetHours" name="offsetHours" value="0" step="1" min="-12" max="14" oninput="updateOffsetNotice()">
        <div id="offsetAlert" class="offset-warning-banner neutral">
          ✓ <strong>Native Time (0h Offset):</strong> Broadcasts authentic station time. Recommended for MultiBand 6 watches to maintain accurate World Time.
        </div>
      </div>

      <div class="checkbox-group">
        <input type="checkbox" id="scheduleEnabled" name="scheduleEnabled" checked>
        <label for="scheduleEnabled">Enable Daily Scheduled Broadcast <button type="button" class="info-btn" data-title="Overnight Calibration" data-content="<p>Casio MultiBand 6 watches automatically attempt radio calibration starting at midnight, 1:00 AM, 2:00 AM, 3:00 AM, 4:00 AM, and 5:00 AM until a successful sync occurs.</p><p>02:00 AM is the standard auto-receive window.</p>">i</button></label>
      </div>

      <div class="form-group">
        <label for="broadcastTime">Daily Broadcast Start Time (Local)</label>
        <input type="time" id="broadcastTime" name="broadcastTime" value="02:00">
      </div>

      <div class="card-title" style="margin-top: 20px;">NETWORK SETTINGS</div>

      <div class="form-group">
        <label for="wifiSsid">Wi-Fi SSID</label>
        <input type="text" id="wifiSsid" name="wifiSsid" placeholder="Your Wi-Fi Network Name">
      </div>

      <div class="form-group">
        <label for="wifiPassword">Wi-Fi Password</label>
        <input type="password" id="wifiPassword" name="wifiPassword" placeholder="••••••••">
      </div>

      <div class="form-group">
        <label for="ntpServer">NTP Time Server <button type="button" class="info-btn" data-title="NTP Atomic Precision" data-content="<p>The Network Time Protocol server used by the ESP32 to discipline its internal RTC. The default <code>pool.ntp.org</code> automatically routes to geographically close Stratum-1/Stratum-2 atomic reference servers.</p>">i</button></label>
        <input type="text" id="ntpServer" name="ntpServer" value="pool.ntp.org">
      </div>

      <button type="submit" class="btn btn-primary">Save Settings</button>
      <div id="alertBox" class="alert"></div>
    </form>
  </div>

  <!-- Global Info Popover -->
  <div id="globalPopover" class="popover-box" role="tooltip" aria-hidden="true">
    <div class="popover-title">
      <span id="popTitle">Information</span>
      <button type="button" class="popover-close" onclick="hideInfo(true)" aria-label="Close">&times;</button>
    </div>
    <div id="popBody" class="popover-content"></div>
  </div>

  <script>
    let isTransmitting = false;
    let carouselMode = false;

    function setMode(isCarousel) {
      carouselMode = isCarousel;
      document.getElementById('btnModeSingle').classList.toggle('active', !isCarousel);
      document.getElementById('btnModeCarousel').classList.toggle('active', isCarousel);
      document.getElementById('singleStationSection').style.display = isCarousel ? 'none' : 'block';
      document.getElementById('carouselSection').style.display = isCarousel ? 'block' : 'none';
      updateCarouselCalc();
    }

    function getCarouselMask() {
      let mask = 0;
      for (let i = 0; i < 6; i++) {
        const cb = document.getElementById('car_' + i);
        if (cb && cb.checked) mask |= (1 << i);
      }
      return mask;
    }

    function setCarouselMask(mask) {
      for (let i = 0; i < 6; i++) {
        const cb = document.getElementById('car_' + i);
        if (cb) cb.checked = (mask & (1 << i)) !== 0;
      }
      updateCarouselCalc();
    }

    function updateCarouselCalc() {
      let count = 0;
      for (let i = 0; i < 6; i++) {
        const cb = document.getElementById('car_' + i);
        if (cb && cb.checked) count++;
      }
      const dur = parseInt(document.getElementById('carouselDurationMin').value) || 15;
      const total = count * dur;
      const notice = document.getElementById('carouselCalcNotice');
      if (notice) {
        if (count === 0) {
          notice.innerText = '⚠️ Please select at least one station for the carousel.';
          notice.style.borderColor = '#ef4444';
          notice.style.color = '#fca5a5';
        } else {
          notice.innerText = `⏱ Total Carousel Duration: ${total} minutes (${count} stations × ${dur} min)`;
          notice.style.borderColor = 'rgba(56, 189, 248, 0.25)';
          notice.style.color = '#38bdf8';
        }
      }
    }

    function formatSec(s) {
      if (!s || s <= 0) return '0:00';
      const m = Math.floor(s / 60);
      const sec = s % 60;
      return `${m}:${sec < 10 ? '0' : ''}${sec}`;
    }

    async function loadStatus() {
      try {
        const res = await fetch('/api/status');
        const data = await res.json();
        document.getElementById('liveClock').innerText = data.time || '--:--:--';
        document.getElementById('activeStation').innerText = data.stationName;
        document.getElementById('carrierFreq').innerText = (data.carrierHz || 0).toLocaleString() + ' Hz';
        document.getElementById('nextSync').innerText = data.nextSync || 'Disabled';
        if (data.antennaPin !== undefined) {
          document.getElementById('antennaPin').innerText = 'GPIO ' + data.antennaPin + ' (LEDC PWM)';
          const pinBtn = document.getElementById('antennaPinInfoBtn');
          if (pinBtn) {
            pinBtn.dataset.content = `<p>GPIO ${data.antennaPin} outputs the LF square-wave carrier generated by the ESP32 hardware LEDC PWM timer.</p><p>Drives the C1815 NPN transistor through a 1k&Omega; base resistor to resonate the 16&times;18 mm 3.5 mH I-type inductor tank circuit.</p>`;
          }
        }

        // Mode & Stage display
        const isCar = data.carouselEnabled;
        document.getElementById('modeDisplay').innerText = isCar ? '🔄 Multi-Station Carousel' : 'Single Station';

        const stageRow = document.getElementById('stageRow');
        const countRow = document.getElementById('countdownRow');
        isTransmitting = data.transmitting;

        if (isTransmitting) {
          if (isCar && data.carouselTotalStages > 1) {
            stageRow.style.display = 'flex';
            document.getElementById('stageDisplay').innerText = `Stage ${data.carouselStage} of ${data.carouselTotalStages} (${data.stationName})`;
          } else {
            stageRow.style.display = 'none';
          }

          countRow.style.display = 'flex';
          if (isCar && data.carouselTotalStages > 1) {
            document.getElementById('countdownDisplay').innerText = `Stage: ${formatSec(data.stageRemainSec)} • Total: ${formatSec(data.totalRemainSec)}`;
          } else {
            document.getElementById('countdownDisplay').innerText = `${formatSec(data.totalRemainSec)} remaining`;
          }
        } else {
          stageRow.style.display = 'none';
          countRow.style.display = 'none';
        }
        
        const badge = document.getElementById('statusBadge');
        const btn = document.getElementById('btnTransmitNow');
        if (isTransmitting) {
          badge.className = 'status-badge transmitting';
          badge.innerText = isCar ? `⚡ CAROUSEL (${data.carouselStage}/${data.carouselTotalStages})` : '⚡ BROADCASTING';
          btn.className = 'btn btn-stop';
          btn.innerText = '⏹ Stop Broadcast';
        } else {
          badge.className = 'status-badge idle';
          badge.innerText = '● IDLE';
          btn.className = 'btn btn-transmit';
          btn.innerText = '⚡ Broadcast Now (Test Mode)';
        }

        // Populate form fields on initial load
        if (!document.getElementById('station').dataset.loaded) {
          document.getElementById('station').value = data.station;
          document.getElementById('offsetHours').value = data.offsetHours;
          updateOffsetNotice();
          document.getElementById('scheduleEnabled').checked = data.scheduleEnabled;
          document.getElementById('broadcastTime').value = data.broadcastTime;
          document.getElementById('broadcastDuration').value = data.broadcastDuration;
          document.getElementById('wifiSsid').value = data.wifiSsid || '';
          document.getElementById('ntpServer').value = data.ntpServer || 'pool.ntp.org';

          if (data.carouselDurationMin) {
            document.getElementById('carouselDurationMin').value = data.carouselDurationMin;
          }
          if (data.carouselMask !== undefined) {
            setCarouselMask(data.carouselMask);
          }
          setMode(!!data.carouselEnabled);

          document.getElementById('station').dataset.loaded = 'true';
        }
      } catch (e) {
        console.log('Fetching status...');
      }
    }

    async function toggleTransmit() {
      const endpoint = isTransmitting ? '/api/stop' : '/api/transmit';
      await fetch(endpoint, { method: 'POST' });
      loadStatus();
    }

    async function saveConfig(e) {
      e.preventDefault();
      const form = document.getElementById('configForm');
      const formData = new FormData(form);
      const data = {
        station: parseInt(formData.get('station')),
        offsetHours: parseInt(formData.get('offsetHours')),
        scheduleEnabled: document.getElementById('scheduleEnabled').checked,
        broadcastTime: formData.get('broadcastTime'),
        broadcastDuration: parseInt(formData.get('broadcastDuration')),
        carouselEnabled: carouselMode,
        carouselMask: getCarouselMask(),
        carouselDurationMin: parseInt(document.getElementById('carouselDurationMin').value) || 15,
        wifiSsid: formData.get('wifiSsid'),
        wifiPassword: formData.get('wifiPassword'),
        ntpServer: formData.get('ntpServer')
      };

      const alertBox = document.getElementById('alertBox');
      try {
        const res = await fetch('/api/save', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify(data)
        });
        alertBox.className = 'alert alert-success';
        alertBox.innerText = 'Settings saved successfully to flash memory!';
        alertBox.style.display = 'block';
        setTimeout(() => alertBox.style.display = 'none', 4000);
        loadStatus();
      } catch (err) {
        alertBox.className = 'alert alert-error';
        alertBox.innerText = 'Error saving settings: ' + err;
        alertBox.style.display = 'block';
      }
    }

    function updateOffsetNotice() {
      const input = document.getElementById('offsetHours');
      const alert = document.getElementById('offsetAlert');
      if (!input || !alert) return;
      const val = parseInt(input.value) || 0;
      if (val === 0) {
        alert.className = 'offset-warning-banner neutral';
        alert.innerHTML = '✓ <strong>Native Time (0h Offset):</strong> Broadcasts authentic station time. Recommended for MultiBand 6 watches to maintain accurate World Time.';
      } else {
        alert.className = 'offset-warning-banner warn';
        const sign = val > 0 ? '+' : '';
        alert.innerHTML = `⚠️ <strong>Timezone Shift (${sign}${val}h):</strong> Adjusts watch display, but shifts its internal UTC reference. <strong>All World Time cities on MultiBand 6 watches will also be shifted by ${sign}${val} hours.</strong>`;
      }
    }

    // Interactive Info Popover Engine (Mouse hover + Touch/Click toggle)
    let activeInfoBtn = null;
    let isPinned = false;
    let hideTimer = null;

    const popover = document.getElementById('globalPopover');
    const popTitle = document.getElementById('popTitle');
    const popBody = document.getElementById('popBody');

    function showInfo(btn, pinned = false) {
      clearTimeout(hideTimer);
      activeInfoBtn = btn;
      isPinned = pinned;

      popTitle.innerText = btn.dataset.title || 'Information';
      popBody.innerHTML = btn.dataset.content || '';

      const rect = btn.getBoundingClientRect();
      const popWidth = Math.min(320, window.innerWidth - 32);
      popover.style.width = popWidth + 'px';

      let left = rect.left + rect.width / 2 - popWidth / 2;
      left = Math.max(16, Math.min(left, window.innerWidth - popWidth - 16));

      popover.classList.add('visible');
      const popHeight = popover.offsetHeight;

      let top = rect.bottom + 8;
      if (top + popHeight > window.innerHeight - 16) {
        top = rect.top - popHeight - 8;
      }
      top = Math.max(12, top);

      popover.style.left = left + 'px';
      popover.style.top = top + 'px';

      document.querySelectorAll('.info-btn').forEach(b => b.classList.remove('active'));
      btn.classList.add('active');
    }

    function hideInfo(force = false) {
      if (isPinned && !force) return;
      hideTimer = setTimeout(() => {
        popover.classList.remove('visible');
        if (activeInfoBtn) activeInfoBtn.classList.remove('active');
        activeInfoBtn = null;
        isPinned = false;
      }, 120);
    }

    document.querySelectorAll('.info-btn').forEach(btn => {
      btn.addEventListener('mouseenter', () => {
        if (!isPinned) showInfo(btn, false);
      });
      btn.addEventListener('mouseleave', () => {
        if (!isPinned) hideInfo(false);
      });
      btn.addEventListener('click', (e) => {
        e.preventDefault();
        e.stopPropagation();
        if (activeInfoBtn === btn && isPinned) {
          hideInfo(true);
        } else {
          showInfo(btn, true);
        }
      });
    });

    popover.addEventListener('mouseenter', () => clearTimeout(hideTimer));
    popover.addEventListener('mouseleave', () => { if (!isPinned) hideInfo(false); });

    document.addEventListener('click', (e) => {
      if (!popover.contains(e.target) && !e.target.closest('.info-btn')) {
        hideInfo(true);
      }
    });

    document.addEventListener('keydown', (e) => {
      if (e.key === 'Escape') hideInfo(true);
    });

    setInterval(loadStatus, 1000);
    loadStatus();
    updateOffsetNotice();
  </script>
</body>
</html>
)rawliteral";
