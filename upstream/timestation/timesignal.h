#pragma once

#include <stdatomic.h>
#include <stdint.h>
#include <limits.h>

/**
 * Fixed count of audio samples processed per render.
 *
 * Technically variable, but as of late 2023 all browsers and Emscripten
 * hard-code it to 128.
 */
#define TSIG_RENDER_QUANTUM 128

#define TSIG_AWP_NAME       "time-signal" /** Name of AudioWorkletProcessor. */
#define TSIG_AWP_STACK_SIZE 4096          /** AWP thread stack size. */

#define TSIG_FADE_MS  35
#define TSIG_DELAY_MS 465

#define TSIG_STATION_BPC   0
#define TSIG_STATION_DCF77 1
#define TSIG_STATION_JJY   2
#define TSIG_STATION_MSF   3
#define TSIG_STATION_WWVB  4

#define TSIG_JJYKHZ_40 0
#define TSIG_JJYKHZ_60 1

#define TSIG_STATE_IDLE        0
#define TSIG_STATE_STARTUP     1
#define TSIG_STATE_REQ_PARAMS  2
#define TSIG_STATE_LOAD_PARAMS 3
#define TSIG_STATE_FADE_IN     4
#define TSIG_STATE_RUNNING     5
#define TSIG_STATE_FADE_OUT    6
#define TSIG_STATE_SUSPEND     7

typedef void (*tsig_js_cb_func)(int data);

/** User parameters. */
typedef struct tsig_params_t {
  double offset;   /** User offset in milliseconds. */
  uint8_t station; /** Time station. */
  uint8_t jjy_khz; /** JJY frequency. */
  int16_t dut1;    /** DUT1 value in milliseconds. */
  uint8_t audible; /** Whether to make generated waveform audible. */
  uint8_t noclip;  /** Whether to interpolate gain changes. */
} tsig_params_t;

static inline int64_t tsig_min(int64_t a, int64_t b) {
  return a < b ? a : b;
}

static inline int64_t tsig_max(int64_t a, int64_t b) {
  return a < b ? a : b;
}
