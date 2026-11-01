/**
 Helper functions and types for `<task-card>`.
 */
import {
  SECONDS_PER_HOUR,
  SECONDS_PER_MINUTE,
} from '@monochromatic-dev/const-core/ts';

import type { Task, } from '../../lib/types.ts';

/**
 Configuration for a `<task-card>` instance, passed via `createTaskCard`.
 */
export type TaskCardOptions = {
  /**
   Whether to show a red "blocked" badge chip.
   */
  readonly showBlockedBadge?: boolean;
  /**
   Callback when the card body is clicked (navigates to task detail).
   */
  readonly onOpen: (taskId: string,) => void;
  /**
   Callback when the checkbox is clicked (completes the task).
   */
  readonly onToggleComplete?: (taskId: string,) => Promise<void>;
};

/**
 Cached formatter; `Intl.DurationFormat` is safe to reuse across calls.
 */
const DIGITAL_FORMATTER = new Intl.DurationFormat(
  undefined,
  { style: 'digital', },
);

/**
 Formats a duration in seconds as `H:MM:SS` via `Intl.DurationFormat`.
 Days roll into the hours field so the shape stays stopwatch-like
 regardless of magnitude.
 
 @param seconds - Non-negative duration in seconds; negative or
   fractional inputs are clamped to a non-negative integer
 
 @returns Formatted duration string
 
 @example
 formatTrackedTime(0); // '0:00:00'
 formatTrackedTime(3661); // '1:01:01'
 */
export function formatTrackedTime(seconds: number,): string {
  /**
   Clamps the input so negative or fractional inputs do not propagate into the formatter.
   */
  const totalSeconds = Math.max(
    0,
    Math.floor(seconds,),
  );
  /**
   Hours bucket fed to `Intl.DurationFormat`.
   */
  const hours = Math.floor(totalSeconds / SECONDS_PER_HOUR,);
  /**
   Minutes bucket within the hour.
   */
  const minutes = Math.floor((totalSeconds % SECONDS_PER_HOUR) / SECONDS_PER_MINUTE,);
  /**
   Seconds remainder within the minute.
   */
  const remainingSeconds = totalSeconds % SECONDS_PER_MINUTE;
  return DIGITAL_FORMATTER.format({
    hours,
    minutes,
    seconds: remainingSeconds,
  },);
}

/**
 Collects all metadata chip labels for a task (tags, tracked time formatted
 via {@link formatTrackedTime}, location, etc.). Each entry becomes a
 `<span class="chip">` in the card's shadow DOM.
 
 @param task - Task to build chip labels for
 
 @returns Array of chip label strings
 
 @example
 const chips = buildChipTexts(task);
 // ['#errand', 'tracked: 1:30:00', 'home']
 */
export function buildChipTexts(task: Task,): string[] {
  /**
   Accumulator mutated by the conditional pushes below; tag chips come first.
   */
  const chips: string[] = [];

  if (task.tags
    .length
    > 0)
    chips.push(`# ${task.tags
      .join(', ',)}`,);
  chips.push(`tracked: ${formatTrackedTime(task.trackedTime,)}`,);
  if (task.locations
    .length
    > 0)
    chips.push(`where: ${task.locations
      .join(', ',)}`,);
  if (task.priority
    !== undefined)
    chips.push(`priority: ${task.priority}`,);
  if (task.dueDate
    !== undefined)
    chips.push(`due: ${task.dueDate}`,);
  if (task.complexity
    !== undefined)
    chips.push(`complexity: ${task.complexity}`,);
  if (task.reminders
    .length
    > 0)
    chips.push(`reminders: ${task.reminders[0]}`,);
  if (task.blockedBy
    .length
    > 0)
    chips.push(`blockedBy: ${task.blockedBy
      .length}`,);
  else
    chips.push('blockedBy: none',);

  return chips;
}
