/**
 Task mutation operations against the SQLite database.
 
 Read-only query functions are in `tasks-queries.ts`.
 Timer and completion operations are in `tasks-timer.ts`.
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
} from './tasks-helpers.ts';
import { getTaskById, } from './tasks-queries.ts';
import {
  SQL_DELETE_TASK,
  SQL_INSERT_TASK,
  SQL_UPDATE_TASK,
} from './tasks-sql.ts';

export type {
  BlockerSummary,
  CompleteTaskResult,
} from './tasks-helpers.ts';
export { getTaskById, } from './tasks-queries.ts';
export {
  completeTask,
  startTaskTimer,
  stopTaskTimer,
} from './tasks-timer.ts';

/**
 Inserts a new task with a generated UUID and a {@link nowIso} timestamp,
 normalizing array fields with {@link normalizeStringArray}.
 
 @param input - Task creation payload (only `title` is required)
 
 @returns Freshly created task read back from the database
 
 @throws When {@link getTaskById} returns {@link TASK_NOT_FOUND} for the read-back (should never happen)
 
 @example
 ```ts
 const task = await createTask({ title: 'Buy groceries' });
 ```
 */
export async function createTask(input: TaskCreateInput,): Promise<Task> {
  /**
   Fresh UUID used as both the primary key and the read-back lookup key.
   */
  const id = crypto.randomUUID();
  /**
   Captured once so `created_at` and `updated_at` start at the same value.
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
   Read-back so callers receive the canonical row including server-applied defaults.
   */
  const createdTask = await getTaskById(id,);
  if (createdTask === TASK_NOT_FOUND)
    throw new Error('Failed to read created task',);

  return createdTask;
}

/**
 Applies a partial update to an existing task, normalizing array fields via
 {@link normalizeStringArray} and stamping a fresh {@link nowIso} timestamp.
 
 @param id - Task UUID
 
 @param input - Fields to update (omitted fields keep their current value)
 
 @returns Updated task, or {@link TASK_NOT_FOUND} when the ID does not exist
 
 @example
 ```ts
 const task = await updateTask({ id: 'abc-123', input: { title: 'Updated title', }, });
 ```
 */
export async function updateTask({
  id,
  input,
}: {
  readonly id: string;
  readonly input: TaskUpdateInput;
},): Promise<Task | typeof TASK_NOT_FOUND> {
  /**
   Existing row used as the merge baseline; the sentinel short-circuits not-found.
   */
  const currentTask = await getTaskById(id,);
  if (currentTask === TASK_NOT_FOUND)
    return TASK_NOT_FOUND;

  // Spread-merge: present `input` keys override; absent optional keys inherit from `currentTask`
  /**
   Merged shape: input wins, falling back to the current row, with a fresh updated-at.
   */
  const updatedTask: Task = {
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
    updatedTask.title,
    updatedTask.description
      ?? null,
    JSON.stringify(normalizeStringArray(updatedTask.tags,),),
    JSON.stringify(normalizeStringArray(updatedTask.locations,),),
    updatedTask.priority
      ?? null,
    updatedTask.dueDate
      ?? null,
    updatedTask.complexity
      ?? null,
    JSON.stringify(normalizeStringArray(updatedTask.reminders,),),
    JSON.stringify(normalizeStringArray(updatedTask.blockedBy,),),
    updatedTask.status,
    updatedTask.updatedAt,
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
 const deleted = await deleteTask('abc-123');
 ```
 */
export async function deleteTask(id: string,): Promise<boolean> {
  /**
   Run result; `changes` distinguishes a real delete from a missing row.
   */
  const result = await (await db.prepare(SQL_DELETE_TASK,))
    .run(id,);
  return result.changes
    > 0;
}
