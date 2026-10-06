# MultiBand6-ESP32-AtomicBeacon

An open-hardware, low-power near-field radio transmitter built on the **ESP32-C3-Mini** to simulate LF time signal broadcasts (**BPC, WWVB, MSF, DCF77, JJY40, JJY60**) for synchronizing radio-controlled ("atomic") clocks and wristwatches over a localized range (approx. 0.5 m to 1.0 m).

---

## 1. Bill of Materials (BOM) & Components

| Component | Value / Specification | Role in Circuit | Rationale / Notes |
| :--- | :--- | :--- | :--- |
| **Microcontroller** | **ESP32-C3-Mini** | Core Controller & Carrier Generator | Single-core 32-bit RISC-V (160 MHz), hardware LEDC PWM with fractional clock divider, RTC timer, Wi-Fi 4, BLE 5. |
| **Inductor ($L$)** | **3.5 mH (16×18 mm)** | Magnetic Antenna ($H$-field radiator) | "I-Type" drum/bobbin core power inductor. Open magnetic circuit allows magnetic dipole flux lines to radiate into the room. Low DC resistance. |
| **Tuning Capacitor ($C$)** | **1.5 nF (1500 pF / code `152`)** | Parallel LC Tank Resonator | Metallized Film (CBB / Polypropylene / Polyester) or C0G/NP0 ceramic. Rated $\ge 50\text{V}$. |
| **Damping Resistor ($R_{damp}$)** | **$220\,\Omega$ (1/4 W)** | Bandwidth Broadening & Safety Current Limiter | Lowers tank $Q$ to $\sim 6.5$, broadening bandwidth to $\sim 10\text{ kHz}$. Limits peak GPIO current to $\sim 15\text{ mA}$ (well below the ESP32-C3 20 mA pin limit). |
| **DC Blocking Cap ($C_{block}$)** | **$1\,\mu\text{F}$ (or $100\text{ nF}$)** | DC Isolation & GPIO Protection | Multi-layer ceramic (MLCC) or film capacitor in series with GPIO. Completely blocks DC current so pin cannot burn out if stuck `HIGH`. |
| *(Optional Driver)* | **2N2222 NPN** or **L9110S H-Bridge** | Extended Range Booster | Optional booster to drive the tank at 5V / 10V peak-to-peak for $> 1.0\text{ m}$ range. |

---

## 2. Hardware Circuit Schematics

### Direct GPIO Drive (Minimalist & Protected for Bench / Desk Use)

This configuration connects directly to an ESP32-C3 GPIO pin with built-in safety against DC overcurrent and back-EMF spikes:

```
                          C_block       R_damp
                         [ 1 uF ]     [ 220 ohm ]
   ESP32-C3 GPIO ---------[ || ]--------[ \/\/\ ]---------+-----------------+
   (LEDC PWM Out)                                         |                 |
                                                       +------+          +------+
                                                       |      |          |      |
                                                    3.5 mH  Inductor  1.5 nF Capacitor
                                                       |      |          |      |
                                                       +------+          +------+
                                                          |                 |
   ESP32-C3 GND ------------------------------------------+-----------------+
```

#### Circuit Analysis & Parameters
* **Nominal Resonance:**
  $$f_0 = \frac{1}{2\pi \sqrt{L \cdot C}} = \frac{1}{2\pi \sqrt{3.5\times 10^{-3} \cdot 1.5\times 10^{-9}}} \approx 69.46\text{ kHz}$$
* **Characteristic Impedance ($Z_0$):**
  $$Z_0 = \sqrt{\frac{L}{C}} = \sqrt{\frac{3.5\times 10^{-3}}{1.5\times 10^{-9}}} \approx 1527\,\Omega$$
* **Effective Quality Factor ($Q$ with $R_{damp} = 220\,\Omega$):**
  $$Q \approx \frac{Z_0}{R_{total}} = \frac{1527\,\Omega}{220\,\Omega + R_{coil}} \approx \mathbf{6.5}$$
* **Passband Bandwidth ($\Delta f$):**
  $$\Delta f = \frac{f_0}{Q} \approx \frac{69.5\text{ kHz}}{6.5} \approx \mathbf{10.7\text{ kHz}}\quad (64\text{ kHz} \text{ to } 75\text{ kHz})$$
  * Because the passband is over $10\text{ kHz}$ wide, **BPC at 68.5 kHz sits comfortably near peak resonance**, fully absorbing the $\pm 10\%$ manufacturing tolerances of the coil and capacitor without requiring precision trimming.
* **Pin Current:**
  $$I_{peak} \approx \frac{3.3\text{ V}}{220\,\Omega} \approx \mathbf{15\text{ mA}} \quad (\text{Safe: below ESP32-C3 20 mA maximum})$$


### NPN Transistor Booster Circuit (Recommended for 1.0 m+ Range)

This configuration uses any generic small-signal NPN transistor (2N3904, 2N2222, BC547, S8050, 2SC1815, etc.) powered directly from the **5V USB rail**. It isolates the ESP32 silicon and delivers $3\times$ to $5\times$ more magnetic flux for extended range:

```
                                  +5V (USB / VIN Pin)
                                    │
                                    ├───[ 220 ohm ]─── (R_damp)
                                    │
                                    ├───┬───────────────┬───┐
                                    │   │               │   │
                                    │  [ 3.5 mH ]   [ 1.5 nF ]
                                    │  (Inductor)   (Capacitor)
                                    │   │               │   │
                                    └───┴───────────────┴───┘
                                                │
                                                ▼
                                         [ C ] (Collector)
  ESP32-C3 GPIO ───[ 1k ohm ]─────────── [ B ]   Generic NPN Transistor
  (LEDC PWM)       (R_base)              [ E ] (Emitter)
                                                │
                                                ▼
  ESP32-C3 GND ─────────────────────────────────┴─── Common Ground (GND)
```

#### Why This Works Well:
1. **Zero Stress on ESP32:** The GPIO pin only provides a safe $I_B \approx 2.6\text{ mA}$ through the $1\text{ k}\Omega$ base resistor.
2. **5V Swing:** The LC tank swings around the 5V rail rather than 3.3V, pushing significantly more magnetic current through the 3.5 mH inductor.
3. **Uses Your $220\,\Omega$ Resistor:** The $220\,\Omega$ damping resistor limits peak collector current to a cool, safe $\sim 23\text{ mA}$ while maintaining the same wide $\sim 10\text{ kHz}$ bandwidth.
4. **Broadcast Range:** Up to **1.0 m – 1.2 m**.


### Breadboard Wiring Layout (NPN Booster Circuit)

> 🎨 **Visual Diagram:** A full vector graphic illustration is available in [breadboard_layout.svg](file:///Users/robertlongbottom/dev/esp32_radio_atomic_clock/breadboard_layout.svg) or viewable in your browser via [breadboard_layout.html](file:///Users/robertlongbottom/dev/esp32_radio_atomic_clock/breadboard_layout.html).

Here is the top-down terminal row mapping for a standard breadboard:

```
  (+) POWER RAIL  [+5V from ESP32 VIN/5V] ─── (Red Line)
  (-) GROUND RAIL [GND from ESP32]         ─── (Blue Line)

   Row    [A B C D E]  (Center Ravine)  [F G H I J]
  ────────────────────────────────────────────────────────────────────────
   08:    [ . . . . . ]                  [ . . . . . ] ── Wire to ESP32 GPIO 2
                                               │
                                         [ 1k R_base ]
                                               │
   11:    [ . . . . . ]                  [ . . . . . ] ── Transistor BASE (B)
   10:    [ . . . . . ]                  [ . . . . . ] ── Transistor EMITTER (E) ── Wire to (-) GND
   12:    [ . . . . . ]                  [ . . . . . ] ── Transistor COLLECTOR (C)
                                               │
                                       ┌───────┴───────┐
                                       │ 3.5mH Inductor│  (Parallel LC Tank)
                                       │ 1.5nF Cap     │
                                       └───────┬───────┘
                                               │
   15:    [ . . . . . ]                  [ . . . . . ]
                                               │
                                         [ 220 ohm ]   ── R_damp connects directly
                                               │          from Row 15 to (+) 5V Rail
  ────────────────────────────────────────────────────────────────────────
```

#### Step-by-Step Breadboard Connections:
1. **Power Rails:**
   * Wire ESP32 **5V** (or `VIN`) to the breadboard **(+) Red Rail**.
   * Wire ESP32 **GND** to the breadboard **(-) Blue Rail**.
2. **NPN Transistor:**
   * Plug into Rows 10, 11, and 12 (assuming standard E-B-C pinout like 2N3904 / 2N2222):
     * **Row 10 = Emitter (E)** $\rightarrow$ Add a short jumper wire from Row 10 to **(-) Blue GND Rail**.
     * **Row 11 = Base (B)** $\rightarrow$ Plug one leg of the **$1\text{ k}\Omega$ resistor** here.
     * **Row 12 = Collector (C)** $\rightarrow$ This is the bottom of the LC tank.
3. **Base Drive:**
   * Plug the other leg of the **$1\text{ k}\Omega$ resistor** into **Row 8**.
   * Run a jumper wire from **Row 8** to **ESP32 GPIO 2** (LEDC PWM output).
4. **LC Tank (Inductor + Capacitor in Parallel):**
   * Plug the **3.5 mH Inductor** across **Row 12** and **Row 15**.
   * Plug the **1.5 nF Capacitor** across the exact same rows: **Row 12** and **Row 15**.
5. **Damping Resistor:**
   * Plug one leg of the **$220\,\Omega$ resistor** into **Row 15**.
   * Plug the other leg directly into the **(+) Red 5V Rail**.

---



## 3. Physical Antenna Alignment & Orientation

* The **16×18 mm I-type inductor** radiates a magnetic dipole field oriented along its **vertical cylindrical axis** (top and bottom flat faces).
* Most atomic clocks and watches contain a **horizontal ferrite rod** inside the case.
* **Optimal Placement:** Position the inductor so that the magnetic flux lines looping out of the inductor's ends pass directly through the clock's internal antenna bar.
* **Broadcast Range:** 
  * Direct GPIO with $220\,\Omega$: Approx. **0.5 m to 0.7 m** (ideal for bedside table or desk).

---

## 4. Supported Stations & Broadcast Signals

| Station | Location | Frequency | Modulation Protocol | Frame Duration |
| :--- | :--- | :---: | :--- | :---: |
| **BPC** | China (Shangqiu) | **68.5 kHz** | 4-state pulse width keying (100–400 ms) | 20 seconds |
| **WWVB** | USA (Colorado) | **60.0 kHz** | AM pulse width keying (0.2s = 0, 0.5s = 1, 0.8s = marker) | 60 seconds |
| **MSF** | UK (Anthorn) | **60.0 kHz** | OOK (Carrier 100% off for 100/200/300/500 ms) | 60 seconds |
| **DCF77** | Germany (Mainflingen) | **77.5 kHz** | AM carrier reduction to 15% (100 ms = 0, 200 ms = 1) | 60 seconds |
| **JJY40** | Japan East (Fukushima) | **40.0 kHz** | AM pulse width keying | 60 seconds |
| **JJY60** | Japan West (Kyushu) | **60.0 kHz** | AM pulse width keying | 60 seconds |
| **Custom** | Global | User Selectable | User-defined UTC offset & simulated station format | — |

---

## 5. Software Stack & Architecture

### Recommended Language & Framework: **Arduino C++ (PlatformIO / Arduino IDE)**
* **Direct C Code Portability:** Reuses proven transmission bitstream engines directly from [`kangtastic/timestation`](https://github.com/kangtastic/timestation) (`timesignal.c`, `waveform.h`, `datetime.h`).
* **Hardware Fractional Divider:** Native access to the ESP32-C3 `LEDC` peripheral via the 80 MHz APB clock:
  $$\text{LEDC Frequency Error} < 0.0005\%$$
* **Power Management:** Deep sleep with RTC timer wakeup (`esp_deep_sleep_start()`) draws $\sim 5\,\mu\text{A}$.
* **Zero Jitter:** Deterministic interrupt-driven or hardware-timer pulse modulation without Python Garbage Collection (GC) pauses.

---

## 6. System Operating Cycle & Power Flow

```
[ Wake from Deep Sleep / Power On ]
              │
              ▼
    Check Wakeup Reason ──(First Boot or Config Button Held)──► [ Start AP Mode & Web Server ]
              │                                                             │
              ▼ (Scheduled Timer Wakeup)                                    ▼
       [ Connect Wi-Fi ]                                          [ Configure & Save to NVS ]
              │                                                             │
              ▼                                                             ▼
     [ Sync Time via NTP ]                                          [ Re-enter Deep Sleep ]
              │
              ▼
   [ Turn OFF Wi-Fi Completely ]  <── CRITICAL: Eliminates 2.4 GHz RF hash during broadcast
              │
              ▼
    [ Start LEDC Carrier PWM ]
    [ Broadcast Signal for 15-30 Min ]
              │
              ▼
   [ Calculate Next Sleep Window ]
              │
              ▼
    [ Enter ESP32 Deep Sleep ] (Draws ~5 uA until next scheduled broadcast)
```

---

## 7. Web Interface Features

The ESP32-C3 hosts an embedded single-page responsive web dashboard:
1. **Station Selection:** Dropdown for BPC (68.5 kHz), WWVB (60 kHz), MSF (60 kHz), DCF77 (77.5 kHz), JJY40/60, and Custom.
2. **Timezone Offset:** Manual $\pm 12$ hour adjustment to set any destination timezone on the clock.
3. **Broadcast Scheduling:** 
   * *Scheduled Window:* Transmit during clock sync hours (e.g. 02:00 – 02:30 AM), deep sleep otherwise.
   * *Continuous / Test Mode:* Continuous broadcast for bench testing.
4. **Network Credentials:** Wi-Fi SSID / Password configuration with captive portal fallback.
5. **NTP Server:** Configurable time server (default: `pool.ntp.org`).

---

## 8. Credits & Acknowledgments

* **Time Signal Protocols & Algorithms:** The core protocol encoding algorithms (`BPC`, `WWVB`, `MSF`, `DCF77`, `JJY`) and calendar routines are adapted from the open-source [timestation](https://github.com/kangtastic/timestation) project by **James Seo** (`james@equiv.tech`), licensed under the **MIT License**.
* **Date Algorithms:** Howard Hinnant's public-domain Gregorian calendar algorithms.
* **Hardware & Firmware Design:** Tailored for the **ESP32-C3-Mini** microcontroller using hardware LEDC PWM carrier generation and low-power deep sleep scheduling.
