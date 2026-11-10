/**
 The longest a command this package starts may run before it is killed.

 A start waits for its whole unit, so its own limits end it: the unit's `RuntimeMaxSec` is the run's wall clock plus the grace
 the unit is given, and a queue ends at its deadline. This bound is the outer one, for a child wedged in a call that cannot be
 cancelled, which is the case this package reports rather than waits out.

 @module
 */

import { MILLISECONDS_PER_SECOND, } from './driver-constants.ts';

/**
 Hours in one day, which is how long the outer bound lasts.
 */
const HOURS_IN_A_DAY = 24;

/**
 Minutes in one hour.
 */
const MINUTES_IN_AN_HOUR = 60;

/**
 Seconds in one minute.
 */
const SECONDS_IN_A_MINUTE = 60;

/**
 The longest a child command may run: one day, far past the longest wall clock a run is given, and short enough that a child
 that cannot end is reported while the person watching is still there.
 */
export const LONGEST_CHILD_MILLISECONDS: number = HOURS_IN_A_DAY
  * MINUTES_IN_AN_HOUR
  * SECONDS_IN_A_MINUTE
  * MILLISECONDS_PER_SECOND;
