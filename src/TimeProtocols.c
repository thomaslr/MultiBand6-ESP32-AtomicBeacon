/**
 * Radio Atomic Clock Protocol Encoders (BPC, WWVB, MSF, DCF77, JJY)
 * Adapted from timestation (https://github.com/kangtastic/timestation)
 * Copyright (c) 2023 James Seo <james@equiv.tech> (MIT License)
 */

#include "TimeProtocols.h"
#include <string.h>

#define TSIG_WAVEFORM_TICK_MS       50
#define TSIG_WAVEFORM_TICKS_PER_SEC (1000 / TSIG_WAVEFORM_TICK_MS)
#define TSIG_WAVEFORM_SYNC_MARKER   0xff

#define TSIG_WAVEFORM_JJY_ANNOUNCE_MIN  15
#define TSIG_WAVEFORM_JJY_ANNOUNCE_MIN2 45
#define TSIG_WAVEFORM_JJY_MORSE_SEC     40
#define TSIG_WAVEFORM_JJY_MORSE_MS      550
#define TSIG_WAVEFORM_JJY_MORSE_END_SEC 49
#define TSIG_WAVEFORM_JJY_MORSE_TICK \
  ((TSIG_WAVEFORM_JJY_MORSE_SEC * TSIG_WAVEFORM_TICKS_PER_SEC) + \
   (TSIG_WAVEFORM_JJY_MORSE_MS / TSIG_WAVEFORM_TICK_MS))
#define TSIG_WAVEFORM_JJY_MORSE_END_TICK \
  (TSIG_WAVEFORM_JJY_MORSE_END_SEC * TSIG_WAVEFORM_TICKS_PER_SEC)

#define TSIG_WAVEFORM_TICKS_PER_DIT 2
#define TSIG_WAVEFORM_TICKS_PER_DAH 5
#define TSIG_WAVEFORM_TICKS_PER_IEG 1  /* Inter-element gap */
#define TSIG_WAVEFORM_TICKS_PER_ICG 6  /* Inter-character gap */
#define TSIG_WAVEFORM_TICKS_PER_IWG 10 /* Inter-word gap */

static inline uint8_t tsig_even_parity(const uint8_t data[], int lo, int hi) {
  uint8_t parity = 0;
  for (int i = lo; i < hi; i++)
    for (uint8_t byte = data[i]; byte; byte &= byte - 1)
      parity = !parity;
  return parity;
}

static inline uint8_t tsig_odd_parity(const uint8_t data[], int lo, int hi) {
  return !tsig_even_parity(data, lo, hi);
}

/* ========================================================================= */
/* BPC (China, 68.5 kHz)                                                     */
/* ========================================================================= */
static void tsig_xmit_bpc(tsig_datetime_t datetime, int16_t dut1_ms, uint8_t xmit_level[]) {
  (void)dut1_ms;
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

  /* BPC sends three 20-second frames per minute: p = 0, 1, 2 */
  for (int p = 0, j = 0; p < 3; p++) {
    if (p)
      bits[1] = 1 << p;
    if (p == 1)
      bits[10] ^= 1;

    /* Marker: Low for 0 ms, 00: 100 ms, 01: 200 ms, 10: 300 ms, 11: 400 ms */
    for (size_t i = 0; i < sizeof(bits); i++) {
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

/* ========================================================================= */
/* DCF77 (Germany, 77.5 kHz)                                                 */
/* ========================================================================= */
static void tsig_xmit_dcf77(tsig_datetime_t datetime, int16_t dut1_ms, uint8_t xmit_level[]) {
  (void)dut1_ms;
  uint8_t bits[60] = {[20] = 1, [59] = TSIG_WAVEFORM_SYNC_MARKER};

  /* tsig_datetime_is_eu_dst() expects UTC datetime. We have CET (UTC+0100). */
  uint32_t utc_offset = (uint32_t)TSIG_DATETIME_MSECS_HOUR;
  double utc_timestamp = datetime.timestamp - utc_offset;
  tsig_datetime_t utc_datetime = tsig_datetime_parse_timestamp(utc_timestamp);

  /* Transmitted time is the CET/CEST time at the next UTC minute. */
  uint32_t in_mins;
  uint8_t is_cest = tsig_datetime_is_eu_dst(utc_datetime, &in_mins);
  uint8_t is_xmit_cest = is_cest ^ (in_mins == 1);

  bits[16] = in_mins <= 60;
  bits[17] = is_xmit_cest;
  bits[18] = !is_xmit_cest;

  uint32_t cest_offset = is_xmit_cest * (uint32_t)TSIG_DATETIME_MSECS_HOUR;
  uint32_t xmit_offset = (uint32_t)TSIG_DATETIME_MSECS_MIN;
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

  /* Marker: Low for 0 ms, 0: 100 ms, 1: 200 ms */
  for (size_t i = 0, j = 0; i < sizeof(bits); i++) {
    int lo_dsec = bits[i] == TSIG_WAVEFORM_SYNC_MARKER ? 0 : !!bits[i] + 1;
    int lo = 100 * lo_dsec / TSIG_WAVEFORM_TICK_MS;
    int hi = TSIG_WAVEFORM_TICKS_PER_SEC - lo;
    for (; lo; j++, lo--)
      xmit_level[j / CHAR_BIT] &= ~((1 << (j % CHAR_BIT)));
    for (; hi; j++, hi--)
      xmit_level[j / CHAR_BIT] |= 1 << (j % CHAR_BIT);
  }
}

/* ========================================================================= */
/* JJY Morse Code for minutes 15 and 45                                      */
/* ========================================================================= */
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
    /* JJ: .--- .--- */
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
    /* Y: -.-- */
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

/* ========================================================================= */
/* JJY (Japan, 40.0 kHz & 60.0 kHz)                                          */
/* ========================================================================= */
static void tsig_xmit_jjy(tsig_datetime_t datetime, int16_t dut1_ms, uint8_t xmit_level[]) {
  (void)dut1_ms;
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

  uint8_t doy_100 = (uint8_t)(datetime.doy / 100);
  bits[22] = doy_100 & 2;
  bits[23] = doy_100 & 1;

  uint8_t doy_10 = (uint8_t)((datetime.doy % 100) / 10);
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

  /* Marker: Low for 200 ms, 0: 800 ms, 1: 500 ms */
  for (size_t i = 0, j = 0; i < sizeof(bits); i++) {
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

/* ========================================================================= */
/* MSF (UK, 60.0 kHz)                                                        */
/* ========================================================================= */
static void tsig_xmit_msf(tsig_datetime_t datetime, int16_t dut1_ms, uint8_t xmit_level[]) {
  uint8_t bits[60] = {[0] = TSIG_WAVEFORM_SYNC_MARKER};

  int8_t dut1 = (int8_t)(dut1_ms / 100);
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
  uint32_t bst_offset = is_xmit_bst * (uint32_t)TSIG_DATETIME_MSECS_HOUR;
  uint32_t xmit_offset = (uint32_t)TSIG_DATETIME_MSECS_MIN;
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

  /* Marker: Low for 500 ms, 00: 100 ms, 01: 200 ms, 11: 300 ms */
  for (size_t i = 0, j = 0; i < sizeof(bits); i++) {
    int dsec_lo = bits[i] == TSIG_WAVEFORM_SYNC_MARKER ? 5 : !!bits[i] + 1;
    dsec_lo += 53 <= i && i <= 58; /* Secondary 01111110 minute marker */
    int lo = 100 * dsec_lo / TSIG_WAVEFORM_TICK_MS;
    int hi = TSIG_WAVEFORM_TICKS_PER_SEC - lo;
    for (; lo; j++, lo--)
      xmit_level[j / CHAR_BIT] &= ~((1 << (j % CHAR_BIT)));
    for (; hi; j++, hi--)
      xmit_level[j / CHAR_BIT] |= 1 << (j % CHAR_BIT);
  }
}

/* ========================================================================= */
/* WWVB (USA, 60.0 kHz)                                                      */
/* ========================================================================= */
static void tsig_xmit_wwvb(tsig_datetime_t datetime, int16_t dut1_ms, uint8_t xmit_level[]) {
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

  uint8_t doy_100 = (uint8_t)(datetime.doy / 100);
  bits[22] = doy_100 & 2;
  bits[23] = doy_100 & 1;

  uint8_t doy_10 = (uint8_t)((datetime.doy % 100) / 10);
  bits[25] = doy_10 & 8;
  bits[26] = doy_10 & 4;
  bits[27] = doy_10 & 2;
  bits[28] = doy_10 & 1;

  uint8_t doy = datetime.doy % 10;
  bits[30] = doy & 8;
  bits[31] = doy & 4;
  bits[32] = doy & 2;
  bits[33] = doy & 1;

  int8_t dut1 = (int8_t)(dut1_ms / 100);
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

  /* Marker: Low for 800 ms, 0: 200 ms, 1: 500 ms */
  for (size_t i = 0, j = 0; i < sizeof(bits); i++) {
    int dsec_lo = bits[i] == TSIG_WAVEFORM_SYNC_MARKER ? 8 : bits[i] ? 5 : 2;
    int lo = 100 * dsec_lo / TSIG_WAVEFORM_TICK_MS;
    int hi = TSIG_WAVEFORM_TICKS_PER_SEC - lo;
    for (; lo; j++, lo--)
      xmit_level[j / CHAR_BIT] &= ~((1 << (j % CHAR_BIT)));
    for (; hi; j++, hi--)
      xmit_level[j / CHAR_BIT] |= 1 << (j % CHAR_BIT);
  }
}

/* ========================================================================= */
/* Public Interface                                                          */
/* ========================================================================= */

uint32_t timeproto_get_carrier_freq(time_station_t station) {
  switch (station) {
    case STATION_BPC:   return 68500;
    case STATION_DCF77: return 77500;
    case STATION_JJY40: return 40000;
    case STATION_JJY60: return 60000;
    case STATION_MSF:   return 60000;
    case STATION_WWVB:  return 60000;
    default:            return 68500;
  }
}

const char* timeproto_get_station_name(time_station_t station) {
  switch (station) {
    case STATION_BPC:   return "BPC (China 68.5 kHz)";
    case STATION_WWVB:  return "WWVB (USA 60.0 kHz)";
    case STATION_MSF:   return "MSF (UK 60.0 kHz)";
    case STATION_DCF77: return "DCF77 (Germany 77.5 kHz)";
    case STATION_JJY40: return "JJY40 (Japan 40.0 kHz)";
    case STATION_JJY60: return "JJY60 (Japan 60.0 kHz)";
    default:            return "Unknown";
  }
}

uint16_t timeproto_get_frame_seconds(time_station_t station) {
  (void)station;
  return 60; // All stations generate a full 60-second minute buffer
}

int32_t timeproto_get_native_utc_offset(time_station_t station) {
  switch (station) {
    case STATION_BPC:   return 28800; // CST (UTC+8)
    case STATION_DCF77: return 3600;  // CET (UTC+1, winter)
    case STATION_JJY40:
    case STATION_JJY60: return 32400; // JST (UTC+9)
    case STATION_MSF:
    case STATION_WWVB:
    default:            return 0;     // UTC
  }
}

void timeproto_generate_frame(time_station_t station,
                              uint64_t epoch_sec,
                              int32_t user_offset_sec,
                              int16_t dut1_ms,
                              uint8_t *out_level,
                              uint16_t *out_num_ticks) {
  // Clear buffer (all low by default)
  memset(out_level, 0, TIMEPROTO_MAX_LEVEL_BYTES);
  *out_num_ticks = 60 * TSIG_TICKS_PER_SEC; // 1200 ticks

  int32_t total_offset_sec = timeproto_get_native_utc_offset(station) + user_offset_sec;
  double adjusted_timestamp_ms = ((double)epoch_sec + total_offset_sec) * 1000.0;
  tsig_datetime_t dt = tsig_datetime_parse_timestamp(adjusted_timestamp_ms);

  switch (station) {
    case STATION_BPC:
      tsig_xmit_bpc(dt, dut1_ms, out_level);
      break;
    case STATION_DCF77:
      tsig_xmit_dcf77(dt, dut1_ms, out_level);
      break;
    case STATION_JJY40:
    case STATION_JJY60:
      tsig_xmit_jjy(dt, dut1_ms, out_level);
      break;
    case STATION_MSF:
      tsig_xmit_msf(dt, dut1_ms, out_level);
      break;
    case STATION_WWVB:
      tsig_xmit_wwvb(dt, dut1_ms, out_level);
      break;
    default:
      break;
  }
}
