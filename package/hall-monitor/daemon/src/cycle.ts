import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';
import { BYTES_PER_KIB, } from '@monochromatic-dev/const-core/ts';

import {
  analyze,
  parseVerdict,
} from './analyze.ts';
import {
  start as startLlama,
  stop as stopLlama,
} from './analyze/llama.ts';
import {
  getRecent,
  store,
} from './analyze/memory.ts';
import { isBlackFrame, } from './infra/blackdetect.ts';
import {
  captureScreenshot,
  captureWebcam,
} from './infra/capture.ts';
import { sendNotification, } from './infra/notification.ts';
import { isScreenLocked, } from './infra/screenlock.ts';
import { log, } from './infra/syslog.ts';

//region Decision tracking
// Sliding window of last 5 verdicts. A ring buffer would work here, but a
// fixed-length tuple gives compile-time size enforcement and the shift cost
// for 5 elements is negligible.

/**
 Possible productivity verdict values.
 */
type Decision = 'PRODUCTIVE' | 'UNPRODUCTIVE';

/**
 Fixed-size tuple tracking the 5 most recent verdicts.
 */
type DecisionWindow = [
  Decision,
  Decision,
  Decision,
  Decision,
  Decision,
];

/**
 Module-singleton mutable state for the rolling decision window; wrapped so it satisfies no-module-root-let.
 */
const state: { decisions: DecisionWindow; } = {
  decisions: [
    'PRODUCTIVE',
    'PRODUCTIVE',
    'PRODUCTIVE',
    'PRODUCTIVE',
    'PRODUCTIVE',
  ],
};
//endregion

/**
 Checks whether a decision is unproductive.
 
 @param d - decision to check
 
 @returns true when the decision is UNPRODUCTIVE
 */
function isUnproductive(d: Decision,): boolean {
  return d === 'UNPRODUCTIVE';
}

/**
 Executes one capture-analyze-notify cycle.
 Captures a screenshot ({@link captureScreenshot}) and webcam frame
 ({@link captureWebcam}), buffers the pair via {@link store} and reads the
 recent window back with {@link getRecent}, feeds them to the local vision
 LLM by bracketing {@link analyze} with {@link startLlama} and
 {@link stopLlama}, records the verdict via {@link parseVerdict}, and sends
 a desktop notification ({@link sendNotification}) when 5 consecutive
 cycles are unproductive.
 
 Skips the cycle when {@link isScreenLocked} reports the screen is locked
 or {@link isBlackFrame} reports the webcam cover is down.
 
 @example
 ```ts
 await cycle();
 ```
 */
export async function cycle(): Promise<void> {
  // Skip the entire cycle when the session is locked: no point capturing
  // a lock screen, and it avoids waking the GPU for inference.
  if (await isScreenLocked()) {
    log.debug('[cycle] Screen is locked: skipping cycle',);
    return;
  }

  /**
   Capture-cycle timestamp; reused for both the log line and the buffered capture set.
   */
  const ts = Date.now();
  log.debug(`[${new Date(ts,).toLocaleTimeString()}] Starting capture cycle...`,);

  try {
    /**
     Screenshot and webcam buffers captured in parallel to keep latency minimal.
     */
    const [screenshot, webcam,] = await Promise.all([
      captureScreenshot(),
      captureWebcam(),
    ],);

    // If the webcam image is mostly black the privacy cover is probably down;
    // skip analysis so the user isn't penalised while away.
    if (await isBlackFrame(webcam,)) {
      log.debug('[cycle] Webcam image is black (cover down?), skipping cycle',);
      return;
    }
    log.debug(
      `[capture] Screenshot: ${
        (screenshot.length
          / BYTES_PER_KIB).toFixed(0,)
      }KiB, Webcam: ${(webcam.length
        / BYTES_PER_KIB).toFixed(0,)}KiB`,
    );

    store({
      timestamp: ts,
      screenshot,
      webcam,
    },);
    /**
     Recent capture sets snapshot fed to the LLM; includes the just-stored entry.
     */
    const sets = getRecent();
    log.debug(`[memory] ${sets.length} capture set(s) in buffer`,);

    await startLlama();
    /**
     Raw LLM response text; both logged verbatim and parsed for the verdict line.
     */
    const result = await analyze(sets,);
    await stopLlama();

    /**
     Verdict extracted from the LLM response and pushed into the sliding decision buffer.
     */
    const verdict = parseVerdict(result,);
    /* oxlint-disable no-magic-numbers -- sliding window indices 1..4 */
    state.decisions = [
      state.decisions[1],
      state.decisions[2],
      state.decisions[3],
      state.decisions[4],
      verdict,
    ];
    /* oxlint-enable no-magic-numbers */
    /**
     Count of unproductive verdicts in the current 5-cycle window; surfaced in the log line as `streak: N/5`.
     */
    const streakCount = state
      .decisions
      .filter(function checkUnproductive(d,) {
        return isUnproductive(d,);
      },)
      .length;

    log.info(`[report] ${result}`,);
    log.info(`[verdict] ${verdict} (streak: ${streakCount}/5)`,);

    if (state.decisions
      .every(function checkUnproductive(d,) {
        return isUnproductive(d,);
      },)) {
      await sendNotification(result,);
      state.decisions = [
        'PRODUCTIVE',
        'PRODUCTIVE',
        'PRODUCTIVE',
        'PRODUCTIVE',
        'PRODUCTIVE',
      ];
    }
  }
  catch (err: unknown) {
    /**
     Normalised error string so both Error instances and arbitrary throws log readable output.
     */
    const message = caughtValueText(err,);
    console.error(`[error] ${message}`,);
    log.error(`[error] ${message}`,);
    try {
      await stopLlama();
    }
    catch (cleanupError) {
      /**
       Cleanup failure string for best-effort GPU process shutdown logging.
       */
      const cleanupMessage = caughtValueText(cleanupError,);
      log.debug(`[cleanup] ${cleanupMessage}`,);
    }
  }
}
