# MultiBand6-ESP32-AtomicBeacon

An open-hardware, low-power near-field radio transmitter built on the **ESP32-C3-Mini** to simulate LF time signal broadcasts (**BPC, WWVB, MSF, DCF77, JJY40, JJY60**) for synchronizing radio-controlled ("atomic") clocks and wristwatches over a localized range (approx. 0.5 m to 1.0 m).

Why?? - I have a collection of multiband6 casios but I live in Singapore were there is no signal from any of the time towers. This device solves that problem by broadcasting the signals locally from my a small device next to my bed, about 0.5m from my watches.

It's still under testing.



---

## 1. Bill of Materials (BOM) & Components

| Component | Value / Specification | Role in Circuit | Rationale / Notes |
| :--- | :--- | :--- | :--- |
| **Microcontroller** | **ESP32-C3-Mini** | Core Controller & Carrier Generator | Single-core 32-bit RISC-V (160 MHz), hardware LEDC PWM with fractional clock divider, RTC timer, Wi-Fi 4, BLE 5. |
| **NPN Transistor** | **C1815 (2SC1815)** | Power Switch / RF Driver (Option A) | Small-signal NPN transistor (TO-92 package, E-C-B pinout) switching the LC tank at 5V for extended 1.0 m+ range. *(Note: You could also use a 2p2222 / 2N2222, but that transistor's different E-B-C pinout must be respected!)* |
| **Base Resistor ($R_{base}$)** | **$1\text{ k}\Omega$ (1/4 W)** | Transistor Base Drive (Option A) | Connects between GPIO 2 and C1815 Base (Pin 3). Limits base drive current to a safe $\sim 2.6\text{ mA}$ for saturation switching. |
| **Inductor ($L$)** | **3.5 mH (16×18 mm)** | Magnetic Antenna ($H$-field radiator) | "I-Type" drum/bobbin core power inductor. Open magnetic circuit allows magnetic dipole flux lines to radiate into the room. Low DC resistance. |
| **Tuning Capacitor ($C$)** | **1.5 nF (1500 pF / code `152`)** | Parallel LC Tank Resonator | Metallized Film (CBB / Polypropylene / Polyester) or C0G/NP0 ceramic. Rated $\ge 50\text{V}$. |
| **Damping Resistor ($R_{damp}$)** | **$220\,\Omega$ (1/4 W)** | Bandwidth Broadening & Safety Current Limiter | Lowers tank $Q$ to $\sim 6.5$, broadening bandwidth to $\sim 10\text{ kHz}$. Limits peak current to $\approx 23\text{ mA}$ in transistor booster or $\approx 15\text{ mA}$ in direct GPIO drive. |
| **DC Blocking Cap ($C_{block}$)** | **$1\,\mu\text{F}$ (or $100\text{ nF}$)** | DC Isolation & GPIO Protection (Option B) | Only required if running Option B (Direct GPIO Drive without transistor). Completely blocks DC current so pin cannot burn out if stuck `HIGH`. |

---

## 2. Hardware Circuit Schematics & Pinout

### Default Pin Assignment & ESP32-C3 SuperMini Pinout

The firmware defaults to **GPIO 2** for the antenna carrier output (`#define ANTENNA_PIN 2` in `src/main.cpp`).

| Board Pin | ESP32-C3 Signal | Role | Circuit Connection |
| :--- | :--- | :--- | :--- |
| **GPIO 2** | `IO2` (LEDC Ch 0) | **Default RF Carrier Out** | Connects to base resistor ($1\text{ k}\Omega$) driving C1815 Base (Pin 3), or DC-blocking cap ($1\,\mu\text{F}$) for direct GPIO |
| **5V / VIN** | `VBUS` (USB 5V) | Primary Power Rail | Supplies the C1815 booster circuit & damping resistor |
| **GND** | `GND` | Common Ground Return | Connects to C1815 transistor Emitter (Pin 1) and breadboard ground rail |

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

### Option A: C1815 NPN Transistor Booster Circuit (Recommended for 1.0 m+ Range)

This configuration uses the **C1815 (2SC1815) NPN** transistor powered directly from the **5V USB rail**. It isolates the ESP32 silicon and delivers $3\times$ to $5\times$ more magnetic flux for extended range:

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
                                         [ C ] Collector (Pin 2)
  ESP32-C3 GPIO 2 ──[ 1k ohm ]────────── [ B ] Base (Pin 3)       C1815 NPN Transistor
  (Default PWM Out) (R_base)             [ E ] Emitter (Pin 1)
                                                │
                                                ▼
  ESP32-C3 GND ─────────────────────────────────┴─── Common Ground (GND)
```

#### Why This Works Well:
1. **Zero Stress on ESP32:** The GPIO 2 pin only sources a safe $I_B \approx 2.6\text{ mA}$ into the base through the $1\text{ k}\Omega$ base resistor ($V_{GPIO} = 3.3\text{V}$, $V_{BE} \approx 0.7\text{V}$).
2. **5V Swing:** The LC tank swings around the 5V rail rather than 3.3V, pushing significantly more magnetic current through the 3.5 mH inductor.
3. **Uses Your $220\,\Omega$ Resistor:** The $220\,\Omega$ damping resistor limits peak collector current to a cool, safe $\approx 23\text{ mA}$ while maintaining the broad $\sim 10.7\text{ kHz}$ bandwidth so 68.5 kHz BPC resonates smoothly.
4. **Broadcast Range:** Up to **1.0 m – 1.2 m**.

#### C1815 Transistor Pinout & Pinout Connections (TO-92 Package)

The circuit is designed specifically around the **C1815 (2SC1815)** small-signal NPN transistor.

Looking at the **flat printed face** (labeled "C1815") with the leads pointing downwards, the pinout follows the standard Asian / JIS **E-C-B** configuration:

```
          TO-92 Package (Front View)
               ┌───────────┐
               │   C1815   │   <── Flat Printed Face
               │   GR 331  │
               └─┬───┬───┬─┘
                 │   │   │
                 1   2   3
                 E   C   B
```

| Pin # | Lead Name | Circuit Function | Breadboard Connection |
| :---: | :--- | :--- | :--- |
| **Pin 1 (Left)** | **Emitter (E)** | Common Ground Return | Connects directly to **(-) Blue GND Rail** (Row 13) |
| **Pin 2 (Center)** | **Collector (C)** | Switched LC Tank Low Side | Connects to **Row 12** (bottom of 3.5 mH inductor & 1.5 nF cap) |
| **Pin 3 (Right)** | **Base (B)** | RF Carrier PWM Input | Connects to **Row 11** (receives drive from GPIO 2 via $1\text{ k}\Omega$ resistor) |

> [!NOTE]
> **Orientation Tip:** Because **Base is Pin 3 (Right)** on the C1815, mounting the transistor with its flat face facing right puts Pin 3 (Base) at **Row 11**, directly adjacent to ESP32 GPIO 2 (Row 6). This allows the $1\text{ k}\Omega$ base resistor to bridge Rows 6 and 11 neatly without any crossing wires.

> [!WARNING]
> **Can you use a 2p2222 (2N2222 / PN2222) instead?**
>
> You **can** use a **2p2222** (commonly labeled **2N2222** or **PN2222**) NPN transistor if a C1815 is unavailable, **BUT that transistor's completely different pinout must be respected!**
>
> * **C1815 (Asian / JIS):** `Pin 1 = Emitter (E) | Pin 2 = Collector (C) | Pin 3 = Base (B)` — **E - C - B**
> * **2p2222 / 2N2222 (American / JEDEC TO-92):** `Pin 1 = Emitter (E) | Pin 2 = Base (B) | Pin 3 = Collector (C)` — **E - B - C**
>
> **Crucial wiring differences to respect when using 2p2222:**
> 1. **Base pin location is swapped:** On a 2p2222, **Base is the middle pin (Pin 2)** instead of Pin 3. The $1\text{ k}\Omega$ base resistor from GPIO 2 must be routed to **Pin 2 (Center)**.
> 2. **Collector pin location is swapped:** On a 2p2222, **Collector is the right pin (Pin 3)** instead of Pin 2. The LC tank low side must be routed to **Pin 3 (Right)**.
> 3. **Emitter:** Pin 1 (Left) remains Emitter (GND) on both transistors.
>
> If you drop a 2p2222 / 2N2222 directly into the breadboard using the C1815 layout without adjusting the pin wiring, the Base and Collector will be reversed, the transistor will not switch, and the beacon will fail to broadcast.

| Transistor Model | Pinout Standard | Pin 1 (Left) | Pin 2 (Center) | Pin 3 (Right) | Required Wiring in this Circuit |
| :--- | :---: | :---: | :---: | :---: | :--- |
| **C1815 (Default)** | **JIS (E-C-B)** | **Emitter (E)** | **Collector (C)** | **Base (B)** | Pin 1 $\rightarrow$ GND, Pin 2 $\rightarrow$ LC Tank (Row 12), Pin 3 $\rightarrow$ $1\text{ k}\Omega$ to GPIO 2 (Row 11) |
| **2p2222 / 2N2222 (Alternative)** | **JEDEC (E-B-C)** | **Emitter (E)** | **Base (B)** | **Collector (C)** | Pin 1 $\rightarrow$ GND, Pin 2 $\rightarrow$ $1\text{ k}\Omega$ to GPIO 2, Pin 3 $\rightarrow$ LC Tank *(Pinout must be respected!)* |

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

### Breadboard Wiring Layout (C1815 NPN Booster Circuit)

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
   11:    [ . . . . . ]                  [ . . . . . ] ── C1815 BASE (B, Pin 3) ◄─┘ (Closest to IO2!)
   12:    [ . . . . . ]                  [ . . . . . ] ── C1815 COLLECTOR (C, Pin 2)
                                               │
                                       ┌───────┴───────┐
                                       │ 3.5mH Inductor│  (Parallel LC Tank: Rows 12–16)
                                       │ 1.5nF Cap     │  (Col F: Cap, Col H: Inductor)
                                       └───────┬───────┘
                                               │
   13:    [ . . . . . ]                  [ . . . . . ] ── C1815 EMITTER (E, Pin 1) ── Wire to (-) Blue GND
   ...
   16:    [ . . . . . ]                  [ . . . . . ] ── LC Tank High Side
                                               │
                                         [ 220 ohm ]   ── R_damp connects directly
                                               │          from Row 16 to (+) 5V Rail
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
3. **C1815 NPN Transistor (JIS E-C-B Pinout):**
   * Plug into **Rows 11, 12, and 13** in Column G with the **flat printed face facing right** (curved dome to the left):
     * **Row 11 = Base (B, Pin 3)** $\rightarrow$ Closest to GPIO 2; receives base drive from the $1\text{ k}\Omega$ resistor.
     * **Row 12 = Collector (C, Pin 2)** $\rightarrow$ Switched output; connects to the parallel LC tank low side.
     * **Row 13 = Emitter (E, Pin 1)** $\rightarrow$ Add a short black jumper from Row 13 (Col I) to the **(-) Blue GND Rail**.
   > [!NOTE]
   > **Note on substituting a 2p2222 (2N2222 / PN2222):** You could use a 2p2222 / 2N2222 instead, but you must respect that transistor's different **E-B-C** pinout: its center pin (Pin 2) is Base and its right pin (Pin 3) is Collector. If substituted, the $1\text{ k}\Omega$ resistor from GPIO 2 must go to the center pin (Base), and the LC tank must connect to the right pin (Collector).
4. **Base Drive Resistor ($1\text{ k}\Omega$):**
   * Plug one leg of the **$1\text{ k}\Omega$ resistor** into **Row 6 (Col I)** (directly taps ESP32 GPIO 2).
   * Plug the other leg into **Row 11 (Col I)** (directly taps the C1815 Base, Pin 3). No loose wires required!
5. **LC Tank (Parallel Inductor + Capacitor):**
   * Plug the **1.5 nF Capacitor** across **Row 12 (Col F)** and **Row 16 (Col F)** (inner position, towards center ravine for clearance).
   * Plug the **3.5 mH Inductor** across **Row 12 (Col H)** and **Row 16 (Col H)** (outer position, nearer breadboard edge for maximum RF radiation & easy watch placement).
6. **Damping Resistor ($220\,\Omega$):**
   * Plug one leg of the **$220\,\Omega$ resistor** into **Row 16 (Col J)**.
   * Plug the other leg directly into the **(+) Red 5V Rail**.
---



## 3. Physical Antenna Alignment & Orientation

* The **16×18 mm I-type inductor** radiates a magnetic dipole field oriented along its **vertical cylindrical axis** (top and bottom flat faces).
* Most atomic clocks and watches contain a **horizontal ferrite rod** inside the case.
* **Optimal Placement:** Position the inductor so that the magnetic flux lines looping out of the inductor's ends pass directly through the clock's internal antenna bar.
* **Broadcast Range:** 
  * **C1815 NPN Booster (Option A):** Approx. **1.0 m to 1.2 m** (extended coverage for watches across a room or dresser).
  * **Direct GPIO with $220\,\Omega$ (Option B):** Approx. **0.5 m to 0.7 m** (ideal for close proximity bedside table or desk).

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

## 7. Web Interface Features & Multi-Station Carousel

The ESP32-C3 hosts an embedded single-page responsive web dashboard:
1. **Operating Modes:**
   * **Single Station Mode:** Broadcasts a single chosen station (BPC, WWVB, MSF, DCF77, JJY40, JJY60) for a set duration.
   * **Multi-Station Carousel Mode ("World Tour"):** Automatically sequences through multiple selected stations during the scheduled broadcast window so all watches in the room sync regardless of which regional home city they are configured for:
     * Select any combination of stations via checklist (e.g. BPC 68.5 kHz $\rightarrow$ JJY60 60.0 kHz $\rightarrow$ WWVB 60.0 kHz).
     * Configurable stage duration (e.g. 15 minutes per station $\times$ 3 stations = 45-minute nightly window).
     * **Clean Minute Boundary Handoff:** The ESP32-C3 hardware LEDC timer retunes carrier frequency dynamically on the fly at the exact `:00` second mark, ensuring no watch receiver encounters truncated minute frames.
     * Live dashboard displays active carousel stage and real-time countdown timers.
2. **Timezone Offset:** Manual $\pm 12$ hour adjustment to set any destination timezone on the clock.
3. **Broadcast Scheduling:** 
   * *Scheduled Window:* Transmit during clock sync hours (e.g. 02:00 AM daily), modem-sleep or deep sleep otherwise.
   * *Manual Test Mode:* On-demand broadcast button for bench testing with live countdown.
4. **Network Credentials:** Wi-Fi SSID / Password configuration with captive portal fallback.
5. **NTP Server:** Configurable time server (default: `pool.ntp.org`).

---

## 8. Credits & Acknowledgments

* **Time Signal Protocols & Algorithms:** The core protocol encoding algorithms (`BPC`, `WWVB`, `MSF`, `DCF77`, `JJY`) and calendar routines are adapted from the open-source [timestation](https://github.com/kangtastic/timestation) project by **James Seo** (`james@equiv.tech`), licensed under the **MIT License**.
* **Date Algorithms:** Howard Hinnant's public-domain Gregorian calendar algorithms.
* **Hardware & Firmware Design:** Tailored for the **ESP32-C3-Mini** microcontroller using hardware LEDC PWM carrier generation and low-power deep sleep scheduling.

---

## 9. License

This project is licensed under the **MIT License** — see the [LICENSE](LICENSE) file for details.

