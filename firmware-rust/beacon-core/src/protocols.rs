//! Radio Atomic Clock Protocol Encoders (BPC, WWVB, MSF, DCF77, JJY40, JJY60)
//!
//! Adapted from timestation (https://github.com/kangtastic/timestation)
//! Copyright (c) 2023 James Seo <james@equiv.tech> (MIT License)

use serde::{Deserialize, Serialize};
use crate::datetime::{self, TsigDateTime};

pub const TSIG_TICKS_PER_SEC: usize = 20;
pub const TSIG_TICK_MS: usize = 50;
pub const TIMEPROTO_MAX_LEVEL_BYTES: usize = 150; // 60s * 20 ticks / 8 = 150 bytes
pub const TOTAL_FRAME_TICKS: usize = 60 * TSIG_TICKS_PER_SEC; // 1200 ticks

const TSIG_WAVEFORM_SYNC_MARKER: u8 = 0xff;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum TimeStation {
    Bpc = 0,    // China (68.5 kHz, 20s frame)
    Wwvb = 1,   // USA (60.0 kHz, 60s frame)
    Msf = 2,    // UK (60.0 kHz, 60s frame)
    Dcf77 = 3,  // Germany (77.5 kHz, 60s frame)
    Jjy40 = 4,  // Japan East (40.0 kHz, 60s frame)
    Jjy60 = 5,  // Japan West (60.0 kHz, 60s frame)
}

pub const ALL_STATIONS: [TimeStation; 6] = [
    TimeStation::Bpc,
    TimeStation::Wwvb,
    TimeStation::Msf,
    TimeStation::Dcf77,
    TimeStation::Jjy40,
    TimeStation::Jjy60,
];

impl TimeStation {
    pub fn from_u8(val: u8) -> Option<Self> {
        match val {
            0 => Some(TimeStation::Bpc),
            1 => Some(TimeStation::Wwvb),
            2 => Some(TimeStation::Msf),
            3 => Some(TimeStation::Dcf77),
            4 => Some(TimeStation::Jjy40),
            5 => Some(TimeStation::Jjy60),
            _ => None,
        }
    }

    pub fn carrier_freq_hz(&self) -> u32 {
        match self {
            TimeStation::Bpc => 68_500,
            TimeStation::Dcf77 => 77_500,
            TimeStation::Jjy40 => 40_000,
            TimeStation::Jjy60 => 60_000,
            TimeStation::Msf => 60_000,
            TimeStation::Wwvb => 60_000,
        }
    }

    pub fn station_name(&self) -> &'static str {
        match self {
            TimeStation::Bpc => "BPC (China 68.5 kHz)",
            TimeStation::Wwvb => "WWVB (USA 60.0 kHz)",
            TimeStation::Msf => "MSF (UK 60.0 kHz)",
            TimeStation::Dcf77 => "DCF77 (Germany 77.5 kHz)",
            TimeStation::Jjy40 => "JJY40 (Japan 40.0 kHz)",
            TimeStation::Jjy60 => "JJY60 (Japan 60.0 kHz)",
        }
    }

    pub fn native_utc_offset_sec(&self) -> i32 {
        match self {
            TimeStation::Bpc => 28_800,  // CST (UTC+8)
            TimeStation::Dcf77 => 3_600, // CET (UTC+1)
            TimeStation::Jjy40 | TimeStation::Jjy60 => 32_400, // JST (UTC+9)
            TimeStation::Msf | TimeStation::Wwvb => 0,          // UTC
        }
    }
}

fn even_parity(data: &[u8], lo: usize, hi: usize) -> u8 {
    let mut parity = 0u8;
    for &byte in &data[lo..hi] {
        let mut b = byte;
        while b != 0 {
            parity ^= 1;
            b &= b - 1;
        }
    }
    parity
}

fn odd_parity(data: &[u8], lo: usize, hi: usize) -> u8 {
    1 ^ even_parity(data, lo, hi)
}

#[inline]
pub fn get_tick_level(level_buf: &[u8], tick_idx: usize) -> bool {
    ((level_buf[tick_idx / 8] >> (tick_idx % 8)) & 1) != 0
}

fn set_bit(buf: &mut [u8], idx: usize, val: bool) {
    if val {
        buf[idx / 8] |= 1 << (idx % 8);
    } else {
        buf[idx / 8] &= !(1 << (idx % 8));
    }
}

// -----------------------------------------------------------------------------
// BPC (China, 68.5 kHz)
// -----------------------------------------------------------------------------
fn xmit_bpc(dt: TsigDateTime, _dut1_ms: i16, xmit_level: &mut [u8; TIMEPROTO_MAX_LEVEL_BYTES]) {
    let mut bits = [0u8; 20];
    bits[0] = TSIG_WAVEFORM_SYNC_MARKER;

    let hour_12h = dt.hour % 12;
    bits[3] = (hour_12h >> 2) & 0x3;
    bits[4] = hour_12h & 0x3;

    let min = dt.min;
    bits[5] = (min >> 4) & 0x3;
    bits[6] = (min >> 2) & 0x3;
    bits[7] = min & 0x3;

    let dow = if dt.dow != 0 { dt.dow } else { 7 };
    bits[8] = (dow >> 2) & 0x1;
    bits[9] = dow & 0x3;

    let is_pm = if dt.hour >= 12 { 1 } else { 0 };
    bits[10] = (is_pm << 1) | even_parity(&bits, 1, 10);

    let day = dt.day;
    bits[11] = (day >> 4) & 0x1;
    bits[12] = (day >> 2) & 0x3;
    bits[13] = day & 0x3;

    let mon = dt.mon;
    bits[14] = (mon >> 2) & 0x3;
    bits[15] = mon & 0x3;

    let year = (dt.year % 100) as u8;
    bits[16] = (year >> 4) & 0x3;
    bits[17] = (year >> 2) & 0x3;
    bits[18] = year & 0x3;
    bits[19] = ((year >> 5) & 0x2) as u8 | even_parity(&bits, 11, 19);

    let mut j = 0usize;
    for p in 0..3 {
        bits[1] = p as u8;
        if p == 1 {
            bits[10] ^= 1;
        }

        for &b in &bits {
            let lo_dsec = if b == TSIG_WAVEFORM_SYNC_MARKER { 0 } else { b as usize + 1 };
            let lo = 100 * lo_dsec / TSIG_TICK_MS;
            let hi = TSIG_TICKS_PER_SEC - lo;

            for _ in 0..lo {
                set_bit(xmit_level, j, false);
                j += 1;
            }
            for _ in 0..hi {
                set_bit(xmit_level, j, true);
                j += 1;
            }
        }
    }
}

// -----------------------------------------------------------------------------
// DCF77 (Germany, 77.5 kHz)
// -----------------------------------------------------------------------------
fn xmit_dcf77(dt: TsigDateTime, _dut1_ms: i16, xmit_level: &mut [u8; TIMEPROTO_MAX_LEVEL_BYTES]) {
    let mut bits = [0u8; 60];
    bits[20] = 1;
    bits[59] = TSIG_WAVEFORM_SYNC_MARKER;

    let utc_offset = datetime::TSIG_DATETIME_MSECS_HOUR as f64;
    let utc_timestamp = dt.timestamp - utc_offset;
    let utc_datetime = datetime::parse_timestamp(utc_timestamp);

    let mut in_mins = 0u32;
    let is_cest = datetime::is_eu_dst(&utc_datetime, &mut in_mins);
    let is_xmit_cest = is_cest ^ (in_mins == 1);

    bits[16] = if in_mins <= 60 { 1 } else { 0 };
    bits[17] = if is_xmit_cest { 1 } else { 0 };
    bits[18] = if !is_xmit_cest { 1 } else { 0 };

    let cest_offset = if is_xmit_cest { datetime::TSIG_DATETIME_MSECS_HOUR as f64 } else { 0.0 };
    let xmit_offset = datetime::TSIG_DATETIME_MSECS_MIN as f64;
    let xmit_timestamp = dt.timestamp + cest_offset + xmit_offset;
    let xmit_datetime = datetime::parse_timestamp(xmit_timestamp);

    bits[20] = 1;

    let min = xmit_datetime.min % 10;
    bits[21] = min & 1;
    bits[22] = (min >> 1) & 1;
    bits[23] = (min >> 2) & 1;
    bits[24] = (min >> 3) & 1;

    let min_10 = xmit_datetime.min / 10;
    bits[25] = min_10 & 1;
    bits[26] = (min_10 >> 1) & 1;
    bits[27] = (min_10 >> 2) & 1;

    bits[28] = even_parity(&bits, 21, 28);

    let hour = xmit_datetime.hour % 10;
    bits[29] = hour & 1;
    bits[30] = (hour >> 1) & 1;
    bits[31] = (hour >> 2) & 1;
    bits[32] = (hour >> 3) & 1;

    let hour_10 = xmit_datetime.hour / 10;
    bits[33] = hour_10 & 1;
    bits[34] = (hour_10 >> 1) & 1;

    bits[35] = even_parity(&bits, 29, 35);

    let day = xmit_datetime.day % 10;
    bits[36] = day & 1;
    bits[37] = (day >> 1) & 1;
    bits[38] = (day >> 2) & 1;
    bits[39] = (day >> 3) & 1;

    let day_10 = xmit_datetime.day / 10;
    bits[40] = day_10 & 1;
    bits[41] = (day_10 >> 1) & 1;

    let dow = if xmit_datetime.dow != 0 { xmit_datetime.dow } else { 7 };
    bits[42] = dow & 1;
    bits[43] = (dow >> 1) & 1;
    bits[44] = (dow >> 2) & 1;

    let mon = xmit_datetime.mon % 10;
    bits[45] = mon & 1;
    bits[46] = (mon >> 1) & 1;
    bits[47] = (mon >> 2) & 1;
    bits[48] = (mon >> 3) & 1;

    let mon_10 = xmit_datetime.mon / 10;
    bits[49] = mon_10 & 1;

    let year = xmit_datetime.year % 10;
    bits[50] = (year & 1) as u8;
    bits[51] = ((year >> 1) & 1) as u8;
    bits[52] = ((year >> 2) & 1) as u8;
    bits[53] = ((year >> 3) & 1) as u8;

    let year_10 = (xmit_datetime.year % 100) / 10;
    bits[54] = (year_10 & 1) as u8;
    bits[55] = ((year_10 >> 1) & 1) as u8;
    bits[56] = ((year_10 >> 2) & 1) as u8;
    bits[57] = ((year_10 >> 3) & 1) as u8;

    bits[58] = even_parity(&bits, 36, 58);

    let mut j = 0usize;
    for &b in &bits {
        let lo_dsec = if b == TSIG_WAVEFORM_SYNC_MARKER {
            0
        } else {
            (if b != 0 { 1 } else { 0 }) + 1
        };
        let lo = 100 * lo_dsec / TSIG_TICK_MS;
        let hi = TSIG_TICKS_PER_SEC - lo;

        for _ in 0..lo {
            set_bit(xmit_level, j, false);
            j += 1;
        }
        for _ in 0..hi {
            set_bit(xmit_level, j, true);
            j += 1;
        }
    }
}

// -----------------------------------------------------------------------------
// JJY (Japan, 40.0 kHz & 60.0 kHz)
// -----------------------------------------------------------------------------
fn xmit_jjy(dt: TsigDateTime, _dut1_ms: i16, xmit_level: &mut [u8; TIMEPROTO_MAX_LEVEL_BYTES]) {
    let mut bits = [0u8; 60];
    bits[0] = TSIG_WAVEFORM_SYNC_MARKER;
    bits[9] = TSIG_WAVEFORM_SYNC_MARKER;
    bits[19] = TSIG_WAVEFORM_SYNC_MARKER;
    bits[29] = TSIG_WAVEFORM_SYNC_MARKER;
    bits[39] = TSIG_WAVEFORM_SYNC_MARKER;
    bits[49] = TSIG_WAVEFORM_SYNC_MARKER;
    bits[59] = TSIG_WAVEFORM_SYNC_MARKER;

    let min_10 = dt.min / 10;
    bits[1] = (min_10 >> 2) & 1;
    bits[2] = (min_10 >> 1) & 1;
    bits[3] = min_10 & 1;

    let min = dt.min % 10;
    bits[5] = (min >> 3) & 1;
    bits[6] = (min >> 2) & 1;
    bits[7] = (min >> 1) & 1;
    bits[8] = min & 1;

    let hour_10 = dt.hour / 10;
    bits[12] = (hour_10 >> 1) & 1;
    bits[13] = hour_10 & 1;

    let hour = dt.hour % 10;
    bits[15] = (hour >> 3) & 1;
    bits[16] = (hour >> 2) & 1;
    bits[17] = (hour >> 1) & 1;
    bits[18] = hour & 1;

    let doy_100 = (dt.doy / 100) as u8;
    bits[22] = (doy_100 >> 1) & 1;
    bits[23] = doy_100 & 1;

    let doy_10 = ((dt.doy % 100) / 10) as u8;
    bits[25] = (doy_10 >> 3) & 1;
    bits[26] = (doy_10 >> 2) & 1;
    bits[27] = (doy_10 >> 1) & 1;
    bits[28] = doy_10 & 1;

    let doy = (dt.doy % 10) as u8;
    bits[30] = (doy >> 3) & 1;
    bits[31] = (doy >> 2) & 1;
    bits[32] = (doy >> 1) & 1;
    bits[33] = doy & 1;

    bits[36] = even_parity(&bits, 12, 19);
    bits[37] = even_parity(&bits, 1, 9);

    // Always encode Year and Day-of-Week on all minutes (NICT discontinued
    // the :15/:45 Morse callsign override in March 2024, and watches require
    // Year/DOW to validate a date+time frame).
    let year_10 = ((dt.year % 100) / 10) as u8;
    bits[41] = (year_10 >> 3) & 1;
    bits[42] = (year_10 >> 2) & 1;
    bits[43] = (year_10 >> 1) & 1;
    bits[44] = year_10 & 1;

    let year = (dt.year % 10) as u8;
    bits[45] = (year >> 3) & 1;
    bits[46] = (year >> 2) & 1;
    bits[47] = (year >> 1) & 1;
    bits[48] = year & 1;

    let dow = dt.dow;
    bits[50] = (dow >> 2) & 1;
    bits[51] = (dow >> 1) & 1;
    bits[52] = dow & 1;

    let mut j = 0usize;
    for &b in &bits {
        let hi_dsec = if b == TSIG_WAVEFORM_SYNC_MARKER {
            2
        } else if b != 0 {
            5
        } else {
            8
        };
        let hi = 100 * hi_dsec / TSIG_TICK_MS;
        let lo = TSIG_TICKS_PER_SEC - hi;

        for _ in 0..hi {
            set_bit(xmit_level, j, true);
            j += 1;
        }
        for _ in 0..lo {
            set_bit(xmit_level, j, false);
            j += 1;
        }
    }
}

// -----------------------------------------------------------------------------
// MSF (UK, 60.0 kHz)
// -----------------------------------------------------------------------------
fn xmit_msf(dt: TsigDateTime, dut1_ms: i16, xmit_level: &mut [u8; TIMEPROTO_MAX_LEVEL_BYTES]) {
    let mut bits = [0u8; 60];
    bits[0] = TSIG_WAVEFORM_SYNC_MARKER;

    let mut dut1 = (dut1_ms / 100) as i8;
    let lt0 = if dut1 < 0 { 8 } else { 0 };
    if dut1 < 0 {
        dut1 = -dut1;
    }
    bits[1 + lt0] = if dut1 >= 1 { 1 } else { 0 };
    bits[2 + lt0] = if dut1 >= 2 { 1 } else { 0 };
    bits[3 + lt0] = if dut1 >= 3 { 1 } else { 0 };
    bits[4 + lt0] = if dut1 >= 4 { 1 } else { 0 };
    bits[5 + lt0] = if dut1 >= 5 { 1 } else { 0 };
    bits[6 + lt0] = if dut1 >= 6 { 1 } else { 0 };
    bits[7 + lt0] = if dut1 >= 7 { 1 } else { 0 };
    bits[8 + lt0] = if dut1 >= 8 { 1 } else { 0 };

    let mut in_mins = 0u32;
    let is_bst = datetime::is_eu_dst(&dt, &mut in_mins);

    let is_xmit_bst = is_bst ^ (in_mins == 1);
    let bst_offset = if is_xmit_bst { datetime::TSIG_DATETIME_MSECS_HOUR as f64 } else { 0.0 };
    let xmit_offset = datetime::TSIG_DATETIME_MSECS_MIN as f64;
    let xmit_timestamp = dt.timestamp + bst_offset + xmit_offset;
    let xmit_datetime = datetime::parse_timestamp(xmit_timestamp);

    let year_10 = ((xmit_datetime.year % 100) / 10) as u8;
    bits[17] = (year_10 >> 3) & 1;
    bits[18] = (year_10 >> 2) & 1;
    bits[19] = (year_10 >> 1) & 1;
    bits[20] = year_10 & 1;

    let year = (xmit_datetime.year % 10) as u8;
    bits[21] = (year >> 3) & 1;
    bits[22] = (year >> 2) & 1;
    bits[23] = (year >> 1) & 1;
    bits[24] = year & 1;

    let mon_10 = xmit_datetime.mon / 10;
    bits[25] = mon_10 & 1;

    let mon = xmit_datetime.mon % 10;
    bits[26] = (mon >> 3) & 1;
    bits[27] = (mon >> 2) & 1;
    bits[28] = (mon >> 1) & 1;
    bits[29] = mon & 1;

    let day_10 = xmit_datetime.day / 10;
    bits[30] = (day_10 >> 1) & 1;
    bits[31] = day_10 & 1;

    let day = xmit_datetime.day % 10;
    bits[32] = (day >> 3) & 1;
    bits[33] = (day >> 2) & 1;
    bits[34] = (day >> 1) & 1;
    bits[35] = day & 1;

    let dow = xmit_datetime.dow;
    bits[36] = (dow >> 2) & 1;
    bits[37] = (dow >> 1) & 1;
    bits[38] = dow & 1;

    let hour_10 = xmit_datetime.hour / 10;
    bits[39] = (hour_10 >> 1) & 1;
    bits[40] = hour_10 & 1;

    let hour = xmit_datetime.hour % 10;
    bits[41] = (hour >> 3) & 1;
    bits[42] = (hour >> 2) & 1;
    bits[43] = (hour >> 1) & 1;
    bits[44] = hour & 1;

    let min_10 = xmit_datetime.min / 10;
    bits[45] = (min_10 >> 2) & 1;
    bits[46] = (min_10 >> 1) & 1;
    bits[47] = min_10 & 1;

    let min = xmit_datetime.min % 10;
    bits[48] = (min >> 3) & 1;
    bits[49] = (min >> 2) & 1;
    bits[50] = (min >> 1) & 1;
    bits[51] = min & 1;

    bits[53] = if in_mins <= 61 { 1 } else { 0 };
    bits[54] = odd_parity(&bits, 17, 25);
    bits[55] = odd_parity(&bits, 25, 36);
    bits[56] = odd_parity(&bits, 36, 39);
    bits[57] = odd_parity(&bits, 39, 52);
    bits[58] = if is_xmit_bst { 1 } else { 0 };

    let mut j = 0usize;
    for i in 0..bits.len() {
        let mut dsec_lo = if bits[i] == TSIG_WAVEFORM_SYNC_MARKER {
            5
        } else {
            (if bits[i] != 0 { 1 } else { 0 }) + 1
        };
        if (53..=58).contains(&i) {
            dsec_lo += 1; // Secondary minute marker
        }
        let lo = 100 * dsec_lo / TSIG_TICK_MS;
        let hi = TSIG_TICKS_PER_SEC - lo;

        for _ in 0..lo {
            set_bit(xmit_level, j, false);
            j += 1;
        }
        for _ in 0..hi {
            set_bit(xmit_level, j, true);
            j += 1;
        }
    }
}

// -----------------------------------------------------------------------------
// WWVB (USA, 60.0 kHz)
// -----------------------------------------------------------------------------
fn xmit_wwvb(dt: TsigDateTime, dut1_ms: i16, xmit_level: &mut [u8; TIMEPROTO_MAX_LEVEL_BYTES]) {
    let mut bits = [0u8; 60];
    bits[0] = TSIG_WAVEFORM_SYNC_MARKER;
    bits[9] = TSIG_WAVEFORM_SYNC_MARKER;
    bits[19] = TSIG_WAVEFORM_SYNC_MARKER;
    bits[29] = TSIG_WAVEFORM_SYNC_MARKER;
    bits[39] = TSIG_WAVEFORM_SYNC_MARKER;
    bits[49] = TSIG_WAVEFORM_SYNC_MARKER;
    bits[59] = TSIG_WAVEFORM_SYNC_MARKER;

    let min_10 = dt.min / 10;
    bits[1] = (min_10 >> 2) & 1;
    bits[2] = (min_10 >> 1) & 1;
    bits[3] = min_10 & 1;

    let min = dt.min % 10;
    bits[5] = (min >> 3) & 1;
    bits[6] = (min >> 2) & 1;
    bits[7] = (min >> 1) & 1;
    bits[8] = min & 1;

    let hour_10 = dt.hour / 10;
    bits[12] = (hour_10 >> 1) & 1;
    bits[13] = hour_10 & 1;

    let hour = dt.hour % 10;
    bits[15] = (hour >> 3) & 1;
    bits[16] = (hour >> 2) & 1;
    bits[17] = (hour >> 1) & 1;
    bits[18] = hour & 1;

    let doy_100 = (dt.doy / 100) as u8;
    bits[22] = (doy_100 >> 1) & 1;
    bits[23] = doy_100 & 1;

    let doy_10 = ((dt.doy % 100) / 10) as u8;
    bits[25] = (doy_10 >> 3) & 1;
    bits[26] = (doy_10 >> 2) & 1;
    bits[27] = (doy_10 >> 1) & 1;
    bits[28] = doy_10 & 1;

    let doy = (dt.doy % 10) as u8;
    bits[30] = (doy >> 3) & 1;
    bits[31] = (doy >> 2) & 1;
    bits[32] = (doy >> 1) & 1;
    bits[33] = doy & 1;

    let mut dut1 = (dut1_ms / 100) as i8;
    bits[36] = if dut1 >= 0 { 1 } else { 0 };
    bits[37] = if dut1 < 0 { 1 } else { 0 };
    bits[38] = if dut1 >= 0 { 1 } else { 0 };
    if dut1 < 0 {
        dut1 = -dut1;
    }
    bits[40] = ((dut1 >> 3) & 1) as u8;
    bits[41] = ((dut1 >> 2) & 1) as u8;
    bits[42] = ((dut1 >> 1) & 1) as u8;
    bits[43] = (dut1 & 1) as u8;

    let year_10 = ((dt.year % 100) / 10) as u8;
    bits[45] = (year_10 >> 3) & 1;
    bits[46] = (year_10 >> 2) & 1;
    bits[47] = (year_10 >> 1) & 1;
    bits[48] = year_10 & 1;

    let year = (dt.year % 10) as u8;
    bits[50] = (year >> 3) & 1;
    bits[51] = (year >> 2) & 1;
    bits[52] = (year >> 1) & 1;
    bits[53] = year & 1;

    bits[55] = if datetime::is_leap(dt.year) { 1 } else { 0 };
    let mut dst_end = 0u8;
    // NIST WWVB spec: Bit 57 = DST status at 00:00 UTC today, Bit 58 = DST status at 24:00 UTC today
    bits[57] = datetime::is_us_dst(&dt, &mut dst_end);
    bits[58] = dst_end;

    let mut j = 0usize;
    for &b in &bits {
        let dsec_lo = if b == TSIG_WAVEFORM_SYNC_MARKER {
            8
        } else if b != 0 {
            5
        } else {
            2
        };
        let lo = 100 * dsec_lo / TSIG_TICK_MS;
        let hi = TSIG_TICKS_PER_SEC - lo;

        for _ in 0..lo {
            set_bit(xmit_level, j, false);
            j += 1;
        }
        for _ in 0..hi {
            set_bit(xmit_level, j, true);
            j += 1;
        }
    }
}

/// Generates a full 60-second frame bitstream for the given timestamp.
pub fn generate_frame(
    station: TimeStation,
    epoch_sec: u64,
    user_offset_sec: i32,
    dut1_ms: i16,
) -> ([u8; TIMEPROTO_MAX_LEVEL_BYTES], u16) {
    let mut out_level = [0u8; TIMEPROTO_MAX_LEVEL_BYTES];
    let num_ticks = TOTAL_FRAME_TICKS as u16;

    let total_offset_sec = station.native_utc_offset_sec() + user_offset_sec;
    let adjusted_timestamp_ms = ((epoch_sec as f64) + (total_offset_sec as f64)) * 1000.0;
    let dt = datetime::parse_timestamp(adjusted_timestamp_ms);

    match station {
        TimeStation::Bpc => xmit_bpc(dt, dut1_ms, &mut out_level),
        TimeStation::Dcf77 => xmit_dcf77(dt, dut1_ms, &mut out_level),
        TimeStation::Jjy40 | TimeStation::Jjy60 => xmit_jjy(dt, dut1_ms, &mut out_level),
        TimeStation::Msf => xmit_msf(dt, dut1_ms, &mut out_level),
        TimeStation::Wwvb => xmit_wwvb(dt, dut1_ms, &mut out_level),
    }

    (out_level, num_ticks)
}
