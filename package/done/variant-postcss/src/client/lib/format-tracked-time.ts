/**
 Duration formatting utilities for tracked task time.
 
 Renders elapsed seconds as `H:MM:SS` via `Intl.DurationFormat` with
 `style: 'digital'`. Days roll into the hours field so the shape stays
 stopwatch-like regardless of magnitude (a 73-hour task is `73:00:00`,
 not `3 days, 1:00:00`). Locale is left undefined so numerals follow
 the host's `Intl` default.
 
 Pure functions with no component dependencies; used by both
 `\<task-card\>` and `\<task-detail\>` to display elapsed time.
 */
import {
  MS_PER_SECOND,
  SECONDS_PER_HOUR,
  SECONDS_PER_MINUTE,
} from '@monochromatic-dev/const-core/ts';

import type { Task, } from '../../lib/types.ts';

/**
 Cached formatter; `Intl.DurationFormat` is safe to reuse across calls.
 */
const DIGITAL_FORMATTER = new Intl.DurationFormat(
  undefined,
  { style: 'digital', },
);

/**
 Formats a duration in seconds as `H:MM:SS`.
 
 @param seconds - Non-negative duration in seconds; negative or
   fractional inputs are clamped to a non-negative integer
 
 @returns Formatted duration string
 
 @example
 formatTrackedTime(0); // '0:00:00'
 formatTrackedTime(5400); // '1:30:00'
 formatTrackedTime(263_400); // '73:10:00'
 */
export function formatTrackedTime(seconds: number,): string {
  /**
   Non-negative integer seconds; negative or fractional inputs are clamped here.
   */
  const totalSeconds = Math.max(
    0,
    Math.floor(seconds,),
  );
  /**
   Whole-hour portion; days roll into hours so the digital format stays stopwatch-like.
   */
  const hours = Math.floor(totalSeconds / SECONDS_PER_HOUR,);
  /**
   Whole-minute portion of the remainder after extracting hours.
   */
  const minutes = Math.floor((totalSeconds % SECONDS_PER_HOUR) / SECONDS_PER_MINUTE,);
  /**
   Remaining seconds after extracting hours and minutes.
   */
  const remainingSeconds = totalSeconds % SECONDS_PER_MINUTE;
  return DIGITAL_FORMATTER.format({
    hours,
    minutes,
    seconds: remainingSeconds,
  },);
}

/**
 Formats tracked time including elapsed seconds from a running timer.
 If no timer is active, returns the static `trackedTime` formatted.
 
 @param task - Task with optional running timer
 
 @returns Formatted duration string
 
 @example
 const display = formatRunningTrackedTime(task);
 */
export function formatRunningTrackedTime(task: Task,): string {
  if (task.timerStartedAt
    === undefined)
    return formatTrackedTime(task.trackedTime,);

  /**
   Seconds elapsed since the timer started; clamped to non-negative for clock skew safety.
   */
  const elapsedSeconds = Math.max(
    0,
    Math.floor((Date.now()
      - Date
        .parse(task.timerStartedAt,)) / MS_PER_SECOND,),
  );
  return formatTrackedTime(task.trackedTime
    + elapsedSeconds,);
}
