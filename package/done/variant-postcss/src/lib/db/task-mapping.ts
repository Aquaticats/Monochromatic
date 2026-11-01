/**
 Row mapping and utility functions for the task data-access layer.
 
 Shared by both query and mutation modules.
 */
import type { Task, } from '../types.ts';
import type { TaskRow, } from './task-sql.ts';

/**
 Returns the current timestamp in ISO 8601 format for database writes.
 
 @returns Current ISO timestamp string
 
 @example
 ```ts
 const timestamp = nowIso(); // '2026-04-05T12:00:00.000Z'
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
 parseStringArray('["a","b"]'); // ['a', 'b']
 ```
 */
export function parseStringArray(value: string,): string[] {
  try {
    /**
     Raw JSON.parse output typed as `unknown` until the array shape check runs.
     */
    const parsed = JSON.parse(value,) as unknown;
    if (!Array.isArray(parsed,))
      return [];

    return parsed.filter(function isString(entry,): entry is string {
      return (typeof entry) === 'string';
    },);
  }
  catch (error) {
    console.error(
      'String array column JSON parse failed:',
      error,
    );
    return [];
  }
}

/**
 Deduplicates, trims, and filters empty strings from an optional array.
 
 @param values - Raw string array, or `undefined` to produce an empty result
 
 @returns Normalized deduplicated array
 
 @example
 ```ts
 normalizeStringArray([' a ', 'b', 'a']); // ['a', 'b']
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
