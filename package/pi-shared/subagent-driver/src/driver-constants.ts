/**
 Fixed values of the driver, each with where it comes from. The limits of a run are options, so none is shipped here.

 @module
 */

import type { RunFailure, } from './driver-types.ts';

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

 Pi answered the first prompt 5.2 to 5.4 seconds after starting in four real runs on 2026-11-08
 (`doc/agent/drive-pi-as-a-coding-subagent.md`); this leaves room for a loaded host.
 */
export const COMMAND_ANSWER_MILLISECONDS = 30_000;

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
 What the summary says when the model reported an error and said nothing about it.
 */
export const MODEL_ERROR_WITHOUT_TEXT = 'the model reported an error and said nothing about it';

/**
 How many attempts the driver makes after a model error another attempt may fix, chosen to match the `retry.maxRetries` default
 Pi's own automatic retry uses (`docs/settings.md` in Pi 1.5.0) rather than guessed here.
 */
export const MODEL_ERROR_RETRY_CAP = 3;

/**
 Seconds before the first attempt after a model error, doubling with each attempt, chosen to match the `retry.baseDelayMs`
 default Pi's own automatic retry uses (`docs/settings.md` in Pi 1.5.0).
 */
export const MODEL_ERROR_RETRY_BASE_SECONDS = 2;

/**
 Longest wait before one attempt after a model error, chosen to match the `retry.maxAgentDelayMs` default Pi's own automatic
 retry uses (`docs/settings.md` in Pi 1.5.0). It bounds a provider that names an hour of backoff, and it bounds the growth of a
 delay that doubles, so no run waits longer than this for one attempt.
 */
export const MODEL_ERROR_RETRY_MAX_DELAY_MILLISECONDS = 60_000;

/**
 How long a run waits, in all, for a provider to have a request slot free after saying its limit on requests at once is reached.

 The number of attempts is not what bounds this wait, because what it waits for is another run of the provider's pool finishing.
 Measured on 2026-11-11T03:00Z over the 1,052 finished runs in `${HOME}/temp/agent/pi-runs`, a run lasts 1,744,872 ms at the
 median (29 minutes) and 6,458,155 ms at the 90th percentile (108 minutes), so a pool of six slots frees one about one run length
 after it fills. Forty minutes is above that median and well inside the 90th percentile, and the run's own wall clock is the
 outer bound either way: `--wall-seconds` of 14,400 in the coordinator's launcher leaves this wait several times over.
 */
export const MODEL_ERROR_CONCURRENCY_WAIT_SECONDS = 2_400;

/**
 The provider whose spent quota moves a run to another provider.

 Synthetic sets no limit on runs at once, so its 429 means the quota is spent and a later attempt on Synthetic cannot succeed
 (`.agents/skills/pi-as-subagent/SKILL.md`). The owner's rule, 2026-11-10: "Use neuralwatt/kimi-k3.1-pro-flex whenever
 Synthetic 429s."
 */
export const PROVIDER_WHOSE_QUOTA_MOVES_A_RUN = 'synthetic';

/**
 The provider a run whose provider's quota is spent goes on with.
 */
export const QUOTA_FALLBACK_PROVIDER = 'neuralwatt';

/**
 The model a run whose provider's quota is spent goes on with.
 */
export const QUOTA_FALLBACK_MODEL = 'kimi-k3.1-pro-flex';

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
