/**
 Fixed values of the driver, each with where it comes from. The limits of a run are options, so none is shipped here.

 @module
 */

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
 Longest wait for Pi's answer to one command. Pi answered the first prompt 5.2 to 5.4 seconds after starting in four real runs
 on 2026-11-08 (`doc/agent/drive-pi-as-a-coding-subagent.md`); this leaves room for a loaded host.
 */
export const COMMAND_ANSWER_MILLISECONDS = 30_000;

/**
 Longest wait for Pi to leave after its input closes, and again after each signal. Pi left 19 milliseconds after its input
 closed in a real run on 2026-11-08 (`doc/agent/drive-pi-as-a-coding-subagent.md`).
 */
export const SHUTDOWN_GRACE_MILLISECONDS = 5_000;

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
} as const;

/**
 Exit status for options the driver cannot use, from `sysexits.h`.
 */
export const EXIT_USAGE = 64;
