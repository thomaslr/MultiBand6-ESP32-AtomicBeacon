# MultiBand6-ESP32-AtomicBeacon

An open-hardware, low-power near-field radio transmitter built on the **ESP32-C3-Mini** to simulate LF time signal broadcasts (**BPC, WWVB, MSF, DCF77, JJY40, JJY60**) for synchronizing radio-controlled ("atomic") clocks and wristwatches over a localized range (approx. 0.5 m to 1.0 m).

Why?? - I have a collection of multiband6 casios but I live in Singapore were there is no signal from any of the time towers. This device solves that problem by broadcasting the signals locally from my a small device next to my bed, about 0.5m from my watches.

It's still under testing.



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

## 2. Hardware Circuit Schematics & Pinout

### Default Pin Assignment & ESP32-C3 SuperMini Pinout

The firmware defaults to **GPIO 2** for the antenna carrier output (`#define ANTENNA_PIN 2` in `src/main.cpp`).

| Board Pin | ESP32-C3 Signal | Role | Circuit Connection |
| :--- | :--- | :--- | :--- |
| **GPIO 2** | `IO2` (LEDC Ch 0) | **Default RF Carrier Out** | Connects to base resistor ($1\text{ k}\Omega$) or DC-blocking cap ($1\,\mu\text{F}$) |
| **5V / VIN** | `VBUS` (USB 5V) | Primary Power Rail | Supplies the transistor booster circuit & damping resistor |
| **GND** | `GND` | Common Ground Return | Connects to transistor emitter and breadboard ground rail |

> **Why GPIO 2?**
> * **LEDC Peripheral:** Native hardware routing to LEDC channel 0 with fractional divider clocking.
> * **Boot Stability:** Not a bootstrapping/strapping pin that interferes with USB flashing, serial upload, or cold boot.
> * **Physical Placement:** Located on the accessible edge header of both the compact **ESP32-C3 SuperMini** and standard **ESP32-C3-DevKitM-1** boards.

```
      TOP VIEW (Component Side — looking at chip & buttons)
                       ┌───[ USB-C ]───┐
     [BOOT Button Side]│               │[RST Button Side]
              GPIO 5   │ [ ]       [ ] │   5V          <── Top-Right (next to USB)
              GPIO 6   │ [ ]       [ ] │   GND (G)     <── Next to 5V
              GPIO 7   │ [ ]       [ ] │   3V3 (3.3)   <── Regulated 3.3V
              GPIO 8   │ [ ]       [ ] │   GPIO 4
              GPIO 9   │ [ ]       [ ] │   GPIO 3
             GPIO 10   │ [ ]       [■] │   GPIO 2      <── [■] ANTENNA OUT (Pin 6)
             GPIO 20   │ [ ]       [ ] │   GPIO 1
   (Pin 21)  GPIO 21   │ [ ]       [ ] │   GPIO 0      (Pin 0)
                       └──[ Antenna ]──┘

      BOTTOM VIEW (Underside — looking at PCB silkscreen labels)
                       ┌───[ USB-C ]───┐
                       │               │
     Top-Left (5V) ──► │ [ ]       [ ] │   GPIO 5
              GND (G)  │ [ ]       [ ] │   GPIO 6
            3V3 (3.3)  │ [ ]       [ ] │   GPIO 7
               GPIO 4  │ [ ]       [ ] │   GPIO 8
               GPIO 3  │ [ ]       [ ] │   GPIO 9
   ANTENNA OUT ──────► │ [■]       [ ] │   GPIO 10
               GPIO 1  │ [ ]       [ ] │   GPIO 20
     (Pin 0)   GPIO 0  │ [ ]       [ ] │   GPIO 21     (Pin 21)
                       └───────────────┘
```

---

### Option A: NPN Transistor Booster Circuit (Recommended for 1.0 m+ Range)

This configuration uses any generic small-signal NPN bipolar junction transistor (BJT) powered directly from the **5V USB rail**. It isolates the ESP32 silicon and delivers $3\times$ to $5\times$ more magnetic flux for extended range:

```
                                  +5V (from ESP32 5V / VIN Pin)
                                    │
                                    ├───[ 220 ohm ]─── (R_damp)
                                    │
                                    ├───┬───────────────┬───┐
                                    │   │               │   │
                                    │  [ 3.5 mH ]   [ 1.5 nF ]  (Parallel LC Tank)
                                    │  (Inductor)   (Capacitor)
                                    │   │               │   │
                                    └───┴───────────────┴───┘
                                                │
                                                ▼
                                         [ C ] (Collector)
  ESP32-C3 GPIO 2 ──[ 1k ohm ]────────── [ B ]   Generic NPN Transistor
  (Default PWM Out) (R_base)             [ E ] (Emitter)
                                                │
                                                ▼
  ESP32-C3 GND ─────────────────────────────────┴─── Common Ground (GND)
```

#### Why This Works Well:
1. **Zero Stress on ESP32:** The GPIO 2 pin only sources a safe $I_B \approx 2.6\text{ mA}$ into the base through the $1\text{ k}\Omega$ base resistor ($V_{GPIO} = 3.3\text{V}$, $V_{BE} \approx 0.7\text{V}$).
2. **5V Swing:** The LC tank swings around the 5V rail rather than 3.3V, pushing significantly more magnetic current through the 3.5 mH inductor.
3. **Uses Your $220\,\Omega$ Resistor:** The $220\,\Omega$ damping resistor limits peak collector current to a cool, safe $\approx 23\text{ mA}$ while maintaining the broad $\sim 10.7\text{ kHz}$ bandwidth so 68.5 kHz BPC resonates smoothly.
4. **Broadcast Range:** Up to **1.0 m – 1.2 m**.

#### Transistor Pinout Quick Reference (TO-92 Package)

When using standard through-hole transistors from your parts drawer, identify the pin sequence looking at the **flat printed face** with the legs pointing down:

| Transistor Pattern | Common Part Numbers | Leg 1 (Left) | Leg 2 (Center) | Leg 3 (Right) |
| :--- | :--- | :---: | :---: | :---: |
| **American (E-B-C)** | **2N3904, 2N2222, 2N4401** | **Emitter (E)** | **Base (B)** | **Collector (C)** |
| **European (C-B-E)** | **BC547, BC548, BC337, BC549** | **Collector (C)** | **Base (B)** | **Emitter (E)** |
| **Asian (E-C-B)** | **S8050, 2SC1815, C945, SS8050** | **Emitter (E)** | **Collector (C)** | **Base (B)** |

---

### Option B: Direct GPIO Drive (Minimalist Bench / Desk Use)

This configuration connects directly to ESP32-C3 GPIO 2 with built-in safety against DC overcurrent and back-EMF spikes:

```
                           C_block       R_damp
                          [ 1 uF ]     [ 220 ohm ]
  ESP32-C3 GPIO 2 ---------[ || ]--------[ \/\/\ ]---------+-----------------+
  (Default PWM Out)                                        |                 |
                                                        +------+          +------+
                                                        |      |          |      |
                                                     3.5 mH  Inductor  1.5 nF Capacitor
                                                        |      |          |      |
                                                        +------+          +------+
                                                           |                 |
  ESP32-C3 GND --------------------------------------------+-----------------+
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

---

### Breadboard Wiring Layout (NPN Booster Circuit)

> 🎨 **Visual Diagram:** A full vector graphic illustration is available in [breadboard_layout.svg](breadboard_layout.svg) or viewable in your browser via [breadboard_layout.html](breadboard_layout.html).

Here is the top-down terminal row mapping for a standard breadboard with the **ESP32-C3 SuperMini mounted directly across the center ravine**:

```
  (+) POWER RAIL  [+5V from ESP32 Pin F1]  ─── (Red Line)
  (-) GROUND RAIL [GND from ESP32 Pin F2]  ─── (Blue Line)

   Row    [A B C D E]  (Center Ravine)  [F G H I J]
  ────────────────────────────────────────────────────────────────────────
   01:    [ . . . . . ] ── [ESP32-C3] ── [ 5V ]  [ . . . . ] ── Wire to (+) 5V Rail
   02:    [ . . . . . ] ── [SuperMini]── [ GND ] [ . . . . ] ── Wire to (-) GND Rail
   03:    [ . . . . . ] ── [ Rows   ] ── [ 3V3 ] [ . . . . ]
   04:    [ . . . . . ] ── [ 1 to 8 ] ── [ IO4 ] [ . . . . ]
   05:    [ . . . . . ] ── [ across ] ── [ IO3 ] [ . . . . ]
   06:    [ . . . . . ] ── [ ravine ] ── [ IO2 ] [ . . . . ] ── [ 1k R_base ]
   07:    [ . . . . . ] ── [        ] ── [ IO1 ] [ . . . . ]          │
   08:    [ . . . . . ] ── [        ] ── [ IO0 ] [ . . . . ]          │
                                                                      │
   11:    [ . . . . . ]                  [ . . . . . ] ── Transistor EMITTER (E) ── Wire to (-) GND
   12:    [ . . . . . ]                  [ . . . . . ] ── Transistor BASE (B) ◄─────┘
   13:    [ . . . . . ]                  [ . . . . . ] ── Transistor COLLECTOR (C)
                                               │
                                       ┌───────┴───────┐
                                       │ 3.5mH Inductor│  (Parallel LC Tank)
                                       │ 1.5nF Cap     │
                                       └───────┬───────┘
                                               │
   17:    [ . . . . . ]                  [ . . . . . ]
                                               │
                                         [ 220 ohm ]   ── R_damp connects directly
                                               │          from Row 17 to (+) 5V Rail
  ────────────────────────────────────────────────────────────────────────
```

#### Step-by-Step Breadboard Connections:
1. **Mount the ESP32-C3 SuperMini:**
   * Plug the module directly into **Rows 1 to 8** spanning the center ravine:
     * **Column E (Left):** GPIO 5 down to GPIO 21 (all left pins open).
     * **Column F (Right):** Pin **F1 = 5V**, **F2 = GND**, Pin **F6 = GPIO 2 (Antenna Out)**.
2. **Power Rails:**
   * Add a short red jumper wire from **Row 1 (Col G)** to the **(+) Red Rail**.
   * Add a short black jumper wire from **Row 2 (Col G)** to the **(-) Blue Rail**.
3. **NPN Transistor:**
   * Plug into **Rows 11, 12, and 13** in Column G (standard 2N3904 / 2N2222 E-B-C pinout):
     * **Row 11 = Emitter (E)** $\rightarrow$ Add a short black jumper from Row 11 (Col H) to **(-) Blue GND Rail**.
     * **Row 12 = Base (B)** $\rightarrow$ Receives base drive from the $1\text{ k}\Omega$ resistor.
     * **Row 13 = Collector (C)** $\rightarrow$ Connects to the bottom of the LC tank.
4. **Base Drive Resistor ($1\text{ k}\Omega$):**
   * Plug one leg of the **$1\text{ k}\Omega$ resistor** into **Row 6 (Col I)** (directly taps ESP32 GPIO 2).
   * Plug the other leg into **Row 12 (Col I)** (directly taps the transistor Base). No loose wires required!
5. **LC Tank (Parallel Inductor + Capacitor):**
   * Plug the **1.5 nF Capacitor** across **Row 13 (Col F)** and **Row 17 (Col F)** (inner position, towards center ravine for clearance).
   * Plug the **3.5 mH Inductor** across **Row 13 (Col H)** and **Row 17 (Col H)** (outer position, nearer breadboard edge for maximum RF radiation & easy watch placement).
6. **Damping Resistor ($220\,\Omega$):**
   * Plug one leg of the **$220\,\Omega$ resistor** into **Row 17 (Col J)**.
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
* **Constant 80 MHz Low-Power Clock Architecture:**
  * **Zero Clock Overhead (Constant 80 MHz):** The CPU remains at a cool, efficient 80 MHz at all times. Because the ESP32-C3 APB peripheral bus clock is locked at 80 MHz, the hardware LEDC timer produces the exact same fractional-accuracy PWM carrier at 80 MHz as it would at 160 MHz.
  * **24/7 Web Standby:** Engages 802.11 Modem-Sleep (`WiFi.setSleep(true)`), keeping the web server and mDNS responder (`http://timestation.local`) accessible 24/7 at ~15–20 mA average current.
  * **Clean Broadcast Rail:** Automatically pauses Wi-Fi sleep (`WiFi.setSleep(false)`) during active broadcasts to keep the 3.3V power rail flat and ripple-free for the LC antenna.
  * **Optional Deep Sleep:** Supports true battery-powered deep sleep with RTC timer wakeup (`esp_deep_sleep_start()`) drawing $\sim 5\,\mu\text{A}$.
* **Zero Jitter:** Deterministic interrupt-driven or hardware-timer pulse modulation without Python Garbage Collection (GC) pauses.

---

## 6. System Operating Cycle & Power Flow

```
[ Power On / Boot ]
        │
        ▼
[ Read Flash Preferences (NVS) ]
        │
        ├───(No Saved Wi-Fi)────────► [ Start SoftAP "TimeStation-Setup" ]
        │                                         │
        ▼ (Saved Wi-Fi Found)                     ▼
 [ Connect to Wi-Fi ] ────────────────► [ Web Setup & Config Portal ]
        │
        ▼
 [ Synchronize NTP Time ]
        │
        ▼
 ┌─────────────────────────────────────────────────────────────┐
 │            24/7 Ultra-Low-Power Standby Mode                │
 │  • CPU running at constant 80 MHz (cool & efficient)        │
 │  • 802.11 Modem-Sleep enabled (~15-20 mA)                   │
 │  • Web Dashboard & mDNS (timestation.local) live 24/7       │
 │  • FreeRTOS yields idle time slices                         │
 └──────────────────────────────┬──────────────────────────────┘
                                │
          ┌─────────────────────┴─────────────────────┐
          │                                           │
          ▼ (Scheduled Time OR "Broadcast Now")       ▼ (User Opens Browser)
 ┌──────────────────────────────────────────┐  ┌───────────────────────────┐
 │        Atomic Broadcast Active           │  │    Serve HTTP Web UI      │
 │  • CPU at 80 MHz (80 MHz APB Bus PWM)    │  │  • Real-time NTP clock    │
 │  • Wi-Fi Sleep paused (clean 3.3V rail)  │  │  • Station selector       │
 │  • 50ms tick ISR modulates carrier       │  │  • Offset & duration cfg  │
 │  • Duration: 5 - 30 minutes              │  └───────────────────────────┘
 └────────────────────┬─────────────────────┘
                      │
                      ▼ (Broadcast Finishes)
 [ Re-enable Wi-Fi Modem-Sleep Standby ]
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
