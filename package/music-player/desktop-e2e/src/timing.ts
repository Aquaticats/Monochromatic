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
 Time to wait for the first state line of a freshly started session.
 */
export const SESSION_READY_MS = 20_000;

/**
 Short wait that lets a key press reach the app before the next read.
 */
export const BRIEF_MS = 120;

/**
 Wait before the compositor takes a screenshot, so a pointer move has been drawn.
 */
export const SHOT_SETTLE_MS = 350;

/**
 Polling attempts and spacing for the compositor socket appearing after launch.
 */
export const SOCKET_POLL = {
  attempts: 120,
  intervalMs: 250,
} as const;

/**
 Polling attempts and spacing for a unit to leave the active states after a quit.
 */
export const STOP_POLL = {
  attempts: 40,
  intervalMs: 200,
} as const;
