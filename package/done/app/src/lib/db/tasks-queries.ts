/**
 Read-only query functions for the task data-access layer.
 */
import db from '../db.ts';
import {
  type BlockedTaskLink,
  type SearchTask,
  type Task,
  TASK_NOT_FOUND,
} from '../types.ts';
import {
  getTaskRowById,
  mapTask,
  type TaskRow,
} from './tasks-helpers.ts';
import {
  SQL_SEARCH_FTS,
  SQL_SEARCH_LIKE,
  SQL_SELECT_ALL_TAGS,
  SQL_SELECT_BLOCKED_INBOX,
  SQL_SELECT_FOR_BLOCKER_PICKER,
  SQL_SELECT_IN_PROGRESS,
  SQL_SELECT_INBOX_UNBLOCKED,
} from './tasks-sql.ts';

/**
 Retrieves a single task by UUID via {@link getTaskRowById}, mapped to the
 application type via {@link mapTask}.
 
 @param id - Task UUID
 
 @returns Mapped task, or {@link TASK_NOT_FOUND} when the ID does not exist
 
 @example
 ```ts
 const task = await getTaskById('abc-123');
 ```
 */
export async function getTaskById(id: string,): Promise<Task | typeof TASK_NOT_FOUND> {
  /**
   Raw row; the not-found sentinel propagates to the caller below.
   */
  const taskRow = await getTaskRowById(id,);
  return taskRow === TASK_NOT_FOUND ? TASK_NOT_FOUND : mapTask(taskRow,);
}

/**
 Lists inbox tasks that have no blockers, newest first, mapped via {@link mapTask}.
 
 @returns Unblocked inbox tasks sorted by creation date
 
 @example
 ```ts
 const tasks = await listInboxUnblockedTasks();
 ```
 */
export async function listInboxUnblockedTasks(): Promise<Task[]> {
  /* oxlint-disable typescript/no-unsafe-type-assertion -- database query returns TaskRow shape */
  /**
   Raw rows from the inbox query, mapped through `mapTask` before returning.
   */
  const rows = (await (await db.prepare(SQL_SELECT_INBOX_UNBLOCKED,))
    .all()) as TaskRow[];
  /* oxlint-enable typescript/no-unsafe-type-assertion */
  return rows.map(function toTask(row,) {
    return mapTask(row,);
  },);
}

/**
 Lists inbox tasks that are blocked, paired with each blocker ID for nesting,
 mapped via {@link mapTask}.
 
 @returns Blocked task links with blocker IDs
 
 @example
 ```ts
 const links = await listBlockedInboxTasks();
 ```
 */
export async function listBlockedInboxTasks(): Promise<BlockedTaskLink[]> {
  /* oxlint-disable typescript/no-unsafe-type-assertion -- database query returns TaskRow with blocker_id join column */
  /**
   Raw rows including the join column used to assemble the blocked-link tuple.
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
 Lists tasks with active timers (`status = 'in_progress'`), most recently
 updated first, mapped via {@link mapTask}.
 
 @returns In-progress tasks sorted by update date
 
 @example
 ```ts
 const running = await listInProgressTasks();
 ```
 */
export async function listInProgressTasks(): Promise<Task[]> {
  /* oxlint-disable typescript/no-unsafe-type-assertion -- database query returns TaskRow shape */
  /**
   Raw rows for in-progress tasks; mapped to the application type below.
   */
  const rows = (await (await db.prepare(SQL_SELECT_IN_PROGRESS,))
    .all()) as TaskRow[];
  /* oxlint-enable typescript/no-unsafe-type-assertion */
  return rows.map(function toTask(row,) {
    return mapTask(row,);
  },);
}

/**
 Lists non-done tasks excluding the given task, for the blocker picker UI,
 mapped via {@link mapTask}.
 
 @param taskId - Task to exclude (the task being edited)
 
 @returns Available tasks for blocker selection
 
 @example
 ```ts
 const candidates = await listTasksForBlockerPicker('current-task-id');
 ```
 */
export async function listTasksForBlockerPicker(taskId: string,): Promise<Task[]> {
  /* oxlint-disable typescript/no-unsafe-type-assertion -- database query returns TaskRow shape */
  /**
   Raw candidate rows excluding the current task, mapped to application objects below.
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
 const tags = await listAllTags();
 // ['errand', 'home', 'work']
 ```
 */
export async function listAllTags(): Promise<string[]> {
  /* oxlint-disable typescript/no-unsafe-type-assertion -- database query returns rows with tag column */
  /**
   Single-column projection; the tag string is unwrapped from each row below.
   */
  const rows = (await (await db.prepare(SQL_SELECT_ALL_TAGS,))
    .all()) as { readonly tag: string; }[];
  /* oxlint-enable typescript/no-unsafe-type-assertion */
  return rows.map(function extractTag(row,) {
    return row.tag;
  },);
}

/**
 Full-text searches tasks by title, description, and tags, mapping rows via
 {@link mapTask}. Falls back to LIKE matching when the FTS query syntax is invalid.
 
 @param searchQuery - User-entered search string
 
 @returns Matching tasks with blocked status
 
 @example
 ```ts
 const results = await searchTasks('groceries');
 ```
 */
export async function searchTasks(searchQuery: string,): Promise<SearchTask[]> {
  /**
   Trimmed query reused by both the FTS attempt and the LIKE fallback.
   */
  const normalizedSearchQuery = searchQuery.trim();
  if (normalizedSearchQuery.length
    === 0)
    return [];

  try {
    /* oxlint-disable typescript/no-unsafe-type-assertion -- database FTS query returns TaskRow with is_blocked column */
    /**
     FTS rows including the blocked-flag join column for the search-card UI.
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
  catch (ftsQueryError: unknown) {
    // Turso FTS rejects some user query syntax, and errors when the index is
    // absent on a build without it; log the cause, then fall back to LIKE.
    console.error(
      'searchTasks FTS query failed; falling back to LIKE matching:',
      ftsQueryError,
    );
    /* oxlint-disable typescript/no-unsafe-type-assertion -- database LIKE query returns TaskRow with is_blocked column */
    /**
     Fallback LIKE rows when the FTS syntax is rejected by SQLite.
     */
    const rows = (await (await db
      .prepare(SQL_SEARCH_LIKE,))
      .all(
      `%${normalizedSearchQuery}%`,
      `%${normalizedSearchQuery}%`,
      )) as (TaskRow & {
        readonly is_blocked: number;
      })[];
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
