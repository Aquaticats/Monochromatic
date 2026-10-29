# Done; Implementation plan

This plan no longer follows a competition deadline.
The day-by-day sequence and hour estimates now serve as implementation guidance only,
 not as a hard timeline.
Items without a priority marker are implicitly highest priority.

> Stale-warning,
>  2026-05-13:
>  lower historical sections still contain the original
> `Bun.serve()` / `Bun.build()` sketch.
>  Current implementation uses h3 route
> registration,
>  h-css runtime-generated styles,
>  and tsdown client bundles.
>  Trust
> `README.md` and `src/server.ts` over older embedded snippets.

## Architecture overview

No framework.
 Vanilla TypeScript on both server and client,
 unified by Node and tsdown.

- **Server:
  ** h3 `H3` route registration handles page and API routes.
   Static serving handles built client assets from `dist/client/`.
- **Client:
  ** Plain TypeScript with `document.createElement` for DOM construction.
   Custom elements where reuse is needed (task card,
   chip editor,
   collapsible section).
- **Build:
  ** h-css runtime-generated styles and tsdown client bundles are built through mise tasks.
- **CSS:
  ** CSS files use `@mixin`/`@apply` syntax processed by `@monochromatic-dev/build-css`.
   Processed CSS is imported as text in client TS and injected at runtime;
   no separate `<link>` tags needed.
- **Dev:
  ** `mise watch -w src -r -- node src/server.ts` restarts the server process on source changes.
- **Operational advantage:
  ** The orchestrator spawns per-user Node processes.
   Each process runs the build pipeline at startup,
   so new/restarted processes immediately serve the latest code.

See `FRAMEWORK_EVALUATION.md` for why this approach was chosen over SvelteKit,
 Vue Vapor,
 or Web Components frameworks.

## Day 1 (Tue): Scaffolding and data layer (~4h)

### 1.1 Project setup (~1.5h) **done**

- (0.25h) Initialize project at `package/site/done/` with `package.json` (`workspace:*` references to monorepo packages)
- (0.25h) Add `mise.toml` with `dev`,
   `build`,
   and `start` tasks
- (0.25h) Create `src/server.ts`:
   entry point with h3 `H3` route registration and static serving
- (0.25h) Create `src/client/` directory with a minimal `inbox.ts` entry point to verify the build pipeline
- (0.25h) Verify:
   `mise run dev:site` starts the Node server,
   serves the built JS,
   and restarts on file change

Historical Bun server-entry sketch,
 superseded by the current h3 implementation in `src/server.ts`:

```ts
// src/server.ts -- entry point for each user's Done instance
// Pipeline: build-css -> Bun.build() -> Bun.serve().
// In dev (bun --watch), server restarts on any file change -> full rebuild is automatic.
import { build as buildCSS, } from '@monochromatic-dev/build-css/ts';
import './lib/db';
import {
  handleCreateTask,
  handleDeleteTask,
  handleUpdateTask,
} from './server/api/tasks';
import {
  handleCompleteTask,
  handleStartTimer,
  handleStopTimer,
} from './server/api/timer';
import { inProgressPage, } from './server/page/in-progress';
import { inboxPage, } from './server/page/inbox';
import { searchPage, } from './server/page/search';
import { settingsPage, } from './server/page/settings';
import { taskDetailsPage, } from './server/page/task-details';
// ... other API imports

// Step 1: Process CSS -- resolves @import, expands @mixin/@apply into plain CSS.
await buildCSS({
  input: './src/client/styles.css',
  output: './dist/client/styles.css',
},);

// Step 2: Bundle client TS -- imports the processed CSS as text string.
const buildResult = await Bun.build({
  entrypoints: [
    './src/client/inbox.ts',
    './src/client/in-progress.ts',
    './src/client/task-details.ts',
    './src/client/search.ts',
    './src/client/settings.ts',
  ],
  outdir: './dist/client',
  target: 'browser',
  minify: process.env.NODE_ENV === 'production',
},);

if (!buildResult.success) {
  console.error('Client build failed:', buildResult.logs,);
  process.exit(1,);
}

// Step 3: Serve. Bun's built-in router handles :param parsing and per-method dispatch.
// No separate router.ts needed -- routes are declarative, type-safe, SIMD-accelerated.
Bun.serve({
  port: Number(process.env.PORT,) || 3000,
  routes: {
    // Page routes -- return HTML with embedded data + script tag
    '/': () => inboxPage(),
    '/in-progress': () => inProgressPage(),
    '/tasks/:id': req => taskDetailsPage(req.params.id,),
    '/search': req => searchPage(new URL(req.url,),),
    '/settings': () => settingsPage(),

    // API routes -- per-method dispatch, typed params
    '/api/tasks': { POST: req => handleCreateTask(req,), },
    '/api/tasks/:id': {
      PUT: req => handleUpdateTask(req, req.params.id,),
      DELETE: req => handleDeleteTask(req.params.id,),
    },
    '/api/tasks/:id/start': { POST: req => handleStartTimer(req.params.id,), },
    '/api/tasks/:id/stop': { POST: req => handleStopTimer(req.params.id,), },
    '/api/tasks/:id/complete': {
      POST: req => handleCompleteTask(req.params.id,),
    },
    // ... ai, attachments, settings, sync routes
  },
  // Fallback: static assets from build output, or 404.
  // Bun.file() auto-detects Content-Type from file extension.
  async fetch(req,) {
    const path = new URL(req.url,).pathname;
    if (path.startsWith('/dist/client/',)) {
      const file = Bun.file(`.${path}`,);
      if (await file.exists())
        return new Response(file,);
    }
    return new Response('Not found', { status: 404, },);
  },
},);
```

### Page rendering pattern

Each page handler queries the DB,
 serializes the data into a JSON script tag,
 and returns a full HTML document.
 The client TS reads the embedded data and builds the DOM.
 No client-side fetch on initial load.
 CSS is bundled into the JS (imported as text,
 injected at runtime) so no `<link>` tag is needed.

```ts
// src/server/page/inbox.ts
export async function inboxPage(): Promise<Response> {
  const tasks = await getInboxTasks(db,);
  const settings = await getAllSettings(db,);

  const html = `<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>Inbox - Done</title>
</head>
<body>
  <script type="application/json" id="page-data">${
    JSON.stringify({ tasks, settings, },)
  }</script>
  <script type="module" src="/dist/client/inbox.js"></script>
</body>
</html>`;

  return new Response(html, {
    headers: { 'Content-Type': 'text/html; charset=utf-8', },
  },);
}
```

Client-side (CSS injection validated in bun-test):

```ts
// src/client/inbox.ts
// CSS is imported as a text string by Bun.build(), injected into <head> at runtime.
import styles from '../dist/client/styles.css' with { type: 'text', };
import { injectCSS, } from './lib/inject-css';
injectCSS(styles,);

// Reads server-embedded data, builds DOM. No fetch on load.
const data = JSON.parse(document.getElementById('page-data',)!.textContent!,);
buildInboxScreen(data.tasks, data.settings,);
```

```ts
// src/client/lib/inject-css.ts -- 3 lines, validated in bun-test
export function injectCSS(css: string,): void {
  const style = document.createElement('style',);
  style.textContent = css;
  document.head.appendChild(style,);
}
```

### 1.2 Database schema and migration (~3h) **done**

- (0.5h) Install `@libsql/client`,
   create `src/lib/db.ts` with `createClient({ url: "file:/data/<user-id>/done.db" })` and migration runner
- (1.0h) Write core tables SQL (tasks,
   attachments,
   settings) and run migration on first startup
- (0.5h) Write FTS5 virtual table + sync triggers (insert/update/delete)
- (0.5h) Write indexes for common query patterns
- (0.5h) Verify migration runs cleanly on a fresh `.db` file,
   test FTS5 triggers manually

libsql is SQLite-compatible,
 so all standard SQLite features work.
The schema below is the complete initial migration:
 run it once on first startup.

```sql
-- ============================================================
-- Core tables
-- ============================================================

-- Every task lives here. JSON columns store arrays as text
-- (SQLite has no native array type). Parse them in application code
-- with JSON.parse().
CREATE TABLE tasks (
  id TEXT PRIMARY KEY,              -- ULID (sortable unique ID, generated in app code)
  title TEXT NOT NULL,
  description TEXT,
  tags TEXT NOT NULL DEFAULT '[]',  -- JSON array of strings, e.g. '["shopping","errands"]'
  locations TEXT NOT NULL DEFAULT '[]', -- JSON array of strings, e.g. '["Walmart","Loblaws"]'
  priority TEXT CHECK (priority IN ('low', 'medium', 'high')),
  due_date TEXT,                    -- ISO8601 string, e.g. '2026-02-14T00:00:00Z'
  complexity TEXT CHECK (complexity IN ('low', 'medium', 'high')),
  reminders TEXT NOT NULL DEFAULT '[]', -- JSON array of ISO8601 timestamps
  blocked_by TEXT NOT NULL DEFAULT '[]', -- JSON array of task ID strings
  tracked_time INTEGER NOT NULL DEFAULT 0, -- cumulative seconds spent on task
  timer_started_at TEXT,            -- ISO8601 timestamp when timer was last started; NULL = not running
  status TEXT NOT NULL DEFAULT 'inbox' CHECK (status IN ('inbox', 'in_progress', 'done')),
  source TEXT NOT NULL DEFAULT 'local' CHECK (source IN ('local', 'github', 'linear', 'calendar', 'codebase')),
  source_id TEXT,                   -- external identifier for sync (e.g. github issue URL, file:line for TODOs)
  source_meta TEXT,                 -- full JSON from external source, preserved for lossless round-trip sync
  created_at TEXT NOT NULL,         -- ISO8601, set once on creation
  updated_at TEXT NOT NULL          -- ISO8601, updated on every modification
);

-- File and photo attachments. Stored as BLOBs directly in the database
-- (acceptable for single-tenant with modest attachment sizes).
CREATE TABLE attachments (
  id TEXT PRIMARY KEY,              -- ULID
  task_id TEXT NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
  filename TEXT NOT NULL,           -- original filename, e.g. 'receipt.jpg'
  mime_type TEXT NOT NULL,          -- e.g. 'image/jpeg'
  data BLOB NOT NULL,              -- raw file bytes
  created_at TEXT NOT NULL
);

-- Key-value store for instance configuration.
-- Avoids needing a config file; everything lives in the same DB that gets backed up.
CREATE TABLE settings (
  key TEXT PRIMARY KEY,
  value TEXT NOT NULL
);
-- Expected keys:
--   'llama_model'           -- preferred model name from the shared llama.cpp instance
--   'user_email'           -- for sending reminders and daily backups
--   'user_location'        -- current/default location name
--   'focus_directive'      -- free-text AI focus instruction
--   'github_token'         -- personal access token for GitHub sync
--   'github_repos'         -- JSON array of 'owner/repo' strings to sync
--   'codebase_paths'       -- JSON array of local directory paths to scan for TODOs

-- ============================================================
-- Full-text search (FTS5)
-- ============================================================

-- FTS5 is a SQLite extension for full-text search. It tokenizes text
-- and builds an inverted index so you can search by words/phrases
-- instead of exact string matching (LIKE '%...%' is slow and limited).
--
-- content=tasks means FTS5 doesn't store its own copy of the text --
-- it references the tasks table. This saves disk space but means
-- we must manually keep it in sync via triggers (below).

CREATE VIRTUAL TABLE tasks_fts USING fts5(
  title,                            -- searchable: task title
  description,                      -- searchable: task description
  tags,                             -- searchable: tags as JSON string (matches individual tag names)
  content=tasks,                    -- "contentless" mode: reads from tasks table, doesn't duplicate data
  content_rowid=rowid               -- links FTS rows to tasks via SQLite's implicit rowid
);

-- Triggers keep FTS5 in sync with the tasks table.
-- Without these, search results would go stale after inserts/updates/deletes.

-- On INSERT: add the new task's searchable fields to the FTS index.
CREATE TRIGGER tasks_fts_insert AFTER INSERT ON tasks BEGIN
  INSERT INTO tasks_fts(rowid, title, description, tags)
  VALUES (new.rowid, new.title, new.description, new.tags);
END;

-- On UPDATE: remove the old entry, add the new one.
-- FTS5 doesn't support UPDATE in-place for content= tables,
-- so we delete-then-insert.
CREATE TRIGGER tasks_fts_update AFTER UPDATE ON tasks BEGIN
  INSERT INTO tasks_fts(tasks_fts, rowid, title, description, tags)
  VALUES ('delete', old.rowid, old.title, old.description, old.tags);
  INSERT INTO tasks_fts(rowid, title, description, tags)
  VALUES (new.rowid, new.title, new.description, new.tags);
END;

-- On DELETE: remove the entry from the FTS index.
CREATE TRIGGER tasks_fts_delete AFTER DELETE ON tasks BEGIN
  INSERT INTO tasks_fts(tasks_fts, rowid, title, description, tags)
  VALUES ('delete', old.rowid, old.title, old.description, old.tags);
END;

-- ============================================================
-- Indexes
-- ============================================================

-- Speed up the most common queries:

-- Inbox screen: WHERE status = 'inbox' (most tasks live here)
-- In Progress screen: WHERE status = 'in_progress'
CREATE INDEX idx_tasks_status ON tasks(status);

-- Suggestion engine filters by status + source
CREATE INDEX idx_tasks_status_source ON tasks(status, source);

-- Reminder email job: find tasks with upcoming reminders
-- (scans reminders JSON, but index on status narrows the scan)
CREATE INDEX idx_tasks_due_date ON tasks(due_date) WHERE due_date IS NOT NULL;

-- Sync: find tasks by external source for conflict detection
CREATE INDEX idx_tasks_source_id ON tasks(source, source_id) WHERE source_id IS NOT NULL;

-- Attachments: look up all attachments for a task
CREATE INDEX idx_attachments_task_id ON attachments(task_id);
```

### 1.3 Database helper functions (~3h) **partial** (tasks.ts done; attachments, settings, reminders not started)

- (0.5h) Write `src/lib/db/tasks.ts`:
   CRUD functions (create,
   read,
   update,
   delete task)
- (0.5h) Write `src/lib/db/tasks.ts`:
   timer functions (startTimer,
   stopTimer,
   completeTask with blocker check)
- (0.5h) Write `src/lib/db/tasks.ts`:
   query functions (inbox unblocked,
   inbox blocked-by-parent,
   in-progress,
   search)
- (0.5h) Write `src/lib/db/attachments.ts`:
   create,
   list-by-task,
   get-by-id,
   delete
- (0.5h) Write `src/lib/db/settings.ts`:
   get,
   set,
   getAll
- (0.5h) Write `src/lib/db/reminders.ts`:
   getDueReminders,
   removeReminder

The SQL patterns below are reference material for implementing these helpers.

### How to query full-text search

```sql
-- Search for tasks matching a query string (e.g. user types "milk walmart")
-- FTS5 tokenizes the query and matches against title, description, tags.
-- Results are ranked by relevance (BM25 algorithm, built into FTS5).
SELECT tasks.*
FROM tasks_fts
JOIN tasks ON tasks.rowid = tasks_fts.rowid
WHERE tasks_fts MATCH ?   -- ? = the search query, e.g. 'milk walmart'
ORDER BY rank;             -- rank is a hidden column FTS5 provides (lower = more relevant)

-- Prefix search (as-you-type): append * to the query
-- e.g. 'wal*' matches 'walmart', 'wallet', etc.
SELECT tasks.*
FROM tasks_fts
JOIN tasks ON tasks.rowid = tasks_fts.rowid
WHERE tasks_fts MATCH 'wal*'
ORDER BY rank;
```

### How to read/write JSON array columns

```ts
// libsql API: client.execute({ sql, args }) returns { rows, columns, rowsAffected }

// Reading: parse the JSON string from SQLite into a JS array
const result = await client.execute({ sql: 'SELECT * FROM tasks WHERE id = ?',
  args: [taskId,], },);
const task = result.rows[0];
const tags: string[] = JSON.parse(task.tags as string,); // '["shopping"]' -> ['shopping']
const locations: string[] = JSON.parse(task.locations as string,);

// Writing: stringify the JS array back to JSON for SQLite
await client.execute({ sql: 'UPDATE tasks SET tags = ? WHERE id = ?',
  args: [JSON.stringify(['shopping', 'errands',],), taskId,], },);

// Querying tasks that contain a specific tag (uses SQLite JSON functions):
// json_each() expands a JSON array into rows, so we can match individual elements.
const tagResult = await client.execute({
  sql:
    'SELECT tasks.* FROM tasks, json_each(tasks.tags) AS tag WHERE tag.value = ?',
  args: ['shopping',],
},);
const tasksWithTag = tagResult.rows;
```

### How timers work in SQL

```ts
// Start timer: record the current UTC timestamp
await client.execute({
  sql: `UPDATE tasks
        SET timer_started_at = datetime('now'),
            status = 'in_progress',
            updated_at = datetime('now')
        WHERE id = ?`,
  args: [taskId,],
},);

// Stop timer: calculate elapsed seconds, add to cumulative total, clear the timestamp
await client.execute({
  sql: `UPDATE tasks
        SET tracked_time = tracked_time + CAST(
              (julianday('now') - julianday(timer_started_at)) * 86400
            AS INTEGER),
            timer_started_at = NULL,
            status = 'inbox',
            updated_at = datetime('now')
        WHERE id = ?`,
  args: [taskId,],
},);

// Read current elapsed time for display (without stopping):
// In the page handler, return both tracked_time and timer_started_at.
// The client computes: displaySeconds = trackedTime + (Date.now() - Date.parse(timerStartedAt)) / 1000
```

### How blocking validation works in SQL

```ts
// Before completing a task, check if all blockers are done.
// blocked_by is a JSON array of task IDs like '["01HX...","01HY..."]'.
// We expand it with json_each and check if any blocker still exists (not yet deleted/done).

const result = await client.execute({
  sql: `SELECT blocker.value AS blocker_id, tasks.title AS blocker_title
        FROM json_each(
          (SELECT blocked_by FROM tasks WHERE id = ?)
        ) AS blocker
        JOIN tasks ON tasks.id = blocker.value
        WHERE tasks.status != 'done'`,
  args: [taskId,],
},);

if (result.rows.length > 0) {
  return new Response(
    JSON.stringify({ error: 'Task is blocked', blockedBy: result.rows, },),
    {
      status: 409,
      headers: { 'Content-Type': 'application/json', },
    },
  );
}

// If no active blockers, proceed with completion
await client.execute({ sql: 'DELETE FROM tasks WHERE id = ?',
  args: [taskId,], },);
```

### How to query tasks with nested blocked children

```ts
// Fetch top-level (unblocked) inbox tasks for "All" section
const topLevel = await client.execute({
  sql:
    `SELECT * FROM tasks WHERE status = 'inbox' AND blocked_by = '[]' ORDER BY created_at DESC`,
  args: [],
},);

// Batched: fetch ALL blocked inbox tasks and their blocker IDs in one query
// Group by blocker on the client side (avoids N+1)
const blocked = await client.execute({
  sql: `SELECT tasks.*, blocker.value AS blocker_id
        FROM tasks, json_each(tasks.blocked_by) AS blocker
        WHERE tasks.status = 'inbox' AND tasks.blocked_by != '[]'`,
  args: [],
},);

// Search always includes blocked tasks (with a badge)
const searchResults = await client.execute({
  sql: `SELECT tasks.*,
          CASE WHEN blocked_by != '[]' THEN 1 ELSE 0 END AS is_blocked
        FROM tasks_fts
        JOIN tasks ON tasks.rowid = tasks_fts.rowid
        WHERE tasks_fts MATCH ?
        ORDER BY rank`,
  args: [query,],
},);
```

### How reminder scheduling works

```ts
// Reminders are stored as a JSON array of ISO8601 timestamps:
// '["2026-02-14T09:00:00Z","2026-02-14T17:00:00Z"]'
//
// A background loop runs every 60 seconds, finds due reminders,
// sends emails, then removes the fired reminder from the array.

const dueResult = await client.execute({
  sql:
    `SELECT tasks.id, tasks.title, tasks.description, reminder.value AS reminder_time
        FROM tasks, json_each(tasks.reminders) AS reminder
        WHERE reminder.value <= datetime('now')
          AND tasks.status != 'done'`,
  args: [],
},);

for (const row of dueResult.rows) {
  await sendReminderEmail(row,);

  // Remove the fired reminder from the JSON array
  await client.execute({
    sql: `UPDATE tasks
          SET reminders = (
            SELECT json_group_array(r.value)
            FROM json_each(tasks.reminders) AS r
            WHERE r.value != ?
          ),
          updated_at = datetime('now')
          WHERE id = ?`,
    args: [row.reminder_time, row.id,],
  },);
}
```

### How daily backup works

```ts
// Export task data as JSON for the daily email attachment.
// This runs once per day on a schedule (e.g., setTimeout loop or Bun cron).
// BLOBs (attachments.data) and the raw .db file are EXCLUDED to stay within SMTP size limits.

import { createMessage, } from '@upyo/core';
import { SmtpTransport, } from '@upyo/smtp';

// @upyo/smtp actual API: host/port/secure/auth (not hostname/username/password)
const transport = new SmtpTransport({
  host: SMTP_HOST,
  port: Number(SMTP_PORT,),
  secure: true,
  auth: { user: SMTP_USER, pass: SMTP_PASS, },
},);

const tasks = await client.execute({ sql: 'SELECT * FROM tasks', args: [], },);
const attachments = await client.execute({
  sql: 'SELECT id, task_id, filename, mime_type, created_at FROM attachments',
  args: [],
},);
const settings = await client.execute({ sql: 'SELECT * FROM settings',
  args: [], },);

const jsonBackup = JSON.stringify({
  tasks: tasks.rows,
  attachments: attachments.rows,
  settings: settings.rows,
}, null, 2,);

// @upyo/core actual API: content.text (not bare text), returns receipt with .successful
const message = createMessage({
  from: SMTP_FROM,
  to: userEmail,
  subject: `Done - Daily Backup - ${new Date().toISOString().slice(0, 10,)}`,
  content: { text: 'Your daily Done database backup is attached.', },
  attachments: [{ filename: 'done-backup.json',
    content: new TextEncoder().encode(jsonBackup,), },],
},);

// Fire and forget -- if SMTP fails, that's the provider's problem
const receipt = await transport.send(message,).catch(console.error,);
await transport.close();
```

### Route structure

Single `Bun.serve()` process handles both pages (HTML responses) and API (JSON responses).

```text
src/
  server.ts                         -- entry point: build-css + Bun.build() + Bun.serve({ routes })
  server/
    pages/                          -- each returns full HTML with embedded JSON data
      inbox.ts                      -- GET /
      in-progress.ts                -- GET /in-progress
      task-details.ts               -- GET /tasks/:id
      search.ts                     -- GET /search?q=
      settings.ts                   -- GET /settings
    api/                            -- each returns JSON, wired via routes { METHOD: handler }
      tasks.ts                      -- POST/PUT/DELETE /api/tasks/*
      timer.ts                      -- POST /api/tasks/:id/start|stop|complete
      ai-autofill.ts                -- POST /api/ai/autofill
      ai-suggest.ts                 -- POST /api/ai/suggest
      attachments.ts                -- POST /api/tasks/:id/attach, GET /api/attachments/:id
      settings.ts                   -- PUT /api/settings
      sync.ts                       -- GET/POST /api/sync
  client/
    styles.css                      -- base styles with @import 'mixin.css' and @apply rules
    mixin.css                       -- @mixin definitions (processed by build-css)
    inbox.ts                        -- imports processed CSS as text, reads #page-data, builds DOM
    in-progress.ts                  -- same pattern, plus setInterval timer tick
    task-details.ts                 -- same pattern, builds editable form
    search.ts                       -- same pattern, wires debounced input
    settings.ts                     -- same pattern, builds settings form
    components/
      task-card.ts                  -- custom element <task-card>
      chip-editor.ts                -- custom element <chip-editor>
      collapsible-section.ts        -- custom element <collapsible-section>
      fab.ts                        -- FAB button (plain function)
      toast.ts                      -- error toast display (plain function)
    lib/
      api.ts                        -- fetch wrapper with error toast
      inject-css.ts                 -- 3-line CSS injection helper
      dom.ts                        -- DOM construction helpers (if needed)
  lib/
    db.ts                           -- libsql client setup + migration runner
    db/
      tasks.ts                      -- CRUD, timer, query functions
      attachments.ts                -- attachment CRUD
      settings.ts                   -- settings get/set
      reminders.ts                  -- reminder queries
    ai/
      client.ts                     -- llama.cpp HTTP client + rate limiter
      prompts.ts                    -- prompt templates
    email/
      transport.ts                  -- @upyo/smtp wrapper
      reminders.ts                  -- reminder email sender
      backup.ts                     -- daily backup email
```

### Mutation pattern

No form actions or progressive enhancement.
 Client-side `fetch()` to API routes,
 then re-render or navigate.

```ts
// Client-side: create a task
const res = await fetch('/api/tasks', {
  method: 'POST',
  headers: { 'Content-Type': 'application/json', },
  body: JSON.stringify({ title, tags, locations, },),
},);
if (!res.ok) {
  const err = await res.json();
  showToast(err.error,);
  return;
}
// Navigate to refresh data (full page load -- simple, no client-side state to sync)
globalThis.location.reload();
```

```ts
// Server-side: API handler
export async function handleCreateTask(req: Request,): Promise<Response> {
  const body = await req.json();
  // validate with zod-mini
  const parsed = TaskCreateSchema.safeParse(body,);
  if (!parsed.success) {
    return new Response(JSON.stringify({ error: parsed.error.message, },), {
      status: 400,
      headers: { 'Content-Type': 'application/json', },
    },);
  }
  const task = await createTask(db, parsed.data,);
  return new Response(JSON.stringify(task,), {
    status: 201,
    headers: { 'Content-Type': 'application/json', },
  },);
}
```

### Error handling pattern

Happy-path only for competition.
 All errors echo to the user in a toast.

```ts
// Client-side: src/client/lib/api.ts
// Wraps fetch with error handling -- all API calls go through this.
export async function api(path: string, options?: RequestInit,): Promise<any> {
  const res = await fetch(path, {
    ...options,
    headers: { 'Content-Type': 'application/json', ...options?.headers, },
  },);
  if (!res.ok) {
    const err = await res.json().catch(() => ({ error: 'Request failed', }));
    showToast(err.error || 'Something went wrong',);
    throw new Error(err.error,);
  }
  return res.json();
}
```

```ts
// Server-side: all API handlers catch errors and return JSON
try {
  // ... handler logic
}
catch (e) {
  return new Response(JSON.stringify({ error: String(e,), },), {
    status: 500,
    headers: { 'Content-Type': 'application/json', },
  },);
}
```

### Orchestrator database schema (orchestrator.db)

The orchestrator maintains its own SQLite database for user accounts,
 sessions,
 and process tracking.
This is separate from each user's `done.db`.

```sql
CREATE TABLE users (
  id TEXT PRIMARY KEY,                -- ULID
  email TEXT NOT NULL UNIQUE,
  password_hash TEXT NOT NULL,        -- argon2id via Bun.password.hash()
  email_verified INTEGER NOT NULL DEFAULT 0,
  verification_token TEXT,            -- random token sent via email, nulled after verification
  created_at TEXT NOT NULL            -- ISO8601
);

CREATE TABLE sessions (
  id TEXT PRIMARY KEY,                -- random hex string (32 bytes)
  user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  expires_at TEXT NOT NULL,           -- ISO8601, e.g., 7 days from creation
  created_at TEXT NOT NULL
);

CREATE TABLE processes (
  user_id TEXT PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
  port INTEGER NOT NULL UNIQUE,       -- allocated from PORT_RANGE_START..PORT_RANGE_END
  pid INTEGER,                        -- OS process ID, null if not running
  last_request_at TEXT                -- ISO8601, updated on each proxied request
);

CREATE INDEX idx_sessions_user ON sessions(user_id);
CREATE INDEX idx_sessions_expires ON sessions(expires_at);
```

### Session cookie format

The session cookie is a simple lookup token,
 not a signed JWT.
The `sessions` table is the source of truth:
 no crypto to get wrong.

```http
Cookie: session=<session-id>; Path=/u/<user-id>/; HttpOnly; Secure; SameSite=Strict
```

- `session-id`:
   32-byte random hex string (via `crypto.randomUUID()` or `crypto.getRandomValues()`)
- On each request,
   orchestrator looks up `session-id` in the `sessions` table
- If found and not expired,
   extract `user_id` and compare against the path's `<user-id>`
- If expired,
   delete the row and redirect to login
- No HMAC signing needed;
   the session ID itself is the secret (high entropy,
   stored server-side)

### Orchestrator request flow (pseudocode)

```ts
// Main request handler -- Bun.serve() in the orchestrator process
function handleRequest(req: Request,): Response | Promise<Response> {
  const url = new URL(req.url,);
  const path = url.pathname;

  // 1. Public routes (no auth required)
  if (path === '/register')
    return handleRegister(req,);
  if (path === '/login')
    return handleLogin(req,);
  if (path === '/verify')
    return handleEmailVerification(req,);
  if (path === '/')
    return Response.redirect('/login',);

  // 2. Extract user-id from path: /u/<user-id>/...
  const match = path.match(/^\/u\/([A-Z0-9]{26})\//,); // ULID is 26 chars
  if (!match)
    return new Response('Not found', { status: 404, },);
  const pathUserId = match[1];

  // 3. Validate session cookie
  const sessionId = parseCookie(req.headers.get('cookie',), 'session',);
  if (!sessionId)
    return Response.redirect('/login',);

  const session = await lookupSession(sessionId,); // DB query
  if (!session || new Date(session.expires_at,) < new Date())
    return Response.redirect('/login',);

  // 4. Path ACL: session user must match path user
  if (session.user_id !== pathUserId)
    return new Response('Forbidden', { status: 403, },);

  // 5. Ensure user process is running
  const proc = await ensureProcessRunning(pathUserId,); // spawn if needed

  // 6. Update last-request timestamp (fire and forget)
  updateLastRequest(pathUserId,);

  // 7. Proxy to user's process, stripping /u/<user-id> prefix
  const stripped = path.replace(`/u/${pathUserId}`, '',) || '/';
  const proxyUrl = `http://localhost:${proc.port}${stripped}${url.search}`;
  return fetch(proxyUrl, {
    method: req.method,
    headers: req.headers,
    body: req.body,
  },);
}
```

### Registration and login page sketches

The orchestrator serves plain HTML pages for auth flows.
Minimal styling:
 these are functional forms,
 not the app itself.

**Registration (`/register`):
**

```html
<form
  method='POST'
  action='/register'>
  <h1>Create your Done account</h1>
  <label>Email <input
      type='email'
      name='email'
      required></label>
  <label>Password <input
      type='password'
      name='password'
      required
      minlength='8'></label>
  <button type='submit'>Register</button>
  <!-- On error, re-render page with error message in a <p class="error"> -->
  <!-- On success, show "Check your email for verification link" -->
</form>
```

**Login (`/login`):
**

```html
<form
  method='POST'
  action='/login'>
  <h1>Log in to Done</h1>
  <label>Email <input
      type='email'
      name='email'
      required></label>
  <label>Password <input
      type='password'
      name='password'
      required></label>
  <button type='submit'>Log in</button>
  <!-- Same error for wrong email and wrong password: "Invalid email or password" -->
  <!-- On success, Set-Cookie + redirect to /u/<user-id>/ -->
</form>
```

**Email verification (`/verify?token=<token>`):
**

- GET request with token query param
- Look up token in `users` table,
   set `email_verified = 1`,
   null the token
- On success:
   redirect to `/login` with "Email verified" message
- On failure:
   "Invalid or expired verification link"

### AI model recommendation

For CPU-only inference on a competition server,
 use the smallest model that reliably produces structured JSON.
Our prompts are simple (infer tags from a title,
 rank tasks by context):
 not hard reasoning problems.
Speed matters more than capability here because autofill runs on every keystroke (debounced).

#### Primary choice: Qwen3-1.7B (Q4_K_M quantization)

- ~1.2GB RAM,
   good structured JSON output (community-validated with Ollama JSON schema constraints)
- Use **non-thinking mode** (`/no_think` or temperature=0) for fast autofill without chain-of-thought overhead
- Official GGUF from Qwen team:
   `Qwen/Qwen3-1.7B-GGUF`
- llama.
  cpp auto-downloads:
   `--hf-repo Qwen/Qwen3-1.7B-GGUF --hf-file Qwen3-1.7B-Q4_K_M.gguf --ctx-size 4096`

#### Fallback: LFM2.5-1.2B-Instruct (GGUF)

- Under 1GB RAM,
   239 tok/s on AMD CPU;
   remarkably fast
- IFEval 86.23% (instruction following) is strong for its size
- Official GGUF:
   `llama-cli -hf LiquidAI/LFM2.5-1.2B-Instruct-GGUF`
- Use if Qwen3 is too slow on the competition server's CPU

Both support the OpenAI-compatible `/v1/chat/completions` endpoint in llama.
cpp server mode.

### AI prompt templates (reference for day 4)

#### Autofill prompt (4.2)

```text
System: You are a task metadata assistant. Given a task title, infer metadata.
Return ONLY valid JSON matching this schema, no other text:
{
  "tags": string[],          // e.g., ["shopping", "errands"]
  "locations": string[],     // e.g., ["Walmart"] -- places where this task can be done
  "priority": "low" | "medium" | "high" | null,
  "complexity": "low" | "medium" | "high" | null,
  "splitSuggestion": null | { "reason": string, "tasks": string[] }
}

For consistency, prefer these existing tags: <existing_tags>
For consistency, prefer these existing locations: <existing_locations>

If the task implies multiple locations with AND semantics (must visit A then B),
set splitSuggestion with the reason and proposed sub-task titles.

<user_task>
{title}
</user_task>
```

#### Suggestion ranking prompt (5.1)

```text
System: You are a task prioritization assistant. Given a list of tasks and the user's
current context, return the task IDs ranked by what the user should do next.
Return ONLY a JSON array of task ID strings, most important first. No other text.

<user_context>
Location: {currentLocation}
Focus: {focusDirective}
Time: {currentTime}
</user_context>

<user_tasks>
{tasks as JSON array of {id, title, tags, locations, priority, dueDate, complexity}}
</user_tasks>
```

**Task splitting prompt (4.3):
**
Handled by the autofill prompt above via the `splitSuggestion` field.
When the autofill response includes a non-null `splitSuggestion`,
 the UI shows a confirmation dialog.

### UI specifications (reference for day 2)

**Fonts and icons:
**

- Body text:
   Inter
- Monospace (tracked time,
   code):
   JetBrains Mono
- Icons:
   Google Material Symbols (outline style,
   variable weight)
  - Hamburger menu:
     `menu`
  - Search:
     `search`
  - Suggested section:
     `auto_awesome` (sparkle)
  - All section:
     `all_inclusive` (infinity)
  - Location pin:
     `location_on`
  - Timer:
     `timer`
  - Tag hash:
     `tag`
  - FAB plus:
     `add`
  - Checkbox:
     `check_box_outline_blank` / `check_box`
  - Collapse toggle:
     `arrow_drop_down` / `arrow_drop_up`
  - Focus icon:
     `psychology`

**Inbox screen layout (from Figma):
**

- Top bar:
   hamburger (left),
   "Inbox" title (center),
   search icon (right)
- **Suggested** section (collapsible,
   default open,
   ephemeral state):
  - Section header:
     sparkle icon + "Suggested" + collapse triangle
  - **My location** subsection:
     "autodetect" toggle + location chips (e.g.,
     "Walmart" with pin icon)
    - Autodetect toggle:
       checkbox-style,
       tells AI to use current geolocation
    - Location chips:
       manually pinned places,
       used by AI to filter/rank suggestions
    - This controls what the suggestion engine sees,
       not task editing
  - **My focus** subsection:
     combobox with saved presets (e.g.,
     "Adulting tasks first")
    - Free-text input with dropdown of previously used directives
    - AI uses this to bias suggestion ranking
  - Task cards:
     AI-ranked list of suggested tasks
- **All** section (collapsible,
   default open,
   ephemeral state):
  - Section header:
     infinity icon + "All" + collapse triangle
  - All unblocked inbox tasks,
     chronological

**Task card layout:
**

- Checkbox (left) + title text (wraps to 2 lines max)
- Metadata chips row below title:
   `# tag`,
   `timer: Xs`,
   `where: Place`:
   horizontal scroll on overflow,
   no wrapping
- Tapping the card navigates to task details;
   tapping the checkbox completes the task

**FAB (floating action button):
**

- Bottom-right,
   black circle with white `+` icon
- Opens new task creation (inline or overlay;
   TBD based on other screen designs)

**Color palette:
**

- Monochrome:
   black text on white background
- Accent:
   muted red/brown for tags (`# shopping` text color)
- No shadows,
   no gradients,
   no border-radius on cards;
   deliberately bare-bones

## Day 2 (Wed): Core CRUD UI (~4h)

### 2.1 Layout shell and navigation (~2h) **done**

- (0.5h) Create shared HTML shell function (`src/server/page/layout.ts`):
   returns the `<!DOCTYPE html>` wrapper with nav drawer,
   styles,
   and script tag slot
- (0.5h) Build drawer navigation as part of the shell:
   Inbox,
   In Progress,
   Settings links.
   Hamburger toggle.
   Pure HTML + CSS (no JS needed for drawer;
   use `<details>` or checkbox hack).
- (0.5h) Create FAB function in `src/client/component/fab.ts`:
   creates a fixed-position button element,
   wires click to open new-task form
- (0.5h) Set up `src/client/styles.css` with `@import 'mixin.css'` and `@apply` rules.
   Create `src/client/mixin.css` with shared mixins.
   CSS is processed by build-css,
   then imported as text in client TS and injected at runtime (no `<link>` tag).
   Include Inter + JetBrains Mono fonts (via CDN),
   Material Symbols icon font,
   CSS variables for colors.

### 2.2 Inbox screen (~2.5h) **done**

- (0.5h) Create `src/server/page/inbox.ts`:
   queries unblocked inbox tasks,
   embeds as JSON,
   returns HTML
- (0.5h) Create `src/server/api/tasks.ts`:
   POST handler for creating tasks (validates with zod-mini)
- (0.5h) Create `<task-card>` custom element (`src/client/component/task-card.ts`):
   displays title,
   tags,
   tracked time,
   location;
   click navigates to `/tasks/:id`,
   checkbox click calls complete API
- (0.5h) Create `<collapsible-section>` custom element:
   header with icon + title + toggle,
   content slot
- (0.5h) Create `src/client/inbox.ts`:
   reads page data,
   builds Suggested + All sections using custom elements,
   wires FAB

### 2.3 Task details screen (~2.5h) **done**

- (0.5h) Create `src/server/page/task-details.ts`:
   queries single task + attachments,
   embeds as JSON,
   returns HTML
- (0.5h) Create `src/server/api/tasks.ts`:
   PUT handler for updating tasks,
   DELETE handler
- (0.5h) Create `<chip-editor>` custom element:
   tappable chip that expands to inline editor on click (text input or select),
   calls API on blur/enter
- (0.5h) Wire all chips:
   tags,
   location,
   priority,
   due date,
   complexity,
   reminders,
   blockedBy
- (0.5h) Create `src/client/task-details.ts`:
   reads page data,
   builds editable form with chips,
   save button calls PUT API

### 2.4 API wiring and verification (~1h) **done**

- (0.5h) Wire all API routes in `src/server/api/tasks.ts`:
   create,
   update,
   delete with zod-mini validation
- (0.5h) Verify round-trip:
   create task -> appears in inbox -> click to details -> edit -> save -> inbox reflects changes

## Day 3 (Thu): Overflow from days 1-2, then medium/low items (~4h)

### 3.1 Server-side timer logic (~2h) `priority:low` **done**

- (0.5h) Add API routes `POST /api/tasks/:id/start` and `POST /api/tasks/:id/stop` in `src/server/api/timer.ts`
- (0.5h) Add API route `POST /api/tasks/:id/complete`:
   validate no unresolved blockers,
   calculate final time,
   set status to `done`,
   then delete
- (0.5h) Test all three actions manually:
   start -> stop -> verify cumulative time;
   start -> complete -> verify blocker rejection
- (0.5h) Wire start/stop/complete buttons in task details client code

### 3.2 In Progress screen (~2.5h) `priority:low` **done**

- (0.5h) Create `src/server/page/in-progress.ts`:
   queries all tasks where `status = 'in_progress'`,
   embeds as JSON
- (0.5h) Create `src/client/in-progress.ts`:
   reads page data,
   builds task card list with live timer display
- (0.5h) Implement timer tick:
   single `setInterval(1000)` updates all visible timer displays
   by computing `trackedTime + (Date.now() - Date.parse(timerStartedAt)) / 1000`
- (0.5h) Format timer display as `HH:MM:SS`,
   handle edge cases (no timerStartedAt,
   timer just started)
- (0.5h) Verify:
   start timer on task -> navigate to In Progress -> see ticking timer -> stop -> time persists

### 3.3 Blocking UI (~2h) `priority:medium` **done**

- (0.5h) Update inbox page handler:
   fetch blocked tasks per visible parent (batched query),
   include in embedded data
- (0.5h) Update inbox client:
   render blocked tasks indented under their blockers with a "blocked" badge
- (0.5h) Add blocker picker to task details:
   `blockedBy` chip opens a searchable task list,
   saves selected IDs via API
- (0.5h) Disable complete button when blockers exist,
   show explanation with list of blocking tasks

### 3.4 Search screen (~1.5h) `priority:low` **done**

- (0.5h) Create `src/server/page/search.ts`:
   runs FTS5 query from `?q=` param,
   embeds results with `is_blocked` flag
- (0.5h) Create `src/client/search.ts`:
   search input with debounced navigation to `/search?q=...`,
   result list with blocked badges
- (0.5h) Test:
   create tasks with various tags -> search by tag -> verify results include blocked tasks with badge

## Day 4 (Fri): AI integration; autofill and suggestions (~4h)

AI is the core differentiator.
 Prompt engineering and structured output parsing are the focus.

### 4.1 llama.cpp client and rate limiting (~2h) **done**

- (0.5h) Create `src/lib/ai/client.ts`:
   HTTP client wrapper for llama.
  cpp's OpenAI-compatible chat completions endpoint (reads `CHAT_COMPLETIONS_URL` env var,
   e.g.,
   `http://llama-cpp:8080/v1/chat/completions`).
   Use `response_format: { type: "json_object" }` to force JSON output.
- (0.5h) Implement in-memory rate limiter:
   sliding window counter,
   max 30 requests/minute per process,
   returns 429 when exceeded
- (0.5h) Create `src/lib/ai/prompts.ts`:
   system prompt templates with clear separation of instructions vs user data (user content in `<user_tasks>` block)
- (0.5h) Test:
   verify client connects to llama.
  cpp,
   sends a simple prompt,
   parses response;
   verify rate limiter rejects burst requests

### 4.2 Autofill endpoint (~3h) **done**

- (0.5h) Create `src/server/api/ai-autofill.ts`:
   POST handler accepting `{ title: string }`
- (0.5h) Write autofill prompt:
   include title text,
   existing tags/locations from DB (for consistency),
   instructions to infer tags/location/priority/complexity
- (0.5h) Implement structured JSON output parsing:
   define expected schema with zod-mini,
   validate response,
   fallback to empty on parse failure
- (0.5h) Wire client-side:
   debounced fetch (500ms) on title input keyup,
   loading indicator,
   pre-fill chip values from response
- (0.5h) Handle edge cases:
   empty title,
   AI timeout,
   malformed response,
   all fields remain manually editable regardless
- (0.5h) Test end-to-end:
   type "Buy milk from Walmart" -> see `#shopping`,
   `where: Walmart`,
   `complexity: low` auto-filled

### 4.3 Task splitting (~1.5h)

- (0.5h) Extend autofill prompt:
   detect multi-location AND semantics,
   return `splitSuggestion` in response when applicable
- (0.5h) Build split confirmation UI:
   a `<dialog>` showing proposed task titles with confirm/cancel buttons
- (0.5h) Wire confirm action:
   POST to create N tasks with sequential `blockedBy` chains

### 4.4 Prompt injection hardening (~1.5h)

- (0.5h) Review all prompts:
   verify system prompt separated from user data,
   user content always in delimited blocks
- (0.5h) Verify structured JSON output schema validation rejects unexpected fields
- (0.5h) Verify server never passes AI output as raw HTML;
   all user-facing text is set via `textContent`,
   never `innerHTML`

## Day 5 (Mon): Suggestions, orchestrator, and deployment (~4h)

Two-day gap (Sat-Sun) before this session.
 Budget time for context recovery.

### 5.1 Suggestion engine; server side (~3h)

- (0.5h) Write suggestion prompt:
   include all unblocked inbox tasks (metadata only),
   user's current location,
   focus directive
- (0.5h) Implement structured output:
   AI returns ranked array of task IDs as JSON,
   validate against known task IDs
- (1.0h) Integrate into inbox page handler:
   call suggestion engine,
   sort "Suggested" section by AI ranking,
   embed ranked data in page JSON
- (0.5h) Handle AI unavailable:
   if llama.
  cpp is down or rate-limited,
   fall back to simple heuristic (due date,
   priority)
- (0.5h) Test:
   create 10 tasks with varied locations/priorities,
   set location to "Walmart",
   verify shopping tasks surface first

### 5.2 Suggestion engine; UI polish (~2.5h)

- (0.5h) Add loading state for suggestions section (skeleton cards while AI processes;
   or just show "Loading suggestions..." text)
- (0.5h) Add "Suggested" section header with explanation tooltip ("Based on your location and focus")
- (0.5h) Handle empty suggestions gracefully:
   show "No suggestions right now" message
- (0.5h) Tune suggestion count:
   limit to top 5-8 tasks to avoid overwhelming the user
- (0.5h) Test suggestion refresh:
   change location in settings -> reload inbox -> verify new suggestions

### 5.3 AI model tuning and prompt iteration (~2.5h)

- (1.0h) Test autofill with diverse task titles,
   iterate on prompt wording for accuracy
- (1.0h) Test suggestion ranking with various location/focus combos,
   iterate on ranking prompt
- (0.5h) Document final prompt templates in `src/lib/ai/prompts.ts` with rationale comments

### 7.1 Orchestrator script (~4h)

The orchestrator is a standalone Bun/TypeScript process that manages the entire multi-tenant lifecycle.
It lives at `package/site/done/orchestrator/` with its own entry point.
It handles auth,
 reverse proxy,
 and process management:
 no Caddy or AuthCrunch needed.
Coolify's reverse proxy terminates HTTPS upstream;
 the orchestrator listens on HTTP (port 3000).

#### 7.1a Registration and login (~1.5h)

- (0.5h) Create `orchestrator/src/auth.ts`:
   registration form (HTML page served by orchestrator),
   email verification via `@upyo/smtp`
- (0.5h) Implement login:
   verify password with `Bun.password.verify()`,
   create session row in `orchestrator.db` with random 32-byte hex ID,
   set cookie (`HttpOnly`,
   `SameSite=Strict`,
   `Secure`,
   `Path=/u/<user-id>/`),
   rate limit login attempts (10/min per IP)
- (0.5h) On verified registration:
   generate ULID user ID,
   hash password with `Bun.password.hash()` (argon2id),
   store in `orchestrator.db`,
   create `/data/<user-id>/` directory,
   initialize empty `done.db` with migration

#### 7.1b Process management (~1.5h)

- (0.5h) Create `orchestrator/src/process-manager.ts`:
   spawn per-user Bun process (`bun src/server.ts --port=XXXX --db=/data/<user-id>/done.db`).
   Each spawn runs `Bun.build()` at startup,
   so new processes always serve fresh client code.
- (0.5h) Implement port allocation:
   track PID + port + user-id mapping in `orchestrator.db`;
   allocate ports from a configurable range (e.g.,
   3100-3999)
- (0.5h) Implement process health check:
   periodic liveness probe,
   restart crashed processes,
   log failures

**7.1c Idle suspension and wake-on-request (~0.5h)** `priority:min`

- (0.25h) Monitor last-request timestamp per user (updated on each proxied request)
- (0.25h) Suspend idle processes after configurable timeout (kill + respawn on next request);
   orchestrator returns loading page briefly during cold start

#### 7.1d HTTP reverse proxy and path ACL (~0.5h)

- (0.25h) Create `orchestrator/src/proxy.ts`:
   on each request to `/u/<user-id>/*`,
   validate session cookie,
   check session user-id matches path user-id,
   proxy to `localhost:$PORT` with prefix stripped
- (0.25h) Handle edge cases:
   unauthenticated -> redirect to login,
   wrong user -> 403,
   user process not running -> spawn and queue request

### 7.4 Docker Compose for Coolify (~1h)

The entire stack deploys as a single `docker-compose.yml` at `package/site/done/docker-compose.yml`.
Coolify picks it up and manages the deployment.
Coolify's reverse proxy handles HTTPS termination:
 the orchestrator only listens on HTTP.

**Services:
**

<table>
<thead>
<tr>
<th>Service</th>
<th>Image</th>
<th>Purpose</th>
</tr>
</thead>
<tbody>
<tr>
<td>`orchestrator`</td>
<td>Custom (Bun + app code)</td>
<td>Auth, reverse proxy, process management</td>
</tr>
<tr>
<td>`llama-cpp`</td>
<td>`ghcr.io/ggml-org/llama.cpp:server` (CPU)</td>
<td>Shared AI inference, OpenAI-compatible API</td>
</tr>
</tbody>
</table>

The orchestrator container spawns per-user Bun processes as child processes within itself (not separate containers).
Each child process runs `Bun.build()` at startup to bundle client assets:
 no separate build step in the Dockerfile.
User data lives on a named volume mounted at `/data/` inside the orchestrator container.
The orchestrator keeps all routing state in memory (rebuilt from `orchestrator.db` on startup):
 no external config files to manage.

**Atomic tasks:
**

- (0.25h) Write `docker-compose.yml`:
   two services,
   inline Dockerfile for orchestrator,
   volumes (`done-data`)
- (0.25h) Configure environment variables:
   `SMTP_*`,
   `CHAT_COMPLETIONS_URL`,
   port range
- (0.25h) Test locally:
   `docker compose up`,
   register user,
   verify full flow (registration -> login -> process spawn -> proxy -> AI suggestion)

**Example `docker-compose.yml` with inline Dockerfile:
**

No framework build step.
 The orchestrator runs TS directly via Bun.
Child processes run `bun src/server.ts` which calls `Bun.build()` at startup for client assets.
The llama-cpp container auto-downloads the model on first start via `--hf-repo` + `--hf-file`.

```yaml
services:
  orchestrator:
    build:
      context: .
      dockerfile_inline: |
        FROM oven/bun:1
        WORKDIR /app
        COPY package.json bun.lock ./
        RUN bun install --frozen-lockfile
        COPY . .
        EXPOSE 3000
        ENV NODE_ENV=production
        CMD ["bun", "run", "orchestrator/src/index.ts"]
    ports:
    - '127.0.0.1:3000:3000'
    volumes:
    - done-data:/data
    environment:
    - SMTP_HOST
    - SMTP_PORT
    - SMTP_USER
    - SMTP_PASS
    - SMTP_FROM
    - CHAT_COMPLETIONS_URL=http://llama-cpp:8080/v1/chat/completions
    - PORT_RANGE_START=3100
    - PORT_RANGE_END=3999
    depends_on:
    - llama-cpp

  llama-cpp:
    image: ghcr.io/ggml-org/llama.cpp:server
    command:
    - '--host'
    - '0.0.0.0'
    - '--port'
    - '8080'
    - '--hf-repo'
    - 'Qwen/Qwen3-1.7B-GGUF'
    - '--hf-file'
    - 'Qwen3-1.7B-Q4_K_M.gguf'
    - '--ctx-size'
    - '4096'
    volumes:
    - llama-models:/root/.cache/llama.cpp

volumes:
  done-data:
  llama-models:
```

### 7.5 Deployment verification (~0.5h)

- (0.25h) Verify full stack via `docker compose up`:
   registration -> email verification -> login -> process spawn -> proxy -> inbox loads -> AI works
- (0.25h) Verify auth enforcement:
   unauthenticated request redirects to login,
   wrong user path returns 403

### 7.6 Testing (~0h, stretch goal)

- Unit tests for critical database functions (module-test):
   CRUD,
   timer,
   blocking validation
- Basic Playwright smoke test:
   create task -> appears in inbox -> start timer -> appears in In Progress

### 7.7 Documentation (~0h, only if time permits)

- Brief README with:
   what it is,
   how to deploy with Coolify,
   env var reference

## Deferred items (build when time allows, ordered by priority)

### 6.1 PWA configuration (~1.5h) `priority:low`

- (0.25h) Create `manifest.json`:
   app name,
   icons,
   theme color,
   `display: standalone`
- (0.5h) Create service worker:
   cache static assets (JS,
   CSS,
   icons) for faster repeat loads
- (0.25h) Add offline detection:
   show "offline" banner when connectivity is lost (no full offline mode;
   all actions require server)
- (0.5h) Test:
   install as PWA on phone/desktop,
   verify cached shell loads fast,
   verify offline banner appears when disconnected

### 6.2 Geolocation (~1h) `priority:low`

- (0.25h) Add Browser Geolocation API call to detect current position
- (0.5h) Reverse geocode coordinates to place name (via llama.
  cpp or a free geocoding API)
- (0.25h) Store detected location in settings,
   send with suggestion requests

### 6.3 Camera and attachments (~1h) `priority:low`

- (0.25h) Add `<input type="file" accept="image/*" capture="environment">` to task details
- (0.5h) Wire API route `POST /api/tasks/:id/attach`:
   validate file type/size,
   store as BLOB in attachments table
- (0.25h) Wire API route `GET /api/attachments/:id`:
   download attachment by ID

### 6.4 Email; reminder notifications (~2h) `priority:medium`

- (0.5h) Create `src/lib/email/transport.ts`:
   `@upyo/smtp` wrapper configured from SMTP env vars
- (0.5h) Create `src/lib/email/reminders.ts`:
   query due reminders,
   format email body (task title,
   description,
   link)
- (0.5h) Wire `setTimeout` loop in server startup:
   check every 60 seconds,
   send due reminders,
   remove fired reminders from JSON array
- (0.5h) Test:
   create task with reminder 1 minute from now -> verify email arrives -> verify reminder removed from task

### 6.5 Email; daily database backup (~1h) `priority:medium`

- (0.25h) Create `src/lib/email/backup.ts`:
   export tasks + settings + attachment metadata as JSON (no BLOBs)
- (0.5h) Wire scheduled job:
   run once daily at 3am UTC,
   send JSON attachment via `@upyo/smtp`
- (0.25h) Test:
   trigger backup manually -> verify email arrives with valid JSON attachment

### 6.6 Settings screen (~1.5h) `priority:low` **partial** (UI skeleton only; no API route, no DB persistence)

- (0.5h) Create `src/server/page/settings.ts`:
   queries all settings,
   embeds as JSON
- (0.5h) Create `src/client/settings.ts`:
   builds settings form (AI model,
   email,
   connected apps,
   location),
   wire save to `PUT /api/settings`
- (0.5h) Verify round-trip:
   change setting -> save -> reload -> persisted

### 7.2 GitHub integration (~1.5h) `priority:low`

- (0.25h) Create `src/lib/sync/github.ts`:
   GitHub API client using personal access token from settings
- (0.5h) Implement import:
   fetch issues from configured repos,
   map to tasks (title,
   body->description,
   labels->tags),
   store `source_meta` for lossless round-trip
- (0.25h) Implement issue dependency detection:
   parse "depends on #X" in issue body,
   map to `blockedBy`
- (0.25h) Implement write-back:
   completing a task closes the GitHub issue (PATCH issue status)
- (0.25h) Wire into settings:
   configure repos,
   trigger sync,
   show sync status

### 7.3 Codebase TODO sync (~0.5h) `priority:low`

Read-only inbound for MVP.
 Outbound writes deferred post-competition.

- (0.25h) Create `src/lib/sync/codebase.ts`:
   regex scanner for `TODO`,
   `FIXME`,
   `HACK` comments in configured directories;
   extract file path + line number + context;
   create tasks with `source: "codebase"`,
   `sourceId: "repo:file:line"`
- (0.25h) Wire into settings:
   configure repo paths,
   trigger scan,
   show scan results

## Validated by bun-test (package/site/bun-test)

A throwaway flashcard app was built pre-competition to validate the architecture.
These patterns are confirmed working:
 no surprises expected during implementation.

<table>
<thead>
<tr>
<th>Pattern</th>
<th>Status</th>
<th>Notes</th>
</tr>
</thead>
<tbody>
<tr>
<td>build-css -> Bun.build() -> Bun.serve() pipeline</td>
<td>**validated**</td>
<td>Runs at startup, restarts cleanly with `bun --watch`</td>
</tr>
<tr>
<td>Bun.serve() `routes` with `:param` and per-method dispatch</td>
<td>**validated**</td>
<td>Type-safe params, SIMD-accelerated, replaces hand-written router</td>
</tr>
<tr>
<td>Embedded JSON (`<script type="application/json">`)</td>
<td>**validated**</td>
<td>Client reads with `JSON.parse(el.textContent)`</td>
</tr>
<tr>
<td>CSS-as-text-import (Bun.build() inlines CSS strings)</td>
<td>**validated**</td>
<td>`import styles from "..." with { type: "text" }` works</td>
</tr>
<tr>
<td>@mixin/@apply via @monochromatic-dev/build-css</td>
<td>**validated**</td>
<td>LightningCSS + PostCSS expansion, oxc-resolver works under Bun</td>
</tr>
<tr>
<td>Custom elements</td>
<td>**validated**</td>
<td>`<flash-card>` with shadow DOM, attributes, events</td>
</tr>
<tr>
<td>zod validation on API routes</td>
<td>**validated**</td>
<td>Schema validation with error messages</td>
</tr>
<tr>
<td>fetch() mutations + globalThis.location.reload()</td>
<td>**validated**</td>
<td>Simple mutation pattern, no client-side state sync</td>
</tr>
<tr>
<td>bun:sqlite in-memory DB</td>
<td>**validated**</td>
<td>CRUD, foreign keys, WAL mode</td>
</tr>
<tr>
<td>Bun.file() auto Content-Type</td>
<td>**validated**</td>
<td>No manual content-type mapping needed</td>
</tr>
<tr>
<td>Static imports for DB + route handlers</td>
<td>**validated**</td>
<td>Dynamic imports unnecessary: top-level await ensures build completes first</td>
</tr>
</tbody>
</table>

### Not yet tested (verify early in implementation)

<table>
<thead>
<tr>
<th>Pattern</th>
<th>Risk</th>
<th>Mitigation</th>
</tr>
</thead>
<tbody>
<tr>
<td>FTS5 full-text search</td>
<td>low</td>
<td>Standard SQLite extension, well-documented</td>
</tr>
<tr>
<td>llama.cpp HTTP client + structured JSON output</td>
<td>medium</td>
<td>Test on day 4 with real model before wiring UI</td>
</tr>
<tr>
<td>File/BLOB attachments in SQLite</td>
<td>low</td>
<td>Standard bun:sqlite, deferred feature anyway</td>
</tr>
<tr>
<td>5 client entrypoints (vs 2 tested)</td>
<td>low</td>
<td>Bun.build() is fast, linear scaling expected</td>
</tr>
<tr>
<td>setInterval timer tick on client</td>
<td>low</td>
<td>Standard browser API, trivial to test</td>
</tr>
<tr>
<td>@upyo/smtp email sending</td>
<td>medium</td>
<td>External dependency, test with real SMTP early</td>
</tr>
<tr>
<td>Orchestrator multi-process spawning</td>
<td>high</td>
<td>Most complex untested piece: budget extra time on day 5</td>
</tr>
</tbody>
</table>

## Risk areas

### AI autofill latency (typing feels sluggish)

- Debounce 500ms,
   show loading indicator,
   make all fields manually editable
- llama.
  cpp latency depends on hardware;
   test early,
   adjust model size if needed

### GitHub sync complexity (two-way is hard)

- Start with one-way import,
   add write-back as stretch

### Timer accuracy across timezones

- Store all timestamps as UTC,
   compute display client-side

### No full offline mode

- App requires connectivity for all actions;
   show "offline" banner when disconnected
- Avoids the massive complexity of local write queue + sync-on-reconnect

### llama.cpp resource contention

- Shared instance serving multiple users could bottleneck on concurrent requests
- Rate limiting per process helps;
   queue depth monitoring would be ideal
- For competition scale (few users),
   this is fine

### BLOB attachments bloat the database

- A few photos can make the `.db` file large
- Daily backup email excludes BLOBs intentionally
- For competition MVP this is acceptable;
   long-term,
   move attachments to filesystem

### Orchestrator complexity underestimated

- Process management,
   auth,
   reverse proxy,
   and registration flow are non-trivial;
   but simpler than configuring Caddy + AuthCrunch
- Cold-start wake-on-request adds latency;
   users may see a brief loading page
- Mitigation:
   keep the orchestrator minimal for MVP (no graceful suspension,
   just kill + respawn),
   test registration flow early on day 5

### Blocked-tasks N+1 query

- Fetching dependents per visible task is N+1;
   batch into one query:

```sql
-- Fetch ALL blocked inbox tasks and their blocker IDs in one query
-- Group by blocker on the client side
SELECT tasks.*, blocker.value AS blocker_id
FROM tasks, json_each(tasks.blocked_by) AS blocker
WHERE tasks.status = 'inbox'
  AND tasks.blocked_by != '[]'
```

### Bun.build() startup time

- Validated in bun-test:
   2 entrypoints build in <100ms. 5 should be similar.
- For competition,
   the simple always-rebuild approach is confirmed fine.

## Priority summary

Items without a marker are implicitly highest priority and form the 20h core plan.
Marked items are built if time allows,
 in descending priority order.

**Core (done):
** 1.1,
 1.2,
 2.1,
 2.2,
 2.3,
 2.4,
 4.1,
 4.2
**Core (partial):
** 1.3 (tasks + settings done;
 attachments/reminders not started)
**Core (not started):
** 4.3,
 4.4,
 5.1,
 5.2,
 5.3,
 7.1a,
 7.1b,
 7.1d,
 7.4,
 7.5
Note:
 Dockerfile simplified (no framework build step):
 single stage,
 Bun runs TS directly.

**`priority:medium`:
 done:
** 3.3 (blocking UI)
**`priority:medium`:
 not started:
** 6.4 (email reminders),
 6.5 (daily backup)

**`priority:low`:
 done:
** 3.1 (timer logic),
 3.2 (in-progress screen),
 3.4 (search)
**`priority:low`:
 partial:
** 6.6 (settings screen;
 UI skeleton only)
**`priority:low`:
 not started:
** 6.1 (PWA),
 6.2 (geolocation),
 6.3 (camera),
 7.2 (GitHub sync),
 7.3 (codebase TODO sync)

**`priority:min`:
** 7.1c (idle suspension)

**Post-competition:
** Codebase TODO outbound writes,
 Linear sync,
 Calendar sync,
 full offline mode,
 comprehensive test suite,
 evaluate Vue Vapor rewrite once 3.6 stable
