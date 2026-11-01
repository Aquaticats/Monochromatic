/**
 Task timer and completion operations.
 */
import { MS_PER_SECOND, } from '@monochromatic-dev/const-core/ts';

import db from '../db.ts';
import {
  type Task,
  TASK_NOT_FOUND,
} from '../types.ts';
import {
  type CompleteTaskResult,
  nowIso,
} from './tasks-helpers.ts';
import { getTaskById, } from './tasks-queries.ts';
import {
  SQL_DELETE_TASK,
  SQL_SELECT_BLOCKERS,
  SQL_START_TIMER,
  SQL_STOP_TIMER,
} from './tasks-sql.ts';

/**
 Starts the timer on a task, transitioning its status to `in_progress`.
 Stamps the start time with {@link nowIso} and reads the updated row back
 via {@link getTaskById}.
 
 @param id - Task UUID
 
 @returns Updated task, or {@link TASK_NOT_FOUND} when the ID does not exist
 
 @example
 ```ts
 const task = await startTaskTimer('abc-123');
 ```
 */
export async function startTaskTimer(id: string,): Promise<Task | typeof TASK_NOT_FOUND> {
  /**
   Captured once so both the `started_at` and `updated_at` columns share the value.
   */
  const timestamp = nowIso();
  await (await db.prepare(SQL_START_TIMER,))
    .run(
    timestamp,
    timestamp,
    id,
    );
  return getTaskById(id,);
}

/**
 Stops the running timer, accumulates elapsed seconds into `trackedTime`,
 and transitions the task back to `inbox` status. Reads the pre-update
 snapshot via {@link getTaskById} and stamps the update with {@link nowIso}.
 
 @param id - Task UUID
 
 @returns Updated task, or {@link TASK_NOT_FOUND} when the ID does not exist
 
 @example
 ```ts
 const task = await stopTaskTimer('abc-123');
 ```
 */
export async function stopTaskTimer(id: string,): Promise<Task | typeof TASK_NOT_FOUND> {
  /**
   Pre-update snapshot used to compute the elapsed delta.
   */
  const currentTask = await getTaskById(id,);
  if (currentTask === TASK_NOT_FOUND)
    return TASK_NOT_FOUND;

  /**
   Live seconds between `timerStartedAt` and now; zero when no timer was running.
   */
  const elapsedSeconds = currentTask.timerStartedAt
    === undefined
    ? 0
    : Math.max(
      0,
      Math.floor(
        (Date.now()
          - Date
            .parse(currentTask.timerStartedAt,)) / MS_PER_SECOND,
      ),
    );
  /**
   Accumulated total persisted to the row's `tracked_time` column.
   */
  const updatedTrackedTime = currentTask.trackedTime
    + elapsedSeconds;
  /**
   Captured once so the `updated_at` column carries the same value as the calculation.
   */
  const timestamp = nowIso();

  await (await db.prepare(SQL_STOP_TIMER,))
    .run(
    updatedTrackedTime,
    timestamp,
    id,
    );

  return getTaskById(id,);
}

/**
 Attempts to complete a task: reads it via {@link getTaskById}, stops any
 running timer via {@link stopTaskTimer}, then deletes it. Refuses completion
 when the task has unresolved blockers.
 
 @param id - Task UUID
 
 @returns Completion result with blocker information
 
 @example
 ```ts
 const result = await completeTask('abc-123');
 ```
 */
export async function completeTask(id: string,): Promise<CompleteTaskResult> {
  /**
   Snapshot needed for the timer-stop branch below; the sentinel distinguishes not-found.
   */
  const currentTask = await getTaskById(id,);
  if (currentTask === TASK_NOT_FOUND) {
    return {
      completed: false,
      notFound: true,
      blockedBy: [],
    };
  }

  /* oxlint-disable typescript/no-unsafe-type-assertion -- database query returns blocker join columns */
  /**
   Rows of unresolved blockers; empty allows completion.
   */
  const blockingRows = (await (await db
    .prepare(SQL_SELECT_BLOCKERS,))
    .all(id,)) as {
      readonly blocker_id: string;
      readonly blocker_title: string;
    }[];
  /* oxlint-enable typescript/no-unsafe-type-assertion */

  /**
   Reshaped blocker summaries returned to the API caller for UI rendering.
   */
  const blockedBy = blockingRows.map(function toBlockerSummary(row,) {
    return {
      blockerId: row.blocker_id,
      blockerTitle: row.blocker_title,
    };
  },);
  if (blockedBy.length
    > 0) {
    return {
      completed: false,
      notFound: false,
      blockedBy,
    };
  }

  if (currentTask.timerStartedAt
    !== undefined)
    await stopTaskTimer(id,);

  await (await db.prepare(SQL_DELETE_TASK,))
    .run(id,);
  return {
    completed: true,
    notFound: false,
    blockedBy: [],
  };
}
