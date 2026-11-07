/**
 Timing constants shared by the polling helpers and the checks.
 
 @example
 ```ts
 import { STATE_TIMEOUT_MS, } from './timing.ts';
 ```
 
 @packageDocumentation
 */

/**
 How long a check waits for a state line that satisfies its predicate.
 */
export const STATE_TIMEOUT_MS = 6_000;

/**
 Pause between two reads of the app's log while waiting for a state line.
 */
export const STATE_POLL_MS = 40;

/**
 Time the app needs to settle after a key press that must change nothing.
 */
export const NO_EFFECT_SETTLE_MS = 500;

/**
 Time the app needs to show a notice or finish a copy after a key press.
 */
export const EFFECT_SETTLE_MS = 700;

/**
 Longest a step that loads files may take before the run gives up on it: the compositor's device listing, its
 `--version`, the unit start, the control socket appearing and the app's first state line.

 Sized from the measured start times of 2026-11-07 on this host (28 hardware threads, load average near 100, other
 sessions building): a start with the files of the compositor, Mesa, LLVM and the app in the page cache puts the socket
 at 0.06 to 0.26 s and the first state line at 0.26 to 1.1 s over 11 starts (renderD129, renderD128, auto and
 software); a start after those files were evicted from the page cache put the socket at 9.9 to 34.9 s and the first
 state line at 11.6 to 56.6 s over 7 starts (four after eviction, three first starts of a batch; CPU time of each unit
 0.2 to 0.9 s, `nr_throttled` 0, so neither the CPU quota nor computing is the wait: the wait is page-in under
 contention, with the unit's `io.pressure` `full` total at 1.6 to 9.8 s). `--list-gpus` alone took 31.9 s on a first
 call, 7.0 s on a later first call and 0.09 to 0.32 s on the other seven of that batch. The widest
 measured wait is 56.6 s, and 120 s is about twice that. The value is long so that a cold start passes, not because any
 check needs that long: a warm start is over in about a second, and the wait ends as soon as the line is there. Record:
 `doc/troubleshooting/music-player-e2e-search-answer-and-headless-start.md`.
 */
export const COLD_START_BOUND_MS = 120_000;

/**
 Time to wait for the first state line of a freshly started session; the bound of a cold start.
 */
export const SESSION_READY_MS: number = COLD_START_BOUND_MS;

/**
 Short wait that lets a key press reach the app before the next read.
 */
export const BRIEF_MS = 120;

/**
 Wait before the compositor takes a screenshot, so a pointer move has been drawn.
 */
export const SHOT_SETTLE_MS = 350;

/**
 Spacing between two looks for the compositor socket after launch.
 */
const SOCKET_POLL_INTERVAL_MS = 250;

/**
 Polling attempts and spacing for the compositor socket appearing after launch; together they span
 `COLD_START_BOUND_MS` (the value was 30 s before the cold-start measurement, which found a cold socket at up to 34.9 s).
 */
export const SOCKET_POLL: {
  readonly attempts: number;
  readonly intervalMs: number;
} = {
  attempts: COLD_START_BOUND_MS / SOCKET_POLL_INTERVAL_MS,
  intervalMs: SOCKET_POLL_INTERVAL_MS,
};

/**
 Polling attempts and spacing for a unit to leave the active states after a quit.
 */
export const STOP_POLL = {
  attempts: 40,
  intervalMs: 200,
} as const;
