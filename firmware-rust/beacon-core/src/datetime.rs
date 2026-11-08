//! Date and time calendar utility functions for atomic time signal encoding.
//!
//! Adapted from timestation (https://github.com/kangtastic/timestation)
//! Copyright (c) 2023 James Seo <james@equiv.tech> (MIT License)

pub const TSIG_DATETIME_NOT_SOON: u32 = u32::MAX;
pub const TSIG_DATETIME_MSECS_DAY: u64 = 86_400_000;
pub const TSIG_DATETIME_MSECS_HOUR: u64 = 3_600_000;
pub const TSIG_DATETIME_MSECS_MIN: u64 = 60_000;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TsigDateTime {
    pub timestamp: f64,
    pub year: u16,
    pub mon: u8,
    pub day: u8,
    pub doy: u16,
    pub dow: u8, // 0 = Sunday, 1 = Monday, ..., 6 = Saturday
    pub hour: u8,
    pub min: u8,
    pub sec: u8,
    pub msec: u16,
}

#[inline]
pub fn is_leap(year: u16) -> bool {
    (year % 4 == 0) && (year % 100 != 0 || year % 400 == 0)
}

pub fn parse_timestamp(timestamp: f64) -> TsigDateTime {
    let mut msec = timestamp as u64;

    let day = msec / TSIG_DATETIME_MSECS_DAY;
    let dse = day + 719468;
    let era = (dse / 146097) as u32;
    let doe = (dse - (era as u64) * 146097) as u32;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let m = (5 * doy + 2) / 153;

    let year = (y + (if m >= 10 { 1 } else { 0 })) as u16;
    let mon = if m < 10 { (m + 3) as u8 } else { (m - 9) as u8 };
    let day_of_mon = (doy - (153 * m + 2) / 5 + 1) as u8;
    let doy_res = if m < 10 {
        (doy + 60 + (if is_leap(year) { 1 } else { 0 })) as u16
    } else {
        (doy - 305) as u16
    };
    let dow = ((day + 4) % 7) as u8;

    msec %= TSIG_DATETIME_MSECS_DAY;
    let hour = (msec / TSIG_DATETIME_MSECS_HOUR) as u8;
    msec %= TSIG_DATETIME_MSECS_HOUR;
    let min = (msec / TSIG_DATETIME_MSECS_MIN) as u8;
    msec %= TSIG_DATETIME_MSECS_MIN;
    let sec = (msec / 1000) as u8;
    let msec_res = (msec % 1000) as u16;

    TsigDateTime {
        timestamp,
        year,
        mon,
        day: day_of_mon,
        doy: doy_res,
        dow,
        hour,
        min,
        sec,
        msec: msec_res,
    }
}

pub fn is_eu_dst(datetime: &TsigDateTime, out_in_mins: &mut u32) -> bool {
    let mut in_mins = TSIG_DATETIME_NOT_SOON;
    let mon = datetime.mon;
    let mut is_est = false;

    if (4..=9).contains(&mon) {
        is_est = true;
    } else if mon == 3 || mon == 10 {
        let hour = datetime.hour;
        let min = datetime.min;
        let day = datetime.day;
        let dow = datetime.dow;

        let fsom = (((day as i32 - 1) + (if dow != 0 { 7 - dow as i32 } else { 0 })) % 7) + 1;
        let lsom = fsom + ((31 - fsom) / 7) * 7;
        let is_changed = (day as i32 == lsom && hour >= 1) || day as i32 > lsom;

        is_est = (mon == 3) == is_changed;

        if day as i32 == lsom - 1 {
            in_mins = (60 * (24 - hour as i32) + 60 - min as i32) as u32;
        } else if day as i32 == lsom && hour < 1 {
            in_mins = (60 - min as i32) as u32;
        }
    }

    *out_in_mins = in_mins;
    is_est
}

pub fn is_us_dst(datetime: &TsigDateTime, out_end: &mut u8) -> u8 {
    let mon = datetime.mon;
    let mut is_dst_end = 0u8;
    let mut is_dst = 0u8;

    if (4..=10).contains(&mon) {
        is_dst_end = 1;
        is_dst = 1;
    } else if mon == 3 || mon == 11 {
        let sunday = if mon == 3 { 8 } else { 1 };
        let day = datetime.day as i32;
        let dow = datetime.dow as i32;

        let change_day = (((day - 1) + (if dow != 0 { 7 - dow } else { 0 })) % 7) + sunday;
        is_dst_end = if mon == 3 {
            if day >= change_day { 1 } else { 0 }
        } else {
            if day < change_day { 1 } else { 0 }
        };
        is_dst = if mon == 3 {
            if day > change_day { 1 } else { 0 }
        } else {
            if day <= change_day { 1 } else { 0 }
        };
    }

    *out_end = is_dst_end;
    is_dst
}
