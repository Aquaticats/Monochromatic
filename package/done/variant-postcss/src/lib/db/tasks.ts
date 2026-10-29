/**
 Task CRUD operations against the SQLite database.
 
 Read-only queries are in task-queries.ts; timer and completion
 operations are in task-timer.ts. Re-exports all query and timer
 functions so existing consumers can import from one place.
 */
import db from '../db.ts';
import {
  type Task,
  type TaskCreateInput,
  TASK_NOT_FOUND,
  type TaskUpdateInput,
} from '../types.ts';
import {
  normalizeStringArray,
  nowIso,
} from './task-mapping.ts';
import { getTaskById, } from './task-queries.ts';
import {
  SQL_DELETE_TASK,
  SQL_INSERT_TASK,
  SQL_UPDATE_TASK,
} from './task-sql.ts';

// Re-export all query functions for backward compatibility
export {
  getTaskById,
  listAllTags,
  listBlockedInboxTasks,
  listInboxUnblockedTasks,
  listInProgressTasks,
  listTasksForBlockerPicker,
  searchTasks,
} from './task-queries.ts';

// Re-export timer and completion functions
export {
  type BlockerSummary,
  completeTask,
  type CompleteTaskResult,
  startTaskTimer,
  stopTaskTimer,
} from './task-timer.ts';

/**
 Inserts a new task with a generated UUID and current timestamp.
 
 @param input - Task creation payload
 
 @returns Freshly created task
 
 @throws When the read-back fails
 
 @example
 ```ts
 const task = await createTask({ title: 'Buy groceries', tags: ['shopping'] });
 ```
 */
export async function createTask(input: TaskCreateInput,): Promise<Task> {
  /**
   Server-generated UUID assigned before the insert so the read-back can find the row.
   */
  const id = crypto.randomUUID();
  /**
   ISO timestamp reused for both `created_at` and `updated_at` to keep them aligned.
   */
  const timestamp = nowIso();

  await (await db.prepare(SQL_INSERT_TASK,))
    .run(
    id,
    input.title
      .trim(),
    input.description
      ?? null,
    JSON.stringify(normalizeStringArray(input.tags,),),
    JSON.stringify(normalizeStringArray(input.locations,),),
    input.priority
      ?? null,
    input.dueDate
      ?? null,
    input.complexity
      ?? null,
    JSON.stringify(normalizeStringArray(input.reminders,),),
    JSON.stringify(normalizeStringArray(input.blockedBy,),),
    0,
    null,
    'inbox',
    'local',
    null,
    null,
    timestamp,
    timestamp,
    );

  /**
   Read-back of the freshly inserted row; absence indicates a write/read race.
   */
  const createdTask = await getTaskById(id,);
  if (createdTask === TASK_NOT_FOUND)
    throw new Error('Failed to read created task',);
  return createdTask;
}

/**
 Applies a partial update to an existing task.
 
 @param id - Task UUID
 
 @param input - Fields to update
 
 @returns Updated task, or {@link TASK_NOT_FOUND} when not found
 
 @example
 ```ts
 const task = await updateTask({ id: 'uuid-123', input: { title: 'Updated title' } });
 ```
 */
export async function updateTask(
  {
    id,
    input,
  }: {
    readonly id: string;
    readonly input: TaskUpdateInput;
  },
): Promise<Task | typeof TASK_NOT_FOUND> {
  /**
   Existing task used as the merge base for the partial update.
   */
  const currentTask = await getTaskById(id,);
  if (currentTask === TASK_NOT_FOUND)
    return TASK_NOT_FOUND;

  // Spread-merge: present `input` keys override; absent optional keys inherit from `currentTask`
  /**
   Merged task object combining the previous values with any fields supplied in `input`.
   */
  const updated: Task = {
    ...currentTask,
    ...input,
    title: input.title
      ?.trim()
      ?? currentTask
        .title,
    updatedAt: nowIso(),
  };

  await (await db.prepare(SQL_UPDATE_TASK,))
    .run(
    updated.title,
    updated.description
      ?? null,
    JSON.stringify(normalizeStringArray(updated.tags,),),
    JSON.stringify(normalizeStringArray(updated.locations,),),
    updated.priority
      ?? null,
    updated.dueDate
      ?? null,
    updated.complexity
      ?? null,
    JSON.stringify(normalizeStringArray(updated.reminders,),),
    JSON.stringify(normalizeStringArray(updated.blockedBy,),),
    updated.status,
    updated.updatedAt,
    id,
    );

  return getTaskById(id,);
}

/**
 Permanently removes a task by UUID.
 
 @param id - Task UUID
 
 @returns `true` when the task existed and was deleted
 
 @example
 ```ts
 const deleted = await deleteTask('uuid-123');
 ```
 */
export async function deleteTask(id: string,): Promise<boolean> {
  /**
   Run result whose `.changes` distinguishes a successful delete from a no-op.
   */
  const result = await (await db.prepare(SQL_DELETE_TASK,))
    .run(id,);
  return result.changes
    > 0;
}
