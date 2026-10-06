/**
 * Radio Atomic Clock Protocol Encoders (BPC, WWVB, MSF, DCF77, JJY)
 * Adapted from timestation (https://github.com/kangtastic/timestation)
 * Copyright (c) 2023 James Seo <james@equiv.tech> (MIT License)
 */

#pragma once

#ifdef __cplusplus
extern "C" {
#endif

#include <stdint.h>
#include <stddef.h>
#include "datetime.h"

#define TSIG_TICKS_PER_SEC 20
#define TSIG_TICK_MS       (1000 / TSIG_TICKS_PER_SEC) // 50 ms

// Maximum buffer size needed (60 seconds * 20 ticks/sec / 8 = 150 bytes)
#define TIMEPROTO_MAX_LEVEL_BYTES 150

typedef enum {
    STATION_BPC = 0,    // China (68.5 kHz, 20s frame)
    STATION_WWVB = 1,   // USA (60.0 kHz, 60s frame)
    STATION_MSF = 2,    // UK (60.0 kHz, 60s frame)
    STATION_DCF77 = 3,  // Germany (77.5 kHz, 60s frame)
    STATION_JJY40 = 4,  // Japan East (40.0 kHz, 60s frame)
    STATION_JJY60 = 5,  // Japan West (60.0 kHz, 60s frame)
    STATION_COUNT = 6
} time_station_t;

/**
 * Returns the exact RF carrier frequency for a station in Hz.
 */
uint32_t timeproto_get_carrier_freq(time_station_t station);

/**
 * Returns the human-readable name of the station.
 */
const char* timeproto_get_station_name(time_station_t station);

/**
 * Returns the frame duration in seconds (20s for BPC, 60s for all others).
 */
uint16_t timeproto_get_frame_seconds(time_station_t station);

/**
 * Returns the native UTC offset in seconds (e.g. +28800 for BPC / UTC+8).
 */
int32_t timeproto_get_native_utc_offset(time_station_t station);

/**
 * Generates the full frame bitstream for the given timestamp.
 * 
 * @param station Target broadcast station.
 * @param epoch_sec Current UTC timestamp in seconds (from NTP).
 * @param user_offset_sec Additional user offset in seconds (0 for native).
 * @param dut1_ms DUT1 offset in milliseconds (-800 to +800, default 0).
 * @param out_level Buffer of at least 150 bytes to receive 50ms tick bits.
 * @param out_num_ticks Out pointer receiving total tick count (400 or 1200).
 */
void timeproto_generate_frame(time_station_t station,
                              uint64_t epoch_sec,
                              int32_t user_offset_sec,
                              int16_t dut1_ms,
                              uint8_t *out_level,
                              uint16_t *out_num_ticks);

/**
 * Checks whether the carrier should be ON (1) or OFF (0) at the given tick index.
 */
static inline uint8_t timeproto_get_tick_level(const uint8_t *level, uint16_t tick_idx) {
    return (level[tick_idx / 8] >> (tick_idx % 8)) & 1;
}

#ifdef __cplusplus
}
#endif
