# Module-logger log accumulation (issue 576)

Proposal answering the three questions of `Aquaticat/Monochromatic#576`
(retention,
location,
one file per invocation)
plus two follow-ups the measurements force
(test inflow and the existing backlog).
Status:
accepted as the L2+F1+R1+I1+B1 package and implemented in `module-logger` `0.4.5`;
[the decision record](../decision/module-logger-log-accumulation.md) is canonical.
The option set in [Open questions](#open-questions) and the measurements stay here as the evidence trail.

## Measurements (2026-10-22)

- The main checkout's `node_modules/.monochromatic` holds 19 GB (`du -sh`).
  Two top-level listings minutes apart counted 1,761,863 and 1,761,868 entries,
  essentially all `*.log.jsonl`;
  inflow continued while measured,
  so both counts are lower bounds at their instant.
- Issue 576 measured 2026-09-26 at 8.8 GB in 499,257 files.
  The tree gained about 10 GB and 1.26 million files in the 26 days since.
- Files per mtime day (`ls -l --time-style=+%Y-%m-%d`),
  2026-10-08 to 2026-10-22:
  2026-10-12 holds 107,082,
  2026-10-11 holds 101,380,
  2026-10-10 holds 85,381,
  and the quietest days hold under 1,100.
  Day-to-day variance spans five orders of magnitude.
- The 400 newest files (`stat --printf='%s'`):
  median 209 bytes,
  mean 721 bytes,
  maximum 159,161 bytes.
  A median file holds a handful of lines,
  so per-file block overhead dominates the tree.
- The `translation-repair` worktree measured 2026-09-27 in the issue:
  34,919 top-level `*.log.jsonl` files,
  846 MB together with the deliberate run directories.

## What the code does today

- The default Node sink list contains the file sink
  (`package/logger/core/src/default-sinks.node.ts:41`),
  so every Node process that logs anything writes a file.
- One file per process and sink instance:
  `{timestamp}.{uuid}.log.jsonl`
  (`package/logger/core/src/sink/file-open.ts:58-63`),
  created and probe round-tripped at verification.
  The probe line `{"test":true,...}` is written into every file,
  so the 40 byte sample minimum shows probe-only files exist.
- Destination order:
  explicit `createFileSink({ directory })`,
  else `MONOCHROMATIC_LOG_DIRECTORY`
  (`package/logger/core/src/sink/file.ts:52-54`),
  else the nearest ancestor `node_modules/.monochromatic`
  (`package/logger/core/src/sink/file-destination.ts:52-77`),
  else verified user state
  (`package/logger/core/src/sink/file-state.ts`).
- No retention,
  rotation,
  or disable switch exists.
  The issue's 2026-09-27 comment predates `MONOCHROMATIC_LOG_DIRECTORY`,
  which now exists and is strict,
  but it bounds nothing:
  it only redirects where files accumulate.
- `LogRecord` is `{level, message, timestamp}`
  (`package/logger/core/src/types.ts:9-13`).
  Records carry no process identity,
  so a merged file would lose attribution.
- `node_modules/.monochromatic/` also hosts the deliberate
  `translation-repair-runs*` run directories
  (`package/translation-repair/run/src/corpus-run/runs-layout.ts`).
  Those are out of scope here.

## Constraints already settled

- The fix belongs in the logger.
  cli-git's linked-worktree copy keeps zero repository-specific exclusions
  (behavior ledger section `Linked-worktree ignored-state copy`),
  so no copier-side filter is on offer.
- `@monochromatic-dev/logger-core` 0.4.4 is published on npm
  (`npm view` confirms 0.4.4),
  so default destination changes are compatibility-visible.
- Incident docs cite individual log files as evidence,
  for example `doc/troubleshooting/pi-ask-user-question-startup.md`
  and `doc/troubleshooting/pi-sdk-staging.md`.
  The one-process-one-file workflow is worth preserving if affordable.
- Issue 573 (sink verification timeout noise on stderr) is a separate incident
  and stays separate (audit ledger T9 notes both).

## Decision 1: location

Shared scenario for every option:
a wrapped git command at 2026-10-22T10:00 in the `translation-repair` worktree
logs four records.

### Option L1: per-project user state

Files land under `$XDG_STATE_HOME/monochromatic/log/<project-key>/`
(platform equivalents from `file-state.ts`),
where `<project-key>` hashes the project root path,
and a `project-path` manifest line records that root.
Visible result:
`~/.local/state/monochromatic/log/8f2c…/2026-10-22/2026-10-22T14-00-01.123Z.<uuid>.log.jsonl`,
and `git worktree add` copies no log tree because none lives under `node_modules`.

Pros:

- Removes the worktree-copy symptom by construction,
  the issue's main complaint.
- Survives `node_modules` reinstalls,
  which today destroy forensic history silently.
- Matches the XDG "logs are persistent state" rule the fallback already cites
  in `package/logger/core/doc/file-destinations.md`.
- `MONOCHROMATIC_LOG_DIRECTORY` stays strict,
  so callers wanting a chosen directory
  (translation-repair child runs,
  auto-mode)
  keep working unchanged.

Cons:

- Logs stop sitting beside the checkout,
  so incident citations need a path mapping
  (hash to root via the manifest).
- Every npm consumer of 0.4.x sees the default destination change on upgrade.
- Findability depends on the manifest convention being documented.

### Option L2: stay project-local, rely on retention

Default stays `node_modules/.monochromatic`;
only retention bounds it.
Visible result:
same paths as today,
but `git worktree add` still copies whatever window retention keeps
(for example 14 days of files)
into every new worktree,
and the copy stall risk shrinks proportionally to the window only.

Pros:

- Zero consumer-visible path change;
  smallest diff.
- Logs stay beside the tree they describe.

Cons:

- Does not remove the reported symptom,
  only shrinks it;
  a peak day of 107,082 files is still copy cargo.
- `node_modules` wipes keep destroying history.
- Sweeps run inside checkouts that tooling copies and caches.

### Option L3: user state default with a project-local opt-in

Same as L1,
plus a repo-level opt-in that restores project-local placement.
Visible result:
L1's layout by default;
a repo that sets the opt-in writes back into its own `node_modules/.monochromatic`.

Pros:

- Escape hatch for projects whose logs should travel with the tree.
- No cli-git change needed for the default case.

Cons:

- Two live layouts to document and test forever.
- The escape hatch reimports the copy cost exactly where it hurts.

### Ranking

L1 > L3 > L2,
because L1 removes the copy cost by construction
while L3 keeps a door back into it,
and L3 > L2 because L2 leaves the reported symptom standing in proportion to the retention window.

## Decision 2: what one log file holds

Shared scenario:
one unit suite run spawns 22 processes that log,
at 2026-10-22T10:00.

### Option F1: one file per process, created on first record

Visible result:
22 files under the day directory,
each holding only its process's lines;
a process that logs nothing leaves no file and no probe line.
Verification probes writability with a temp file it removes.

Pros:

- Keeps the exclusive-ownership contract of `file-open.ts`
  and the one-process-one-file pattern incident citations use.
- Removes probe-only junk files
  and the bogus `{"test":true}` first record consumers parse today.
- No cross-process append semantics enter the design.

Cons:

- File count still tracks process count,
  bounded only by retention caps.
- `file-open.ts` verification changes from final-file probe to temp probe.

### Option F2: one file per process, exactly as today

Visible result:
22 files at verification time,
some containing only `{"test":true,...}`.

Pros:

- No change to `file-open.ts`;
  smallest diff.

Cons:

- Probe-only files and the bogus probe record stay.
- The measured 209 byte median keeps costing a filesystem block each.

### Option F3: one shared rolling file per project

Visible result:
`current.log.jsonl` plus rotated `current.1.log.jsonl` and so on;
all 22 processes append,
each line needs a new `pid` field to attribute,
and per-project file count caps at the rotation depth.

Pros:

- Best history per byte:
  no per-file block overhead against a measured 209 byte median.
- File count bounded absolutely.
- One grep covers the whole project window.

Cons:

- Reopens the ownership model in `file-open.ts`
  and batch appends in `file-append.ts`:
  cross-process `O_APPEND` atomicity assumptions,
  weaker on NFS and SMB.
- `LogRecord` gains fields
  or attribution is lost for merged streams.
- Every consumer treating a file as one process's stream changes;
  largest test surface of the three.

### Ranking

F1 > F3 > F2,
because F1 drops the probe-only files the 40 byte minimum proves exist
while keeping the per-process files incident evidence cites,
and F3 > F2 because F2 keeps every defect of per-invocation files
while F3 at least converts them into bounded count and denser history.

## Decision 3: retention depth

Mechanism adopted under every option:
at sink open the sink sweeps only day directories of its project log dir;
whole day directories older than the age cap go first,
then oldest day directories while over the size or count cap.
Sweep failures report through `reportLoggerInternalError`
and never block logging.
Overriding environment variables mirror `MONOCHROMATIC_LOG_DIRECTORY` naming.

### Options

- Option R1:
  14 days,
  512 MiB,
  50,000 files per project (Recommended).
  Visible result at the measured peak day of 107,082 files:
  one busy day fits under the file cap,
  and history older than two weeks is gone.
- Option R2:
  7 days,
  256 MiB,
  20,000 files.
  Visible result:
  one busy stretch plus its surrounding silence;
  smallest footprint.
- Option R3:
  30 days,
  2 GiB,
  200,000 files.
  Visible result:
  a full month including several peak days;
  largest footprint.
- Option R4:
  size and count caps only,
  no age expiry.
  Visible result:
  quiet weeks never evict,
  so a slow trickle of small files can survive far longer than any window.

### Ranking

R1 > R3 > R2 > R4,
because R1 covers the two-week investigation window this issue's own
2026-09-26 to 2026-10-20 timeline spans at a bounded footprint,
R3 > R2 because measured variance
(7 files on one day,
107,082 on another)
means a 7-day window can hold barely one busy day while 30 days holds a whole incident cycle,
and R2 > R4 because unbounded age is exactly the failure mode that produced today's 19 GB.

## Decision 4: test and CI inflow

Shared scenario:
the translation-repair unit suite adds about 22 files per run
(issue comment 2026-09-27),
and audit ledger entry T9 records the suite as non-hermetic because of it.

### Option I1: kill-switch plus harness redirection

`MONOCHROMATIC_LOG_FILE=off` disables the file sink for a process tree;
the module-test harness also sets `MONOCHROMATIC_LOG_DIRECTORY`
to a per-run scratch directory as defense in depth.
Visible result:
`mise run //package/...:test` leaves zero files in any checkout,
while failed runs keep their scratch logs for debugging.

Pros:

- Answers the T9 ask for a switch that turns the sink off in tests.
- Documented defaults stay intact for normal runs.
- Redirection remains a second line of defense.

Cons:

- One more environment knob to document.
- Tests wanting file evidence must unset it.

### Option I2: redirection only

Harnesses and CI set `MONOCHROMATIC_LOG_DIRECTORY` to scratch directories.
Visible result:
clean checkouts wherever the harness remembers the variable;
a consumer without harness support keeps writing into its checkout.

Pros:

- No new knob;
  uses the strict destination that exists today.

Cons:

- T9's ask stays unanswered.
- Correctness depends on every harness setting the variable.

### Option I3: nothing beyond retention

Visible result:
test runs keep adding files and sweeps reap them later.

Pros:

- Zero new surface.

Cons:

- Peak inflow days keep churning disk and sweep work.

### Ranking

I1 > I2 > I3,
because I1 answers the audit's ask and keeps debugging evidence in scratch,
and I2 > I3 because it keeps checkouts clean without waiting for sweeps.

## Decision 5: the existing backlog

The new layout never visits the old flat `*.log.jsonl` files,
so nothing reaps today's 19 GB automatically.

- Option B1:
  apply the retention window to every worktree of this repo now (Recommended).
  Visible result:
  each `node_modules/.monochromatic` drops to the chosen window;
  `translation-repair-runs*` directories stay.
- Option B2:
  main checkout only.
  Visible result:
  the main checkout shrinks;
  worktrees keep their trees.
- Option B3:
  no deletion now.
  Visible result:
  the 19 GB stays until deleted by hand.

### Ranking

B1 > B2 > B3,
because the copies of the tree live in the worktrees too,
and B2 > B3 because a partial purge still frees the largest tree.

## Overall recommendation

Combine **L1 + F1 + R1 + I1 + B1**.

Landing order:

1.  Retention and day-directory layout inside the logger
   (pure addition;
   old placement keeps working).
2.  Location default flip to per-project user state with the manifest,
   released as 0.4.5 with changelog and migration notes.
3.  Kill-switch and harness wiring for hermetic tests.
4.  Authorized backlog purge across worktrees.
5.  Decision record in `doc/decision/`,
   issue comment on 576,
   and the audit ledger T9 status update.

## Rollout (2026-10-22)

- The retention sweep's first real run bounded the main checkout
  before the purge:
  a `file-enforcer` verify at 08:15 UTC
  (stamp `node_modules/.monochromatic/.logger-retention-sweep`)
  trimmed the measured 19 GB and 1,761,904 entries
  to the R1 caps in one pass.
- The authorized purge then removed every pre-2026-10-08 logger file
  from all 153 linked worktrees:
  44,570 files and 722 MiB deleted,
  53,313 files and 535.6 MiB kept in the window,
  537 files and 49.1 MiB preserved in `translation-repair-runs*`.
  A confirming walk after deletion reports 0 expired entries.
- The main checkout after both passes holds
  47,005 in-window files (511 MiB) spanning 2026-10-17 to 2026-10-22,
  plus the sweep stamp.
- 0.4.5 shipped the same day (release commit `4b0caa5191`).
  The auto-publish lane failed on the `pnpm` frozen-install drift pair
  (`doc/troubleshooting/pnpm-frozen-install-validation-drift.md`),
  and the targeted `npm-publish.yml` run 39749801676
  published it at 04:44 America/New_York with signed provenance
  (transparency log index 3382101501).

## Risks and limits

- npm consumers of 0.4.x see the destination change on upgrade.
  Setting `MONOCHROMATIC_LOG_DIRECTORY` to the old path restores old placement;
  the changelog must say so.
- Per-project hashing never shares a log directory across checkouts of one repo.
  "All logs of repo X" needs a manifest scan.
  Keying by git common dir would share them but couples the logger to git,
  which `file-project.ts` deliberately avoids.
- The sweep costs one small directory walk per process start;
  day directories bound it even at the measured peak of 107,082 files in a day.
- Windows and macOS layouts resolve through `file-state.ts`
  but are verified only through injected path catalogs here
  (`package/logger/core/doc/file-destinations.md`),
  not on those hosts.
- All counts in [Measurements (2026-10-22)](#measurements-2026-10-22)
  moved while read;
  treat them as lower bounds at their timestamp.

## Open questions

1.  Location:
   per-project user state (L1),
   project-local (L2),
   or user state with a project-local opt-in (L3)?
2.  File granularity:
   per-process files created on first record (F1),
   per-process files as today (F2),
   or a shared rolling file (F3)?
3.  Retention depth:
   14 days and 512 MiB and 50,000 files (R1),
   7 days (R2),
   30 days (R3),
   or caps without age (R4)?
4.  Test inflow:
   kill-switch plus harness redirection (I1),
   redirection only (I2),
   or nothing (I3)?
5.  Backlog purge authorization:
   every worktree (B1),
   main checkout only (B2),
   or no deletion (B3)?
