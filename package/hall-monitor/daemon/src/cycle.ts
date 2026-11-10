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
import {
  DECISION_WINDOW_SIZE,
  initialDecisionWindow,
  recordVerdict,
  type DecisionWindow,
} from './analyze/decision-window.ts';
import { isBlackFrame, } from './infra/blackdetect.ts';
import {
  captureScreenshot,
  captureWebcam,
} from './infra/capture.ts';
import { sendNotification, } from './infra/notification.ts';
import { isScreenLocked, } from './infra/screenlock.ts';
import { log, } from './infra/syslog.ts';

//region Decision tracking
// The window of recent verdicts and the streak policy live in
// `analyze/decision-window.ts` as pure functions; this module only keeps the
// one live window between cycles.

/**
 Module-singleton mutable state for the rolling decision window; wrapped so it satisfies no-module-root-let.
 */
const state: { decisions: DecisionWindow; } = { decisions: initialDecisionWindow(), };
//endregion

/**
 Executes one capture-analyze-notify cycle.
 Captures a screenshot ({@link captureScreenshot}) and webcam frame
 ({@link captureWebcam}), buffers the pair via {@link store} and reads the
 recent window back with {@link getRecent}, feeds them to the local vision
 LLM by bracketing {@link analyze} with {@link startLlama} and
 {@link stopLlama}, records the verdict via {@link parseVerdict} and
 {@link recordVerdict}, and sends a desktop notification
 ({@link sendNotification}) when every verdict in the window is
 unproductive.
 
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
     Verdict extracted from the LLM response, recorded into the sliding decision window; the rest of the line
     reports its unproductive count and whether the user is told.
     */
    const verdict = parseVerdict(result,);
    /**
     The window with this verdict in it, its unproductive count, and whether it completes the streak.
     */
    const {
      window: filledWindow,
      streakCount,
      notified,
    } = recordVerdict({
      window: state.decisions,
      verdict,
    },);
    state.decisions = filledWindow;

    log.info(`[report] ${result}`,);
    log.info(`[verdict] ${verdict} (streak: ${streakCount}/${DECISION_WINDOW_SIZE})`,);

    if (notified) {
      // The window is reset only after the user has actually been told, so a failed notification keeps the
      // completed streak and the next unproductive cycle reports it again instead of losing it.
      await sendNotification(result,);
      state.decisions = initialDecisionWindow();
    }
  }
  catch (err: unknown) {
    /**
     Normalised error string so both Error instances and arbitrary throws log readable output.
     */
    const message = caughtValueText(err,);
    // One event, one record: the journal sink is the only channel, so a second
    // console write here filed every failure twice.
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
