/**
 Fixed values of the driver, each with where it comes from. The limits of a run are options, so none is shipped here.

 @module
 */

import type {
  PiRetryGaveUp,
  RunFailure,
} from './driver-types.ts';

/**
 Set in Pi's environment. A driver that finds it was started by a Pi the driver runs, and refuses, so an agent cannot start
 agents without end.
 */
export const NESTED_RUN_VARIABLE = 'MONOCHROMATIC_PI_SUBAGENT_DRIVER_RUNNING';

/**
 Milliseconds in a second.
 */
export const MILLISECONDS_PER_SECOND = 1_000;

/**
 Milliseconds between looks at the inbox, the stop file and the limits. Only how soon a steer or a stop is noticed depends on it.
 */
export const POLL_MILLISECONDS = 300;

/**
 Longest silence Pi may keep while a command waits for its answer. A compaction Pi reports under way is not silence, since Pi is
 working on its own session then, so this bound stops counting until the compaction ends; the deadline a run gives the wait is
 the outer bound.

 This is the bound of an answer Pi owes while it is already at work, such as a steer from the inbox or an attempt after a model
 error. The three answers of a run's start-up wait under {@link START_UP_ANSWER_MILLISECONDS} instead, because Pi is loading
 its session and its extensions then.

 Pi answered the first prompt 5.2 to 5.4 seconds after starting in four real runs on 2026-11-08
 (`doc/agent/drive-pi-as-a-coding-subagent.md`); this leaves room for a loaded host.
 */
export const COMMAND_ANSWER_MILLISECONDS = 30_000;

/**
 Longest silence Pi may keep while the driver starts it up: the state and the commands it is asked for, and the answer to the
 first prompt. Pi is loading its session and its extensions then, and a loaded host stretches that, so the bound for an answer
 Pi owes while it is already at work does not cover these.

 Measured on this machine on 2026-11-12 over the runs in `${HOME}/temp/agent/pi-runs`: of the 1,214 runs that recorded a first
 prompt, the longest answer Pi gave took 76,322 ms (`kimi-review-since-alpha8-all-waits-3`, which ended `idle`), and the
 99th percentile was 1,086 ms. Four runs were still waiting 90 to 91 seconds after Pi started and were cut off as
 `prompt-rejected`, three of them with summary files stamped 2026-11-11T00:50-05:00
 (`kimi-never-stuck-audit-findings-1`, `commands-never-wait-on-or-fail-with-a-silent-service-1`,
 `killed-add-leftover-copy-is-not-another-commands-wait-1`, and `kimi-review-merge-route-round-9-2` a day earlier): each waited
 the old thirty seconds for Pi's state, thirty more for its commands, and were due the prompt's answer at 90 seconds.

 Two minutes sits above both figures. It stays a silence bound, never the run's length: the run's own wall clock is still the
 outer bound of every wait.
 */
export const START_UP_ANSWER_MILLISECONDS = 120_000;

/**
 Longest wait for Pi to leave after its input closes and again after each signal, and for its output to end once it has gone.
 Pi left 19 milliseconds after its input closed in a real run on 2026-11-08 (`doc/agent/drive-pi-as-a-coding-subagent.md`).
 */
export const SHUTDOWN_GRACE_MILLISECONDS = 5_000;

/**
 The stages of ending Pi that each wait for Pi: its input closing, then one signal, then the last signal, then its standard
 error copy. The driver divides the time it has left for ending by this count, so one slow stage cannot spend another's time.
 */
export const SHUTDOWN_STAGES = 4;

/**
 Longest wait for Pi's last two answers while a run is put away: its session statistics, then its abort. Both are a courtesy,
 so they get far less than an ordinary command: a Pi at the end of its wall clock may never answer, and the summary must still
 appear. Pi answered the first prompt 5.2 to 5.4 seconds after starting in four real runs on 2026-11-08
 (`doc/agent/drive-pi-as-a-coding-subagent.md`).
 */
export const TEARDOWN_ANSWER_MILLISECONDS = 5_000;

/**
 Longest the driver spends putting a run away: those two answers, then Pi's stages of leaving, then the feed's last write. The
 wall limit bounds the run itself; this bounds everything after it, so a run whose Pi never answers still ends.
 */
export const TEARDOWN_MILLISECONDS: number = (2 * TEARDOWN_ANSWER_MILLISECONDS)
  + ((SHUTDOWN_STAGES + 1) * SHUTDOWN_GRACE_MILLISECONDS);

/**
 Name of the file that ends a run early.
 */
export const STOP_FILE_NAME = 'stop';

/**
 The failure of a run that has none, so no caller has to carry a nullish member for absence.
 */
export const NO_RUN_FAILURE: RunFailure = Object.freeze({ failed: false, });

/**
 What the driver knows of Pi's own automatic retry before Pi says anything of it, so a caller reads one shape rather than a
 nullish member.
 */
export const NO_PI_RETRY_GAVE_UP: PiRetryGaveUp = Object.freeze({ gaveUp: false, });

/**
 What the summary says when the model reported an error and said nothing about it.
 */
export const MODEL_ERROR_WITHOUT_TEXT = 'the model reported an error and said nothing about it';

/**
 How many attempts the driver makes after a model error another attempt may fix, chosen to match the `retry.maxRetries` default
 Pi's own automatic retry uses (`docs/settings.md` in Pi 1.5.0) rather than guessed here. It bounds only the errors the driver
 owns: a 429, in any form, is Pi's own retry's to answer.
 */
export const MODEL_ERROR_RETRY_CAP = 3;

/**
 Seconds before the first attempt after a model error, doubling with each attempt, chosen to match the `retry.baseDelayMs`
 default Pi's own automatic retry uses (`docs/settings.md` in Pi 1.5.0), so the two layers back off alike. It shapes only the
 errors the driver owns, which a 429 is not.
 */
export const MODEL_ERROR_RETRY_BASE_SECONDS = 2;

/**
 Longest wait before one attempt after a model error, chosen to match the `retry.maxAgentDelayMs` default Pi's own automatic
 retry uses (`docs/settings.md` in Pi 1.5.0). It bounds a provider that names an hour of backoff, and it bounds the growth of a
 delay that doubles, so no run waits longer than this for one attempt. It shapes only the errors the driver owns, and a 429 is
 one it leaves to Pi.
 */
export const MODEL_ERROR_RETRY_MAX_DELAY_MILLISECONDS = 60_000;

/**
 Longest text of one kind kept on a feed line, carried over from the driver the recorded runs used. They bound the size of a feed
 line and change nothing about the run.
 */
export const FEED_PREVIEW_CHARACTERS = {
  toolArguments: 700,
  toolResult: 500,
  toolError: 300,
  dialogTitle: 160,
  dialogMessage: 200,
  assistantText: 900,
  eventDetail: 300,
  inboxMessage: 300,
  unreadableLine: 160,
  modelError: 700,
} as const;
