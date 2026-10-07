/**
 * ESP32 Worldwide Radio Atomic Clock Simulator
 * Supports BPC, WWVB, MSF, DCF77, JJY40, JJY60
 * 
 * Hardware:
 *   - GPIO 2: Antenna Output (LEDC PWM Carrier)
 *   - Optional NPN Transistor Booster or Direct GPIO with R_damp (220 ohm)
 */

#include <Arduino.h>
#include <WiFi.h>
#include <WebServer.h>
#include <DNSServer.h>
#include <ESPmDNS.h>
#include <Preferences.h>
#include <time.h>
#include <esp_sleep.h>
#include <esp_timer.h>
#include "TimeProtocols.h"
#include "WebPage.h"

// --- Compile-Time Fallbacks (.env via scripts/load_env.py) ---
#ifndef DEFAULT_WIFI_SSID
  #ifdef WIFI_SSID
    #define DEFAULT_WIFI_SSID WIFI_SSID
  #else
    #define DEFAULT_WIFI_SSID ""
  #endif
#endif

#ifndef DEFAULT_WIFI_PASS
  #ifdef WIFI_PASSWORD
    #define DEFAULT_WIFI_PASS WIFI_PASSWORD
  #else
    #define DEFAULT_WIFI_PASS ""
  #endif
#endif

#ifndef DEFAULT_NTP_SERVER
  #ifdef NTP_SERVER
    #define DEFAULT_NTP_SERVER NTP_SERVER
  #else
    #define DEFAULT_NTP_SERVER "pool.ntp.org"
  #endif
#endif

// --- Pin Definitions ---
#ifndef ANTENNA_PIN
  #ifdef GPIO_PIN
    #define ANTENNA_PIN GPIO_PIN
  #else
    #define ANTENNA_PIN 2
  #endif
#endif
#define PWM_CHANNEL      0
#define PWM_RESOLUTION   8

// --- Onboard Status LED (ESP32-C3 SuperMini: GPIO 8, Active LOW) ---
#ifndef STATUS_LED_PIN
  #define STATUS_LED_PIN       8
#endif
#define LED_PWM_CHANNEL      1
#define LED_PWM_FREQ         1000  // 1 kHz flicker-free PWM
#define LED_PWM_RESOLUTION   8     // 8-bit (0-255)
#define LED_BREATHE_PERIOD   4000  // 4.0 seconds calm breathing cycle
#define LED_MAX_BRIGHTNESS   75    // Capped at ~30% peak brightness for nightstand comfort

#ifndef DEFAULT_STATION
  #define DEFAULT_STATION 0 // 0 = BPC (China 68.5 kHz)
#endif

#ifndef DEFAULT_OFFSET_HOURS
  #define DEFAULT_OFFSET_HOURS 0
#endif

#ifndef DEFAULT_SCHEDULE_ENABLED
  #define DEFAULT_SCHEDULE_ENABLED 1
#endif

#ifndef DEFAULT_BROADCAST_HOUR
  #define DEFAULT_BROADCAST_HOUR 2 // 02:00 AM
#endif

#ifndef DEFAULT_BROADCAST_MINUTE
  #define DEFAULT_BROADCAST_MINUTE 0
#endif

#ifndef DEFAULT_BROADCAST_DURATION
  #define DEFAULT_BROADCAST_DURATION 25 // 25 minutes
#endif

#ifndef DEFAULT_CAROUSEL_ENABLED
  #define DEFAULT_CAROUSEL_ENABLED 0
#endif
#ifndef DEFAULT_CAROUSEL_MASK
  #define DEFAULT_CAROUSEL_MASK 0x23 // BPC (0), WWVB (1), JJY60 (5)
#endif
#ifndef DEFAULT_CAROUSEL_DURATION
  #define DEFAULT_CAROUSEL_DURATION 15 // 15 minutes per station
#endif

// --- Preferences / NVS Storage ---
Preferences prefs;

// --- Config State ---
time_station_t activeStation = (time_station_t)DEFAULT_STATION;
int32_t userOffsetHours = DEFAULT_OFFSET_HOURS;
bool scheduleEnabled = (DEFAULT_SCHEDULE_ENABLED != 0);
int broadcastHour = DEFAULT_BROADCAST_HOUR;
int broadcastMinute = DEFAULT_BROADCAST_MINUTE;
int broadcastDurationMin = DEFAULT_BROADCAST_DURATION;
String wifiSsid = "";
String wifiPassword = "";
String ntpServer = "pool.ntp.org";

// --- Carousel Config State ---
bool carouselEnabled = (DEFAULT_CAROUSEL_ENABLED != 0);
uint8_t carouselMask = DEFAULT_CAROUSEL_MASK;
int carouselDurationPerStationMin = DEFAULT_CAROUSEL_DURATION;

// --- Carousel Runtime State ---
time_station_t carouselStations[STATION_COUNT];
int carouselStageCount = 1;
volatile int currentStageIndex = 0;
uint32_t stageStartSec = 0;
uint32_t stageDurationSec = 0;

// --- Runtime State ---
WebServer server(80);
DNSServer dnsServer;
bool isAPMode = false;
volatile bool isTransmitting = false;

// Transmission Buffers
uint8_t tickBuffer[TIMEPROTO_MAX_LEVEL_BYTES];
uint16_t totalFrameTicks = 1200;
volatile uint16_t currentTickIndex = 0;
uint64_t transmitStartSec = 0;
uint64_t transmitDurationSec = 0;

// High Resolution Timer for 50ms Ticks
esp_timer_handle_t tickTimer = nullptr;

// Forward Declarations
void buildCarouselStationList();
void startTransmission(uint32_t durationSec);
void stopTransmission();
void setupLEDC(uint32_t freqHz);
void onTickTimer(void* arg);
void scheduleNextDeepSleep();
void syncNtpTime();
void handleRoot();
void handleStatus();
void handleSave();
void handleTransmit();
void handleStop();
void updateBreathingLED();

// ============================================================================
// Setup
// ============================================================================
void setup() {
    Serial.begin(115200);
    delay(500);
    Serial.println("\n==================================================");
    Serial.println("  ESP32-C3 Radio Atomic Clock Simulator Booting");
    Serial.println("==================================================");

    // 1. Initialize Antenna Pin (Carrier Off initially)
    pinMode(ANTENNA_PIN, OUTPUT);
    digitalWrite(ANTENNA_PIN, LOW);

    // Initialize Onboard Status LED (GPIO 8, Active LOW: 255 = OFF)
    ledcSetup(LED_PWM_CHANNEL, LED_PWM_FREQ, LED_PWM_RESOLUTION);
    ledcAttachPin(STATUS_LED_PIN, LED_PWM_CHANNEL);
    ledcWrite(LED_PWM_CHANNEL, 255); // 100% OFF in standby

    // 2. Load Settings from Flash (NVS)
    prefs.begin("timestation", false);
    activeStation = (time_station_t)prefs.getInt("station", DEFAULT_STATION);
    userOffsetHours = prefs.getInt("offsetH", DEFAULT_OFFSET_HOURS);
    scheduleEnabled = prefs.getBool("schedEn", (DEFAULT_SCHEDULE_ENABLED != 0));
    broadcastHour = prefs.getInt("bcHour", DEFAULT_BROADCAST_HOUR);
    broadcastMinute = prefs.getInt("bcMin", DEFAULT_BROADCAST_MINUTE);
    broadcastDurationMin = prefs.getInt("bcDur", DEFAULT_BROADCAST_DURATION);
    wifiSsid = prefs.getString("ssid", DEFAULT_WIFI_SSID);
    wifiPassword = prefs.getString("pass", DEFAULT_WIFI_PASS);
    ntpServer = prefs.getString("ntp", DEFAULT_NTP_SERVER);
    carouselEnabled = prefs.getBool("carEn", (DEFAULT_CAROUSEL_ENABLED != 0));
    carouselMask = prefs.getUInt("carMask", DEFAULT_CAROUSEL_MASK);
    carouselDurationPerStationMin = prefs.getInt("carDur", DEFAULT_CAROUSEL_DURATION);

    // Fall back to compile-time .env defaults if NVS credentials are blank
    if (wifiSsid.length() == 0 && String(DEFAULT_WIFI_SSID).length() > 0) {
        wifiSsid = DEFAULT_WIFI_SSID;
        wifiPassword = DEFAULT_WIFI_PASS;
    }

    Serial.printf("[Config] Station: %s\n", timeproto_get_station_name(activeStation));
    Serial.printf("[Config] Antenna Pin: GPIO %d\n", ANTENNA_PIN);
    Serial.printf("[Config] Carousel Mode: %s (Mask: 0x%02X, %d min/station)\n",
                  carouselEnabled ? "ENABLED" : "Disabled", carouselMask, carouselDurationPerStationMin);
    Serial.printf("[Config] Wi-Fi Target: '%s'\n", wifiSsid.c_str());
    Serial.printf("[Config] Schedule: %02d:%02d, %d min (Enabled: %d)\n",
                  broadcastHour, broadcastMinute, broadcastDurationMin, scheduleEnabled);

    // 3. Check Wakeup Reason
    esp_sleep_wakeup_cause_t wakeup_reason = esp_sleep_get_wakeup_cause();
    Serial.printf("[Power] Wakeup Cause: %d\n", wakeup_reason);

    // If woken by timer for a scheduled broadcast:
    if (wakeup_reason == ESP_SLEEP_WAKEUP_TIMER && scheduleEnabled) {
        Serial.println("[Schedule] Woken by RTC Timer for scheduled broadcast!");
        
        // Connect Wi-Fi & sync NTP
        syncNtpTime();

        // CRITICAL: Shut OFF Wi-Fi before RF transmission to kill 2.4 GHz noise!
        Serial.println("[RF Safety] Shutting off Wi-Fi radio to eliminate RF hash...");
        WiFi.disconnect(true);
        WiFi.mode(WIFI_OFF);
        delay(100);

        // Start broadcast
        startTransmission(broadcastDurationMin * 60);

        // Run transmission loop until duration expires
        while (isTransmitting) {
            updateBreathingLED();
            delay(20);
            time_t nowSec = time(nullptr);
            if (nowSec - transmitStartSec >= transmitDurationSec) {
                Serial.println("[Schedule] Broadcast window finished!");
                stopTransmission();
                break;
            }
        }

        // Enter deep sleep until next daily schedule
        scheduleNextDeepSleep();
        return;
    }

    // 4. Normal Boot: Connect to Wi-Fi or Start AP Mode
    if (wifiSsid.length() > 0) {
        Serial.printf("[WiFi] Connecting to '%s'...\n", wifiSsid.c_str());
        
        WiFi.mode(WIFI_STA);
        WiFi.setAutoReconnect(true);
        WiFi.begin(wifiSsid.c_str(), wifiPassword.c_str());
        
        int retries = 0;
        while (WiFi.status() != WL_CONNECTED && retries < 30) {
            delay(500);
            Serial.print(".");
            retries++;
            if (retries == 15) {
                // If router had a stale session from a reset, re-trigger
                Serial.print(" [re-associating] ");
                WiFi.disconnect();
                delay(200);
                WiFi.begin(wifiSsid.c_str(), wifiPassword.c_str());
            }
        }

        if (WiFi.status() == WL_CONNECTED) {
            WiFi.setSleep(true); // Enable 802.11 Modem-Sleep for lowest power consumption
            setCpuFrequencyMhz(80); // Drop CPU clock to 80 MHz for power savings
            Serial.printf("\n[WiFi] Connected! IP Address: %s\n", WiFi.localIP().toString().c_str());
            Serial.println("[Power] 80 MHz CPU & Wi-Fi Modem-Sleep active (ultra-low-power standby).");
            if (MDNS.begin("timestation")) {
                MDNS.addService("http", "tcp", 80);
                Serial.println("[mDNS] Responder active at: http://timestation.local");
            }
            syncNtpTime();
        } else {
            Serial.println("\n[WiFi] Connection timed out. Falling back to SoftAP mode.");
            isAPMode = true;
        }
    } else {
        Serial.println("[WiFi] No saved Wi-Fi credentials. Starting Setup AP.");
        isAPMode = true;
    }

    if (isAPMode) {
        WiFi.mode(WIFI_AP);
        WiFi.softAP("TimeStation-Setup");
        Serial.printf("[AP] Access Point started: 'TimeStation-Setup'\n");
        Serial.printf("[AP] Open your browser to: http://%s\n", WiFi.softAPIP().toString().c_str());
        dnsServer.start(53, "*", WiFi.softAPIP());
    }

    // 5. Start Web Server
    server.on("/", HTTP_GET, handleRoot);
    server.on("/api/status", HTTP_GET, handleStatus);
    server.on("/api/save", HTTP_POST, handleSave);
    server.on("/api/transmit", HTTP_POST, handleTransmit);
    server.on("/api/stop", HTTP_POST, handleStop);
    server.begin();
    Serial.println("[HTTP] Web server started on port 80.");
}

// ============================================================================
// Loop
// ============================================================================
static int lastScheduledBroadcastDay = -1;

void loop() {
    if (isAPMode) {
        dnsServer.processNextRequest();
    }
    server.handleClient();

    // Check if test or scheduled transmission has timed out
    if (isTransmitting && transmitDurationSec > 0) {
        time_t nowSec = time(nullptr);
        if (nowSec - transmitStartSec >= transmitDurationSec) {
            Serial.println("[Transmitter] Broadcast duration completed.");
            stopTransmission();
        }
    }

    // Check daily scheduled broadcast (runs in low-power standby without sleeping)
    if (scheduleEnabled && !isTransmitting && !isAPMode) {
        time_t now = time(nullptr);
        if (now > 1700000000) { // Valid NTP epoch
            time_t localTime = now + timeproto_get_native_utc_offset(activeStation) + (userOffsetHours * 3600);
            struct tm* tm_local = gmtime(&localTime);
            if (tm_local && tm_local->tm_hour == broadcastHour && 
                tm_local->tm_min == broadcastMinute && 
                tm_local->tm_yday != lastScheduledBroadcastDay) {
                
                lastScheduledBroadcastDay = tm_local->tm_yday;
                if (carouselEnabled) {
                    buildCarouselStationList();
                    int totalMin = carouselStageCount * carouselDurationPerStationMin;
                    Serial.printf("[Schedule] Starting scheduled Carousel broadcast (%d min, %d stages) at %02d:%02d...\n",
                                  totalMin, carouselStageCount, broadcastHour, broadcastMinute);
                    startTransmission(totalMin * 60);
                } else {
                    Serial.printf("[Schedule] Starting scheduled broadcast (%d min) for %s at %02d:%02d...\n",
                                  broadcastDurationMin, timeproto_get_station_name(activeStation),
                                  broadcastHour, broadcastMinute);
                    startTransmission(broadcastDurationMin * 60);
                }
            }
        }
    }

    updateBreathingLED();
    delay(20); // Yields CPU to FreeRTOS power-saving idle thread
}

// ============================================================================
// NTP Time Synchronization
// ============================================================================
void syncNtpTime() {
    Serial.printf("[NTP] Fetching time from %s...\n", ntpServer.c_str());
    configTime(0, 0, ntpServer.c_str());

    time_t now = time(nullptr);
    int retries = 0;
    while (now < 1700000000 && retries < 25) { // Year > 2023
        delay(300);
        now = time(nullptr);
        retries++;
    }

    if (now > 1700000000) {
        struct tm* timeinfo = gmtime(&now);
        Serial.printf("[NTP] Time synchronized! Current UTC: %04d-%02d-%02d %02d:%02d:%02d\n",
                      timeinfo->tm_year + 1900, timeinfo->tm_mon + 1, timeinfo->tm_mday,
                      timeinfo->tm_hour, timeinfo->tm_min, timeinfo->tm_sec);
    } else {
        Serial.println("[NTP] Warning: Sync timed out, using internal RTC.");
    }
}

// ============================================================================
// RF Carrier Generation & 50ms Pulse Modulator
// ============================================================================
void buildCarouselStationList() {
    carouselStageCount = 0;
    for (int i = 0; i < STATION_COUNT; i++) {
        if (carouselMask & (1 << i)) {
            carouselStations[carouselStageCount++] = (time_station_t)i;
        }
    }
    if (carouselStageCount == 0) {
        carouselStations[0] = activeStation;
        carouselStageCount = 1;
    }
}

void setupLEDC(uint32_t freqHz) {
    ledcSetup(PWM_CHANNEL, freqHz, PWM_RESOLUTION);
    ledcAttachPin(ANTENNA_PIN, PWM_CHANNEL);
    ledcWrite(PWM_CHANNEL, 0); // Carrier OFF by default
}

void onTickTimer(void* arg) {
    if (!isTransmitting) return;

    // Check if we need to regenerate a new frame
    if (currentTickIndex >= totalFrameTicks) {
        time_t now = time(nullptr);

        // Check if Carousel stage handoff is due at this clean frame boundary
        if (carouselEnabled && carouselStageCount > 1) {
            if ((now - stageStartSec) >= stageDurationSec) {
                if (currentStageIndex + 1 < carouselStageCount) {
                    currentStageIndex++;
                    activeStation = carouselStations[currentStageIndex];
                    stageStartSec = now;
                    uint32_t newFreq = timeproto_get_carrier_freq(activeStation);
                    setupLEDC(newFreq);
                    Serial.printf("[Carousel] Handoff at :00 to Stage %d/%d: %s (%u Hz)\n",
                                  currentStageIndex + 1, carouselStageCount,
                                  timeproto_get_station_name(activeStation), newFreq);
                }
            }
        }

        timeproto_generate_frame(activeStation, now, userOffsetHours * 3600, 0,
                                 tickBuffer, &totalFrameTicks);
        currentTickIndex = 0;
    }

    uint8_t carrierState = timeproto_get_tick_level(tickBuffer, currentTickIndex);
    currentTickIndex++;

    if (carrierState) {
        ledcWrite(PWM_CHANNEL, 128); // 50% duty cycle (Carrier ON)
    } else {
        ledcWrite(PWM_CHANNEL, 0);   // 0% duty cycle (Carrier OFF)
    }
}

// ============================================================================
// Onboard LED Breathing Pulse (4.0s Calm Nightstand Rhythm)
// ============================================================================
void updateBreathingLED() {
    if (isTransmitting) {
        uint32_t nowMs = millis();
        // Phase ranges from 0.0 to 2*PI across the 4000ms period
        float phase = ((float)(nowMs % LED_BREATHE_PERIOD) / (float)LED_BREATHE_PERIOD) * 2.0f * PI;
        // Smooth sine wave: (1 - cos(phase)) / 2 goes 0.0 -> 1.0 -> 0.0
        float factor = (1.0f - cosf(phase)) * 0.5f;
        // Natural perceptual curve: factor^2
        uint32_t brightness = (uint32_t)(factor * factor * (float)LED_MAX_BRIGHTNESS);
        // Active LOW: 255 is completely OFF, 0 is full brightness
        ledcWrite(LED_PWM_CHANNEL, 255 - brightness);
    } else {
        // Completely dark in standby
        ledcWrite(LED_PWM_CHANNEL, 255);
    }
}

void startTransmission(uint32_t durationSec) {
    if (isTransmitting) return;

    // Maintain constant 80 MHz clock (APB bus locked at 80 MHz) and pause Wi-Fi sleep for clean 3.3V rail
    WiFi.setSleep(false);
    Serial.println("[Power] Broadcast active at 80 MHz (Wi-Fi sleep paused for rail stability).");

    time_t now = time(nullptr);

    if (carouselEnabled) {
        buildCarouselStationList();
        currentStageIndex = 0;
        activeStation = carouselStations[0];
        stageDurationSec = (uint32_t)carouselDurationPerStationMin * 60;
        durationSec = carouselStageCount * stageDurationSec;
        Serial.printf("[Carousel] Starting Carousel Mode: %d stages (%d min each, total %d min)\n",
                      carouselStageCount, carouselDurationPerStationMin, (int)(durationSec / 60));
    } else {
        carouselStageCount = 1;
        currentStageIndex = 0;
        stageDurationSec = durationSec;
    }

    uint32_t carrierFreq = timeproto_get_carrier_freq(activeStation);
    Serial.printf("[Transmitter] Starting %s carrier at %u Hz on GPIO %d\n",
                  timeproto_get_station_name(activeStation), carrierFreq, ANTENNA_PIN);

    setupLEDC(carrierFreq);

    // Generate initial frame
    timeproto_generate_frame(activeStation, now, userOffsetHours * 3600, 0,
                             tickBuffer, &totalFrameTicks);
    currentTickIndex = 0;

    transmitStartSec = now;
    stageStartSec = now;
    transmitDurationSec = durationSec;
    isTransmitting = true;

    // Start 50 ms (20 Hz) high-resolution timer
    esp_timer_create_args_t timerArgs = {
        .callback = &onTickTimer,
        .arg = nullptr,
        .dispatch_method = ESP_TIMER_TASK,
        .name = "50ms_tick"
    };
    esp_timer_create(&timerArgs, &tickTimer);
    esp_timer_start_periodic(tickTimer, 50000); // 50,000 microseconds = 50 ms
}

void stopTransmission() {
    if (!isTransmitting) return;

    isTransmitting = false;
    if (tickTimer) {
        esp_timer_stop(tickTimer);
        esp_timer_delete(tickTimer);
        tickTimer = nullptr;
    }

    ledcWrite(PWM_CHANNEL, 0);
    digitalWrite(ANTENNA_PIN, LOW);
    ledcWrite(LED_PWM_CHANNEL, 255); // Ensure LED is 100% OFF in standby
    Serial.println("[Transmitter] Transmission stopped. Carrier & LED OFF.");

    // Re-engage Wi-Fi Modem-Sleep for low-power standby (CPU stays at 80 MHz)
    WiFi.setSleep(true);
    Serial.println("[Power] 80 MHz CPU & Wi-Fi Modem-Sleep active (standby).");
}

// ============================================================================
// Deep Sleep Scheduling
// ============================================================================
void scheduleNextDeepSleep() {
    time_t now = time(nullptr);
    struct tm* tm_now = gmtime(&now);

    int currentMinutesToday = tm_now->tm_hour * 60 + tm_now->tm_min;
    int targetMinutesToday = broadcastHour * 60 + broadcastMinute;

    int minutesToSleep = targetMinutesToday - currentMinutesToday;
    if (minutesToSleep <= 0) {
        minutesToSleep += 24 * 60; // Wake up tomorrow at target time
    }

    uint64_t sleepUs = (uint64_t)minutesToSleep * 60ULL * 1000000ULL;
    Serial.printf("[Power] Entering Deep Sleep for %d minutes (%llu us). Goodnight!\n",
                  minutesToSleep, sleepUs);

    esp_sleep_enable_timer_wakeup(sleepUs);
    esp_deep_sleep_start();
}

// ============================================================================
// Web Server Route Handlers
// ============================================================================
void handleRoot() {
    server.send_P(200, "text/html", INDEX_HTML);
}

void handleStatus() {
    time_t now = time(nullptr);
    struct tm* tm_info = gmtime(&now);
    char timeStr[32];
    snprintf(timeStr, sizeof(timeStr), "%02d:%02d:%02d UTC",
             tm_info->tm_hour, tm_info->tm_min, tm_info->tm_sec);

    char nextSyncStr[32];
    snprintf(nextSyncStr, sizeof(nextSyncStr), "%02d:%02d (Daily)", broadcastHour, broadcastMinute);

    int stageRemainSec = 0;
    int totalRemainSec = 0;
    if (isTransmitting) {
        time_t nowSec = now;
        if (nowSec >= stageStartSec && stageDurationSec > 0) {
            uint32_t elapsed = nowSec - stageStartSec;
            stageRemainSec = (elapsed < stageDurationSec) ? (stageDurationSec - elapsed) : 0;
        }
        if (nowSec >= transmitStartSec && transmitDurationSec > 0) {
            uint32_t elapsed = nowSec - transmitStartSec;
            totalRemainSec = (elapsed < transmitDurationSec) ? (transmitDurationSec - elapsed) : 0;
        }
    }

    String json = "{";
    json += "\"time\":\"" + String(timeStr) + "\",";
    json += "\"station\":" + String((int)activeStation) + ",";
    json += "\"stationName\":\"" + String(timeproto_get_station_name(activeStation)) + "\",";
    json += "\"carrierHz\":" + String(timeproto_get_carrier_freq(activeStation)) + ",";
    json += "\"offsetHours\":" + String(userOffsetHours) + ",";
    json += "\"scheduleEnabled\":" + String(scheduleEnabled ? "true" : "false") + ",";
    json += "\"broadcastTime\":\"" + String(broadcastHour < 10 ? "0" : "") + String(broadcastHour) + ":" + (broadcastMinute < 10 ? "0" : "") + String(broadcastMinute) + "\",";
    json += "\"broadcastDuration\":" + String(broadcastDurationMin) + ",";
    json += "\"nextSync\":\"" + String(scheduleEnabled ? nextSyncStr : "Disabled") + "\",";
    json += "\"wifiSsid\":\"" + wifiSsid + "\",";
    json += "\"ntpServer\":\"" + ntpServer + "\",";
    json += "\"antennaPin\":" + String(ANTENNA_PIN) + ",";
    json += "\"transmitting\":" + String(isTransmitting ? "true" : "false") + ",";
    json += "\"carouselEnabled\":" + String(carouselEnabled ? "true" : "false") + ",";
    json += "\"carouselMask\":" + String(carouselMask) + ",";
    json += "\"carouselDurationMin\":" + String(carouselDurationPerStationMin) + ",";
    json += "\"carouselStage\":" + String(currentStageIndex + 1) + ",";
    json += "\"carouselTotalStages\":" + String(carouselStageCount) + ",";
    json += "\"stageRemainSec\":" + String(stageRemainSec) + ",";
    json += "\"totalRemainSec\":" + String(totalRemainSec);
    json += "}";

    server.send(200, "application/json", json);
}

void handleSave() {
    if (!server.hasArg("plain")) {
        server.send(400, "text/plain", "Bad Request");
        return;
    }

    String body = server.arg("plain");
    // Simple JSON extraction
    int stIdx = body.indexOf("\"station\":");
    if (stIdx != -1) {
        int stVal = body.substring(stIdx + 10).toInt();
        if (stVal >= 0 && stVal < STATION_COUNT) {
            activeStation = (time_station_t)stVal;
            prefs.putInt("station", stVal);
        }
    }

    int offIdx = body.indexOf("\"offsetHours\":");
    if (offIdx != -1) {
        userOffsetHours = body.substring(offIdx + 14).toInt();
        prefs.putInt("offsetH", userOffsetHours);
    }

    int schIdx = body.indexOf("\"scheduleEnabled\":");
    if (schIdx != -1) {
        scheduleEnabled = body.substring(schIdx + 18).startsWith("true");
        prefs.putBool("schedEn", scheduleEnabled);
    }

    int durIdx = body.indexOf("\"broadcastDuration\":");
    if (durIdx != -1) {
        broadcastDurationMin = body.substring(durIdx + 20).toInt();
        prefs.putInt("bcDur", broadcastDurationMin);
    }

    int timeIdx = body.indexOf("\"broadcastTime\":\"");
    if (timeIdx != -1) {
        String tStr = body.substring(timeIdx + 17, timeIdx + 22);
        broadcastHour = tStr.substring(0, 2).toInt();
        broadcastMinute = tStr.substring(3, 5).toInt();
        prefs.putInt("bcHour", broadcastHour);
        prefs.putInt("bcMin", broadcastMinute);
    }

    // Carousel settings
    int carEnIdx = body.indexOf("\"carouselEnabled\":");
    if (carEnIdx != -1) {
        carouselEnabled = body.substring(carEnIdx + 18).startsWith("true");
        prefs.putBool("carEn", carouselEnabled);
    }

    int carMaskIdx = body.indexOf("\"carouselMask\":");
    if (carMaskIdx != -1) {
        carouselMask = (uint8_t)body.substring(carMaskIdx + 15).toInt();
        prefs.putUInt("carMask", carouselMask);
    }

    int carDurIdx = body.indexOf("\"carouselDurationMin\":");
    if (carDurIdx != -1) {
        carouselDurationPerStationMin = body.substring(carDurIdx + 22).toInt();
        if (carouselDurationPerStationMin < 1) carouselDurationPerStationMin = 15;
        prefs.putInt("carDur", carouselDurationPerStationMin);
    }

    int ssidIdx = body.indexOf("\"wifiSsid\":\"");
    if (ssidIdx != -1) {
        int endSsid = body.indexOf("\"", ssidIdx + 12);
        String newSsid = body.substring(ssidIdx + 12, endSsid);
        if (newSsid.length() > 0) {
            wifiSsid = newSsid;
            prefs.putString("ssid", wifiSsid);
        }
    }

    int passIdx = body.indexOf("\"wifiPassword\":\"");
    if (passIdx != -1) {
        int endPass = body.indexOf("\"", passIdx + 16);
        String pass = body.substring(passIdx + 16, endPass);
        if (pass.length() > 0) {
            wifiPassword = pass;
            prefs.putString("pass", wifiPassword);
        }
    }

    int ntpIdx = body.indexOf("\"ntpServer\":\"");
    if (ntpIdx != -1) {
        int endNtp = body.indexOf("\"", ntpIdx + 13);
        ntpServer = body.substring(ntpIdx + 13, endNtp);
        prefs.putString("ntp", ntpServer);
    }

    Serial.println("[Config] Settings saved to flash!");
    server.send(200, "application/json", "{\"status\":\"saved\"}");
}

void handleTransmit() {
    uint32_t dur = broadcastDurationMin * 60;
    if (carouselEnabled) {
        buildCarouselStationList();
        dur = carouselStageCount * carouselDurationPerStationMin * 60;
    }
    startTransmission(dur);
    server.send(200, "application/json", "{\"status\":\"transmitting\"}");
}

void handleStop() {
    stopTransmission();
    server.send(200, "application/json", "{\"status\":\"stopped\"}");
}
