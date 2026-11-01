# Prisma ORM for all Turso needs: vet report

> Moved 2026-10-29 by the generic package category elimination
> (`doc/planning/generic-category-elimination.md`);
> paths and names below this note are historical:
> `package/webapp-productivity/done-postcss` is now `package/done/variant-postcss`;
> `package/webapp-productivity/done` is now `package/done/app`.

## Report metadata

- Status:
   complete.
- Lifecycle phase:
   screened;
   terminal outcome `no serious alternative` (screening hard-gate exit).
- Subject:
   `prisma-orm-for-turso-needs`.
- Scope:
   replace all repo Turso database access,
   active embedded `@tursodatabase/database` packages and paused libSQL/Kysely usage,
   with Prisma ORM (user pointed at the Prisma Next docs,
   <https://www.prisma.io/docs/next>).
- Started:
   2026-07-20.
- Last updated:
   2026-07-20.
- Governing skill commit:
   `a05818ad70a40e5769a36de669697ba109891b31` (`.agents/skills/choosing-technology/SKILL.md`).
- Governing skill SHA-256:
   `393eb68c5b2b2f7b16c8f7f90c100fb8be43eefa4501511360cd0572e4ae8087`.
- Compatibility fingerprint (SHA-256 of canonicalized context JSON):
   `dbff0104ca5e6894083e70b049ebd5805dea22212b76cf2788a0b6e88378791d`.
- Audit owner:
   Claude Code session `f1c75178` for Aquaticat.
- Prior compatible report:
   none;
   `doc/audit/` has no `tech-*` report with this subject slug
   (nearest related artifact is `doc/audit/turso-cargo-toml-0.6.1.md`,
   an incumbent audit,
   not a replacement vet).

## Hard constraints

1. The replacement must drive the embedded `@tursodatabase/database` engine
   that every active Turso consumer uses,
   or provide equivalent behavior on the same data.
2. The replacement must preserve the native full-text index surface the active packages depend on:
   `CREATE INDEX ... USING fts`,
   `fts_match()`,
   `fts_score()`,
   available only behind the engine-specific `experimental: ['index_method']` connect option
   (`doc/troubleshooting/turso-fts5-native-fts.md`).

## Base components and overlays

- Candidate Prisma ORM (7.x line):
   inspectable open-source local technology;
   overlays:
   incumbent replacement,
   native/prebuilt boundary (query engine and generated client).
- Candidate Prisma Next (Early Access rebuild):
   inspectable open-source local technology;
   same overlays.
- Prisma Postgres (the hosted service Prisma markets alongside Next) is out of scope:
   the request is about Turso needs,
   not a Postgres migration,
   so no SaaS component was classified.
- Incumbent `@tursodatabase/database`:
   retained-candidate under the replacement parity overlay.

## Context: measured incumbent inventory

Active packages (the only non-paused Turso consumers):

- `package/webapp-productivity/done`:
   `@tursodatabase/database` via catalog (`pnpm-workspace.yaml:47`,
   `>=0.6.1`);
   uses the native FTS index method with `experimental: ['index_method']`;
   the FTS remediation and its constraints are recorded in
   `doc/troubleshooting/turso-fts5-native-fts.md`.
- `package/webapp-productivity/done-postcss`:
   same dependency and FTS usage.

Paused packages (`package-paused/`,
 excluded from the hard constraints but inventoried):

- `package-paused/webapp-forge/server`:
   `@libsql/client`,
   `@libsql/kysely-libsql`,
   `kysely`,
   plus `@tursodatabase/database`.
- `package-paused/webapp-content/messages-demo` and `package-paused/webapp-forge/seed`:
   `@tursodatabase/database`.

`doc/dependency-blocklist.md` has no entry for Prisma,
Turso,
or libSQL,
so no repo policy pre-decides this question.

## Candidate ledger

### Prisma ORM 7.x (current GA line)

- Discovery source:
   named by the user.
- Base category:
   inspectable open-source local.
- Screening result:
   exited at screening hard gate (category and hard-constraint mismatch;
   see hard-gate exits).

### Prisma Next (Early Access, the docs the user linked)

- Discovery source:
   named by the user via <https://www.prisma.io/docs/next>.
- Base category:
   inspectable open-source local (Early Access rebuild).
- Screening result:
   exited at screening hard gate
   (no SQLite-family connector exists in the Next line at all).

### Community or Turso-authored Prisma driver adapter for `@tursodatabase/database`

- Discovery source:
   widened search (RXH) across GitHub code search,
   npm-oriented web search,
   and the Turso monorepo.
- Screening result:
   no such candidate exists to screen (see query records).

### Incumbent retention: `@tursodatabase/database` direct SQL

- Discovery source:
   replacement parity overlay requires keeping the incumbent as a candidate.
- Screening result:
   survives by default;
   no alternative survived screening.

## Query and evidence records

All evidence accessed 2026-07-20 unless noted.

### Repo measurements

- `rg --ignore-case 'libsql|turso|kysely|prisma|drizzle'` over `package/` and `package-paused/`
  `package.json` files:
   produced the incumbent inventory quoted in the context section.
- `rg --ignore-case 'turso|libsql|prisma' doc/dependency-blocklist.md`:
   one incidental `@libsql/hrana-client` mention (line 298),
   no blocklist entry.

### Prisma monorepo clone (primary source)

- Clone:
   `gh repo clone prisma/prisma ~/temp/agent/prisma-20260720 -- --depth 1`;
   HEAD `a6d0155` (2026-07-20),
   tag `7.9.0-dev.31`,
   confirming `main` is the Prisma 7 line.
- `packages/adapter-libsql/package.json:49` in that clone:
   the adapter's runtime dependency is `"@libsql/client": "^0.17.0"`.
   The adapter drives libSQL,
   full stop.
- `rg --ignore-case --count-matches 'tursodatabase' ~/temp/agent/prisma-20260720`:
   exit 1,
   zero matches across the whole monorepo,
   run uncapped and unfiltered (QRY sanity check).
- `ls packages | rg adapter`:
   the complete adapter roster is
   `adapter-better-sqlite3`,
   `adapter-d1`,
   `adapter-libsql`,
   `adapter-mariadb`,
   `adapter-mssql`,
   `adapter-neon`,
   `adapter-pg`,
   `adapter-planetscale`,
   `adapter-ppg`.
   No adapter targets the embedded Turso engine.

### Prisma documentation (vendor primary source)

- <https://www.prisma.io/docs/next>:
   Prisma Next is an Early Access ground-up rebuild;
   its quickstart and connector navigation covers PostgreSQL,
   MongoDB,
   and Prisma Postgres only;
   no SQLite,
   libSQL,
   Turso,
   or D1 mention anywhere in the section.
   (Two fetches,
   one asking for the full navigation;
   `https://www.prisma.io/docs/next/fundamentals` returned 404.)
- <https://www.prisma.io/docs/orm/overview/databases/turso>:
   Turso support on the current ORM line goes through `@prisma/adapter-libsql`;
   Prisma Migrate is incompatible with libSQL's HTTP transport,
   so schema changes go through `prisma migrate diff` plus the Turso CLI.
- <https://www.prisma.io/blog/prisma-turso-ea-support-rXGd_Tmy3UXX> and
  <https://github.com/prisma/prisma/discussions/21345>:
   Turso support has been labeled Early Access since Prisma 5.4.2 and remains so.

### Widened adapter discovery (absence evidence)

- `gh search code '"@tursodatabase/database" "driver-adapter"' --limit 10`:
   zero results.
- `gh search code '"@tursodatabase/database" "SqlDriverAdapter"' --limit 10`:
   zero results.
- Web search
  `npm prisma driver adapter "tursodatabase" OR "turso database" embedded rust sqlite rewrite`:
   every hit resolves to `@prisma/adapter-libsql` for libSQL/Turso Cloud;
   nothing wraps the embedded engine.
- `gh search code --repo tursodatabase/turso 'prisma'`:
   matches only in `bindings/javascript/package-lock.json` (`@prisma/client`),
   and `gh api` listings of `bindings/javascript/perf` and `bindings/javascript/packages`
   show a benchmark harness (`perf-turso.js`,
   `perf-better-sqlite3.js`) and binding packages
   (`common`,
   `native`,
   `wasm-common`,
   `wasm`),
   no Prisma adapter shipped or in progress there.

## Hard-gate exits

### Prisma ORM 7.x: category and hard-constraint mismatch

Prisma cannot connect to the database our active packages actually run:

1. No first-party adapter for `@tursodatabase/database` exists
   (adapter roster and zero-match search in the clone).
2. No community or Turso-authored adapter exists (widened discovery above).
3. `@prisma/adapter-libsql` wraps `@libsql/client`,
   a different engine and client API;
   it cannot pass the `experimental: ['index_method']` connect option,
   which only `@tursodatabase/database.connect()` accepts.
4. Even under a hypothetical future adapter,
   hard constraint 2 fails:
   Prisma's schema DSL and migration engine cannot express
   `CREATE INDEX ... USING fts` or query through `fts_match()`/`fts_score()`;
   every FTS touchpoint would drop to raw SQL escape hatches,
   which removes the type-safety value an ORM migration would be buying.
5. Independently,
   the file-format route (pointing a SQLite-family adapter at the `done` database file)
   fails because the schema contains `CREATE INDEX ... USING fts`,
   which no non-Turso engine parses.

Secondary (not needed for the exit,
recorded for completeness):
even for genuine libSQL targets,
Prisma's Turso support is Early Access and Prisma Migrate does not work against it.

### Prisma Next: required category absent

The Next line the user linked supports PostgreSQL,
MongoDB,
and Prisma Postgres only.
There is no SQLite-family connector to evaluate,
so "migrate to Prisma Next for Turso needs" has no implementable form today.

## Gates not reached, with reasons

- SaaS historical and operational domains:
   not applicable;
   no hosted component is in scope and both candidates exited at screening.
- Open-source deep source/test/CI audit,
   maintenance audit,
   finalist validation,
   execution gate:
   not reached;
   no candidate survived screening,
   and no third-party code was executed (clone inspection only).
- Weighted scoring and sensitivity:
   not applicable;
   `score: not applicable`,
   no finalist exists.

## Ranking and recommendation

Ranking of what remains:

1. Keep the incumbent `@tursodatabase/database` direct-SQL layer (only surviving candidate).
2. Prisma ORM 7.x (exited:
    cannot drive the incumbent engine or its FTS surface).
3. Prisma Next (exited:
    no SQLite-family support at all).

Prisma 7.x ranks above Prisma Next because 7.x at least has an Early Access path
to libSQL-flavored Turso,
while Next currently has none.

Recommendation:
**none**;
the migration as asked is not currently possible.
Terminal outcome `no serious alternative`.

Open constraint question for the user
(the skill requires asking which named hard constraint,
if any,
should change):

- Keep hard constraint 1 (stay on the embedded Turso engine):
   Prisma is out;
   revisit only if Prisma or Turso ships a driver adapter for `@tursodatabase/database`.
- Drop hard constraint 1 (switch the active packages' engine,
   e.g. to `@libsql/client` or plain SQLite):
   Prisma 7.x becomes evaluable,
   but a fresh vet would be required,
   and constraint 2 still bites:
   native `USING fts` does not exist on those engines,
   so search would need redesign (FTS5 or `LIKE`),
   a regression path already analyzed in `doc/troubleshooting/turso-fts5-native-fts.md`.

## Evidence limits

- Prisma documentation pages were read through summarizing fetches,
   not raw HTML;
   the two load-bearing claims (adapter dependency,
   monorepo `tursodatabase` absence) were instead verified from the cloned source.
- GitHub code search does not index all forks or very recent pushes;
   absence evidence is as of 2026-07-20.
- The Prisma clone is shallow at one commit (`a6d0155`);
   unmerged branches were not searched for embedded-Turso adapter work.
- Prisma Next is Early Access;
   its connector roster can grow.
   The `@prisma/adapter-libsql` published version was reported as 7.8.0 by search results,
   not verified against the npm registry (not load-bearing).
