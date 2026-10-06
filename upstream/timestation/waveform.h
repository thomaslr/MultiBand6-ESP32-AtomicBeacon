#pragma once

#include <stdint.h>
#include <emscripten/webaudio.h>
#include "datetime.h"
#include "iir.h"
#include "timesignal.h"

#define TSIG_WAVEFORM_LERP_RATE             0.015F
#define TSIG_WAVEFORM_LERP_MIN_DELTA        0.005F
#define TSIG_WAVEFORM_SYNC_MARKER           0xff
#define TSIG_WAVEFORM_AUDIBLE_FREQ          1000
#define TSIG_WAVEFORM_SUBHARMONIC_THRESHOLD 20000
#define TSIG_WAVEFORM_SUBHARMONIC_THIRD     3
#define TSIG_WAVEFORM_SUBHARMONIC_FIFTH     5

/* Our internal time quantum is a "tick". */
#define TSIG_WAVEFORM_TICK_MS       50
#define TSIG_WAVEFORM_TICKS_PER_SEC (1000 / TSIG_WAVEFORM_TICK_MS)

/*
 * JJY makes announcements during minutes 15 and 45. From about
 * [40.550-49.000) seconds, it transmits its callsign in Morse code.
 */
#define TSIG_WAVEFORM_JJY_ANNOUNCE_MIN  15
#define TSIG_WAVEFORM_JJY_ANNOUNCE_MIN2 45
#define TSIG_WAVEFORM_JJY_MORSE_SEC     40
#define TSIG_WAVEFORM_JJY_MORSE_MS      550
#define TSIG_WAVEFORM_JJY_MORSE_END_SEC 49
#define TSIG_WAVEFORM_JJY_MORSE_TICK                             \
  ((TSIG_WAVEFORM_JJY_MORSE_SEC * TSIG_WAVEFORM_TICKS_PER_SEC) + \
   (TSIG_WAVEFORM_JJY_MORSE_MS / TSIG_WAVEFORM_TICK_MS))
#define TSIG_WAVEFORM_JJY_MORSE_END_TICK \
  (TSIG_WAVEFORM_JJY_MORSE_END_SEC * TSIG_WAVEFORM_TICKS_PER_SEC)

/* Duration of Morse code symbols as ticks. */
#define TSIG_WAVEFORM_TICKS_PER_DIT 2
#define TSIG_WAVEFORM_TICKS_PER_DAH 5
#define TSIG_WAVEFORM_TICKS_PER_IEG 1  /* Inter-element gap. */
#define TSIG_WAVEFORM_TICKS_PER_ICG 6  /* Inter-character gap. */
#define TSIG_WAVEFORM_TICKS_PER_IWG 10 /* Inter-word gap. */

static void tsig_xmit_bpc(tsig_datetime_t, tsig_params_t *, uint8_t[]);
static void tsig_xmit_dcf77(tsig_datetime_t, tsig_params_t *, uint8_t[]);
static void tsig_xmit_jjy(tsig_datetime_t, tsig_params_t *, uint8_t[]);
static void tsig_xmit_msf(tsig_datetime_t, tsig_params_t *, uint8_t[]);
static void tsig_xmit_wwvb(tsig_datetime_t, tsig_params_t *, uint8_t[]);

typedef void (*waveform_xmit_func)(tsig_datetime_t datetime,
                                   tsig_params_t *params, uint8_t xmit_level[]);

/** Characteristics of a real time station's signal. */
typedef struct waveform_station_data {
  /** Pointer to a function that generates transmit level flags. */
  waveform_xmit_func gen_xmit;

  uint32_t utc_offset; /** Usual (not summer time) UTC offset. */
  uint32_t target_hz;  /** Actual broadcast frequency. */
  float xmit_low;      /** Low gain in [0.0F-1.0F]. */
} waveform_station_data_t;

static waveform_station_data_t TSIG_WAVEFORM_STATION_DATA[] = {
    [TSIG_STATION_BPC] =
        {
            .gen_xmit = tsig_xmit_bpc,
            .utc_offset = 28800000, /* CST is UTC+0800 */
            .target_hz = 68500,
            .xmit_low = 0.31622776F /* -10 dB */
        },
    [TSIG_STATION_DCF77] =
        {
            .gen_xmit = tsig_xmit_dcf77,
            .utc_offset = 3600000, /* CET is UTC+0100 */
            .target_hz = 77500,
            .xmit_low = 0.14962357F /* -16.5 dB */
        },
    [TSIG_STATION_JJY] =
        {
            .gen_xmit = tsig_xmit_jjy,
            .utc_offset = 32400000, /* JST is UTC+0900 */
            .target_hz = 40000,
            .xmit_low = 0.31622776F /* -10 dB */
        },
    [TSIG_STATION_MSF] =
        {
            .gen_xmit = tsig_xmit_msf,
            .utc_offset = 0, /* UTC */
            .target_hz = 60000,
            .xmit_low = 0.0F /* On-off keying */
        },
    [TSIG_STATION_WWVB] =
        {
            .gen_xmit = tsig_xmit_wwvb,
            .utc_offset = 0, /* UTC */
            .target_hz = 60000,
            .xmit_low = 0.14125375F /* -17 dB */
        },
};

/**
 * Waveform context.
 *
 * Used to generate a waveform similar to that produced by a real time station.
 */
typedef struct tsig_waveform_ctx_t {
  /** Sample rate of AudioContext. */
  uint32_t sample_rate;

  /** Bitfield of per-tick transmit level flags for current station minute. */
  uint8_t xmit_level[60 * TSIG_WAVEFORM_TICKS_PER_SEC / CHAR_BIT];

  double timestamp;   /** Base timestamp of this waveform context. */
  uint32_t samples;   /** Sample count since that timestamp. */
  uint32_t next_tick; /** Sample count at next tick. */
  uint32_t morse_end; /** Sample count when on-off keying should stop. */
  uint16_t tick;      /** Tick index within current station minute. */

  uint32_t max_fade_gain; /** Maximum fade gain. */
  uint32_t fade_gain;     /** Fade gain. Relative to max. */
  float gain;             /** Actual current gain in [0.0F-1.0F]. */

  tsig_iir_t iir; /** IIR filter sine wave generator. */
  uint32_t freq;  /** Frequency of generated waveform. */
  int scale;      /** Scale factor for emulated integer-quantized LPCM. */
} tsig_waveform_ctx_t;

static inline uint32_t tsig_calculate_target_hz(tsig_params_t *params) {
  return params->station != TSIG_STATION_JJY ||
                 params->jjy_khz != TSIG_JJYKHZ_60
             ? TSIG_WAVEFORM_STATION_DATA[params->station].target_hz
             : 60000;
}

static inline uint8_t tsig_calculate_subharmonic(uint32_t target_hz) {
  target_hz /= TSIG_WAVEFORM_SUBHARMONIC_THIRD;
  return target_hz <= TSIG_WAVEFORM_SUBHARMONIC_THRESHOLD
             ? TSIG_WAVEFORM_SUBHARMONIC_THIRD
             : TSIG_WAVEFORM_SUBHARMONIC_FIFTH;
}

static inline uint8_t tsig_even_parity(uint8_t data[], int lo, int hi) {
  uint8_t parity = 0;
  for (int i = lo; i < hi; i++)
    for (uint8_t byte = data[i]; byte; byte &= byte - 1)
      parity = !parity;
  return parity;
}

static inline uint8_t tsig_odd_parity(uint8_t data[], int lo, int hi) {
  return !tsig_even_parity(data, lo, hi);
}

static void tsig_xmit_bpc(tsig_datetime_t datetime, tsig_params_t *params,
                          uint8_t xmit_level[]) {
  uint8_t bits[20] = {[0] = TSIG_WAVEFORM_SYNC_MARKER};

  uint8_t hour_12h = datetime.hour % 12;
  bits[3] = (hour_12h >> 2) & 0x3;
  bits[4] = hour_12h & 0x3;

  uint8_t min = datetime.min;
  bits[5] = (min >> 4) & 0x3;
  bits[6] = (min >> 2) & 0x3;
  bits[7] = min & 0x3;

  uint8_t dow = datetime.dow ? datetime.dow : 7;
  bits[8] = (dow >> 2) & 0x1;
  bits[9] = dow & 0x3;

  uint8_t is_pm = datetime.hour >= 12;
  bits[10] = (is_pm << 1) | tsig_even_parity(bits, 1, 10);

  uint8_t day = datetime.day;
  bits[11] = (day >> 4) & 0x1;
  bits[12] = (day >> 2) & 0x3;
  bits[13] = day & 0x3;

  uint8_t mon = datetime.mon;
  bits[14] = (mon >> 2) & 0x3;
  bits[15] = mon & 0x3;

  uint8_t year = datetime.year % 100;
  bits[16] = (year >> 4) & 0x3;
  bits[17] = (year >> 2) & 0x3;
  bits[18] = year & 0x3;
  bits[19] = ((year >> 5) & 0x2) | tsig_even_parity(bits, 11, 19);

  for (int p = 0, j = 0; p < 3; p++) {
    if (p)
      bits[1] = 1 << p;
    if (p == 1)
      bits[10] ^= 1;

    /* Marker: Low for 0 ms, 00: 100 ms, 01: 200 ms, 10: 300 ms, 11: 400 ms. */
    for (int i = 0; i < sizeof(bits); i++) {
      int lo_dsec = bits[i] == TSIG_WAVEFORM_SYNC_MARKER ? 0 : bits[i] + 1;
      int lo = 100 * lo_dsec / TSIG_WAVEFORM_TICK_MS;
      int hi = TSIG_WAVEFORM_TICKS_PER_SEC - lo;
      for (; lo; j++, lo--)
        xmit_level[j / CHAR_BIT] &= ~((1 << (j % CHAR_BIT)));
      for (; hi; j++, hi--)
        xmit_level[j / CHAR_BIT] |= 1 << (j % CHAR_BIT);
    }
  }
}

static void tsig_xmit_dcf77(tsig_datetime_t datetime, tsig_params_t *params,
                            uint8_t xmit_level[]) {
  uint8_t bits[60] = {[20] = 1, [59] = TSIG_WAVEFORM_SYNC_MARKER};

  /* tsig_datetime_is_eu_dst() expects UTC datetime. We have CET (UTC+0100). */
  uint32_t utc_offset = TSIG_DATETIME_MSECS_HOUR;
  double utc_timestamp = datetime.timestamp - utc_offset;
  tsig_datetime_t utc_datetime = tsig_datetime_parse_timestamp(utc_timestamp);

  /* Transmitted time is the CET/CEST time at the next UTC minute. */
  uint32_t in_mins;
  uint8_t is_cest = tsig_datetime_is_eu_dst(utc_datetime, &in_mins);
  uint8_t is_xmit_cest = is_cest ^ (in_mins == 1);

  bits[16] = in_mins <= 60;
  bits[17] = is_xmit_cest;
  bits[18] = !is_xmit_cest;

  uint32_t cest_offset = is_xmit_cest * TSIG_DATETIME_MSECS_HOUR;
  uint32_t xmit_offset = TSIG_DATETIME_MSECS_MIN;
  double xmit_timestamp = datetime.timestamp + cest_offset + xmit_offset;
  tsig_datetime_t xmit_datetime = tsig_datetime_parse_timestamp(xmit_timestamp);

  bits[20] = 1;

  uint8_t min = xmit_datetime.min % 10;
  bits[21] = min & 1;
  bits[22] = min & 2;
  bits[23] = min & 4;
  bits[24] = min & 8;

  uint8_t min_10 = xmit_datetime.min / 10;
  bits[25] = min_10 & 1;
  bits[26] = min_10 & 2;
  bits[27] = min_10 & 4;

  bits[28] = tsig_even_parity(bits, 21, 28);

  uint8_t hour = xmit_datetime.hour % 10;
  bits[29] = hour & 1;
  bits[30] = hour & 2;
  bits[31] = hour & 4;
  bits[32] = hour & 8;

  uint8_t hour_10 = xmit_datetime.hour / 10;
  bits[33] = hour_10 & 1;
  bits[34] = hour_10 & 2;

  bits[35] = tsig_even_parity(bits, 29, 35);

  uint8_t day = xmit_datetime.day % 10;
  bits[36] = day & 1;
  bits[37] = day & 2;
  bits[38] = day & 4;
  bits[39] = day & 8;

  uint8_t day_10 = xmit_datetime.day / 10;
  bits[40] = day_10 & 1;
  bits[41] = day_10 & 2;

  uint8_t dow = xmit_datetime.dow ? xmit_datetime.dow : 7;
  bits[42] = dow & 1;
  bits[43] = dow & 2;
  bits[44] = dow & 4;

  uint8_t mon = xmit_datetime.mon % 10;
  bits[45] = mon & 1;
  bits[46] = mon & 2;
  bits[47] = mon & 4;
  bits[48] = mon & 8;

  uint8_t mon_10 = xmit_datetime.mon / 10;
  bits[49] = mon_10 & 1;

  uint8_t year = xmit_datetime.year % 10;
  bits[50] = year & 1;
  bits[51] = year & 2;
  bits[52] = year & 4;
  bits[53] = year & 8;

  uint8_t year_10 = (xmit_datetime.year % 100) / 10;
  bits[54] = year_10 & 1;
  bits[55] = year_10 & 2;
  bits[56] = year_10 & 4;
  bits[57] = year_10 & 8;

  bits[58] = tsig_even_parity(bits, 36, 58);

  /* Marker: Low for 0 ms, 0: 100 ms, 1: 200 ms. */
  for (int i = 0, j = 0; i < sizeof(bits); i++) {
    int lo_dsec = bits[i] == TSIG_WAVEFORM_SYNC_MARKER ? 0 : !!bits[i] + 1;
    int lo = 100 * lo_dsec / TSIG_WAVEFORM_TICK_MS;
    int hi = TSIG_WAVEFORM_TICKS_PER_SEC - lo;
    for (; lo; j++, lo--)
      xmit_level[j / CHAR_BIT] &= ~((1 << (j % CHAR_BIT)));
    for (; hi; j++, hi--)
      xmit_level[j / CHAR_BIT] |= 1 << (j % CHAR_BIT);
  }
}

static void tsig_xmit_jjy_morse_pulse(uint8_t xmit_level[], int *k, int ticks) {
  for (int i = 0, j = *k; i < ticks; i++, j++)
    xmit_level[j / CHAR_BIT] |= 1 << (j % CHAR_BIT);
  *k += ticks;
}

static void tsig_xmit_jjy_morse(uint8_t xmit_level[]) {
  int lo = TSIG_WAVEFORM_JJY_MORSE_SEC * TSIG_WAVEFORM_TICKS_PER_SEC;
  int hi = TSIG_WAVEFORM_JJY_MORSE_END_SEC * TSIG_WAVEFORM_TICKS_PER_SEC;
  for (int i = lo; i < hi; i++)
    xmit_level[i / CHAR_BIT] &= ~((1 << (i % CHAR_BIT)));

  int k = TSIG_WAVEFORM_JJY_MORSE_TICK;
  for (int i = 0; i < 2; i++) {
    /* JJ, i.e. .--- .--- */
    for (int j = 0; j < 2; j++) {
      tsig_xmit_jjy_morse_pulse(xmit_level, &k, TSIG_WAVEFORM_TICKS_PER_DIT);
      k += TSIG_WAVEFORM_TICKS_PER_IEG;
      tsig_xmit_jjy_morse_pulse(xmit_level, &k, TSIG_WAVEFORM_TICKS_PER_DAH);
      k += TSIG_WAVEFORM_TICKS_PER_IEG;
      tsig_xmit_jjy_morse_pulse(xmit_level, &k, TSIG_WAVEFORM_TICKS_PER_DAH);
      k += TSIG_WAVEFORM_TICKS_PER_IEG;
      tsig_xmit_jjy_morse_pulse(xmit_level, &k, TSIG_WAVEFORM_TICKS_PER_DAH);
      k += TSIG_WAVEFORM_TICKS_PER_ICG;
    }
    /* Y, i.e. -.-- */
    tsig_xmit_jjy_morse_pulse(xmit_level, &k, TSIG_WAVEFORM_TICKS_PER_DAH);
    k += TSIG_WAVEFORM_TICKS_PER_IEG;
    tsig_xmit_jjy_morse_pulse(xmit_level, &k, TSIG_WAVEFORM_TICKS_PER_DIT);
    k += TSIG_WAVEFORM_TICKS_PER_IEG;
    tsig_xmit_jjy_morse_pulse(xmit_level, &k, TSIG_WAVEFORM_TICKS_PER_DAH);
    k += TSIG_WAVEFORM_TICKS_PER_IEG;
    tsig_xmit_jjy_morse_pulse(xmit_level, &k, TSIG_WAVEFORM_TICKS_PER_DAH);
    k += TSIG_WAVEFORM_TICKS_PER_IWG;
  }
}

static void tsig_xmit_jjy(tsig_datetime_t datetime, tsig_params_t *params,
                          uint8_t xmit_level[]) {
  uint8_t bits[60] = {
      [0] = TSIG_WAVEFORM_SYNC_MARKER,  [9] = TSIG_WAVEFORM_SYNC_MARKER,
      [19] = TSIG_WAVEFORM_SYNC_MARKER, [29] = TSIG_WAVEFORM_SYNC_MARKER,
      [39] = TSIG_WAVEFORM_SYNC_MARKER, [49] = TSIG_WAVEFORM_SYNC_MARKER,
      [59] = TSIG_WAVEFORM_SYNC_MARKER,
  };

  uint8_t min_10 = datetime.min / 10;
  bits[1] = min_10 & 4;
  bits[2] = min_10 & 2;
  bits[3] = min_10 & 1;

  uint8_t min = datetime.min % 10;
  bits[5] = min & 8;
  bits[6] = min & 4;
  bits[7] = min & 2;
  bits[8] = min & 1;

  uint8_t hour_10 = datetime.hour / 10;
  bits[12] = hour_10 & 2;
  bits[13] = hour_10 & 1;

  uint8_t hour = datetime.hour % 10;
  bits[15] = hour & 8;
  bits[16] = hour & 4;
  bits[17] = hour & 2;
  bits[18] = hour & 1;

  uint8_t doy_100 = datetime.doy / 100;
  bits[22] = doy_100 & 2;
  bits[23] = doy_100 & 1;

  uint8_t doy_10 = (datetime.doy % 100) / 10;
  bits[25] = doy_10 & 8;
  bits[26] = doy_10 & 4;
  bits[27] = doy_10 & 2;
  bits[28] = doy_10 & 1;

  uint8_t doy = datetime.doy % 10;
  bits[30] = doy & 8;
  bits[31] = doy & 4;
  bits[32] = doy & 2;
  bits[33] = doy & 1;

  bits[36] = tsig_even_parity(bits, 12, 19);
  bits[37] = tsig_even_parity(bits, 1, 9);

  uint8_t is_announce = datetime.min == TSIG_WAVEFORM_JJY_ANNOUNCE_MIN ||
                        datetime.min == TSIG_WAVEFORM_JJY_ANNOUNCE_MIN2;
  if (!is_announce) {
    uint8_t year_10 = (datetime.year % 100) / 10;
    bits[41] = year_10 & 8;
    bits[42] = year_10 & 4;
    bits[43] = year_10 & 2;
    bits[44] = year_10 & 1;

    uint8_t year = datetime.year % 10;
    bits[45] = year & 8;
    bits[46] = year & 4;
    bits[47] = year & 2;
    bits[48] = year & 1;

    uint8_t dow = datetime.dow;
    bits[50] = dow & 4;
    bits[51] = dow & 2;
    bits[52] = dow & 1;
  }

  /* Marker: Low for 200 ms, 0: 800 ms, 1: 500 ms. */
  for (int i = 0, j = 0; i < sizeof(bits); i++) {
    if (is_announce && i == TSIG_WAVEFORM_JJY_MORSE_SEC) {
      tsig_xmit_jjy_morse(xmit_level);
      i = TSIG_WAVEFORM_JJY_MORSE_END_SEC;
      j = TSIG_WAVEFORM_JJY_MORSE_END_TICK;
    }

    int hi_dsec = bits[i] == TSIG_WAVEFORM_SYNC_MARKER ? 2 : bits[i] ? 5 : 8;
    int hi = 100 * hi_dsec / TSIG_WAVEFORM_TICK_MS;
    int lo = TSIG_WAVEFORM_TICKS_PER_SEC - hi;
    for (; hi; j++, hi--)
      xmit_level[j / CHAR_BIT] |= 1 << (j % CHAR_BIT);
    for (; lo; j++, lo--)
      xmit_level[j / CHAR_BIT] &= ~((1 << (j % CHAR_BIT)));
  }
}

static void tsig_xmit_msf(tsig_datetime_t datetime, tsig_params_t *params,
                          uint8_t xmit_level[]) {
  uint8_t bits[60] = {[0] = TSIG_WAVEFORM_SYNC_MARKER};

  int8_t dut1 = params->dut1 / 100;
  uint8_t lt0 = dut1 < 0 ? 8 : 0;
  if (lt0)
    dut1 = -dut1;
  bits[1 + lt0] = dut1 >= 1;
  bits[2 + lt0] = dut1 >= 2;
  bits[3 + lt0] = dut1 >= 3;
  bits[4 + lt0] = dut1 >= 4;
  bits[5 + lt0] = dut1 >= 5;
  bits[6 + lt0] = dut1 >= 6;
  bits[7 + lt0] = dut1 >= 7;
  bits[8 + lt0] = dut1 >= 8;

  uint32_t in_mins;
  uint8_t is_bst = tsig_datetime_is_eu_dst(datetime, &in_mins);

  /* Transmitted time is the UTC/BST time at the next UTC minute. */
  uint8_t is_xmit_bst = is_bst ^ (in_mins == 1);
  uint32_t bst_offset = is_xmit_bst * TSIG_DATETIME_MSECS_HOUR;
  uint32_t xmit_offset = TSIG_DATETIME_MSECS_MIN;
  double xmit_timestamp = datetime.timestamp + bst_offset + xmit_offset;
  tsig_datetime_t xmit_datetime = tsig_datetime_parse_timestamp(xmit_timestamp);

  uint8_t year_10 = (xmit_datetime.year % 100) / 10;
  bits[17] = year_10 & 8;
  bits[18] = year_10 & 4;
  bits[19] = year_10 & 2;
  bits[20] = year_10 & 1;

  uint8_t year = xmit_datetime.year % 10;
  bits[21] = year & 8;
  bits[22] = year & 4;
  bits[23] = year & 2;
  bits[24] = year & 1;

  uint8_t mon_10 = xmit_datetime.mon / 10;
  bits[25] = mon_10 & 1;

  uint8_t mon = xmit_datetime.mon % 10;
  bits[26] = mon & 8;
  bits[27] = mon & 4;
  bits[28] = mon & 2;
  bits[29] = mon & 1;

  uint8_t day_10 = xmit_datetime.day / 10;
  bits[30] = day_10 & 2;
  bits[31] = day_10 & 1;

  uint8_t day = xmit_datetime.day % 10;
  bits[32] = day & 8;
  bits[33] = day & 4;
  bits[34] = day & 2;
  bits[35] = day & 1;

  uint8_t dow = xmit_datetime.dow;
  bits[36] = dow & 4;
  bits[37] = dow & 2;
  bits[38] = dow & 1;

  uint8_t hour_10 = xmit_datetime.hour / 10;
  bits[39] = hour_10 & 2;
  bits[40] = hour_10 & 1;

  uint8_t hour = xmit_datetime.hour % 10;
  bits[41] = hour & 8;
  bits[42] = hour & 4;
  bits[43] = hour & 2;
  bits[44] = hour & 1;

  uint8_t min_10 = xmit_datetime.min / 10;
  bits[45] = min_10 & 4;
  bits[46] = min_10 & 2;
  bits[47] = min_10 & 1;

  uint8_t min = xmit_datetime.min % 10;
  bits[48] = min & 8;
  bits[49] = min & 4;
  bits[50] = min & 2;
  bits[51] = min & 1;

  bits[53] = in_mins <= 61;
  bits[54] = tsig_odd_parity(bits, 17, 25);
  bits[55] = tsig_odd_parity(bits, 25, 36);
  bits[56] = tsig_odd_parity(bits, 36, 39);
  bits[57] = tsig_odd_parity(bits, 39, 52);
  bits[58] = is_xmit_bst;

  /*
   * Marker: Low for 500 ms, 00: 100 ms, 01: 200 ms, 11: 300 ms.
   * Note that 11 can only occur during the secondary minute marker.
   */
  for (int i = 0, j = 0; i < sizeof(bits); i++) {
    int dsec_lo = bits[i] == TSIG_WAVEFORM_SYNC_MARKER ? 5 : !!bits[i] + 1;
    dsec_lo += 53 <= i && i <= 58; /* Secondary 01111110 minute marker. */
    int lo = 100 * dsec_lo / TSIG_WAVEFORM_TICK_MS;
    int hi = TSIG_WAVEFORM_TICKS_PER_SEC - lo;
    for (; lo; j++, lo--)
      xmit_level[j / CHAR_BIT] &= ~((1 << (j % CHAR_BIT)));
    for (; hi; j++, hi--)
      xmit_level[j / CHAR_BIT] |= 1 << (j % CHAR_BIT);
  }
}

static void tsig_xmit_wwvb(tsig_datetime_t datetime, tsig_params_t *params,
                           uint8_t xmit_level[]) {
  uint8_t bits[60] = {
      [0] = TSIG_WAVEFORM_SYNC_MARKER,  [9] = TSIG_WAVEFORM_SYNC_MARKER,
      [19] = TSIG_WAVEFORM_SYNC_MARKER, [29] = TSIG_WAVEFORM_SYNC_MARKER,
      [39] = TSIG_WAVEFORM_SYNC_MARKER, [49] = TSIG_WAVEFORM_SYNC_MARKER,
      [59] = TSIG_WAVEFORM_SYNC_MARKER,
  };

  uint8_t min_10 = datetime.min / 10;
  bits[1] = min_10 & 4;
  bits[2] = min_10 & 2;
  bits[3] = min_10 & 1;

  uint8_t min = datetime.min % 10;
  bits[5] = min & 8;
  bits[6] = min & 4;
  bits[7] = min & 2;
  bits[8] = min & 1;

  uint8_t hour_10 = datetime.hour / 10;
  bits[12] = hour_10 & 2;
  bits[13] = hour_10 & 1;

  uint8_t hour = datetime.hour % 10;
  bits[15] = hour & 8;
  bits[16] = hour & 4;
  bits[17] = hour & 2;
  bits[18] = hour & 1;

  uint8_t doy_100 = datetime.doy / 100;
  bits[22] = doy_100 & 2;
  bits[23] = doy_100 & 1;

  uint8_t doy_10 = (datetime.doy % 100) / 10;
  bits[25] = doy_10 & 8;
  bits[26] = doy_10 & 4;
  bits[27] = doy_10 & 2;
  bits[28] = doy_10 & 1;

  uint8_t doy = datetime.doy % 10;
  bits[30] = doy & 8;
  bits[31] = doy & 4;
  bits[32] = doy & 2;
  bits[33] = doy & 1;

  int8_t dut1 = params->dut1 / 100;
  bits[36] = dut1 >= 0;
  bits[37] = dut1 < 0;
  bits[38] = dut1 >= 0;
  if (dut1 < 0)
    dut1 = -dut1;
  bits[40] = dut1 & 8;
  bits[41] = dut1 & 4;
  bits[42] = dut1 & 2;
  bits[43] = dut1 & 1;

  uint8_t year_10 = (datetime.year % 100) / 10;
  bits[45] = year_10 & 8;
  bits[46] = year_10 & 4;
  bits[47] = year_10 & 2;
  bits[48] = year_10 & 1;

  uint8_t year = datetime.year % 10;
  bits[50] = year & 8;
  bits[51] = year & 4;
  bits[52] = year & 2;
  bits[53] = year & 1;

  bits[55] = tsig_datetime_is_leap(datetime.year);

  bits[58] = tsig_datetime_is_us_dst(datetime, &bits[57]);

  /* Marker: Low for 800 ms, 0: 200 ms, 1: 500 ms. */
  for (int i = 0, j = 0; i < sizeof(bits); i++) {
    int dsec_lo = bits[i] == TSIG_WAVEFORM_SYNC_MARKER ? 8 : bits[i] ? 5 : 2;
    int lo = 100 * dsec_lo / TSIG_WAVEFORM_TICK_MS;
    int hi = TSIG_WAVEFORM_TICKS_PER_SEC - lo;
    for (; lo; j++, lo--)
      xmit_level[j / CHAR_BIT] &= ~((1 << (j % CHAR_BIT)));
    for (; hi; j++, hi--)
      xmit_level[j / CHAR_BIT] |= 1 << (j % CHAR_BIT);
  }
}

static inline float tsig_gen_next_sample(tsig_waveform_ctx_t *ctx) {
  /*
   * JS wants 32-bit floats, but pure floats may not work. Simulate integer
   * quantization by scaling by some integer factor, flooring, and dividing
   * back to float. Apparently, some devices won't pick up the fundamental we
   * hope we're creating when we play back a subharmonic otherwise. The "best"
   * scale factor varies, but the AudioContext's sample rate divided by the
   * number of the subharmonic we're using should work.
   * cf. https://jjy.luxferre.top/
   */
  double sample = tsig_iir_next(&ctx->iir);
  int lpcm_sample = sample * ctx->gain * ctx->scale;
  return (float)lpcm_sample / ctx->scale;
}

static inline float tsig_lerp(float target_gain, float gain) {
  float delta = target_gain > gain ? target_gain - gain : gain - target_gain;
  return delta > TSIG_WAVEFORM_LERP_MIN_DELTA
             ? (1.0F - TSIG_WAVEFORM_LERP_RATE) * gain +
                   TSIG_WAVEFORM_LERP_RATE * target_gain
             : target_gain;
}

/**
 * Generate audio samples for an emulated time station waveform.
 *
 * `TSIG_RENDER_QUANTUM` samples are generated of an emulated waveform similar
 * to that transmitted by a real time station and written to the provided
 * audio output buffers.
 *
 * @param ctx Pointer to a waveform context.
 * @param params Pointer to a struct containing user parameters.
 * @param state Current time signal generator module state.
 * @param[out] out_next_state Out pointer to the time signal generator module
 *  state at the end of this render quantum.
 * @param n_outputs Count of audio output buffers.
 * @param outputs Array of audio output buffers provided to an audio worklet
 *  processor callback function by the Emscripten Audio Worklets API.
 */
void tsig_waveform_generate(tsig_waveform_ctx_t *ctx, tsig_params_t *params,
                            int state, int *out_next_state, int n_outputs,
                            AudioSampleFrame *outputs) {
  waveform_station_data_t *data = &TSIG_WAVEFORM_STATION_DATA[params->station];
  float xmit_low = data->xmit_low;

  for (int i = 0; i < TSIG_RENDER_QUANTUM; i++) {
    /* Update state for the current tick. */
    if (ctx->samples == ctx->next_tick) {
      double adj_timestamp = 1000.0 * ctx->samples / ctx->sample_rate +
                             ctx->timestamp + params->offset;
      tsig_datetime_t adj_datetime =
          tsig_datetime_parse_timestamp(adj_timestamp);

      uint32_t msec_since_min = 1000 * adj_datetime.sec + adj_datetime.msec;
      ctx->tick = msec_since_min / TSIG_WAVEFORM_TICK_MS;
      ;

      if (!ctx->samples || !ctx->tick)
        data->gen_xmit(adj_datetime, params, ctx->xmit_level);

      uint32_t msec_since_tick = adj_datetime.msec % TSIG_WAVEFORM_TICK_MS;
      uint32_t msec_to_tick = TSIG_WAVEFORM_TICK_MS - msec_since_tick;
      ctx->next_tick += msec_to_tick * ctx->sample_rate / 1000;

      /*
       * Per DCF77's signal format specification, each minute and each transmit
       * power change occurs at a rising zero crossing. We don't have enough
       * control over what actually gets transmitted to reliably emulate this,
       * and it's almost certainly not necessary for our purposes. Still,
       * there's no particular reason not to try, so adjust the initial phase
       * of the waveform such that the beginning of the next minute occurs at
       * such a crossing. The phase change shouldn't matter for other stations.
       */
      if (!ctx->samples) {
        uint32_t msec_to_min = TSIG_DATETIME_MSECS_MIN - msec_since_min;
        int to_min = msec_to_min * ctx->sample_rate / 1000;
        tsig_iir_init(&ctx->iir, ctx->freq, ctx->sample_rate, -to_min);
      }

      /*
       * Using a public WebSDR, it was determined that if JJY is doing an
       * announcement, it transmits its callsign in Morse code from about
       * 40.550 to 48.250 seconds after the minute. During this time, keying is
       * on-off and low gain is 0 instead of the usual -10 dB. Afterwards, low
       * gain delays returning to -10 dB until the marker bit at 49 seconds.
       */
      if (params->station == TSIG_STATION_JJY && !ctx->morse_end) {
        uint8_t min = adj_datetime.min;
        uint8_t is_announce = min == TSIG_WAVEFORM_JJY_ANNOUNCE_MIN ||
                              min == TSIG_WAVEFORM_JJY_ANNOUNCE_MIN2;
        if (is_announce) {
          uint8_t sec = adj_datetime.sec;
          uint16_t msec = adj_datetime.msec;
          uint8_t is_morse = ((sec == TSIG_WAVEFORM_JJY_MORSE_SEC &&
                               msec >= TSIG_WAVEFORM_JJY_MORSE_MS) ||
                              TSIG_WAVEFORM_JJY_MORSE_SEC < sec) &&
                             sec < TSIG_WAVEFORM_JJY_MORSE_END_SEC;
          if (is_morse) {
            uint32_t msec_to_morse_end =
                1000 * TSIG_WAVEFORM_JJY_MORSE_END_SEC - msec_since_min;
            ctx->morse_end =
                ctx->samples + msec_to_morse_end * ctx->sample_rate / 1000;
          }
        }
      }
    }

    if (ctx->morse_end) {
      if (ctx->samples < ctx->morse_end)
        xmit_low = 0;
      else
        ctx->morse_end = 0;
    }

    /* Find and set instantaneous gain, interpolating changes if needed. */
    uint8_t bit = 1 << (ctx->tick % CHAR_BIT);
    uint8_t is_xmit_high = ctx->xmit_level[ctx->tick / CHAR_BIT] & bit;
    float target_gain = is_xmit_high ? 1.0F : xmit_low;
    float gain = ctx->gain;

    if (ctx->fade_gain != ctx->max_fade_gain)
      target_gain *= (float)ctx->fade_gain * ctx->fade_gain /
                     (ctx->max_fade_gain * ctx->max_fade_gain);

    ctx->gain = params->noclip ? tsig_lerp(target_gain, gain) : target_gain;

    /* We are now ready to generate and output a sample. */
    float sample = tsig_gen_next_sample(ctx);

    for (int o = 0; o < n_outputs; o++)
      for (int c = 0; c < outputs[o].numberOfChannels; c++)
        outputs[o].data[c * TSIG_RENDER_QUANTUM + i] = sample;

    ctx->samples++;

    /* Fade in/out. Initiate a phase transition once fade is complete. */
    if (state == TSIG_STATE_FADE_IN) {
      if (ctx->fade_gain < ctx->max_fade_gain)
        ctx->fade_gain++;
      else if (target_gain == ctx->gain)
        *out_next_state = TSIG_STATE_RUNNING;
    }

    else if (state == TSIG_STATE_FADE_OUT) {
      if (ctx->fade_gain)
        ctx->fade_gain--;
      else if (target_gain == ctx->gain)
        *out_next_state = TSIG_STATE_SUSPEND;
    }
  }
}

/**
 * Fill audio output buffers with silence.
 * @param n_outputs Count of audio output buffers.
 * @param outputs Array of audio output buffers provided to an audio worklet
 *  processor callback function by the Emscripten Audio Worklets API.
 */
void tsig_waveform_generate_silence(int n_outputs, AudioSampleFrame *outputs) {
  for (int i = 0; i < n_outputs; i++)
    for (int j = 0; j < TSIG_RENDER_QUANTUM; j++)
      for (int k = 0; k < outputs[i].numberOfChannels; k++)
        outputs[i].data[j + k * TSIG_RENDER_QUANTUM] = 0.0F;
}

/**
 * Initialize a waveform context from a timestamp.
 * @param ctx Pointer to the waveform context to be initialized.
 * @param params Pointer to user parameters.
 */
void tsig_waveform_init(tsig_waveform_ctx_t *ctx, tsig_params_t *params) {
  uint32_t utc_offset = TSIG_WAVEFORM_STATION_DATA[params->station].utc_offset;
  double render_quantum_ms = 1000.0 * TSIG_RENDER_QUANTUM / ctx->sample_rate;
  double timestamp = emscripten_get_now();
  uint32_t sample_rate = ctx->sample_rate;
  uint8_t subharmonic;
  uint32_t target_hz;

  target_hz = tsig_calculate_target_hz(params);
  subharmonic = tsig_calculate_subharmonic(target_hz);

  ctx->timestamp = timestamp + utc_offset + render_quantum_ms;
  ctx->samples = 0;
  ctx->next_tick = 0;
  ctx->morse_end = 0;

  ctx->max_fade_gain = sample_rate * TSIG_FADE_MS / 1000;
  ctx->fade_gain = 0;
  ctx->gain = 0.0;

  ctx->freq =
      params->audible ? TSIG_WAVEFORM_AUDIBLE_FREQ : target_hz / subharmonic;
  ctx->scale = sample_rate / subharmonic;
}
