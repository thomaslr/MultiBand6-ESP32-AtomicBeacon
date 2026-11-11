# MultiBand 6 ESP32 Atomic Beacon — Licensing & Usage Terms

Copyright (c) 2026 Robert Longbottom (@thomaslr)

This project is released under a **Dual-License / Source-Available Model**:
* **100% Free for Makers, Hobbyists, Students, and Tinkerers** (Personal & Non-Commercial Use).
* **Commercial Use, Resale, or For-Profit Manufacturing Requires a Paid Commercial License.**

---

## 🧭 Summary of Terms

| Usage Type | Permissions & Conditions | License Terms |
| :--- | :--- | :--- |
| **Hobbyists & Makers** | Free to build, order PCBs from JLCPCB/PCBWay, hand-solder, flash firmware, modify code, and tinker for personal, home, or educational use. | **Free & Permitted** (under non-commercial terms below) |
| **Open Sharing & Forks** | Permitted with attribution. Derivative hobbyist PCB designs or software improvements must be shared under the same non-commercial terms (ShareAlike). | **Free & Permitted** (with attribution & CC BY-NC-SA / PolyForm) |
| **Commercial Sales** | Selling pre-assembled PCBs, DIY kits, pre-flashed hardware, commercial enclosures, or incorporating into commercial products. | **PROHIBITED** without a separate Commercial License |
| **Enterprise / Business Deployment** | Operating beacons inside commercial facilities, retail stores, or for-profit clock service operations. | **PROHIBITED** without a separate Commercial License |

---

## Part 1: Hardware Designs, Schematics & PCB Layouts

All printed circuit board (PCB) layouts (KiCad files in `pcb/smd/`, `pcb/hole-through/`, `pcb/breadboard/`), schematics, Gerber files, BOMs, CPL centroid files, 3D models, and hardware documentation are licensed under the **Creative Commons Attribution-NonCommercial-ShareAlike 4.0 International License (CC BY-NC-SA 4.0)**.

### Summary of CC BY-NC-SA 4.0:
* **Attribution:** You must give appropriate credit to Robert Longbottom (@thomaslr), provide a link to the original repository (<https://github.com/robertlongbottom/MultiBand6-ESP32-AtomicBeacon>), and indicate if changes were made.
* **NonCommercial:** You may **not** use the material for commercial purposes (including selling PCBs, kits, or finished devices on Tindie, Etsy, eBay, AliExpress, Amazon, or via private sale).
* **ShareAlike:** If you remix, transform, or build upon the material, you must distribute your contributions under the same license as the original.
* **Full Legal Text:** <https://creativecommons.org/licenses/by-nc-sa/4.0/legalcode>

---

## Part 2: Firmware & Software Source Code

All firmware source code (Rust crates in `firmware-rust/`), web interface assets (`data/`), build tools, scripts, and automation workflows are licensed under the **PolyForm Noncommercial License 1.0.0**.

### PolyForm Noncommercial License 1.0.0 Text:
<https://polyformproject.org/licenses/noncommercial/1.0.0>

### 1. Purpose
This license allows you to use the software for noncommercial purposes.

### 2. Agreement
In order to receive this license, you have to agree to its rules. Those rules are both terms of this license and conditions to each of the licenses granted in this license.

### 3. Notices
You must ensure that everyone who gets a copy of any part of this software from you, with or without changes, also gets the text of this license or a link to <https://polyformproject.org/licenses/noncommercial/1.0.0>.

### 4. Noncommercial License
The licensor grants you a copyright license to do everything with the software that would otherwise infringe the licensor's copyright in it, for noncommercial purposes.

### 5. Commercial License
You may not use the software for commercial purposes. To use the software for commercial purposes, you must obtain a separate license from the licensor.

### 6. Definitions
* The **"licensor"** is Robert Longbottom (@thomaslr).
* The **"software"** is the software the licensor makes available under this license, including documentation or other related text or media.
* **"You"** refers to the individual or entity taking advantage of this license.
* **"Noncommercial purposes"** means personal use, education, academic research, and non-commercial hobbyist experimentation. Noncommercial purposes does not include any use for commercial advantage, monetary compensation, or in connection with any business or commercial enterprise.
* **"Changes"** are additions, deletions, or other modifications to the software.

### 7. Patent License
The licensor grants you a patent license to make, have made, use, and import the software, as well as to sell and offer for sale changes you make, for noncommercial purposes.

### 8. Fair Use & Legal Exceptions
You may use the software according to fair use or other lawful exceptions to copyright.

### 9. Disclaimer of Warranties
**THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.**

---

## Part 3: Commercial Licensing Inquiries

If you are a manufacturer, electronics distributor, watch repair service, or business interested in:
* Selling turnkey assembled PCBs or DIY kits
* Pre-flashing and distributing physical hardware
* Embedding this multi-station atomic clock simulator into a commercial product
* Commercial volume licensing or custom hardware engineering

Please contact the author to negotiate a commercial license agreement:
* **Contact / Commercial Inquiries:** Open an issue or inquiry via **[GitHub Discussions](https://github.com/robertlongbottom/MultiBand6-ESP32-AtomicBeacon/discussions)** or email the repository owner.

---

## Part 4: Third-Party Attributions & Open Source Credits

This project includes and builds upon the following open-source works:

1. **Time Signal Protocols & Algorithms:**
   * Adapted from the open-source **[timestation](https://github.com/kangtastic/timestation)** project by **James Seo** (`james@equiv.tech`), licensed under the **MIT License**:
     ```
     Copyright (c) 2023 James Seo <james@equiv.tech>
     Permission is hereby granted, free of charge, to any person obtaining a copy of this software... (MIT License)
     ```
2. **Date & Calendar Computations:**
   * Gregorian calendar date algorithms by **Howard Hinnant** (Public Domain / Boost Software License).
3. **ESP-IDF Framework & Rust Bindings:**
   * `esp-idf-svc`, `esp-idf-hal`, and `esp-idf-sys` developed by Espressif Systems and the Rust on ESP Community, licensed under **Apache-2.0 / MIT**.
