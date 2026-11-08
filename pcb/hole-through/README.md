# MultiBand 6 ESP32 Atomic Beacon — Through-Hole (THT) Edition

A 100% through-hole, hand-solderable DIY carrier board for the MultiBand 6 ESP32 Atomic Clock Beacon transmitter. Designed specifically for easy assembly with standard hobbyist soldering equipment—**zero SMD components required**.

---

## 📐 Board Specifications

| Parameter | Value |
| :--- | :--- |
| **Dimensions** | $70.0\text{ mm} \times 46.0\text{ mm}$ ($2.76" \times 1.81"$) |
| **Layers** | 2-layer FR-4 ($1.6\text{ mm}$ standard thickness, $1\text{ oz}$ copper) |
| **Auxiliary Origin** | Bottom-left corner $(0.0, 46.0)$ canvas $\to$ $(0, 0)$ manufacturing origin |
| **Mounting** | $4\times \text{M3}$ mounting holes ($3.2\text{ mm}$ drill, $6.0\text{ mm}$ annular ring) |
| **Clearances** | $\ge 0.2\text{ mm}$ trace clearance, wide $0.6\text{ mm}$ RF tank buses |
| **Ground Plane** | Continuous solid ground pour on bottom layer (`B.Cu`) with thermal reliefs |
| **DRC Status** | **0 Errors, 0 Warnings, 0 Unconnected Pads** |

---

## 🛠️ Complete Bill of Materials (BOM)

All components use standard through-hole footprints available on DigiKey, Mouser, LCSC, AliExpress, or eBay:

| Ref | Value / Part | Footprint & Lead Pitch | Description & Sourcing Notes |
| :--- | :--- | :--- | :--- |
| **$U_1$** | ESP32-C3 SuperMini | $2\times 1\times 8$ Female Socket Header ($2.54\text{ mm}$ pitch, $15.24\text{ mm}$ / $600\text{ mil}$ row spacing) | RISC-V Wi-Fi / BLE microcontroller module. Socket allows easy removal or flashing. |
| **$Q_1$** | 2SC1815 (or 2N2222) | TO-92 Inline ($2.54\text{ mm}$ lead spacing) | NPN RF switching transistor. Silkscreen marked with explicit **E, C, B** labels. *(See Transistor Pinout Note below)* |
| **$D_1$** | 1N5819 (or 1N4148 / 1N4007) | DO-41 Axial ($10.16\text{ mm}$ / $400\text{ mil}$ pitch) | 1A 40V Schottky flyback clamp diode across the tank inductor. |
| **$R_1$** | $1\text{ k}\Omega$ | Axial DIN0207 ($10.16\text{ mm}$ pitch, 1/4W) | ESP32 GPIO1 base drive resistor ($5\%$ or $1\%$ carbon/metal film). Boot-safe non-strapping pin. |
| **$R_2$** | $220\,\Omega$ | Axial DIN0207 ($10.16\text{ mm}$ pitch, 1/4W) | Tank supply current limiter / Q-damping resistor. |
| **$R_4$** | $4.7\text{ k}\Omega$ | Axial DIN0207 ($10.16\text{ mm}$ pitch, 1/4W) | $I^2C$ SDA line pull-up resistor. |
| **$R_5$** | $4.7\text{ k}\Omega$ | Axial DIN0207 ($10.16\text{ mm}$ pitch, 1/4W) | $I^2C$ SCL line pull-up resistor. |
| **$R_6$** | $10\text{ k}\Omega$ | Axial DIN0207 ($10.16\text{ mm}$ pitch, 1/4W) | Manual sync pushbutton pull-up resistor (to $+3.3\text{V}$). |
| **$C_1$** | $1.5\text{ nF}$ ($1500\text{ pF}$, code `152`) | Radial / Box Film ($5.08\text{ mm}$ / $200\text{ mil}$ pitch) | Main LC resonant tank capacitor ($50\text{V}+$ rated, metallized polypropylene or C0G/NP0 ceramic). |
| **$C_2$** | Tuning Capacitor Socket | Box Film / Radial ($5.08\text{ mm}$ / $200\text{ mil}$ pitch) | Auxiliary parallel capacitor socket for frequency fine-tuning (e.g. $100\text{ pF}$–$470\text{ pF}$ for JJY/WWVB/MSF). |
| **$C_3$** | $10\,\mu\text{F}$ | Radial Electrolytic ($D=5\text{ mm}$, $2.54\text{ mm}$ pitch) | $+5\text{V}$ bulk filter capacitor ($25\text{V}+$ rated). |
| **$C_4$** | $100\text{ nF}$ ($0.1\,\mu\text{F}$, code `104`) | Radial Disc / MLCC ($5.08\text{ mm}$ pitch) | High-frequency ceramic decoupling bypass capacitor ($50\text{V}$ rated). |
| **$L_1$** | $3.5\text{ mH}$ $16\times 18$ | Radial Drum / I-Core ($12.0\text{ mm}$ lead pitch, $D=16\text{ mm}$) | High-Q open-core ferrite inductor serving as the magnetic near-field transmitting antenna. |
| **$J_1$** | KF301-2P | $2$-Pin Screw Terminal ($5.08\text{ mm}$ pitch) | External antenna / loop coil connection block (`+` Tank Hi, `-` Switched Collector). |
| **$SW_1$** | $6\times 6\text{ mm}$ Tactile | $4$-pin THT Pushbutton ($6.5\text{ mm} \times 4.5\text{ mm}$ pin spacing) | Bedside momentary manual sync trigger button. |
| **$J_2$** | $1\times 4$ Pin Header | Male or Female ($2.54\text{ mm}$ pitch) | $I^2C$ OLED Display interface (`GND`, `3V3`, `SCL`, `SDA`). |
| **$J_3$** | $1\times 2$ Pin Header | Male ($2.54\text{ mm}$ pitch) | Remote sync button connection (wired in parallel with $SW_1$). |
| **$H_1$–$H_4$** | M3 Hardware | $3.2\text{ mm}$ drill | $4\times$ corner mounting holes for standard $M3$ standoffs or 3D-printed enclosure. |

---

## ⚡ Transistor Pinout Note ($Q_1$)

The board silkscreen features clear visual pin markings for the TO-92 transistor:
- **Default Footprint:** Asian standard **E-C-B** pinout (e.g., **2SC1815**, **2SC945**, **KSC1815**):
  - Top Pin = **Collector** (`C`) $\to$ Tank Switched Node
  - Middle Pin = **Base** (`B`) $\to$ Base Resistor $R_1$
  - Bottom Pin = **Emitter** (`E`) $\to$ Ground
- **Alternative US Pinout (2N2222 / 2N3904):**
  - Standard American TO-92 pinout is **E-B-C**. If using a 2N2222 or 2N3904, cross the Base and Collector leads or gently splay them to match the labeled holes: Pin 1 (`E`) to Ground, Pin 2 (`C`) to Collector, Pin 3 (`B`) to Base.

---

## 🔌 Headers & Connectors Pinout

### $I^2C$ Display Header ($J_2$)
Matches standard $0.96"$ or $1.3"$ SSD1306 / SH1106 $I^2C$ OLED pinout:
1. **GND** (Square Pad 1)
2. **3V3** (Pad 2)
3. **SCL** (Pad 3 $\to$ ESP32 GPIO5)
4. **SDA** (Pad 4 $\to$ ESP32 GPIO4)

### External Antenna Terminal ($J_1$)
For connecting an external ferrite rod antenna or resonant loop coil:
- **`+` (Pin 1):** Tank Supply Rail (`NET_TANK_HI`, $+5\text{V}$ through $R_2$)
- **`-` (Pin 2):** Switched Collector Rail (`NET_TANK_SW`, $Q_1$ Collector)

---

## 📦 Manufacturing with JLCPCB

The manufacturing package is pre-compiled and fully validated:

- **Gerber & Drill ZIP:** [`MultiBand6_Beacon_THT_Gerbers.zip`](file:///Users/robertlongbottom/dev/MultiBand6-ESP32-AtomicBeacon/pcb/hole-through/MultiBand6_Beacon_THT_Gerbers.zip)
- **Ordering Parameters:**
  - **Base Material:** FR-4
  - **Layers:** 2 Layers
  - **Dimensions:** $70\text{ mm} \times 46\text{ mm}$
  - **PCB Thickness:** $1.6\text{ mm}$
  - **PCB Color:** Green, Blue, Black, Purple, etc. (any standard color)
  - **Surface Finish:** HASL with lead or Lead-free HASL
  - **Copper Weight:** $1\text{ oz}$ ($35\,\mu\text{m}$)
  - **SMT Assembly:** **None / No** (all components are hand-soldered through-hole parts)

---

## 🪛 Recommended Soldering Order

For the cleanest and easiest hand-soldering assembly, solder components in order of physical height:

1. **Lowest Profile:** Resistors $R_1, R_2, R_4, R_5, R_6$ and Diode $D_1$ (bend leads $90^\circ$ and lay flat against the PCB). *Ensure $D_1$ cathode band aligns with the double-line silkscreen mark towards $+5\text{V}$*.
2. **Capacitors & Headers:** Ceramic capacitor $C_4$, film capacitors $C_1, C_2$, pin headers $J_2, J_3$, and the $2\times 1\times 8$ female socket for $U_1$.
3. **Transistor & Button:** $Q_1$ TO-92 transistor (observe `E-C-B` markings) and tactile pushbutton $SW_1$.
4. **Tall Components:** Electrolytic capacitor $C_3$ (observe `+`/`-` polarity), screw terminal block $J_1$, and the $16\times 18\text{ mm}$ power inductor $L_1$.
5. **Final Step:** Insert the ESP32-C3 SuperMini module into the socket headers with the USB-C port facing the board edge.
