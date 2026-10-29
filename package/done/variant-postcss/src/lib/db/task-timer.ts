/**
 Timer and completion operations for tasks.
 
 Handles start/stop timer state transitions and blocker-aware
 task completion. Basic CRUD remains in tasks.ts.
 */
import { MS_PER_SECOND, } from '@monochromatic-dev/const-core/ts';

import db from '../db.ts';
import {
  type Task,
  TASK_NOT_FOUND,
} from '../types.ts';
import { nowIso, } from './task-mapping.ts';
import { getTaskById, } from './task-queries.ts';
import {
  SQL_DELETE_TASK,
  SQL_SELECT_BLOCKERS,
  SQL_START_TIMER,
  SQL_STOP_TIMER,
} from './task-sql.ts';

/**
 Summary of a single blocker task, used to report why completion was refused.
 */
export type BlockerSummary = {
  /**
   UUID of the blocking task.
   */
  blockerId: string;
  /**
   Title of the blocking task.
   */
  blockerTitle: string;
};

/**
 Outcome of a {@link completeTask} call: carries blockers when completion is refused.
 */
export type CompleteTaskResult = {
  /**
   Whether the task was successfully completed and deleted.
   */
  completed: boolean;
  /**
   Whether the task ID was not found in the database.
   */
  notFound: boolean;
  /**
   List of active blockers that prevented completion.
   */
  blockedBy: BlockerSummary[];
};

/**
 Starts the timer on a task, transitioning its status to `in_progress`.
 
 @param id - Task UUID
 
 @returns Updated task, or {@link TASK_NOT_FOUND} when not found
 
 @example
 ```ts
 const task = await startTaskTimer('uuid-123');
 ```
 */
export async function startTaskTimer(id: string,): Promise<Task | typeof TASK_NOT_FOUND> {
  /**
   Single ISO timestamp reused for both `timer_started_at` and `updated_at` to keep them aligned.
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
 Stops the running timer, accumulates elapsed seconds, and resets to inbox.
 
 @param id - Task UUID
 
 @returns Updated task, or {@link TASK_NOT_FOUND} when not found
 
 @example
 ```ts
 const task = await stopTaskTimer('uuid-123');
 ```
 */
export async function stopTaskTimer(id: string,): Promise<Task | typeof TASK_NOT_FOUND> {
  /**
   Existing task; absent task short-circuits with {@link TASK_NOT_FOUND}.
   */
  const currentTask = await getTaskById(id,);
  if (currentTask === TASK_NOT_FOUND)
    return TASK_NOT_FOUND;

  /**
   Seconds the running timer accumulated; zero when no timer was active.
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
   ISO timestamp for the `updated_at` column on the stop write.
   */
  const timestamp = nowIso();
  await (await db.prepare(SQL_STOP_TIMER,))
    .run(
    currentTask.trackedTime
      + elapsedSeconds,
    timestamp,
    id,
    );
  return getTaskById(id,);
}

/**
 Attempts to complete a task: refuses if blockers remain, otherwise deletes.
 
 @param id - Task UUID
 
 @returns Completion result with blocker information
 
 @example
 ```ts
 const result = await completeTask('uuid-123');
 if (!result.completed) console.error('Blocked by', result.blockedBy);
 ```
 */
export async function completeTask(id: string,): Promise<CompleteTaskResult> {
  /**
   Existing task; absent task short-circuits with `notFound: true`.
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
   Raw blocker rows; emptiness implies completion is allowed.
   */
  const blockingRows = (await (await db.prepare(SQL_SELECT_BLOCKERS,))
    .all(id,)) as {
      readonly blocker_id: string;
      readonly blocker_title: string;
    }[];
  /* oxlint-enable typescript/no-unsafe-type-assertion */
  /**
   Application-shaped blocker summaries returned to the caller when completion is refused.
   */
  const blockedBy = blockingRows.map(function toSummary(row,) {
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
