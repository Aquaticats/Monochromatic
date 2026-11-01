/**
 SQL query constants for the task data-access layer.
 
 Each constant is used by exactly one function in `tasks.ts` or `tasks-queries.ts`.
 */

/**
 SQL to select a single task by primary key.
 */
export const SQL_SELECT_TASK_BY_ID = 'SELECT * FROM tasks WHERE id = ?';

/**
 SQL to select unblocked inbox tasks ordered by creation date.
 */
export const SQL_SELECT_INBOX_UNBLOCKED =
  "SELECT * FROM tasks WHERE status = 'inbox' AND blocked_by = '[]' ORDER BY created_at DESC";

/**
 SQL to select blocked inbox tasks with blocker IDs via JSON expansion.
 */
export const SQL_SELECT_BLOCKED_INBOX = `
  SELECT tasks.*, blocker.value AS blocker_id
  FROM tasks, json_each(tasks.blocked_by) AS blocker
  WHERE tasks.status = 'inbox' AND tasks.blocked_by != '[]'
  ORDER BY tasks.created_at DESC`;

/**
 SQL to select in-progress tasks ordered by update date.
 */
export const SQL_SELECT_IN_PROGRESS =
  "SELECT * FROM tasks WHERE status = 'in_progress' ORDER BY updated_at DESC";

/**
 SQL to select non-done tasks excluding a given task for blocker selection.
 */
export const SQL_SELECT_FOR_BLOCKER_PICKER =
  "SELECT * FROM tasks WHERE id != ? AND status != 'done' ORDER BY title ASC";

/**
 SQL to select all unique tags across all tasks.
 */
export const SQL_SELECT_ALL_TAGS =
  'SELECT DISTINCT tag.value AS tag FROM tasks, json_each(tasks.tags) AS tag ORDER BY tag.value ASC';

/**
 SQL for full-text search across task title, description, and tags.
 
 Uses Turso's native `fts_match`/`fts_score` functions against the `tasks_fts`
 index method. The bound query reuses the `?1` numbered parameter across both
 functions. `fts_score` is unreliable on the pinned build (often 0 or tied across
 multiple matches; see `doc/troubleshooting/turso-fts5-native-fts.md`), so
 `updated_at` is a deterministic tiebreaker; genuine relevance ordering activates
 once upstream scoring is fixed.
 */
export const SQL_SEARCH_FTS = `
  SELECT tasks.*, CASE WHEN blocked_by != '[]' THEN 1 ELSE 0 END AS is_blocked
  FROM tasks
  WHERE fts_match(title, description, tags, ?1)
  ORDER BY fts_score(title, description, tags, ?1) DESC, tasks.updated_at DESC`;

/**
 SQL fallback search using LIKE matching on title and description.
 */
export const SQL_SEARCH_LIKE = `
  SELECT tasks.*, CASE WHEN blocked_by != '[]' THEN 1 ELSE 0 END AS is_blocked
  FROM tasks
  WHERE tasks.title LIKE ? OR tasks.description LIKE ?
  ORDER BY tasks.updated_at DESC`;

/**
 SQL to insert a new task with all columns.
 */
export const SQL_INSERT_TASK = `
  INSERT INTO tasks (
    id, title, description, tags, locations, priority, due_date,
    complexity, reminders, blocked_by, tracked_time, timer_started_at,
    status, source, source_id, source_meta, created_at, updated_at
  ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)`;

/**
 SQL to update mutable task fields by ID.
 */
export const SQL_UPDATE_TASK = `
  UPDATE tasks
  SET title = ?, description = ?, tags = ?, locations = ?, priority = ?, due_date = ?,
      complexity = ?, reminders = ?, blocked_by = ?, status = ?, updated_at = ?
  WHERE id = ?`;

/**
 SQL to delete a task by ID.
 */
export const SQL_DELETE_TASK = 'DELETE FROM tasks WHERE id = ?';

/**
 SQL to start a timer on a task, setting status to in_progress.
 */
export const SQL_START_TIMER =
  "UPDATE tasks SET timer_started_at = ?, status = 'in_progress', updated_at = ? WHERE id = ?";

/**
 SQL to stop a timer, accumulating tracked time and resetting status to inbox.
 */
export const SQL_STOP_TIMER =
  "UPDATE tasks SET tracked_time = ?, timer_started_at = NULL, status = 'inbox', updated_at = ? WHERE id = ?";

/**
 SQL to select active blockers for a task via JSON expansion.
 */
export const SQL_SELECT_BLOCKERS = `
  SELECT blocker.value AS blocker_id, tasks.title AS blocker_title
  FROM json_each((SELECT blocked_by FROM tasks WHERE id = ?)) AS blocker
  JOIN tasks ON tasks.id = blocker.value
  WHERE tasks.status != 'done'`;
