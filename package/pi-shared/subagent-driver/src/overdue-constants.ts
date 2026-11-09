/**
 Fixed values of the overdue watch, each with where it comes from. Both limits a running run is compared against are read
 from the finished runs, so neither is shipped here.

 @module
 */

import { MILLISECONDS_PER_SECOND, } from './driver-constants.ts';

/**
 Seconds in one minute.
 */
export const SECONDS_PER_MINUTE = 60;

/**
 Milliseconds in one minute.
 */
export const MILLISECONDS_PER_MINUTE: number = MILLISECONDS_PER_SECOND * SECONDS_PER_MINUTE;

/**
 The share of finished runs a running run must pass to be reported. The owner's rule, quoted in `.agents/skills/pi-as-subagent`
 and in the handover: "Regularly, check in on subagents that should have finished their jobs (based on time estimations 90%+
 percentile) but haven't returned."
 */
export const OVERDUE_PERCENTILE = 0.9;

/**
 The share of finished tool calls a call that is open now must pass to be reported. A false alarm costs one look at the agent,
 so the limit sits in the rare tail rather than above every long test run: over 24,728 tool calls in the 136 finished feeds on
 this machine on 2026-11-08, p99 was 3.4 minutes and p99.9 was 18.7 minutes.
 */
export const OPEN_CALL_PERCENTILE = 0.999;

/**
 Minutes a reported run is left alone before it is reported again: half an hour, so a coordinator that reruns the command on
 every turn is told about one run about twice an hour instead of on every turn.
 */
export const RECHECK_MINUTES = 30;

/**
 Milliseconds a reported run is left alone before it is reported again.
 */
export const RECHECK_MILLISECONDS: number = RECHECK_MINUTES * MILLISECONDS_PER_MINUTE;

/**
 Minutes between looks, and so the longest a run past a limit goes unreported: five. One look reads every finished feed
 (28 MB in 96,232 lines on this machine on 2026-11-08, 0.3 seconds of central processor time), so looking more often costs
 more than the five minutes it saves.
 */
export const LOOK_INTERVAL_MINUTES = 5;

/**
 Milliseconds between looks, and so the longest a run past a limit goes unreported.
 */
export const LOOK_INTERVAL_MILLISECONDS: number = LOOK_INTERVAL_MINUTES * MILLISECONDS_PER_MINUTE;
