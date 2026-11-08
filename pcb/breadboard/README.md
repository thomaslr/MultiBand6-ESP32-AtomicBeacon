# MultiBand 6 ESP32 Atomic Beacon — Solderless Breadboard Prototyping

This directory contains the visual layout and wiring documentation for building the **MultiBand 6 ESP32 Atomic Clock Beacon** on a standard solderless breadboard.

---

## 🎨 Visual Diagram & Interactive Preview

* **Interactive HTML Preview:** Open [`breadboard_layout.html`](breadboard_layout.html) in any web browser.
* **Vector Graphic (SVG):** View [`breadboard_layout.svg`](breadboard_layout.svg) directly in GitHub, VS Code, or an image viewer.

---

## 📌 Circuit Overview & Boot-Safe Pinout

* **MCU:** ESP32-C3 SuperMini mounted across the center ravine (Rows 1 to 8).
* **RF Output:** **GPIO 1** (Pin F7 on ESP32-C3 SuperMini) $\rightarrow$ 100% boot-safe and flash-safe.
* **Switching Transistor ($Q_1$):** 2SC1815 (JIS pinout: Emitter-Collector-Base) or 2N2222 (E-B-C pinout with swapped Base/Collector).
* **LC Tank ($L_1, C_1$):** 16×18 mm 3.5 mH drum core inductor in parallel with 1.5 nF capacitor across Rows 12 and 16.
* **Base Resistor ($R_1$):** $1\text{ k}\Omega$ spanning directly from Row 7 (GPIO 1) to Row 11 (C1815 Base).
* **Damping Resistor ($R_2$):** $220\,\Omega$ connected from Row 16 (LC Tank high side) to the (+) 5V power rail.

---

## 📋 Row Mapping Table (C1815 NPN)

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
   06:    [ . . . . . ] ── [ ravine ] ── [ IO2 ] [ . . . . ]
   07:    [ . . . . . ] ── [        ] ── [ IO1 ] [ . . . . ] ── [ 1k R_base ]
   08:    [ . . . . . ] ── [        ] ── [ IO0 ] [ . . . . ]          │
                                                                      │
   11:    [ . . . . . ]                  [ . . . . . ] ── C1815 BASE (B, Pin 3) ◄─┘ (Closest to IO1!)
   12:    [ . . . . . ]                  [ . . . . . ] ── C1815 COLLECTOR (C, Pin 2)
                                               │
                                       ┌───────┴───────────────┐
                                       │ 16x18 3.5mH I-Type Ind│  (Parallel LC Tank: Rows 12–16)
                                       │ 1.5nF Capacitor       │  (Col F: Cap, Col H: Inductor)
                                       └───────┬───────────────┘
                                               │
   13:    [ . . . . . ]                  [ . . . . . ] ── C1815 EMITTER (E, Pin 1) ── Wire to (-) Blue GND
   ...
   16:    [ . . . . . ]                  [ . . . . . ] ── LC Tank High Side
                                               │
                                         [ 220 ohm ]   ── R_damp connects directly
                                               │          from Row 16 to (+) 5V Rail
  ────────────────────────────────────────────────────────────────────────
```

> [!WARNING]
> **Transistor Pinout Warning:** The Japanese JIS pinout for the 2SC1815 is **Emitter (1), Collector (2), Base (3)** facing the flat side with leads pointing down. If you substitute a western 2N2222 / PN2222 (which has an **Emitter (1), Base (2), Collector (3)** pinout), you must route the $1\text{ k}\Omega$ base resistor to the middle pin (Base) and the LC tank to the right pin (Collector).
