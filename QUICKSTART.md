# ⚡ Quick Start Guide: MultiBand 6 ESP32 Beacon

Get your Casio G-Shock or radio-controlled atomic clock syncing in **under 5 minutes**. 

You do **not** need to install Rust, Python, Git, or any software. Everything can be flashed directly from **Google Chrome** or **Microsoft Edge**.

---

## 🛠️ 1. What You Need

1. **ESP32-C3 SuperMini** board (~$2 on AliExpress/Amazon).
2. **USB-C Data Cable** *(make sure it is a data cable, not charge-only)*.
3. **Antenna Inductor:** 
   * **Minimalist Bench Test:** 16×18 mm 3.5 mH radial drum inductor (or direct wire loop) connected between **GPIO 1** and **GND** (see [schematics](README.md#2-hardware-circuit-schematics--pinout)).
   * **Full Range (1.0 m+):** The C1815 booster circuit or custom [PCB carrier](pcb/README.md).
4. A computer with **Google Chrome**, **Microsoft Edge**, or **Brave** browser.

> [!WARNING]
> **Antenna Pin Rule:** Always connect your antenna circuit to **GPIO 1** (`IO1`).  
> **Never connect anything to GPIO 2:** On the ESP32-C3, GPIO 2 is a hardware strapping pin. Grounding it locks the board and prevents flashing or booting!

---

## 📥 2. Download the Factory Firmware

Go to the **[Latest GitHub Releases](https://github.com/robertlongbottom/MultiBand6-ESP32-AtomicBeacon/releases/latest)** and download:

$$\mathbf{multiband6\text{-}beacon\text{-}factory.bin}$$

> [!TIP]
> **Which file do I download?**
> * **`multiband6-beacon-factory.bin`** *(~1.6 MB)*: **Download this one for initial setup.** It contains the bootloader, Dual-OTA partition table, and the full app.
> * `multiband6-beacon.bin` *(~1.5 MB)*: Used only later for over-the-air (OTA) Wi-Fi updates from the web dashboard.

---

## ⚡ 3. Flash Your ESP32-C3 SuperMini

### 🌟 Option A: 1-Click Online Web Installer (Easiest — No Download Needed!)

If you are using **Google Chrome** or **Microsoft Edge**, you don't even need to download files:

👉 **[Open the TimeStation 1-Click Web Installer](https://robertlongbottom.github.io/MultiBand6-ESP32-AtomicBeacon/)**

1. Plug your ESP32-C3 SuperMini into USB.
2. Put the board into Download Boot Mode: Hold **`BOOT`** $\rightarrow$ tap **`RST`** $\rightarrow$ release **`BOOT`**.
3. Click **"Install TimeStation Beacon"**, select your serial port, and wait for the progress bar to finish.
4. When finished, you can configure your Wi-Fi directly in your browser!

---

### Option B: Manual Web Flasher (Espressif esptool-js)

1. **Plug the ESP32-C3 SuperMini into your computer** using your USB-C data cable.
2. **Put the board into Download Boot Mode:**
   * Press and hold the **`BOOT`** button (labeled `B`).
   * Press and release the **`RST`** button (labeled `R`).
   * Release the **`BOOT`** button.
   *(The onboard blue LED will turn off; the chip is now waiting for firmware).*
3. **Open the Web Flasher:**
   * In Chrome or Edge, go to the official **[Espressif Web Flasher (esptool-js)](https://espressif.github.io/esptool-js/)** *(or [Adafruit WebSerial ESPTool](https://adafruit.github.io/Adafruit_WebSerial_ESPTool/))*.
4. **Connect:**
   * Set Baud Rate to **`460800`**.
   * Click **Connect**.
   * In the browser popup, select your board (e.g. `USB JTAG/serial debug unit` or `usbmodem...`) and click **Connect**.
5. **Program:**
   * In the file list, add a row with:
     * **Flash Address:** `0x0` (or `0x0000`)
     * **File:** Select your downloaded `multiband6-beacon-factory.bin`.
   * Click **Program / Flash**.
6. **Reboot:**
   * When the progress bar hits 100%, press the **`RST`** button on your ESP32-C3 SuperMini once.

---

### Option C: Flash via Terminal (For CLI Users)

If you have Python or `espflash` installed on your computer:

```bash
# Using Python esptool:
pip install esptool
esptool.py --chip esp32c3 write_flash 0x0 multiband6-beacon-factory.bin

# Or using Rust espflash:
espflash write-bin 0x0 multiband6-beacon-factory.bin
```

---

## 📶 4. Connect to Wi-Fi (Setup Portal)

> [!IMPORTANT]
> **Why Wi-Fi & NTP are Essential:**  
> The ESP32-C3 requires an active 2.4 GHz Wi-Fi connection to discipline its clock via Network Time Protocol (NTP) servers (`pool.ntp.org`). MultiBand 6 protocols transmit live date and time frames down to the exact second. The beacon must be positioned within range of your home Wi-Fi network so it can synchronize time on boot and before nightly broadcasts.

When first powered on, the beacon creates its own Wi-Fi hotspot:

1. On your smartphone or laptop, open Wi-Fi settings and connect to:
   $$\mathbf{TimeStation\text{-}Setup\text{-}XXXX}$$
   *(where `XXXX` is unique to your board).*
2. A setup portal will open automatically. If it doesn't, browse to:
   **[http://192.168.4.1](http://192.168.4.1)**
3. In the setup portal:
   * Select your home **2.4 GHz Wi-Fi network** from the list and enter the password.
   * Choose your local **Atomic Time Station**:
     * **WWVB** — USA & Canada (60 kHz)
     * **DCF77** — Germany & Central Europe (77.5 kHz)
     * **MSF** — United Kingdom (60 kHz)
     * **JJY40 / JJY60** — Japan (Fukushima / Kyushu)
     * **BPC** — China & East Asia (68.5 kHz)
     * **Carousel Mode** — Rotates between selected stations automatically!
   * Click **Save & Connect**.
4. The ESP32 will reboot and connect to your home Wi-Fi.

---

## 🌐 5. Open the Web Dashboard

Open any browser on your home network and go to:

$$\mathbf{http://timestation.local}$$

*(If your router does not support mDNS, check your router's client list for the ESP32's IP address).*

You will see the live status dashboard:
* **Current Atomic Time:** Disciplined by Stratum-1 NTP.
* **Active Station & Carousel:** Live frequency, modulation bitstream, and countdown.
* **Nightly Auto-Broadcast:** Pre-configured for nighttime calibration (default 2:00 AM) while you sleep.

---

## ⌚ 6. Sync Your Watch!

1. Place your watch near the inductor antenna:
   * Casio G-Shocks usually have their internal ferrite rod antenna at the **12 o'clock** position (top of the watch face) or **9 o'clock**.
   * Aim the watch's internal antenna toward the inductor.
2. In the web dashboard, click **"Broadcast Now"** to start a manual test transmission (or press the hardware button on **GPIO 3**).
3. Trigger manual sync on your watch:
   * On most Casios, hold the bottom-right button until you hear a beep and see **`RC!`** or **`READY`** flashing on the display.
   * Within 2–5 minutes, the watch display will indicate **`GET`** or show the receiving signal level **`L1 / L2 / L3`**.
   * The watch display will confirm successful reception with a happy beep and updated date/time!

> [!TIP]
> **Universal Radio Watch Compatibility Notice (Casio, Citizen, Seiko, Timex, etc.):**  
> While Casio MultiBand 6 watches are used as the primary reference throughout this guide (as that is what I own and am most familiar with), this beacon broadcasts authentic low-frequency time signals compatible with **every radio-controlled watch**—including **Citizen Eco-Drive**, **Seiko Radio Wave**, **Timex**, **Junghans**, and standalone atomic desk/wall clocks.  
> 
> If you live in Singapore, Australia, or another region outside official atomic towers, your watch may have radio sync disabled by default! See our complete **[Watch Compatibility & Setup Guide](CASIO_GUIDE.md)** to learn how regional radio-locks work, how to configure compatible Home Cities (`HKG`, `TYO`, etc.), and how to check your brand's specific manual for manual sync procedures.

---

## ❓ Troubleshooting

| Issue | Solution |
| :--- | :--- |
| **Browser can't find serial port** | Ensure you are using a **data cable**. Many USB-C phone cables only deliver 5V power and have no data lines. |
| **Flasher times out waiting for packet header** | Manually force boot mode: Hold **`BOOT`**, press and release **`RST`**, then release **`BOOT`**. Try connecting again. |
| **Board enters boot loop after connecting antenna** | Check your wiring. The antenna carrier must be on **GPIO 1**. If connected to **GPIO 2**, the ESP32 cannot boot. |
| **Watch flashes `ERR` or won't sync** | 1. Ensure your watch's **Home City** matches your selected station (e.g. `NYC`/`LAX` for WWVB, `BER`/`PAR` for DCF77, `LON` for MSF, `TYO` for JJY, `HKG` for BPC).<br>2. Keep the watch within 0.5–1.0 m of the inductor and away from PC monitors or switching power bricks.<br>3. Verify the beacon has synced with NTP (check `http://timestation.local`). |
| **How do I update firmware later?** | You never need to plug it into USB again! Go to the **Firmware & OTA** tab on `http://timestation.local` and either click **Update from GitHub** or drag-and-drop the latest `multiband6-beacon.bin`. The Dual-OTA system updates safely over Wi-Fi with automatic crash rollback. |

---

*Need advanced schematics, PCB ordering files, or want to compile from Rust source? See the full [README.md](README.md).*
