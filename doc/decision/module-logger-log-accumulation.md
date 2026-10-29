# Module-logger log accumulation (issue 576)

## Status

Decided 2026-10-22:
**stay project-local with retention (L2),
one file per process created on first record (F1),
retention window 14 days and 512 MiB and 50,000 files (R1),
kill-switch plus harness scratch redirection (I1),
and a one-shot backlog purge across every worktree (B1)**.
The human accepted this combination through the decision panel
recorded in the session,
against the proposal `doc/planning/module-logger-log-accumulation.md`,
whose full option set and measurements stay there as the evidence trail.
The recommended alternative (L1,
per-project user state)
was rejected in favor of logs staying beside their tree.

The day-directory layout under that destination was confirmed 2026-10-22:
the zero-path-change phrasing in the accepted L2 wording is superseded
by the recorded nesting `{YYYY-MM-DD}/{timestamp}.{uuid}.log.jsonl`,
because age sweeps delete whole day directories
instead of walking every file.

Version 0.4.5 shipped 2026-10-22 with this decision (commit `4b0caa5191`).
It published through `.github/workflows/npm-publish.yml` run 39749801676
(04:44 America/New_York) with a signed provenance statement
(transparency log index 3382101501).
The auto-publish lane (`npm-release.yml`) failed three times on the
`pnpm` frozen-install drift pair recorded in
`doc/troubleshooting/pnpm-frozen-install-validation-drift.md`
before the release could move.

## Context

The file sink wrote one `{timestamp}.{uuid}.log.jsonl` per process
into the nearest ancestor `node_modules/.monochromatic`
with no retention,
rotation,
or off switch.
Measured 2026-10-22 in the main checkout:
19 GB in about 1,761,868 top-level files,
with day peaks of 107,082 files
and a 209 byte median file size.
cli-git's linked-worktree copy carries that ignored tree into every new worktree
(`doc/planning/cli-git-rust-behavior-ledger.md`,
section `Linked-worktree ignored-state copy`),
and the copy keeps zero repository-specific exclusions,
so every fix lands in the logger.

## Decision: retention inside the logger

At sink open the sink sweeps its log directory:
day directories (`YYYY-MM-DD`,
UTC date of the file timestamps)
and stray `*.log.jsonl` files older than the age cap go first,
then the oldest day directories while the tree exceeds the size or count cap.
Entries the logger does not name
(for example the deliberate `translation-repair-runs*` directories)
are never touched.
Defaults are 14 days,
536,870,912 bytes,
and 50,000 files,
overridable by `MONOCHROMATIC_LOG_MAX_AGE_DAYS`,
`MONOCHROMATIC_LOG_MAX_BYTES`,
and `MONOCHROMATIC_LOG_MAX_FILES`.
The sweep is throttled per directory,
runs off the logging path,
and reports failures through `reportLoggerInternalError`
without ever blocking a record.

Retention beats a copier-side exclusion,
which the behavior ledger forbids,
and beats moving to user state (proposal option L1),
because incident evidence stays beside the checkout it describes
and npm consumers keep their documented destination.

## Decision: one file per process, created on first record

Verification probes writability with a temp file it removes;
the real `{timestamp}.{uuid}.log.jsonl` is created under the day directory
only when the first record arrives.
Probe-only files and the bogus `{"test":true}` first record disappear,
while one process keeps one file,
the unit `doc/troubleshooting/` incident citations use.
The shared-rolling-file alternative (proposal option F3)
was rejected:
records carry no process identity,
so merging streams loses attribution,
and it reopens the exclusive-ownership contract of `file-open.ts`
for a file-count bound retention already provides.

## Decision: kill-switch plus harness redirection

`MONOCHROMATIC_LOG_FILE=off` disables the file sink for its process tree,
answering audit ledger entry T9
(`package/translation-repair/doc/audit-ledger-test-suite-defects-in-the-tests-themselves.md`).
The module-test harness additionally redirects `MONOCHROMATIC_LOG_DIRECTORY`
to a per-run scratch directory,
so tests stay hermetic even without the switch.
Documented defaults for normal runs do not change.

## Decision: backlog purge

The new layout never visits the old flat files,
so the retention window is applied once,
by hand,
to `*.log.jsonl` files in `node_modules/.monochromatic`
of every worktree of this repository.
The deliberate run directories stay.
This deletion is destructive and was authorized explicitly
("Every worktree of this repo").

## Consequences

- New log files land at
  `node_modules/.monochromatic/{YYYY-MM-DD}/{timestamp}.{uuid}.log.jsonl`.
  Incident citations of the old flat shape remain valid as historical records.
- `@monochromatic-dev/logger-core` needs a 0.4.5 release note covering
  the day-directory layout,
  lazy creation,
  retention defaults and their environment overrides,
  and the kill-switch.
- Age-only expiry is not enough:
  the measured 107,082 file day needs the count and size caps too.
- The worktree copy still carries whatever window retention keeps
  (up to 50,000 files per checkout).
  If that window ever re-triggers the copy stall of issue 571,
  revisiting proposal option L1 is the recorded escape.
