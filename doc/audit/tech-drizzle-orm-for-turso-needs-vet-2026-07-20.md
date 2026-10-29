# Drizzle ORM for all Turso needs: vet report

> Moved 2026-10-29 by the generic package category elimination
> (`doc/planning/generic-category-elimination.md`);
> paths and names below this note are historical:
> `package/webapp-productivity/done` is now `package/done/app`.

## Report metadata

- Status:
   in progress;
   lifecycle phase `hard-gate confirmed` (serious alternative),
   finalist validation pending.
- Subject:
   `drizzle-orm-for-turso-needs`.
- Scope:
   replace all repo Turso database access,
   active embedded `@tursodatabase/database` packages and paused libSQL/Kysely usage,
   with Drizzle ORM.
- Started:
   2026-07-20.
- Last updated:
   2026-07-20.
- Governing skill commit:
   `a05818ad70a40e5769a36de669697ba109891b31` (`.agents/skills/choosing-technology/SKILL.md`).
- Governing skill SHA-256:
   `393eb68c5b2b2f7b16c8f7f90c100fb8be43eefa4501511360cd0572e4ae8087`.
- Compatibility fingerprint:
   `a4e4b942e5f3255bcd274c2ca409d89099792ebba35adf666050e61d183c0864`.
- Audit owner:
   Claude Code session `f1c75178` for Aquaticat.
- Prior related report:
   `doc/audit/tech-prisma-orm-for-turso-needs-vet-2026-07-20.md`
   (same decision context,
   different candidate;
   Prisma exited at screening).

## Hard constraints, base components, overlays

Identical decision context to the Prisma report:

1. Drive the embedded `@tursodatabase/database` engine used by the active packages
   (`package/webapp-productivity/done`,
   `done-postcss`).
2. Preserve the native search surface
   (Turso's Tantivy-backed experimental index method:
   `CREATE INDEX ... USING fts`,
   `fts_match()`,
   `fts_score()`,
   behind `experimental: ['index_method']`).
   This is not SQLite FTS5;
   the engine ships no FTS5 at all
   (`doc/troubleshooting/turso-fts5-native-fts.md`).

Base category:
inspectable open-source local technology.
Overlays:
incumbent replacement;
native-prebuilt boundary (the engine's napi prebuilts,
already the incumbent's boundary).

## Candidate ledger

### Drizzle ORM (named by user)

- Discovery source:
   named by the user.
- Screening:
   passed;
   no screening hard gate failed.
- Targeted evidence:
   passed both hard constraints through a consumer-boundary probe (below).
- Current state:
   serious alternative,
   hard-gate confirmed;
   finalist validation not yet performed.

### Incumbent retention: `@tursodatabase/database` direct SQL

- Replacement parity overlay keeps the incumbent as a candidate.
- State:
   default survivor;
   full parity comparison deferred to finalist validation.

## Evidence records

All evidence accessed 2026-07-20.

### Clone and integration-surface inventory

- Clone:
   `gh repo clone drizzle-team/drizzle-orm ~/temp/agent/drizzle-orm-20260720 -- --depth 1`;
   HEAD `9d64532` (2026-07-10,
   `main`).
- No first-party driver for the embedded Turso engine:
   `rg --ignore-case 'tursodatabase'` over the clone matches only two changelog files,
   and both matches are `github.com/tursodatabase/libsql` documentation URLs,
   not an integration.
   `drizzle-orm/package.json` `peerDependencies` lists `better-sqlite3`,
   `@libsql/client`,
   and other clients,
   but not `@tursodatabase/database`.
- SQLite-family driver roster (`drizzle-orm/src/`):
   `better-sqlite3`,
   `bun-sql`,
   `bun-sqlite`,
   `d1`,
   `durable-sqlite`,
   `expo-sqlite`,
   `libsql`,
   `op-sqlite`,
   `sqlite-proxy`.
   Two roster entries admit the embedded Turso engine without any new driver:
   `better-sqlite3` (because the engine ships a better-sqlite3-compatible sync API)
   and `sqlite-proxy` (bring-your-own-executor callback,
   `drizzle-orm/src/sqlite-proxy/driver.ts:30-34`).
- Drizzle's better-sqlite3 session calls exactly
   `prepare`,
   `transaction`,
   `run`,
   `all`,
   `get`,
   `raw()`,
   and `iterate` on the client
   (`drizzle-orm/src/better-sqlite3/session.ts`),
   all of which `@tursodatabase/database/compat` implements
   (`@tursodatabase/database-common/dist/compat.d.ts`),
   and the compat constructor accepts `DatabaseOpts.experimental`
   (`@tursodatabase/database-common/dist/types.d.ts`).

### Consumer-boundary probes (throwaway fixture)

Execution manifest:

- Fixture:
   session scratchpad `drizzle-turso-probe/` (disposable,
   outside the repo).
- Install:
   `npm install --ignore-scripts --no-audit --no-fund drizzle-orm @tursodatabase/database`;
   resolved `drizzle-orm@0.45.2` (pure JS,
   no runtime deps) and
   `@tursodatabase/database@0.7.0` (napi prebuilts via optional dependencies;
   same artifact family the repo already runs;
   the repo's own `node_modules` also holds 0.7.0).
- A resolution-only stub replaced `better-sqlite3`
   (Drizzle's driver module statically imports it even when the client is injected;
   the stub constructor throws if ever constructed;
   it never was).
- Network only during the npm fetch phase;
   probes ran on `:memory:` databases;
   no credentials;
   wall clock under one minute per run.

Route A,
`drizzle-orm/better-sqlite3` over `@tursodatabase/database/compat` (`probe.mjs`):

- Pass:
   insert,
   select,
   `fts_match` predicate with bound parameter through the query builder,
   `fts_score` ordering with deterministic tiebreaker,
   native-index auto-maintenance across Drizzle `update` and `delete`,
   raw-mode `values()`,
   `get()`,
   and `transaction()`.
   Probe printed `PROBE OK`.
- Fail:
   process teardown aborted (SIGABRT,
   exit 134) with the engine panic
   `FTS Drop: transaction already committed, cannot flush | pending_docs_count=1`
   (`core/index_method/fts.rs:2628`).
- Bisection (probe scripts `control.mjs`,
   `bisect.mjs`,
   `refine.mjs`):
   the trigger is any successful write to a native-FTS-indexed table
   inside an explicit transaction through the **compat** API,
   whether via the `transaction()` wrapper or raw `BEGIN`/`COMMIT`;
   explicit `close()` does not prevent it.
   Non-transactional writes are clean.
   The identical transactional sequence through the **promise** API exits cleanly,
   with and without `close()`.
   Full matrix and upstream cluster:
   `doc/troubleshooting/turso-fts5-native-fts.md`,
   section on the 0.7.0 compat teardown abort.

Route B,
`drizzle-orm/sqlite-proxy` over the promise API (`proxy-probe.mjs`):

- The executor callback is ~15 lines:
   `run` maps to `stmt.run(...params)`,
   `get` to `stmt.raw().get(...params)`,
   `all`/`values` to `stmt.raw().all(...params)`.
- Pass,
   exit 0,
   no teardown panic:
   the whole Route A functional list,
   plus `db.transaction()`.

### Hard-gate outcomes

1. Drive the embedded engine:
   **pass**,
   two routes,
   both exercised at the consumer boundary.
2. Preserve the native search surface:
   **pass**;
   `fts_match`/`fts_score` compose naturally inside Drizzle's `sql` template
   with column references and bound parameters,
   and the native index self-maintains across Drizzle-issued DML.
   The `USING fts` DDL itself stays raw SQL:
   Drizzle's schema DSL cannot express it,
   but the active packages already hand-roll migrations
   (`package/webapp-productivity/done`,
    `src/lib/db-migrations.ts`),
   so no drizzle-kit dependency is required or proposed.

### Route ranking

Route B (sqlite-proxy over promise API) over Route A (better-sqlite3 driver over compat):

- Route B rides the same promise API the active packages already use,
   inherits none of the compat teardown abort,
   and keeps transaction support.
- Route A is more type-idiomatic (no callback shim)
   but is disqualified for transactional workloads
   until the upstream compat bug is fixed,
   and an ORM adoption without transactions forfeits a main benefit.

## Gates not yet run, with reasons

This report is not a recommendation to adopt.
Remaining before any adoption-grade recommendation:

- Drizzle source,
   dependency,
   test,
   CI,
   and fuzzing audits (skill open-source gates);
   only the integration-relevant source paths were read so far.
- Maintenance audit with the skill's issue/PR sampling method.
- Finalist validation:
   running Drizzle's own suite in an isolated environment,
   and equal-depth parity audit against keeping the incumbent direct-SQL layer.
- Replacement parity inventory:
   enumerate every query in `done`/`done-postcss`
   and confirm each is expressible (or acceptably raw) under Drizzle.
- Scoring,
   sensitivity,
   and the incumbent-vs-Drizzle soft-criteria comparison.

## Interim conclusion

Drizzle is a genuine serious alternative where Prisma was a screening exit:
both hard gates pass with primary evidence.
The credible adoption shape is
query-builder-plus-`sql`-template over the existing promise connection via `sqlite-proxy`,
keeping hand-rolled migrations.
No adoption or product change is authorized or performed by this evaluation.

## Evidence limits

- Probes ran one engine version (0.7.0),
   one platform (Linux x64),
   `:memory:` databases only.
- drizzle-kit was not evaluated (out of the proposed adoption shape).
- The teardown abort is an incumbent-engine bug surfaced by the probe,
   not a Drizzle defect;
   it equally constrains any better-sqlite3-compat consumer of the engine.
- GitHub code and issue searches are as of 2026-07-20.
