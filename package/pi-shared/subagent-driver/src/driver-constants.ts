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
 Seconds in a minute, for the figures a message to an agent and a report of a run are read in.
 */
export const SECONDS_PER_MINUTE = 60;

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
 How many times the driver starts Pi in one run before it ends the run on a prompt Pi never took.

 Of the 1,936 finished runs in `${HOME}/temp/agent/pi-runs` read on 2026-11-14, 18 ended with Pi exiting before the prompt took the
 run: 13 on an extension that could not load a module, each start ending in 2,096 to 20,491 ms, one on a logger with no space left
 on the device (6,097 ms), and four on Pi's own retry of a model-store lock another process held, each start spending 127,176 to
 247,556 ms. Most of those attempts were a condition that was passing while it was watched, so two more starts ride it out; a
 fourth would add the cost of a whole Pi without covering anything the three did not.
 */
export const START_ATTEMPTS: number = 3;

/**
 Milliseconds the driver waits before it starts Pi again, after a start ended before it took the prompt.

 The pause leaves a condition that passes on its own the time to pass: a Pi the driver has just ended releasing a lock, another
 agent replacing a file Pi reads as it starts. It stays far below what a failing start itself costs (the runs above took seconds,
 and one of them minutes), so two pauses add 20 seconds to such a run rather than spending its wall clock. It is a bound rather
 than a measurement: Pi spends the time a failed start takes, not the driver.
 */
export const START_PAUSE_MILLISECONDS: number = 10_000;

/**
 Longest the driver's whole sequence of starts may take, counted from the first start.

 One start-up answer is allowed {@link START_UP_ANSWER_MILLISECONDS}, so a condition that passes by itself clears inside the time
 one start gets. A start that already spent that bound ended on a condition standing: Pi's own model-store retry spent 127,176 to
 247,556 ms in the runs above and would spend as much again on every start, so such a start is not made again and the summary says
 what stood instead of a reader waiting three of them out.
 */
export const START_SEQUENCE_MILLISECONDS: number = START_UP_ANSWER_MILLISECONDS;

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
 Seconds a run may say nothing before the driver writes it down as a line of its own: ten minutes.

 What a run says is every record Pi sends over the channel: a tool's output arrives as one `tool_execution_update` per chunk, and a
 model's reply arrives as `message_update` deltas, so a command or a reply that is working is never silent. Silence is therefore a
 command that reports nothing, or a model that answers nothing, for this long.

 Measured on this machine on 2026-11-15 over the 2,346 feeds of `${HOME}/temp/agent/pi-runs` (38 of them left unopened because a
 scan in the secret-values keep folder names their run), holding 248,424 bash commands and 612,100 stretches with no command open:
 a command's p99 was 316 s and its p99.9 1,500 s, 401 of the 152,277 commands that declared no bound of their own ran past 600 s,
 and 24 of those 612,100 command-free stretches ran past it. A notice costs one feed line and ends nothing, so it sits in that
 rare tail rather than above every slow build.
 */
export const SILENT_NOTICE_SECONDS = 600;

/**
 Seconds a command that says nothing is given before the driver ends it: one hour.

 Above every honest silence in the same measurement: the longest stretch with no command open was 1,205 s, three times over, and
 the longest command that declared no bound of its own was 5,512 s. Sixteen of those 152,277 unbounded commands ran past 3,600 s:
 four of them are the hangs that started this work (6,424 s, 24,325 s, 43,535 s and 56,315 s), and the longest of the rest is
 13,990 s, so a silent command past this bound is ended and told about rather than left for its run's wall clock.

 A command that declares a bound of its own is not ended before that bound has passed, because a whole-suite run or a build says
 how long it may take rather than staying inside an hour: 96,147 of the 248,424 commands did, at 30 s to 7,300 s. The run's own
 wall clock stays the outer bound of every wait.
 */
export const SILENT_ENDING_SECONDS = 3_600;

/**
 Seconds past a declared bound before the driver ends the command itself.

 Pi ends a command at its own `timeout` by killing its process tree, so an honest command ends within a moment of that bound.
 Measured on this machine on 2026-11-15, 334 of the 96,158 commands that declared a bound ran past it: 179 within one second and
 215 within five, while 32 ran more than a minute past it. A minute therefore sits above Pi's own kill and leaves the rest to this
 driver, which is the case a bound could not end: a descendant holding the command's output open after its own process went.
 */
export const SILENT_BOUND_GRACE_SECONDS = 60;

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
