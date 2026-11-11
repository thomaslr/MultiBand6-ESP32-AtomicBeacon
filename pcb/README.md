# MultiBand 6 ESP32 Atomic Beacon — Hardware & PCB Designs

This directory contains two complete, production-ready printed circuit board designs for the **MultiBand 6 ESP32 Atomic Clock Beacon**, optimized for different fabrication and assembly workflows.

Both boards integrate the RF switching driver, flyback diode protection, resonant LC tank, bulk/RF power decoupling, tactile manual sync pushbutton, $I^2C$ OLED display header, and dual antenna connections (onboard $16\times 18\text{ mm}$ drum inductor + screw terminal block).

---

## 🧭 Hardware Design Comparison

| Specification | SMT Carrier Board ([`smd/`](smd/)) | Through-Hole DIY ([`hole-through/`](hole-through/)) | Solderless Breadboard ([`breadboard/`](breadboard/)) |
| :--- | :--- | :--- | :--- |
| **Target User** | Automated Turnkey Assembly via JLCPCB | 100% Hand-Solderable DIY Makers | Rapid Prototyping & Testing |
| **Dimensions** | **$58.0\text{ mm} \times 36.0\text{ mm}$** ($2.28" \times 1.42"$) | **$70.0\text{ mm} \times 46.0\text{ mm}$** ($2.76" \times 1.81"$) | Standard Half / Full Breadboard |
| **Component Tech** | SMD 0805, SOT-23, SOD-123 | **100% Through-Hole passives** | Standard Lead Passives & Jumpers |
| **RF Transistor ($Q_1$)** | SOT-23 MMBT2222A / C1815 | **TO-92** C1815 or 2N2222 (with explicit `E, C, B` silk) | **TO-92** C1815 or 2N2222 |
| **Flyback Diode ($D_1$)** | SOD-123 1N5819HW | **DO-41 Axial** 1N5819 (or 1N4148 / 1N4007) | Optional / Built-in clamp |
| **Resistors ($R_1$–$R_6$)** | 0805 SMD resistors | **1/4W Axial DIN0207** ($10.16\text{ mm}$ / $400\text{ mil}$ pitch) | 1/4W Axial Through-Hole |
| **Decoupling ($C_3, C_4$)** | 0805 SMD ceramic ($10\,\mu\text{F}, 100\text{ nF}$) | **Radial Electrolytic** ($10\,\mu\text{F}$) + **Radial Ceramic** ($100\text{ nF}$) | Breadboard rail decoupling |
| **Tank Tuning ($C_1, C_2$)** | 0805 base + 5.08 mm THT socket | **Dual 5.08 mm Box Film / Radial sockets** | 1.5 nF Film across Rows 12–16 |
| **MCU Socket ($U_1$)** | $2\times 1\times 8$ Female Header ($2.54\text{ mm}$) | **$2\times 1\times 8$ Female Header** ($2.54\text{ mm}$, $600\text{ mil}$ row spacing) | Direct socket across center ravine |
| **Antenna Coil ($L_1$)** | $16\times 18\text{ mm}$ 3.5 mH Drum Inductor | **$16\times 18\text{ mm}$ 3.5 mH Drum Inductor** ($12.0\text{ mm}$ pitch) | $16\times 18\text{ mm}$ 3.5 mH Drum Inductor |
| **Assembly Cost** | **$0.00 setup fees** (100% JLCPCB Basic parts) | **$0.00 assembly** (assemble with hand soldering iron) | **$0.00** (Zero soldering required) |
| **DRC Validation** | **0 errors, 0 warnings, 0 unconnected** | **0 errors, 0 warnings, 0 unconnected** | N/A (Plug-and-play wiring) |

---

## 📌 Pin Assignments & Boot-Safe Architecture

Both designs route the RF carrier output to **GPIO 1** to guarantee 100% boot and flashing reliability:

| Board Pin | Signal / Function | Circuit Role | Boot / Strapping Safety |
| :--- | :--- | :--- | :--- |
| **GPIO 1** | **RF Carrier Out** | Drives base of $Q_1$ via $1\text{ k}\Omega$ ($R_1$) | **100% Boot-Safe.** Pure GPIO/ADC on ESP32-C3 with zero strapping functions. Allows flashing over USB while assembled. |
| **GPIO 2** | *Unconnected* | *(Reserved)* | Avoided for RF driving because GPIO 2 is an ESP32-C3 strapping pin that prevents USB/serial download mode if pulled low. |
| **GPIO 3** | **Manual Sync** | Tactile button $SW_1$ + header $J_3$ | Safe. $10\text{ k}\Omega$ pull-up ($R_6$) to $+3.3\text{V}$, active LOW. |
| **GPIO 4** | **I2C SDA** | Display / Sensor Header $J_2$ | Safe. $4.7\text{ k}\Omega$ pull-up ($R_4$) to $+3.3\text{V}$. |
| **GPIO 5** | **I2C SCL** | Display / Sensor Header $J_2$ | Safe. $4.7\text{ k}\Omega$ pull-up ($R_5$) to $+3.3\text{V}$. |
| **GPIO 8** | **Status LED** | Onboard ESP32-C3 blue LED | Calm breathing pulse during broadcast; dark in standby. |
| **GPIO 9** | **BOOT Button** | Onboard ESP32-C3 tactile switch | Internal pull-up to $+3.3\text{V}$. |

---

## 📁 Directory Structure

```
pcb/
├── smd/                            # SMT Turnkey Carrier Board (58×36 mm)
│   ├── MultiBand6_Beacon.kicad_pro
│   ├── MultiBand6_Beacon.kicad_sch
│   ├── MultiBand6_Beacon.kicad_pcb
│   ├── MultiBand6_Beacon_Gerbers.zip   # JLCPCB manufacturing archive
│   ├── MultiBand6_Beacon-drc.rpt       # Clean DRC report (0 violations)
│   ├── make_pcb.py                     # Python generator script
│   ├── MultiBand6_Beacon_Top.png
│   ├── MultiBand6_Beacon_3D_Isometric.png
│   ├── README.md                       # SMT ordering & assembly guide
│   ├── bom.csv                         # BOM with LCSC Basic Part numbers
│   └── positions.csv                   # Centroid CPL pick-and-place file
│
├── hole-through/                   # 100% Through-Hole DIY Board (70×46 mm)
│   ├── MultiBand6_Beacon_THT.kicad_pro
│   ├── MultiBand6_Beacon_THT.kicad_sch
│   ├── MultiBand6_Beacon_THT.kicad_pcb
│   ├── MultiBand6_Beacon_THT_Gerbers.zip # JLCPCB manufacturing archive
│   ├── MultiBand6_Beacon_THT-drc.rpt     # Clean DRC report (0 violations)
│   ├── make_pcb.py                       # Python generator script
│   ├── MultiBand6_Beacon_THT_Top.png
│   ├── MultiBand6_Beacon_THT_3D_Isometric.png
│   ├── bom.csv                           # Complete THT parts list
│   └── README.md                         # DIY build guide & transistor pinout note
│
└── breadboard/                     # Solderless Breadboard Prototyping Layout
    ├── breadboard_layout.html            # Interactive visual diagram
    ├── breadboard_layout.svg             # Full vector graphic wiring diagram
    └── README.md                         # Breadboard wiring guide & row mapping
```

---

## 🚀 Quick Getting Started Guide

1. **For Turnkey Assembled Boards:**
   * Go to [`pcb/smd/`](smd/).
   * Upload `MultiBand6_Beacon_Gerbers.zip` to [JLCPCB](https://jlcpcb.com/).
   * Enable **PCB Assembly** and upload `bom.csv` and `positions.csv`.
   * All 11 SMD parts (including the tactile sync button) incur **$0.00 setup fees** (100% Basic Parts).
2. **For Hand-Soldered DIY Boards:**
   * Go to [`pcb/hole-through/`](hole-through/).
   * Upload `MultiBand6_Beacon_THT_Gerbers.zip` to [JLCPCB](https://jlcpcb.com/).
   * Select 2-layer, 1.6 mm thickness, HASL finish. No SMT assembly needed.
   * Hand solder using standard axial passives, TO-92 transistor, and M3 hardware.
3. **For Solderless Breadboard Prototyping:**
   * Go to [`pcb/breadboard/`](breadboard/).
   * Open [`breadboard_layout.html`](breadboard/breadboard_layout.html) in your browser.
   * Follow the step-by-step row mapping to wire up the circuit in under 5 minutes without soldering.

---

## 📜 Hardware License & Commercial Terms

All hardware design files, schematics, PCB layouts, Gerbers, and BOMs in this directory are licensed under the **[CERN Open Hardware Licence Version 2 - Strongly Reciprocal (CERN-OHL-S v2)](../LICENSES/CERN-OHL-S-v2.txt)**.

* **Free for Makers & Hobbyists:** You may inspect, modify, order, and assemble boards for personal or educational use.
* **Reciprocal Copyleft:** Any commercial distribution or manufacturing of derivative or modified hardware designs must publish the complete corresponding hardware source files under the CERN-OHL-S v2.
* **Commercial Licensing:** If your organization wishes to manufacture or sell pre-assembled devices under proprietary/closed-source terms, commercial licenses are available directly from the copyright holder ([@thomaslr](https://github.com/thomaslr)). See [`LICENSE`](../LICENSE) for details.
