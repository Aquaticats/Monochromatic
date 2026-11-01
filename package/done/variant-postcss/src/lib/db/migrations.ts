/**
 Schema migration SQL and runner for the task database.
 
 Executed once at startup by `db.ts` to create tables, indexes,
 and the native Turso full-text index.
 */
import type { Database, } from '@tursodatabase/database';
import {
  logger,
  tagged,
} from '@monochromatic-dev/logger-core/ts';

/**
 Tagged logger for the migration runner.
 */
const l = tagged({
  tag: 'db-migrations',
  l: logger,
},);

/**
 Core tables, indexes, and CHECK constraints.
 */
const MIGRATION_TABLES_AND_INDEXES = `
  CREATE TABLE IF NOT EXISTS tasks (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    description TEXT,
    tags TEXT NOT NULL DEFAULT '[]',
    locations TEXT NOT NULL DEFAULT '[]',
    priority TEXT CHECK (priority IN ('low', 'medium', 'high')),
    due_date TEXT,
    complexity TEXT CHECK (complexity IN ('low', 'medium', 'high')),
    reminders TEXT NOT NULL DEFAULT '[]',
    blocked_by TEXT NOT NULL DEFAULT '[]',
    tracked_time INTEGER NOT NULL DEFAULT 0,
    timer_started_at TEXT,
    status TEXT NOT NULL DEFAULT 'inbox' CHECK (status IN ('inbox', 'in_progress', 'done')),
    source TEXT NOT NULL DEFAULT 'local' CHECK (source IN ('local', 'github', 'linear', 'calendar', 'codebase')),
    source_id TEXT,
    source_meta TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
  );

  CREATE TABLE IF NOT EXISTS attachments (
    id TEXT PRIMARY KEY,
    task_id TEXT NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
    filename TEXT NOT NULL,
    mime_type TEXT NOT NULL,
    data BLOB NOT NULL,
    created_at TEXT NOT NULL
  );

  CREATE TABLE IF NOT EXISTS settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
  );

  CREATE INDEX IF NOT EXISTS idx_tasks_status ON tasks(status);
  CREATE INDEX IF NOT EXISTS idx_tasks_status_source ON tasks(status, source);
  CREATE INDEX IF NOT EXISTS idx_tasks_due_date ON tasks(due_date) WHERE due_date IS NOT NULL;
  CREATE INDEX IF NOT EXISTS idx_tasks_source_id ON tasks(source, source_id) WHERE source_id IS NOT NULL;
  CREATE INDEX IF NOT EXISTS idx_attachments_task_id ON attachments(task_id);
`;

/**
 Native Turso full-text index over the searchable task columns.
 
 Unlike a SQLite FTS5 virtual table, Turso's `USING fts` index method attaches
 directly to the base table: it indexes existing rows at creation time and stays
 in sync on every write, so no sync triggers or backfill statement are needed.
 Requires the connection to be opened with `experimental: ['index_method']`.
 */
const MIGRATION_FTS = `
  CREATE INDEX IF NOT EXISTS tasks_fts ON tasks USING fts (title, description, tags);
`;

/**
 Attempts to create the native full-text index, reporting whether FTS is available.
 
 Turso ships full-text search as an experimental index method; a build lacking it
 rejects the `USING fts` statement. Rather than crash startup, this logs the cause
 and reports failure so callers (and {@link runMigrations}) let search degrade to
 LIKE matching.
 
 @param database - Connected Turso database instance
 
 @returns `true` when index creation succeeds, `false` when FTS is unavailable
 
 @example
 ```ts
 const ftsEnabled = await tryEnableFts(database);
 ```
 */
export async function tryEnableFts(database: Database,): Promise<boolean> {
  try {
    await database.exec(MIGRATION_FTS,);
    return true;
  }
  catch (ftsError: unknown) {
    l.warn(
      `native FTS index unavailable; search degrades to LIKE matching: ${String(ftsError,)}`,
    );
    return false;
  }
}

/**
 Executes all schema migrations (tables, indexes, and the guarded FTS index).
 
 Table and index creation must succeed; FTS creation is guarded by
 {@link tryEnableFts} so a build without the experimental FTS index method still
 boots, with search falling back to LIKE matching.
 
 @param database - Connected Turso database instance
 
 @example
 ```ts
 await runMigrations(database);
 ```
 */
export async function runMigrations(database: Database,): Promise<void> {
  await database.exec(MIGRATION_TABLES_AND_INDEXES,);
  await tryEnableFts(database,);
}
