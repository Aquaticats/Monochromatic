/**
 Read-only query functions for the task database.
 
 Each function executes a single SQL query and maps results
 to application-level types using `mapTask()`.
 */
import db from '../db.ts';
import {
  type BlockedTaskLink,
  type SearchTask,
  type Task,
  TASK_NOT_FOUND,
} from '../types.ts';
import { mapTask, } from './task-mapping.ts';
import {
  SQL_SEARCH_FTS,
  SQL_SEARCH_LIKE,
  SQL_SELECT_ALL_TAGS,
  SQL_SELECT_BLOCKED_INBOX,
  SQL_SELECT_FOR_BLOCKER_PICKER,
  SQL_SELECT_IN_PROGRESS,
  SQL_SELECT_INBOX_UNBLOCKED,
  SQL_SELECT_TASK_BY_ID,
  type TaskRow,
} from './task-sql.ts';

/**
 Fetches a single task row by primary key without mapping.
 
 @param id - Task UUID
 
 @returns Raw task row, or {@link TASK_NOT_FOUND} when no row matches
 
 @example
 ```ts
 const row = await getTaskRowById('uuid-123');
 ```
 */
export async function getTaskRowById(id: string,): Promise<TaskRow | typeof TASK_NOT_FOUND> {
  /**
   Single-row lookup result; nullish when the ID does not exist.
   */
  const taskRow: unknown = await (await db.prepare(SQL_SELECT_TASK_BY_ID,))
    .get(id,);
  if ((taskRow === undefined) || (taskRow === null))
    return TASK_NOT_FOUND;
  // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- database prepare().get() returns the row shape
  return taskRow as TaskRow;
}

/**
 Retrieves a single task by UUID, mapped to the application type.
 
 @param id - Task UUID
 
 @returns Mapped task, or {@link TASK_NOT_FOUND} when the ID does not exist
 
 @example
 ```ts
 const task = await getTaskById('uuid-123');
 ```
 */
export async function getTaskById(id: string,): Promise<Task | typeof TASK_NOT_FOUND> {
  /**
   Raw row from `getTaskRowById`; mapped to the application shape when present.
   */
  const taskRow = await getTaskRowById(id,);
  return taskRow === TASK_NOT_FOUND ? TASK_NOT_FOUND : mapTask(taskRow,);
}

/**
 Lists inbox tasks that have no blockers, newest first.
 
 @returns Unblocked inbox tasks sorted by creation date
 
 @example
 ```ts
 const tasks = await listInboxUnblockedTasks();
 ```
 */
export async function listInboxUnblockedTasks(): Promise<Task[]> {
  /* oxlint-disable typescript/no-unsafe-type-assertion -- database query returns TaskRow shape */
  /**
   Raw row list for the unblocked-inbox query; mapped element-wise below.
   */
  const rows = (await (await db.prepare(SQL_SELECT_INBOX_UNBLOCKED,))
    .all()) as TaskRow[];
  /* oxlint-enable typescript/no-unsafe-type-assertion */
  return rows.map(function toTask(row,) {
    return mapTask(row,);
  },);
}

/**
 Lists inbox tasks that are blocked, paired with each blocker ID for nesting.
 
 @returns Blocked task links with blocker IDs
 
 @example
 ```ts
 const links = await listBlockedInboxTasks();
 ```
 */
export async function listBlockedInboxTasks(): Promise<BlockedTaskLink[]> {
  /* oxlint-disable typescript/no-unsafe-type-assertion -- database query returns TaskRow with blocker_id join column */
  /**
   Raw row list paired with the join column; rebuilt into blocker links below.
   */
  const rows = (await (await db
    .prepare(SQL_SELECT_BLOCKED_INBOX,))
    .all()) as (TaskRow & { readonly blocker_id: string; })[];
  /* oxlint-enable typescript/no-unsafe-type-assertion */
  return rows.map(function toBlockedLink(row,) {
    return {
      blockerId: row.blocker_id,
      task: mapTask(row,),
    };
  },);
}

/**
 Lists tasks with active timers, most recently updated first.
 
 @returns In-progress tasks sorted by update date
 
 @example
 ```ts
 const tasks = await listInProgressTasks();
 ```
 */
export async function listInProgressTasks(): Promise<Task[]> {
  /* oxlint-disable typescript/no-unsafe-type-assertion -- database query returns TaskRow shape */
  /**
   Raw row list of tasks with active timers; mapped element-wise below.
   */
  const rows = (await (await db.prepare(SQL_SELECT_IN_PROGRESS,))
    .all()) as TaskRow[];
  /* oxlint-enable typescript/no-unsafe-type-assertion */
  return rows.map(function toTask(row,) {
    return mapTask(row,);
  },);
}

/**
 Lists non-done tasks excluding the given task, for the blocker picker UI.
 
 @param taskId - Task to exclude
 
 @returns Available tasks for blocker selection
 
 @example
 ```ts
 const candidates = await listTasksForBlockerPicker('uuid-123');
 ```
 */
export async function listTasksForBlockerPicker(taskId: string,): Promise<Task[]> {
  /* oxlint-disable typescript/no-unsafe-type-assertion -- database query returns TaskRow shape */
  /**
   Raw candidate rows excluding the current task; mapped element-wise below.
   */
  const rows = (await (await db.prepare(SQL_SELECT_FOR_BLOCKER_PICKER,))
    .all(taskId,)) as TaskRow[];
  /* oxlint-enable typescript/no-unsafe-type-assertion */
  return rows.map(function toTask(row,) {
    return mapTask(row,);
  },);
}

/**
 Collects all unique tags across every task, sorted alphabetically.
 
 @returns Sorted array of unique tag strings
 
 @example
 ```ts
 const tags = await listAllTags(); // ['errands', 'shopping']
 ```
 */
export async function listAllTags(): Promise<string[]> {
  /* oxlint-disable typescript/no-unsafe-type-assertion -- database query returns rows with tag column */
  /**
   Single-column tag rows; flattened to a string array below.
   */
  const rows = (await (await db.prepare(SQL_SELECT_ALL_TAGS,))
    .all()) as { readonly tag: string; }[];
  /* oxlint-enable typescript/no-unsafe-type-assertion */
  return rows.map(function extractTag(row,) {
    return row.tag;
  },);
}

/**
 Full-text searches tasks by title, description, and tags.
 Falls back to LIKE matching when the FTS query syntax is invalid.
 
 @param searchQuery - User-entered search string
 
 @returns Matching tasks with blocked status
 
 @example
 ```ts
 const results = await searchTasks('shopping');
 ```
 */
export async function searchTasks(searchQuery: string,): Promise<SearchTask[]> {
  /**
   Whitespace-trimmed query; empty trim short-circuits with no DB call.
   */
  const normalizedSearchQuery = searchQuery.trim();
  if (normalizedSearchQuery.length
    === 0)
    return [];

  try {
    /* oxlint-disable typescript/no-unsafe-type-assertion -- database FTS query */
    /**
     FTS-matched rows joined with the blocked flag; mapped to {@link SearchTask} below.
     */
    const rows = (await (await db.prepare(SQL_SEARCH_FTS,))
      .all(
      normalizedSearchQuery,
      )) as (TaskRow & { readonly is_blocked: number; })[];
    /* oxlint-enable typescript/no-unsafe-type-assertion */
    return rows.map(function toSearchTask(row,) {
      return {
        ...mapTask(row,),
        isBlocked: row.is_blocked
          === 1,
      };
    },);
  }
  catch (error) {
    console.error(
      'Full-text task search failed, using LIKE fallback:',
      error,
    );
    /* oxlint-disable typescript/no-unsafe-type-assertion -- database LIKE query */
    /**
     Fallback LIKE-match rows used when Turso FTS rejects the query syntax or
     the index is absent on a build without it.
     */
    const rows = (await (await db
      .prepare(SQL_SEARCH_LIKE,))
      .all(
      `%${normalizedSearchQuery}%`,
      `%${normalizedSearchQuery}%`,
      )) as (TaskRow & { readonly is_blocked: number; })[];
    /* oxlint-enable typescript/no-unsafe-type-assertion */
    return rows.map(function toSearchTask(row,) {
      return {
        ...mapTask(row,),
        isBlocked: row.is_blocked
          === 1,
      };
    },);
  }
}
