/**
 Shared types and helper functions for the task data-access layer.
 */
import db from '../db.ts';
import {
  type Task,
  type TaskComplexity,
  type TaskPriority,
  type TaskStatus,
  TASK_NOT_FOUND,
} from '../types.ts';
import { SQL_SELECT_TASK_BY_ID, } from './tasks-sql.ts';

/* oxlint-disable no-restricted-syntax/no-nullish-union -- mirrors `@tursodatabase/database` `prepare().get()/.all()` raw row shape: SQLite NULL columns materialize as JS `null` on the returned row object, so the honest type for a nullable column is `T | null`; `mapTask` converts these to absent (`?:`) fields at the application boundary */
/**
 Raw SQLite row shape before mapping to the application-level `Task` type.
 */
export type TaskRow = {
  /**
   Primary key UUID.
   */
  readonly id: string;
  /**
   Task title text.
   */
  readonly title: string;
  /**
   Optional task description.
   */
  readonly description: string | null;
  /**
   JSON-encoded string array (`'["tag1","tag2"]'`).
   */
  readonly tags: string;
  /**
   JSON-encoded string array.
   */
  readonly locations: string;
  /**
   Task priority level or null.
   */
  readonly priority: TaskPriority | null;
  /**
   ISO date string for due date or null.
   */
  readonly due_date: string | null;
  /**
   Task complexity level or null.
   */
  readonly complexity: TaskComplexity | null;
  /**
   JSON-encoded string array.
   */
  readonly reminders: string;
  /**
   JSON-encoded string array of blocker task IDs.
   */
  readonly blocked_by: string;
  /**
   Total tracked seconds.
   */
  readonly tracked_time: number;
  /**
   ISO timestamp when timer was started, or null.
   */
  readonly timer_started_at: string | null;
  /**
   Current task workflow status.
   */
  readonly status: TaskStatus;
  /**
   Source system that created this task.
   */
  readonly source: Task['source'];
  /**
   External source identifier.
   */
  readonly source_id: string | null;
  /**
   Additional source metadata as JSON.
   */
  readonly source_meta: string | null;
  /**
   ISO timestamp of creation.
   */
  readonly created_at: string;
  /**
   ISO timestamp of last update.
   */
  readonly updated_at: string;
};
/* oxlint-enable no-restricted-syntax/no-nullish-union */

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
 Outcome of a {@link completeTask} call; carries blockers when completion is refused.
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
 Returns the current timestamp in ISO 8601 format for database writes.
 
 @returns Current ISO timestamp string
 
 @example
 ```ts
 const timestamp = nowIso();
 // '2026-04-05T12:00:00.000Z'
 ```
 */
export function nowIso(): string {
  return new Date().toISOString();
}

/**
 Safely parses a JSON string expected to contain a string array.
 Returns an empty array on parse failure or unexpected shape.
 
 @param value - Raw JSON text from a SQLite TEXT column
 
 @returns Parsed string array, or empty array on failure
 
 @example
 ```ts
 const tags = parseStringArray('["errand","home"]');
 // ['errand', 'home']
 ```
 */
export function parseStringArray(value: string,): string[] {
  try {
    /**
     Untyped parse result narrowed by the array guard below.
     */
    const parsed = JSON.parse(value,) as unknown;
    if (!Array.isArray(parsed,))
      return [];
    return parsed.filter(function isString(entry,): entry is string {
      return (typeof entry) === 'string';
    },);
  }
  catch (jsonParseError: unknown) {
    // Stored column was not valid JSON; log the cause and treat it as an empty list.
    console.error(
      'parseStringArray could not parse stored JSON array; using empty array:',
      jsonParseError,
    );
    return [];
  }
}

/**
 Deduplicates, trims, and filters empty strings from an optional array.
 Used to normalize user-supplied tag/location/blocker arrays before DB writes.
 
 @param values - Raw string array, or `undefined` to produce an empty result
 
 @returns Normalized deduplicated array
 
 @example
 ```ts
 const tags = normalizeStringArray([' errand ', 'home', '', 'errand']);
 // ['errand', 'home']
 ```
 */
export function normalizeStringArray(values?: readonly string[],): string[] {
  if (values === undefined)
    return [];
  return [...new Set(values
    .map(function trimValue(value,) {
      return value.trim();
    },)
    .filter(function isNonEmpty(value,) {
      return value.length
        > 0;
    },),),];
}

/**
 Converts a raw SQLite {@link TaskRow} to the application-level {@link Task} shape.
 Parses JSON-encoded array columns via {@link parseStringArray} and renames
 snake_case to camelCase.
 
 @param row - Raw database row
 
 @returns Mapped task object
 
 @example
 ```ts
 const task = mapTask(row);
 ```
 */
export function mapTask(row: Readonly<TaskRow>,): Task {
  /**
   Mutable accumulator; nullable SQLite columns are added only when present, so null maps to an absent (`?:`) field.
   */
  const task: { -readonly [K in keyof Task]: Task[K]; } = {
    id: row.id,
    title: row.title,
    tags: parseStringArray(row.tags,),
    locations: parseStringArray(row.locations,),
    reminders: parseStringArray(row.reminders,),
    blockedBy: parseStringArray(row.blocked_by,),
    trackedTime: row.tracked_time,
    status: row.status,
    source: row.source,
    createdAt: row.created_at,
    updatedAt: row.updated_at,
  };
  if (row.description
    !== null)
    task.description = row.description;
  if (row.priority
    !== null)
    task.priority = row.priority;
  if (row.due_date
    !== null)
    task.dueDate = row.due_date;
  if (row.complexity
    !== null)
    task.complexity = row.complexity;
  if (row.timer_started_at
    !== null)
    task.timerStartedAt = row.timer_started_at;
  if (row.source_id
    !== null)
    task.sourceId = row.source_id;
  if (row.source_meta
    !== null)
    task.sourceMeta = row.source_meta;
  return task;
}

/**
 Fetches a single task row by primary key without mapping.
 
 @param id - Task UUID
 
 @returns Raw task row, or {@link TASK_NOT_FOUND} when not found
 
 @example
 ```ts
 const row = await getTaskRowById('abc-123');
 ```
 */
export async function getTaskRowById(id: string,): Promise<TaskRow | typeof TASK_NOT_FOUND> {
  /**
   Raw row read from SQLite; nullish when no row matches the requested ID.
   */
  const taskRow: unknown = await (await db.prepare(SQL_SELECT_TASK_BY_ID,))
    .get(id,);
  if ((taskRow === undefined) || (taskRow === null))
    return TASK_NOT_FOUND;
  // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- database prepare().get() returns the row shape we defined
  return taskRow as TaskRow;
}
