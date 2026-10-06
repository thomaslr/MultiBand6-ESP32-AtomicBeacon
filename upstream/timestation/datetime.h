#pragma once

#include <stdint.h>

#define TSIG_DATETIME_NOT_SOON   UINT32_MAX
#define TSIG_DATETIME_MSECS_DAY  86400000
#define TSIG_DATETIME_MSECS_HOUR 3600000
#define TSIG_DATETIME_MSECS_MIN  60000

/**
 * Date and time. Presented in a friendlier manner than a raw timestamp.
 * @note The original timestamp used to initialize this struct is found in
 *  `timestamp`. Partial milliseconds are not preserved in `msec`.
 */
typedef struct tsig_datetime_t {
  double timestamp; /** Unix timestamp in milliseconds. */
  uint16_t year;    /** Year (0 and up). */
  uint8_t mon;      /** Month (1-12). */
  uint8_t day;      /** Day of month (1-31). */
  uint16_t doy;     /** Day of year (1-366). */
  uint8_t dow;      /** Day of week (0-6, Sunday-Saturday). */
  uint8_t hour;     /** Hour (0-23). */
  uint8_t min;      /** Minute (0-59). */
  uint8_t sec;      /** Second (0-59). */
  uint16_t msec;    /** Millisecond (0-999). */
} tsig_datetime_t;

#ifdef TSIG_DEBUG
static void datetime_print(tsig_datetime_t datetime) {
  printf("datetime = {\n");
  printf("  .year = %u\n", datetime.year);
  printf("  .mon  = %u\n", datetime.mon);
  printf("  .day  = %u\n", datetime.day);
  printf("  .doy  = %u\n", datetime.doy);
  printf("  .dow  = %u\n", datetime.dow);
  printf("  .hour = %u\n", datetime.hour);
  printf("  .min  = %u\n", datetime.min);
  printf("  .sec  = %u\n", datetime.sec);
  printf("  .msec = %u\n", datetime.msec);
  printf("};\n");
}
#endif /* TSIG_DEBUG */

/**
 * Determine whether a year is a leap year.
 * @param year Gregorian year.
 * @return Whether the year is a Gregorian leap year.
 */
uint8_t tsig_datetime_is_leap(uint16_t year) {
  return !(year % 4) && ((year % 100) || !(year % 400));
}

/**
 * Parse a timestamp into a date and time.
 * @param timestamp Unix timestamp in milliseconds.
 * @return A tsig_datetime_t structure.
 */
tsig_datetime_t tsig_datetime_parse_timestamp(double timestamp) {
  tsig_datetime_t datetime = {.timestamp = timestamp};
  uint64_t msec = timestamp;

  /*
   * Certain date calculations are simplified by shifting the
   * epoch to begin on March 1, 0000 instead of January 1, 1970.
   * cf. https://howardhinnant.github.io/date_algorithms.html
   */

  uint64_t day = msec / TSIG_DATETIME_MSECS_DAY;
  uint64_t dse = day + 719468;
  uint32_t era = dse / 146097;
  uint32_t doe = dse - era * 146097;
  uint32_t yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
  uint32_t y = yoe + era * 400;
  uint32_t doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
  uint32_t m = (5 * doy + 2) / 153;

  datetime.year = y + (m >= 10);
  datetime.mon = m < 10 ? m + 3 : m - 9;      /* 1-12 */
  datetime.day = doy - (153 * m + 2) / 5 + 1; /* 1-31 */
  datetime.doy =
      m < 10 ? doy + 60 + tsig_datetime_is_leap(datetime.year) : doy - 305;
  datetime.dow = (day + 4) % 7;
  datetime.hour = (msec %= TSIG_DATETIME_MSECS_DAY) / TSIG_DATETIME_MSECS_HOUR;
  datetime.min = (msec %= TSIG_DATETIME_MSECS_HOUR) / TSIG_DATETIME_MSECS_MIN;
  datetime.sec = (msec %= TSIG_DATETIME_MSECS_MIN) / 1000;
  datetime.msec = msec % 1000;

  return datetime;
}

/**
 * Check if Summer Time is in effect in Germany or the United Kingdom.
 *
 * For Germany and the UK, as well as for many other countries in Europe,
 * summer time begins/ends at 01:00 UTC on the last Sunday of March/October.
 *
 * @param datetime UTC datetime to be checked.
 * @param[out] out_in_mins Optional out pointer to the count of minutes
 *  remaining until the next changeover at the beginning of the provided
 *  minute. `TSIG_DATETIME_NOT_SOON`, a large value, is stored if the next
 *  changeover will occur in more than 25 hours.
 * @return Whether CEST/BST are in effect in Germany/the UK at `datetime`.
 */
uint8_t tsig_datetime_is_eu_dst(tsig_datetime_t datetime,
                                uint32_t *out_in_mins) {
  uint32_t in_mins = TSIG_DATETIME_NOT_SOON;
  uint8_t mon = datetime.mon;
  uint8_t is_est = 0;

  if (3 < mon && mon < 10) {
    is_est = 1;
  } else if (mon == 3 || mon == 10) {
    uint8_t hour = datetime.hour;
    uint8_t min = datetime.min;
    uint8_t day = datetime.day;
    uint8_t dow = datetime.dow;

    uint8_t fsom = (((day - 1) + (dow ? 7 - dow : 0)) % 7) + 1;
    uint8_t lsom = fsom + ((31 - fsom) / 7) * 7;
    uint8_t is_changed = (day == lsom && hour >= 1) || day > lsom;

    is_est = (mon == 3) == is_changed;

    if (day == lsom - 1)
      in_mins = 60 * (24 - hour) + 60 - min;
    else if (day == lsom && hour < 1)
      in_mins = 60 - min;
  }

  if (out_in_mins)
    *out_in_mins = in_mins;

  return is_est;
}

/**
 * Check if Daylight Saving Time is in effect in the United States.
 *
 * Daylight Saving Time begins/ends at 02:00 local time on the second Sunday
 * of March/the first Sunday of November.
 *
 * @param datetime UTC datetime to be checked.
 * @param[out] out_end Optional out pointer to whether DST will be in effect
 *  at the end of the provided UTC day.
 * @return Whether DST is in effect in the United States at the beginning of
 *  the provided UTC day.
 */
uint8_t tsig_datetime_is_us_dst(tsig_datetime_t datetime, uint8_t *out_end) {
  uint8_t mon = datetime.mon;
  uint8_t is_dst_end = 0;
  uint8_t is_dst = 0;

  if (3 < mon && mon < 11) {
    is_dst_end = 1;
    is_dst = 1;
  } else if (mon == 3 || mon == 11) {
    uint8_t sunday = mon == 3 ? 8 : 1;
    uint8_t hour = datetime.hour;
    uint8_t day = datetime.day;
    uint8_t dow = datetime.dow;

    uint8_t change_day = (((day - 1) + (dow ? 7 - dow : 0)) % 7) + sunday;
    is_dst_end = mon == 3 ? day >= change_day : day < change_day;
    is_dst = mon == 3 ? day > change_day : day <= change_day;
  }

  if (out_end)
    *out_end = is_dst_end;

  return is_dst;
}
