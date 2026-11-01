# `@tursodatabase/database` 0.6.1 ships no SQLite FTS5; `USING fts5` throws `no such module: fts5`

Turso is a Rust rewrite of SQLite,
 not a build of SQLite with extensions.
Its `@tursodatabase/database` 0.6.1 npm prebuilt contains **no** FTS5 module,
so any `CREATE VIRTUAL TABLE ... USING fts5(...)` fails at parse time.
Full-text search is instead offered as an experimental *index method*
(`CREATE INDEX ... USING fts`) queried with `fts_match` / `fts_score` / `fts_highlight`.

## Symptom

A migration that creates an FTS5 virtual table throws:

```txt
Error: failed to consume stmt: Parse error: no such module: fts5
```

In `package/done/app` the migration runs as an import side effect
(`src/server.ts` side-effect-imports `./lib/db.ts`,
 whose module body calls
`runMigrations` via top-level `await`),
 so the failure crashes startup before any
route is served rather than surfacing on first search.

The same statement family that fails:

```sql
CREATE VIRTUAL TABLE IF NOT EXISTS tasks_fts USING fts5(title, description, tags, content=tasks, content_rowid=rowid);
```

## Root cause

The loaded native binary implements Turso's own full-text engine and does not
register an `fts5` virtual-table module,
 so SQLite's module lookup for `fts5` fails.

Binary evidence (linux-x64-gnu prebuilt,
`node_modules/.pnpm/@tursodatabase+database-linux-x64-gnu@0.6.1/node_modules/@tursodatabase/database-linux-x64-gnu/turso.linux-x64-gnu.node`):

```txt
strings turso.linux-x64-gnu.node | grep -ic fts5          -> 0
strings turso.linux-x64-gnu.node | grep -c index_method::fts -> 612
```

Zero `fts5` tokens;
 hundreds of `turso_core::index_method::fts::*` symbols (a
Tantivy-backed index method),
 so FTS exists but only under Turso's own API.

The Cargo-level analysis in `doc/audit/turso-cargo-toml-0.6.1.md` corroborates this:
the default feature set enables an `fts` feature that forwards to `turso_core/fts`
(Tantivy) and defines the `fts` index method plus `fts_highlight` and friends,
 and
the SQL translator rejects `MATCH` when FTS is not enabled.
 None of that is SQLite FTS5.

The intended API is shown by Turso's own bundled test,
`node_modules/.pnpm/@tursodatabase+database@0.6.1/node_modules/@tursodatabase/database/dist/promise.test.js:187-212`:

```js
const db = await connect(":memory:", { experimental: ["index_method"] });
await db.exec(`
    CREATE TABLE documents (id INTEGER PRIMARY KEY, title TEXT, body TEXT);
    CREATE INDEX documents_fts ON documents USING fts (title, body);
`);
// search:  WHERE fts_match(title, body, 'programming language')
// score:   fts_score(title, body, 'programming language') as score
// excerpt: fts_highlight(title, '<b>', '</b>', 'Rust')
```

Key differences from FTS5 that follow from this:

- The index attaches to the base table (`ON tasks USING fts (...)`),
   so it is
  auto-maintained on insert/update/delete and populated over existing rows at
  creation time.
   The FTS5 external-content pattern's sync triggers and backfill
  are unnecessary and,
   being written against `tasks_fts` as a virtual table,
   would
  never run here anyway.
- The connection must be opened with `experimental: ['index_method']`.
  `experimental: ['triggers']` (what the FTS5 triggers needed) does not enable it.
- Matching is token-exact with no stemming:
   querying `grocery` does not match a
  row containing only `groceries`.
- `fts_score` is unreliable on this build:
   it returns a real BM25-style value in the
  simplest score-only shape but collapses to `0`,
   or to identical values across rows,
  in other shapes (multiple matches,
   non-covered or joined score projections).
   So
  ordering cannot depend on it and must carry a deterministic tiebreaker.
   This is the
  open upstream cluster #7636 / #7637 / #7532 / #7524.

## Verification

Version under test:
 `@tursodatabase/database@0.6.1`,
`sha512-qg6LO1XHA9kc2s97crg0/nHiKFk3xTzylRuZPYggO0vA4hx2rp8fk0Je2vFGOkfVv+ZmzsXOIb+DBHmdrG+72A==`.

Minimal harness (run from a directory that resolves the package,
 e.g. inside
`package/done/app`):

```js
// probe.mjs
import { connect } from '@tursodatabase/database';

// 1. FTS5 is absent.
{
  const db = await connect(':memory:', { experimental: ['index_method'] });
  await db.exec('CREATE TABLE t (a TEXT, b TEXT)');
  try {
    await db.exec('CREATE VIRTUAL TABLE t_fts USING fts5(a, b)');
    console.log('fts5: unexpectedly succeeded');
  } catch (e) {
    console.log('fts5:', String(e).split('\n')[0]);
  }
}

// 2. Native USING fts: bound ?1 match + auto-maintenance.
{
  const db = await connect(':memory:', { experimental: ['index_method'] });
  await db.exec(`
    CREATE TABLE t (id TEXT PRIMARY KEY, a TEXT, b TEXT, updated_at TEXT);
    INSERT INTO t VALUES ('x', 'buy groceries', 'milk eggs', '2023-01-01');
    CREATE INDEX t_fts ON t USING fts (a, b);
  `);
  const ids = async (term) =>
    (await (await db.prepare('SELECT id FROM t WHERE fts_match(a, b, ?1)')).all(term)).map((r) => r.id);
  console.log('match "groceries":', JSON.stringify(await ids('groceries')));
  console.log('match "eggs":     ', JSON.stringify(await ids('eggs')));
  console.log('match "grocery":  ', JSON.stringify(await ids('grocery')), '(token-exact, no stem)');
  await db.exec("UPDATE t SET a = 'buy produce' WHERE id = 'x'");
  console.log('after update, match "groceries":', JSON.stringify(await ids('groceries')));
}
```

Observed output:

```txt
fts5: Error: failed to consume stmt: Parse error: no such module: fts5
match "groceries": ["x"]
match "eggs":      ["x"]
match "grocery":   [] (token-exact, no stem)
after update, match "groceries": []
```

Works cleanly:

- `CREATE INDEX name ON table USING fts (col, ...)`.
- `fts_match(col, ..., ?1)` accepts a bound parameter;
   `?1` may be reused across
  `fts_match` and `fts_score` to bind the query once.
- Auto-maintenance:
   rows matched after `UPDATE`/`DELETE` reflect the change with no
  triggers;
   an index created after inserts still matches the pre-existing rows.

Fails:

- `CREATE VIRTUAL TABLE ... USING fts5(...)` -> `no such module: fts5`.
- Token-exact matching only:
   `fts_match(a, b, 'grocery')` returns nothing for a row
  containing only `groceries`.
- `fts_score` is inconsistent:
   a lone match returns a real value (~0.29 in this
  harness),
   but across multiple matches or non-covered/joined projections it collapses
  to `0` or to identical values,
   so `ORDER BY fts_score(...) DESC` alone does not
  reliably rank.

## Verified workarounds

The remediation landed in `package/done/app` and `done-postcss`.

1. Replace the FTS5 virtual table plus sync triggers plus backfill with one native
   index,
    and open the connection with `experimental: ['index_method']`:

   ```sql
   CREATE INDEX IF NOT EXISTS tasks_fts ON tasks USING fts (title, description, tags);
   ```

   Tradeoff:
    search becomes token-exact (no substring or stemming),
    a behavior
   change from a `LIKE '%q%'` scan.
    Acceptable here because the FTS5 path never
   actually ran on this build.

2. Query with `fts_match` for the predicate and `fts_score` for ranking,
    reusing
   `?1`,
    with `updated_at` as a deterministic tiebreaker:

   ```sql
   SELECT tasks.*, CASE WHEN blocked_by != '[]' THEN 1 ELSE 0 END AS is_blocked
   FROM tasks
   WHERE fts_match(title, description, tags, ?1)
   ORDER BY fts_score(title, description, tags, ?1) DESC, tasks.updated_at DESC;
   ```

   Tradeoff:
    because `fts_score` is unreliable today (frequently `0` or tied across
   multiple matches),
    ordering is effectively recency-only;
    genuine relevance ordering
   activates for free once upstream scoring is fixed.

3. Guard index creation so a build lacking the index method degrades instead of
   crashing (`src/lib/db-migrations.ts`,
    `tryEnableFts`):
    wrap the `CREATE INDEX` in
   try/catch,
    log a warning,
    and skip it,
    letting `searchTasks` fall back to its
   existing `LIKE` query.

   Tradeoff:
    on a build without `index_method`,
    search silently loses relevance and
   uses substring matching;
    the warning is the only signal.

## What does not work

- **Selecting a different 0.6.1 prebuild.**
   All 0.6.1 platform prebuilts are the same
  Rust engine;
   none ship SQLite FTS5.
   "Pin a build with FTS5" is not achievable for
  Turso without switching the database engine (libsql,
   better-sqlite3,
   node:sqlite).
- **Keeping the FTS5 triggers.**
   They insert into `tasks_fts` as if it were an FTS5
  vtable;
   the native index needs neither triggers nor a backfill statement.
- **Ordering purely by `fts_score`.**
   It is inconsistent on this build (0 or tied
  values across multiple matches and non-covered projections,
   upstream #7636/#7637/
  #7532/#7524),
   so order is unstable without a secondary key.
- **Substring or prefix search via `fts_match`.**
   Matching is token-exact;
   a partial
  word returns nothing (no error,
   so the `searchTasks` try/catch does not fall back).

## Upstream filing decision

Default policy is do not file.
 `.out-of-scope/` has no turso or FTS exemption
(checked;
 none matched).
 Duplicate search on `tursodatabase/turso` (open and closed):

- FTS5 absence is tracked and by-design:
   **#346 "FTS5 support"** (open,
   `help wanted`)
  and **#997 "Full text search extension"** (closed).
   The `no such module: fts5`
  behavior is the expected consequence,
   not a defect.
- `fts_score` returning `0` is already reported:
   **#7636 "using `fts_score` with
  `fts_match` makes all `fts_score` values be 0"**,
   **#7637 "`fts_score()` silently
  returns 0 outside the score-only / exact-columns pattern"**,
   and **#7532
  "fts_score returns 0.0 when projected through joins or non-covered score
  expressions"**.
   Ordering quirks:
   **#7524 "FTS ORDER BY score ASC returns descending
  score order"**.
   Ranking as a feature:
   **#5081 "Support BM25 full text search"**.

6-constraint check:

1. **Upstream's fault?**
    The FTS5 gap is a deliberate architecture choice,
    not a bug.
   The `fts_score` = 0 behavior is a real gap but already filed.
2. **Can upstream fix it?**
    Adding FTS5 is a large feature (#346);
    `fts_score` scoring
   is engine work.
    Both fixable in principle.
3. **Supporting this use case?**
    Native FTS is supported and tested;
    FTS5 explicitly is
   not yet (#346 `help wanted`).
4. **Would the repo welcome a contribution?**
    Not assessed;
    moot given constraint 6.
5. **Will they likely fix it?**
    #346 is open with `help wanted`;
    the `fts_score` issues
   are open and recent.
    No won't-fix signal.
6. **Prototyped a minimal fix?**
    No. Implementing FTS5 or Turso's scoring is deep
   engine work,
    out of scope for this consumer-side task,
    and the relevant behaviors
   are already tracked.

Decision:
 **do not file,
 nothing to add.**
 Every finding here (FTS5 absence,
 native
`USING fts` API,
 `fts_score` = 0,
 score-ordering) is already represented by the issues
above;
 a new issue or a "me too" comment would be a duplicate.
 The durable artifact is
this doc plus the consumer-side workaround already landed.

## 0.7.0: compat API transactional writes to a native-FTS table abort at process teardown

Found 2026-07-20 while probing Drizzle ORM over `@tursodatabase/database@0.7.0`
(vet report:
 `doc/audit/tech-drizzle-orm-for-turso-needs-vet-2026-07-20.md`).
This is an engine bug,
 not a Drizzle bug;
 any better-sqlite3-compat consumer hits it.

### Symptom

A process that successfully commits an explicit transaction containing a write to a
table covered by a native `USING fts` index,
 through the **compat** API
(`@tursodatabase/database/compat`),
 aborts at teardown with SIGABRT (exit 134):

```txt
thread '<unnamed>' panicked at core/index_method/fts.rs:2628:9:
FTS Drop: transaction already committed, cannot flush | pending_docs_count=1
```

All application work completes first;
 the abort fires when the engine drops during
process exit,
 so tests pass their assertions and then the process dies.

### Repro matrix (0.7.0, Linux x64, `:memory:`, `experimental: ['index_method']`)

Every case creates a table,
 a `USING fts` index over it,
 and then:

- compat,
   `transaction()` wrapper insert:
   **aborts** at exit.
- compat,
   raw `BEGIN` + insert + `COMMIT`:
   **aborts** at exit.
- compat,
   transactional insert then explicit `close()`:
   **still aborts**.
- compat,
   non-transactional insert/update/delete:
   clean exit.
- promise API,
   raw `BEGIN` + insert + `COMMIT`,
   with or without `close()`:
   clean exit.
- promise API,
   non-transactional writes:
   clean exit.
- `fts_match`/`fts_score` reads:
   never the trigger on either API.

So the pending-document flush is mishandled only when the committing context is the
compat layer's explicit transaction;
 the promise API path flushes correctly.

### Exposure in this repo

Not exposed today:
 `package/done/app` and `done-postcss` use the
promise API and issue no transactions (`rg --ignore-case 'transaction|BEGIN'` over
`package/done/app/src` matches nothing).
 The hazard binds any future
compat-API adoption,
 notably Drizzle's `better-sqlite3` driver route.

### Workaround

Use the promise API for anything transactional near a native-FTS index.
 For Drizzle,
that means the `sqlite-proxy` driver over a promise connection (verified clean,
including `db.transaction()`),
 not the `better-sqlite3` driver over compat.

### Upstream filing decision

Open adjacent cluster on `tursodatabase/turso` as of 2026-07-20:
 #7259 (Tantivy FTS
panic on cursor Drop after trigger INSERT commits transaction),
 #7520 (FTS cursor Drop
panics after failed multi-row insert leaves pending docs),
 #7530 (FK cascades bypass
FTS maintenance),
 #5027 (fts index rollback on failed insert).
 Same panic family and
the `pending docs` language matches #7520,
 but our variant differs on two axes that
none of the open issues state:
 the insert **succeeds** and the asymmetry is
**compat-only** (promise API is clean).
 That asymmetry is diagnostic signal,
 so filing
plausibly adds value rather than duplicating.
 Filing is an external action and was not
performed by the evaluation session;
 if authorized,
 attach the repro matrix above.
