/**
 The symbols this package's built-artifact tests reach, re-exported from one
 module.

 The package's only shipped entry is the compiled `hall-monitor` binary, which
 starts the daemon loop and never returns, so no test can reach a helper through
 it. This second entry exists so tests import those symbols from built output
 rather than from source, which `test-import/require-eventual-artifact` requires
 of every package with a build.

 Nothing in this package imports it, and no desktop session loads it: these
 exports are not part of the package's API, and they exist only so the built
 artifact under test is the one tests load.

 @module
 */

//region Analysis: the prompt boundary and the verdict the notification window reads

export {
  parseVerdict,
  skipSpacesAndTabs,
} from './analyze.ts';

//endregion Analysis

//region Decision window: the streak policy between verdicts and notifications

export {
  countUnproductive,
  DECISION_WINDOW_SIZE,
  everyDecisionUnproductive,
  initialDecisionWindow,
  recordVerdict,
  shiftDecisionWindow,
} from './analyze/decision-window.ts';
export type {
  Decision,
  DecisionWindow,
} from './analyze/decision-window.ts';

//endregion Decision window

//region Capture buffer: retention and capacity the cycle feeds the model from

export {
  appendedCaptureBuffer,
  getRecent,
  MAX_ENTRIES,
  retainedCaptureBuffer,
  RETENTION_MS,
  store,
} from './analyze/memory.ts';
export type {
  CaptureBuffer,
  CaptureSet,
} from './analyze/memory.ts';

//endregion Capture buffer

//region llama-server: starting a server, stopping one, and the end-of-child wait both depend on

export {
  awaitChildEnd,
  start as startLlamaServer,
  stop as stopLlamaServer,
} from './analyze/llama.ts';

//endregion llama-server

//region Daemon loop: the between-cycle wait and the shutdown that ends it

export {
  createShutdownSignal,
  runDaemonLoop,
  runDetachedShutdown,
} from './daemon-loop.ts';

//endregion Daemon loop

//region Capture: the private handoff directory and the ffmpeg run behind it

export {
  createScreenshotHandoff,
  handoffParentDirectory,
} from './infra/capture.ts';
export type { ScreenshotHandoff, } from './infra/capture.ts';
export { runFfmpeg, } from './infra/ffmpeg-runner.ts';

//endregion Capture

//region Single-instance lock: the takeover signal a stale owner races

export {
  signalProcess,
  splitOnWhitespace,
} from './infra/lock.ts';

//endregion Single-instance lock

//region Session and journal: the optional screen-lock probe and the record sink

export { isScreenLocked, } from './infra/screenlock.ts';
export {
  createJournalSink,
  journalArgumentsFor,
  reportExpectedFailure,
} from './infra/syslog.ts';

//endregion Session and journal
