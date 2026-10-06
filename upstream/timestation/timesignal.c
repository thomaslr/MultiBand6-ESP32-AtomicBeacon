/**
 * WebAssembly C module for generating an emulated time station radio signal.
 *
 * Copyright © 2023 James Seo <james@equiv.tech> (MIT license).
 *
 * Uses the Emscripten Wasm Audio Worklets API to implement a Wasm-based Web
 * Audio AudioWorkletProcessor.process() method which runs in a privileged
 * real-time Audio Worklet thread. The Wasm Audio Worklets API also provides
 * JS-to-Wasm glue code validated not to create JS garbage in hot paths.
 *
 * Building this module requires a number of emcc flags, e.g.:
 *
 *  emcc timesignal.c -o timesignal.js -sEXPORT_NAME=createTimeSignalModule \
 *    -sMODULARIZE -sAUDIO_WORKLET -sWASM_WORKERS -sEXPORT_ES6 \
 *    -sALLOW_TABLE_GROWTH -sSTACK_SIZE=32768 -sINITIAL_MEMORY=65536 -sMALLOC=none \
 *    -sEXPORTED_RUNTIME_METHODS="addFunction,emscriptenGetAudioObject,emscriptenRegisterAudioObject,wasmTable"
 *
 * timesignal.js, timesignal.wasm, timesignal.aw.js, and timesignal.ww.js are
 * created. Only the first 3 are necessary; the .ww.js file can be deleted.
 * However, timesignal.js needs to be modified before it can be used.
 *
 * Prior to v3.1.54 (cf. https://github.com/emscripten-core/emscripten/pull/21192),
 * emscripten generates broken JS glue code for a module using the Wasm Audio
 * Worklets API if compiled with -sEXPORT_ES6. The Wasm binary is fetched in
 * AudioWorkletGlobalScope instead of the main thread, so URL() is undefined.
 * A fix is to turn this:
 *
 *  var wasmBinaryFile;
 *  if (Module['locateFile']) {
 *    wasmBinaryFile = 'timesignal.wasm';
 *    if (!isDataURI(wasmBinaryFile)) {
 *      wasmBinaryFile = locateFile(wasmBinaryFile);
 *    }
 *  } else {
 *    // Use bundler-friendly `new URL(..., import.meta.url)` pattern; works in browsers too.
 *    wasmBinaryFile = new URL('timesignal.wasm', import.meta.url).href;
 *  }
 *
 * into something like this:
 *
 *  var wasmBinaryFile = 'timesignal.wasm';
 *  if (!isDataURI(wasmBinaryFile))
 *    wasmBinaryFile = locateFile(wasmBinaryFile);
 *
 * Also, this:
 *
 *  audioWorklet.addModule('timesignal.aw.js').then(() => {
 *
 * will fail if timesignal.aw.js is not in the server root directory. That is
 * unlikely, so the URL string must either be modified to have the correct
 * prefix relative to the server root or be wrapped in a call to locateFile().
 *
 * The final hurdle is that these changes must be made on minified JS if this
 * module was compiled with -O3.
 *
 * Once built and loaded, use from JavaScript as follows:
 *
 * 0. Define two JS functions to be registered with this module as callbacks.
 *    - The first function will be called with a handle (an opaque pointer-like
 *      number) to the AudioWorkletNode that results from module initialization.
 *    - The second function will be called upon module state transitions (see
 *      timesignal.h) to notify JS in the main thread whenever such a state
 *      transition has occurred.
 *
 * 1. Register the two JS callbacks with this module via addFunction(), which
 *    returns two callback handles. Hold onto them.
 *
 * 2. Create an AudioContext in JS as per usual when using the Web Audio API,
 *    Register it with this module via emscriptenRegisterAudioObject(), which
 *    returns a handle to the AudioContext. Hold onto it.
 *
 * 3. Initialize this module by calling tsig_init(), which takes the 3 handles
 *    from 1) and 2) and the AudioContext's sample rate as its parameters (the
 *    latter because JS object property access in Wasm seems to have a C++
 *    Emscripten API but not a C API). Eventually, the first callback from 0)
 *    is called with a handle to an AudioWorkletNode, which is a good point at
 *    which to (in JS)...
 *
 * 4. Use emscriptenGetAudioObject() on said AudioWorkletNode handle to obtain
 *    an AudioWorkletNode (just like the result of `new AudioWorkletNode()`).
 *    Connect the AudioWorkletNode to the AudioContext as per usual,
 *    e.g. with  `audioWorkletNode.connect(audioContext.destination)`.
 *
 * 5. With the AudioContext's audio routing graph thus built, call
 *    AudioContext.resume(). This returns a Promise that resolves when the
 *    AudioContext has been resumed, which is a good point at which to...
 *
 * 6. Call tsig_start(). The goal is to load user params into the module, but
 *    not just yet. Eventually, the second callback from 0) is called with the
 *    state `TSIG_STATE_REQ_PARAMS`, which is a good point at which to...
 *
 * 6. Call tsig_load_params() to load user params. At last, the module
 *    begins generating and outputting a time station "radio signal".
 *
 * 7. Shutting the module down is another roundabout process that begins with
 *    a call to tsig_stop(). Eventually, the second callback from 0) is called
 *    with the module state `TSIG_STATE_REQ_IDLE`, which is a good point at
 *    which to call AudioContext.suspend().
 *
 * 8. For subsequent startups, simply GOTO 5.
 */

#include <stdio.h>
#include <stdatomic.h>
#include <stdint.h>
#include <emscripten/emscripten.h>
#include <emscripten/webaudio.h>
#include "timesignal.h"
#include "datetime.h"
#include "waveform.h"

/** AudioWorkletProcessor thread state. */
typedef struct tsig_ctx_t {
  /** Overall state of time signal generator module. */
  atomic_int state;

  /** Thread-local copy of user parameters. */
  tsig_params_t params;

  /** Waveform context. */
  tsig_waveform_ctx_t waveform_ctx;

  /** Count of render quantums to delay when starting/stopping. */
  uint32_t delay_quantums;
} tsig_ctx_t;

/** Global stack for all threads in AudioWorkletGlobalScope.*/
uint8_t tsig_awp_stack[TSIG_AWP_STACK_SIZE];

/**
 * JavaScript callback that looks like a C function pointer.
 *
 * Invoked from Wasm to export things like module state changes to JS.
 */
tsig_js_cb_func tsig_js_cb;

tsig_params_t tsig_params = {};
tsig_ctx_t tsig_ctx = {};

static inline uint8_t rearm_state_transition_delay() {
  tsig_ctx.delay_quantums =
      (tsig_ctx.waveform_ctx.sample_rate * TSIG_DELAY_MS) /
      (1000 * TSIG_RENDER_QUANTUM);
  return 1;
}

static inline uint8_t is_state_transition_delay_finished() {
  if (tsig_ctx.delay_quantums && !--tsig_ctx.delay_quantums)
    return rearm_state_transition_delay();
  return 0;
}

/**
 * Process `TSIG_RENDER_QUANTUM` samples of audio.
 * @param n_inputs Count of audio input channels.
 * @param inputs Array of audio input buffers.
 * @param n_outputs Count of audio output channels.
 * @param outputs Array of audio output buffers.
 * @param n_params Count of audio parameters. Unused.
 * @param params Array of audio parameters. Unused.
 * @param userdata Pointer to user data. Unused.
 * @return Always `EM_TRUE`.
 * @note Equivalent to AudioWorkletProcessor.process(). Runs in a real-time
 *  Audio Worklet thread within AudioWorkletGlobalScope.
 */
EM_BOOL tsig_awp_process_cb(int n_inputs, const AudioSampleFrame *inputs,
                            int n_outputs, AudioSampleFrame *outputs,
                            int n_params, const AudioParamFrame *params,
                            void *userdata) {
  int state = atomic_load(&tsig_ctx.state);
  int next_state = state;
  uint8_t silent = 1;

  switch (state) {
    /* Default state immediately following AudioContext.resume(). */
    case TSIG_STATE_IDLE:
      break;

    /*
     * JS in the main thread forced a state transition.
     * Wait for AudioContext.outputLatency to become available.
     */
    case TSIG_STATE_STARTUP:
      if (is_state_transition_delay_finished())
        next_state = TSIG_STATE_REQ_PARAMS;
      break;

    /* JS will notice our state transition and send params. */
    case TSIG_STATE_REQ_PARAMS:
      break;

    /* JS sent params and forced a state transition. */
    case TSIG_STATE_LOAD_PARAMS:
      tsig_ctx.params = tsig_params;

      tsig_waveform_init(&tsig_ctx.waveform_ctx, &tsig_ctx.params);

#ifdef TSIG_DEBUG
      printf("Wasm loaded params at %f, phase delta is %u / %u\n",
             tsig_ctx.waveform_ctx.timestamp, tsig_ctx.waveform_ctx.phase_delta,
             tsig_ctx.waveform_ctx.phase_base);
#endif /* TSIG_DEBUG */

      next_state = TSIG_STATE_FADE_IN;
      break;

    /* Fade in to prevent crackling. Run until tsig_stop() forces us to stop. */
    case TSIG_STATE_FADE_IN:
    case TSIG_STATE_RUNNING:
    case TSIG_STATE_FADE_OUT:
      /* NOTE: tsig_waveform_generate() can initiate state transitions. */
      tsig_waveform_generate(&tsig_ctx.waveform_ctx, &tsig_ctx.params, state,
                             &next_state, n_outputs, outputs);
      silent = 0;
      break;

    /* Delay to ensure no audible pop occurs upon AudioContext.suspend(). */
    case TSIG_STATE_SUSPEND:
      if (is_state_transition_delay_finished())
        next_state = TSIG_STATE_IDLE;
      break;

    default:
      break;
  }

  /* Inform JS about state transitions we initiated. */
  if (next_state != state) {
    atomic_store(&tsig_ctx.state, next_state);
    /* Audio Worklet thread must not block. Inform JS via the main thread. */
    emscripten_audio_worklet_post_function_vi(EMSCRIPTEN_AUDIO_MAIN_THREAD,
                                              tsig_js_cb, next_state);
  }

  if (silent)
    tsig_waveform_generate_silence(n_outputs, outputs);

  return EM_TRUE;
}

/**
 * Create an AudioWorkletNode and export it to JavaScript.
 * @param audio_ctx Handle of a Web Audio API AudioContext.
 * @param success Whether we should create the AudioWorkletNode.
 * @param init_js_cb Pointer to JS callback that will be used for exporting a
 *  handle to the created AudioWorkletNode.
 * @note Final part of time signal generator module initialization.
 *  Invoked when AudioWorkletProcessor is added to AudioWorkletGlobalScope.
 *  Exported handle can be turned into a regular JS AudioWorkletNode object
 *  via emscriptenGetAudioObject().
 */
void tsig_awp_create_cb(EMSCRIPTEN_WEBAUDIO_T audio_ctx, EM_BOOL success,
                        void *init_js_cb) {
#ifdef TSIG_DEBUG
  printf("tsig_awp_create_cb(audio_ctx=%d, success=%d, init_js_cb=%p)\n",
         audio_ctx, success, init_js_cb);
#endif /* TSIG_DEBUG */

  if (!success)
    return;

  int request_output_channels[] = {1};

  EmscriptenAudioWorkletNodeCreateOptions options = {
      .numberOfInputs = 0,
      .numberOfOutputs = 1,
      .outputChannelCounts = request_output_channels,
  };

  EMSCRIPTEN_AUDIO_WORKLET_NODE_T awn_handle =
      emscripten_create_wasm_audio_worklet_node(
          audio_ctx, TSIG_AWP_NAME, &options, tsig_awp_process_cb, NULL);

  ((tsig_js_cb_func)init_js_cb)(awn_handle);
}

/**
 * Create and add an AudioWorkletProcessor to an AudioContext.
 * @param audio_ctx Handle of a Web Audio API AudioContext.
 * @param success Whether we should create the AudioWorkletProcessor.
 * @param init_js_cb Pointer to JS callback that will be used for exporting an
 *  AudioWorkletNode created as the result of module initialization.
 * @note Part of time signal generator module initialization. Invoked when
 *  Wasm module is added to AudioWorkletGlobalScope and is ready for a
 *  AudioWorkletProcessor to be attached to it.
 */
void tsig_aw_thread_init_cb(EMSCRIPTEN_WEBAUDIO_T audio_ctx, EM_BOOL success,
                            void *init_js_cb) {
#ifdef TSIG_DEBUG
  printf("tsig_aw_thread_init_cb(audio_ctx=%d, success=%d, init_js_cb=%p)\n",
         audio_ctx, success, init_js_cb);
#endif /* TSIG_DEBUG */

  if (!success)
    return;

  WebAudioWorkletProcessorCreateOptions opts = {.name = TSIG_AWP_NAME};
  emscripten_create_wasm_audio_worklet_processor_async(
      audio_ctx, &opts, tsig_awp_create_cb, init_js_cb);
}

/**
 * Initialize time signal generator Wasm module.
 * @param audio_ctx Handle of a Web Audio API AudioContext.
 * @param sample_rate Sample rate of the AudioContext.
 * @param init_js_cb Pointer to JS callback that will be used for exporting an
 *  AudioWorkletNode created as the result of module initialization.
 * @note Should be called once per page load. `audio_ctx` can be obtained in
 *  JS via emscriptenRegisterAudioObject() on a preexisting AudioContext.
 */
EMSCRIPTEN_KEEPALIVE void tsig_init(EMSCRIPTEN_WEBAUDIO_T audio_ctx,
                                    uint32_t sample_rate,
                                    tsig_js_cb_func init_js_cb,
                                    tsig_js_cb_func js_cb) {
#ifdef TSIG_DEBUG
  printf("tsig_init(audio_ctx=%d, sample_rate=%u, init_js_cb=%p, js_cb=%p)\n",
         audio_ctx, sample_rate, init_js_cb, js_cb);
#endif /* TSIG_DEBUG */

  atomic_store(&tsig_ctx.state, TSIG_STATE_IDLE);
  tsig_ctx.waveform_ctx.sample_rate = sample_rate;
  rearm_state_transition_delay();
  tsig_js_cb = js_cb;

  emscripten_start_wasm_audio_worklet_thread_async(
      audio_ctx, tsig_awp_stack, sizeof(tsig_awp_stack), tsig_aw_thread_init_cb,
      init_js_cb);
}

/** Start generating a time station signal. */
EMSCRIPTEN_KEEPALIVE void tsig_start() {
  atomic_store(&tsig_ctx.state, TSIG_STATE_STARTUP);
  tsig_js_cb(TSIG_STATE_STARTUP);
}

/**
 * Load user params.
 * @param offset User offset in milliseconds.
 * @param station Time station.
 * @param jjy_khz JJY frequency.
 * @param dut1 DUT1 value in milliseconds.
 * @param audible Whether to make generated waveform audible.
 * @param noclip Whether to interpolate gain changes.
 * @note Should be called by JS in response to being notified of a state
 *  transition to `TSIG_STATE_REQ_PARAMS`.
 */
EMSCRIPTEN_KEEPALIVE void tsig_load_params(double offset, uint8_t station,
                                           uint8_t jjy_khz, int16_t dut1,
                                           uint8_t audible, uint8_t noclip) {
#ifdef TSIG_DEBUG
  printf(
      "tsig_load_params(offset=%f, station=%u, jjy_khz=%u, dut1=%d, "
      "audible=%d, noclip=%d);\n",
      offset, station, jjy_khz, dut1, audible, noclip);
#endif /* TSIG_DEBUG */

  tsig_params.offset = offset;
  tsig_params.station = station;
  tsig_params.jjy_khz = jjy_khz;
  tsig_params.dut1 = dut1;
  tsig_params.audible = audible;
  tsig_params.noclip = noclip;

  atomic_store(&tsig_ctx.state, TSIG_STATE_LOAD_PARAMS);
  tsig_js_cb(TSIG_STATE_LOAD_PARAMS);
}

/** Stop generating a time station signal. */
EMSCRIPTEN_KEEPALIVE void tsig_stop() {
  int state = atomic_load(&tsig_ctx.state);
  int next_state = TSIG_STATE_FADE_OUT;

  /* No need to fade out if playback never started. */
  if (state < TSIG_STATE_FADE_IN) {
    rearm_state_transition_delay();
    next_state = TSIG_STATE_IDLE;
  }

  atomic_store(&tsig_ctx.state, next_state);
  tsig_js_cb(next_state);
}

#ifdef TSIG_DEBUG
EMSCRIPTEN_KEEPALIVE uint32_t tsig_print_timestamp(double timestamp, int n) {
  uint32_t ret = 0;

  tsig_datetime_t datetime = tsig_datetime_parse_timestamp(timestamp);

  datetime_print(datetime);

  double now = emscripten_get_now();

  for (int i = 0; i < n; i++) {
    tsig_datetime_t datetime = tsig_datetime_parse_timestamp(timestamp);
    ret += datetime.day;
  }

  double later = emscripten_get_now();

  printf("%f\n", later - now);

  return ret;
}
#endif /* TSIG_DEBUG */
