# Audit ledger: a pass keeps its records in its run folder

Every top-level `corpus-pass-<name>.log` of the agent runs directory was lost on 2026-10-30 (UTC),
and the run folders `corpus-pass-<name>/` survived.
Those files were the lead's launcher capturing the console of the pass with systemd `StandardOutput=file:...`,
and the run readers parsed that capture.
The owner's answer of 2026-10-31,
verbatim:
"we shouldn't have *.log at all.
Our logger writes *.log.jsonl files.
The existence of them means we didn't architect properly."
Entries B1090 to B1099 say what moved,
what holds it,
what did not move and why.
Entries B1110 to B1121 say what an independent re-check of that round found,
and what moved because of it.

Part of [Audit ledger](audit-ledger.md),
which lists every file of this record.

## B1090: a pass wrote nothing, and readers parsed its capture

Status:
fixed on 2026-10-31 (UTC) in `3679320e90` and `a8c99f75be`,
after the failing cases of `7c51effbe0`,
`b5c2a587aa` and `78e9e0951b`.

What was wrong:
a corpus pass printed its lines with `console.log` and logged the rest through the default logger,
whose console sink printed them.
Nothing in the pass wrote a file.
The record was whatever the launcher caught,
and the five readers (meter,
spend,
run timing,
slice cost and cap census) parsed that text.
A lost capture lost the run's evidence although the run folder survived.

What a pass does now:
`runCorpusPassOver` opens a run log in its run folder once it holds the lock
(`typography/src/run-log.ts`,
`openRunLog`),
builds the shared logger's file sink itself,
and installs it for every module root of the package.
A root is built by `contextRoot`,
which now writes through `runLogger`:
a logger that forwards each call to the installed run log,
or to the default logger while none is installed.
The choice is made at the call and not where the root is built,
because roots are built when their module loads,
long before a pass has a run folder.
The lines a pass prints itself go through `printRunLine`,
which prints the line exactly as before and records it whole as the message of a record.
Fifteen such lines sit in seven modules of the pass and nine in five modules of the slice cache.
The console sink stays for a person watching,
and nothing depends on it.

A pass that cannot write its folder refuses to start,
as a stated refusal naming the directory and the two ways out
(make it writable,
or name another runs directory),
because a run that leaves no record of itself is not worth its quota.
`MONOCHROMATIC_LOG_FILE=off` is ignored by this sink on purpose:
a run that quietly records nothing is worse than one that records.

A throw out of the pass after the log is open is recorded as `[corpus-pass] the pass stopped: <refusal text>`
at level `error` while the log stands,
and thrown on unchanged.
The record is made with `recordRunLine`,
which prints nothing:
the entry states the refusal once on stderr,
and the first form of this record went through the logger's console sink too,
so eight cases of the built command's stderr (`corpus-pass.unit.test.ts` five,
`corpus-pass-refusals.unit.test.ts`
three) failed in the end-of-round suite on a second line,
and the record became a print-free one.
The console gets the refusal only after the run log is closed,
so without this line a pass refused by a generation drift left no word of why in its folder.

What holds of the flush,
said plainly (it was written as "flushes on every exit"):
the run log is flushed when the pass returns,
when it throws out of the pass body,
and when SIGINT or SIGTERM stops it (B1113).
A SIGKILL,
a crash of the process and a power loss flush nothing,
so a record written after the last flush is lost,
and the lock is left for the next pass to judge stale.

What the run folder holds after a pass:
`log/<UTC day>/<timestamp>.<uuid>.log.jsonl`,
one file for each process that opened the run log,
named by the sink and never rotated,
one JSON object per line holding exactly `level`,
`message` and `timestamp`.
The day directory is the sink's own and cannot be turned off.
A pass and a `budget-sample` taken into the same folder each leave a file of their own beside the other's.

A correction to the commit messages:
`3679320e90` says it turns "40 failing cases" green.
The failing cases of `7c51effbe0`,
the commit that precedes it,
are 24
(typography 11,
run 10,
slice cache 2,
corpus pass 1),
and typography's 11 became 6 in `3679320e90` when its three reader suites moved to glossary with the reader
(the re-check of B1121 counted six).
`a8c99f75be` says the cap census walk "goes five levels down a named directory and not three".
What changed is the bound `MAX_DEPTH`,
from 3 to 5:
the walk lists directories down to the fifth level below the named one,
so it reads a record file as deep as six levels below it where it read four.
The case that holds it is titled `GOES SIX LEVELS DOWN`,
counting the file's level,
and it moved from a log in a directory at the fourth level to one at the sixth.

## B1091: how a reader finds one run's records

Status:
fixed on 2026-10-31 (UTC) in `3679320e90`.

A report is named a run folder,
its `log` directory,
a day directory or one `.log.jsonl` file.
`readRunLogText` (`glossary/src/corpus-run/run-log-read.ts`) lists the named directory,
then its day directories,
then the day directories of its `log` directory,
and reads the record files of each in code point order of their names,
which is the order the processes started in.
It renders each record as the console sink prints one,
`[level] [ISO instant] message`,
and hands that text to the parsers,
so no parser changed.

A line is a record only when it is a JSON object with exactly `level`,
`message` and `timestamp`,
the level one the logger has,
the message a string and the timestamp a whole number of milliseconds a date can hold.
Anything else (a line a crash cut off,
a foreign line,
an object with a field the sink never writes)
reaches the parser as it stands,
whole,
so the report counts it as the damaged or foreign line it is.
This is the shape that
[readers accept shapes writers never write](audit-ledger-readers-accept-shapes-writers-never-write.md) asks for.
A message reaches the parser as the console showed it:
the console sink writes control characters other than the line feed and the tab as `\uXXXX`,
and the reader does the same through the logger's own function (B1118).
The debug and trace records,
which the console showed only for a verbose run,
are left out unless the reader is asked for them,
so every count is what it was over a capture (B1117).

A folder that holds no record file is refused in words:
`<path> holds no record file (log/<day>/*.log.jsonl), so there is nothing to count. Name the run folder of a pass that
ran, or one of its record files. Nothing was counted.`
A path that is neither a directory nor a `.log.jsonl` file,
a `.log` text capture of a console among them,
is refused as `<path> is neither a run folder nor a file ending .log.jsonl`.
Neither is an empty report.
The positional of the four reports that read records is named `run folder or log file`.

Evidence,
from `report-run-folders.unit.test.ts` and `report-log-read.unit.test.ts`:
the four `readLogTexts` cases (three path forms read alike,
two processes' files in name order with a cut line and a foreign line kept as they stand,
the empty folder refused,
the text capture refused).

## B1092: no retention sweep can delete a run's records

Status:
held on 2026-10-31 (UTC) by `typography/src/run-log.ts`,
pinned in `typography/src/run-log.unit.test.ts`.

Read in full:
`package/logger/core/src/sink/file-retention-sweep.ts` and `file-retention-clean.ts`.
A sweep runs when a file sink verifies and was given a policy.
It writes a stamp file `.logger-retention-sweep` into the directory it sweeps,
removes every day directory whose date is more than `maxAgeDays` old (14 by default) wholesale,
removes expired files at the root of the directory,
then trims the oldest files until the tree is within `maxBytes` (512 MiB) and `maxFiles` (50,000),
never touching a file modified within the last hour.
Names that do not begin with a date,
and every foreign neighbour,
are left alone.

The environment route cannot hold a run's records:
`MONOCHROMATIC_LOG_DIRECTORY` leaves the policy to `logRetentionPolicyFromEnv`,
whose bounds accept only finite,
non-negative numbers,
so there is no way to say "never",
and a sweep would write its stamp file into the run folder and expire its old day directories.
The run log therefore builds the sink itself with an explicit directory and `NO_LOG_RETENTION_SWEEP`,
which the sink offers for caller-owned directories,
and no change to `package/logger` was needed.

The default runs directory,
`node_modules/.monochromatic/translation-repair-runs`,
is a child of the default logger's own directory,
and a sweep of that parent lists names beginning with a date only,
so the run folder is not listed either.

The case `KEEPS A RUN'S OLD DAY DIRECTORIES` opens a run log over a folder holding a day of 2026-01-01,
logs a record,
and finds the old file and no stamp file.
Its control gives the same folder the sink's default bounds and loses the old day.
The guard was shown to fail:
with `NO_LOG_RETENTION_SWEEP` replaced by the default bounds,
rebuilt,
the case failed with `ENOENT` on the old file,
and with the guard restored it passes.
The pass has the same pin at its own boundary:
`KEEPS ITS LINES AS RECORDS in the run folder` starts with an old day directory in the folder
and finds it,
and no stamp,
after a pass.

## B1093: the five readers read records, with the same reports

Status:
fixed on 2026-10-31 (UTC) in `3679320e90`.

Each reader keeps its output and its counts on the same lines,
because the parsers are untouched and the records are rendered as the lines they were.
Each has a differential case that reads the same logical lines as console text and as the records of a run folder,
built through the real logger's file sink (`run-log-write.test-fixture.ts`),
and a case that prints the report over the folder.

- `meter-report`:
  `readMeterLog` over the folder's text equals `readMeterLog` over the lines,
  and the report prints `meter-report: logs=1 readings=2 unread=0` and the same window.
- `spend-report`:
  `tallySpend` over the folder equals `tallySpend` over the lines
  (a cut record and a bare line of nothing among them),
  and the report prints the same seats and `1 log, 4 lines, 2 seats`.
- `run-timing-report`:
  `readRunTiming` over the folder equals `readRunTiming` over the lines,
  and the report prints the same rounds and calls in flight.
- `slice-cost-report`:
  the report over a folder is the report of the same cost lines,
  to the count of dropped lines.
- `cap-census`:
  the walk takes `.log.jsonl` files and drops a `.log` capture by its name,
  the pass-run head reads the `START tip=` record's message (a record that merely mentions the marker is not one,
  and the bound of a line now bounds the record's line),
  and `readCapLog` over the records equals `readCapLog` over the lines.
  The walk lists directories five levels down a named directory,
  so it reads a record file six levels below it,
  since a run's records sit two levels below its folder.

The cases that held the old text inputs were kept and now write record files:
`report-log-write.test-fixture.ts` writes each console-form line as the record the logger writes,
and any other line as a foreign line,
and `report-log-write.unit.test.ts` holds its bytes to the sink's.

## B1094: four commands of the list read no pass records

Status:
recorded on 2026-10-31 (UTC),
read from source,
nothing changed.

- `repeat-census` reads the page of every entry at the corpus pin.
- `ledger-report` reads the contest ledger directory.
- `verify-published` reads artifacts and published pages.
- `cache-account-audit` reads git history and the cache version declarations.

None of the four opens a log,
and the only `console.log` in each is its own report.

## B1095: the budget sample leaves its reading in a run folder

Status:
fixed on 2026-10-31 (UTC) in `a8c99f75be`,
after `b5c2a587aa`.

`budget-sample` logged its `METERS` line through the default logger,
so an operator captured it into a `.log` and `meter-report` was pointed at the capture,
and `meter-report`'s own NOTHING RECORDED text sends the operator to take a sample.
`sampleIntoRunLog` opens the run log of the runs directory
(`TRANSLATION_REPAIR_RUNS_DIR`,
else the worktree default),
refuses when it cannot be written before any meter is asked,
and takes the sample.
A sample taken into a pass's own folder is read with the pass's readings by `meter-report` on that folder.
Cases:
the reading is one record that `meter-report` counts as `logs=1 readings=1 unread=0`,
and a folder blocked by a file is a stated refusal with no meter asked.

## B1096: what is console only, and what the launcher changed

Status:
recorded on 2026-10-31 (UTC);
the launcher is outside the repository,
and the lead made the edits named here to it the same day,
so none is to do.
What is console only changed in B1111 and B1114.

Console only,
with nothing that parses it:
the seat report and the run end reports that `reportingRefusals` prints to stderr after the pass body returns,
the refusal line itself (the `the pass stopped` record of B1090 carries its text),
the build lines of the mise task (`rolldown v...`),
and the lines a probe prints through `artifact-pool.ts` (`POOL ...`),
which the pass never calls
(the `POOL ...` lines of `readPlacement`,
which the pass does reach,
were missed here and are records now:
B1111).
The readers match only these markers:
` METERS `,
` SPEND `,
the stream completion marker,
` round: `,
`SLICE-COST` and `START tip=`.
None of the console-only lines carries one.

The launcher `/var/home/user/temp/agent/tr-fan/s245/split/launch-rotation.ts` changed in these places,
all done:

- Dropped `--property=StandardOutput=truncate:${log}`,
  and the `existsSync(log)` guard against an earlier capture.
- Reads START from the records:
  polls `${runsDir}/log/*/*.log.jsonl` and parses each line as JSON,
  taking the record whose `message` starts with `START `,
  not a line that starts with `START` in a capture.
  The digest check on `pipeline=${FROZEN} ` then reads that message.
- The `rolldown v` lines come from the mise build the unit runs.
  They reach the unit's journal and no record,
  so the bundler version is read from `journalctl --user --unit=<unit> --output=cat`.
- The `/\[error\]/` early exit reads `level === "error"` of the records.
  A stderr line of the pass is no longer in any file the launcher reads,
  and a refusal before the run log opened is in the journal only (B1114).

## B1097: census of every `.log` the package named, at `d343265b71`

Status:
read on 2026-10-31 (UTC) with `git grep -P '(?<!console)\.log\b'` over `package/translation-repair`,
and each hit handled.

Production TypeScript,
15 lines:

- 5 in `cap-census-walk.ts` (the reader):
  moved to `.log.jsonl`.
- 1 in `slice-cost-report-run.ts` (an example in a TSDoc comment) and 3 in `editor-fidelity-read.ts` (the arm reader):
  the first moved,
  the second is open (B1099).
- 2 in `coverage-census-run.ts`,
  which names the files `suite-<member>.log` and `suite.log` for the stdout of a child
  test runner:
  the captured output of a third party,
  not a pass's evidence,
  and nothing in this package parses it.
- 2 in `editor-fidelity-pool-draw-run.ts`,
  which are `input.log`,
  a logger,
  and no file.
- 1 each in `log-context.ts` and `artifact-two-lane-contract.ts`,
  history that names a past capture:
  the text stays and a note beside it says what the file was.

Test files,
136 lines,
and fixtures,
4 lines:
the readers' cases wrote `.log` text files and now write `.log.jsonl` record files,
the cap census walk cases name `.log.jsonl`,
and the cases of `editor-fidelity-read`,
`coverage-census` and the probe fixtures that name `<arm>.log`
or `suite.log` are open with their runners.

Markdown other than the ledger,
30 lines:
the pass runbook and its checks after the run exits,
`provider-availability.md` and `mistake-prevention.md`
were rewritten to the run folder (`jq` over the records where a `grep` over the capture stood);
`probe/README.md` carries an OPEN note and its 24 lines stand until B1099 lands;
`editor-fidelity-bench-outcome.md` names the three logs of a past bench and stays;
`mistake-prevention-tasks-builds-and-bulk-output.md` names the lead's scratch capture of a test sweep and stays.

The ledger,
209 lines:
history of the logs of past passes,
named as evidence,
which stay as written.

## B1098: no cache version moved

Status:
read on 2026-10-31 (UTC).

The package versions its caches by constants whose names end `CACHE_VERSION`
(`SLICE_CACHE_VERSION`,
`TRANSLATE_SLICE_CACHE_VERSION`,
`REFINE_CACHE_VERSION`,
`CONSOLIDATE_CACHE_VERSION`,
`LANE_CONTEST_CACHE_VERSION`,
`CONSISTENCY_CACHE_VERSION`,
`PAIRING_CACHE_VERSION`,
`PAGE_TITLE_CACHE_VERSION`)
and `cache-key-sheet-blindness.unit.test.ts` holds the five key sheets to theirs.
This round changed where records are written and what the pass prints beside them:
no key material,
no stored record shape,
no prompt,
no sheet and no decision rule.
So no stored slice,
pairing,
title or consolidation record is stale,
and none was bumped.
The pipeline digest moves,
as it does for any commit that changes a built file,
and a resume across it needs `TRANSLATION_REPAIR_ALLOW_GENERATION_DRIFT` as it always did.

## B1099: what was not moved, as open scope

Status:
open on 2026-10-31 (UTC).

The editor fidelity bench still captures each arm's console into `<arm>.log` with systemd
and reads that capture with `editor-fidelity-read`,
along with an `<arm>.exit` file.
Its runbook (`probe/README.md`) greps `high.log`,
`medium.log` and `low.log` in 24 lines.
The arm already runs with `TRANSLATION_REPAIR_RUNS_DIR=$BENCH/$ARM`,
so the same block as the pass (`openRunLog`,
`installRunLog`,
refuse when not recording,
`printRunLine`
for its `TRIALS`,
`GUARD` and `PROMPT-REUSE` lines) fits in the probe entry,
and `editor-fidelity-read` would take the three arm folders.
The files are `probe/src/corpus-run/editor-fidelity-read.ts`,
`editor-fidelity-probe-run.ts`,
their tests,
`run/src/corpus-run/editor-fidelity-*command-line*`,
and the README.

The other runners that call a model,
probe or bench write their console and the default logger,
and are launched with an operator's redirect:
`audit-sensitivity`,
`checker-sensitivity`,
`coverage-control-probe`,
`coverage-probe`,
`displacement-probe`,
`editor-calibrate`,
`editor-width-probe`,
`judge-fidelity-probe`,
`model-health`,
`probe-relabel`,
`probe-sensitivity`,
`probe-verify`,
`producer-calibrate`,
`reader-measure`,
`recall-benchmark`,
`rendering-audit-settled`,
`roster-bench`,
`sentinel-probe`,
`translate-probe` and `window-trial-probe`.
The shape of the fix is one option in the command table that makes `reportingRefusals` open and install the run log of
the runs directory around the body,
with the same refusal,
and one reader rule for their reports.
It is a round of its own.

## B1110: the loggers of a pass bypassed its run log

Status:
fixed on 2026-10-31 (UTC) in `bbc2952859` and `b48b8da3e5`,
after the failing cases of `fbe3d6d29f`.

What was wrong:
`tagged` writes through logger-core's default logger when it is given no `l`.
`contextRoot` writes through `runLogger` (B1090),
but the pass built its entry loggers as `sliceTagged({ l: tagged({ tag: entryId }) })`,
which never reaches the installed run log.
So the `round:` lines of the stages,
the `SLICE-COST` lines of the lanes and the consolidation,
the consolidate driver's lines and every entry-level warning and error reached the console alone,
and `run-timing-report` and `slice-cost-report` read nothing on a real run.
B1093 did not see it:
the case that ran a pass (`corpus-pass-run.unit.test.ts`) ran `--plan` only,
and its comparison dropped every message that begins with `[`.

Census at `86c1722f6e`,
by syntax tree over the production source of every package of the family and under whatever local name a file binds `tagged`:
201 calls,
171 with an `l` and 30 without.
Of the 30,
11 are on the pass path in 8 files:
corpus pass 7 (`pass-entry` 1,
`pass-lanes` 1,
`pass-prepare-reseat` 1,
`pass-insertion-reseat` 1,
`pass-reseat` 3),
slice cache 2 (`pass-decline`),
run 2 (`required-providers`,
`run-providers`).
The other 19 sit in 17 files of runners that are no pass:
the bench 7,
the probe 10 and the coverage census 2.
No other logger of the pass bypasses the run log:
the default `logger` is imported by name in `bench/.../model-health.ts` and `probe/.../damage-sample.ts` only,
both runners,
and `createLogger` outside typography is the silent logger of `page-check/.../source-ships-tiling.ts`.

What holds now:
`entryLoggerOf` (`corpus-pass/src/corpus-run/pass-entry-logger.ts`) builds the one entry logger,
slice-tagged and rooted on `runLogger`,
and the 7 corpus pass sites call it;
the 4 sites of slice cache and run give `tagged` an `l: runLogger`.
Where no run log is installed the logger writes to the default logger as before.
The scan `source-scan/src/tagged-parent-scan.unit.test.ts` fails on any call of `tagged` without a parent in the family's source,
but for the 17 runner files it lists with their counts and the reason of B1099:
the list can only shrink as those runners move.
The pass-level case is in `pass-entry.unit.test.ts`:
it settles an entry with a scripted client through `settleEntry` under an installed run log,
settles a second entry that is declined while a page an earlier crash left stands,
and finds in the folder a `round:` record,
a `SLICE-COST` record and the `warn` of the removed page.
At `86c1722f6e` it found no `round:` record.
The seam is `settleEntry`,
which `runCorpusPassOver` hands each entry,
because the `runCorpusPassOver` cases script their settlement;
the cases that run the whole pass over a scripted settlement hold the lines the pass prints itself.

What the scan cannot see:
a function that receives its `l` from a caller that built it with the default logger is a call the scan has no reason to read,
and a `tagged` called by a name that is neither an import nor a const alias of it.
The behavioural case sees the first;
nothing in the family builds the second.

## B1111: console-only lines the pass reached are records now

Status:
fixed on 2026-10-31 (UTC) in `6d57f27934`,
after the failing cases of `e34df38acf`.

B1096 listed what the pass leaves on the console.
Three things on that list were reached by a pass and belong in its folder:

- The `POOL ...` lines of `readPlacement` (`artifact-pool`),
  4 of them,
  reached through the resume guard's census.
  They went through `console.log`;
  they go through `printRunLine` now,
  which prints the line as before (the probes' stdout cases hold it) and records it.
  The `POOL` lines that `artifact-pool.ts` prints for a probe stay console only,
  since the pass never calls it.
- The seat report (`SEAT ...`,
  `SEATS DARK:`) and the end reports (NeuralWatt's plan energy) were printed by `reportingRefusals` after the pass body
  returned,
  that is after the run log was closed.
  `reportingRefusals` builds those lines once (`ClosingReport`:
  the end reports empty their registry and may ask a provider for a reading,
  so they cannot be built twice),
  hands them to the body as `closing`,
  and prints the same lines when the body returns.
  The pass records them as info records last,
  while its log stands,
  after any `the pass stopped` record.
- The stack frames of an unexpected fault were printed after the log closed.
  A throw that is neither a stated refusal nor an unreadable run file (`isUnexpectedFault`) records the frames the command
  prints (`framesOf`:
  the frames without the message or the cause) as `[corpus-pass] the stack frames of the fault that stopped the pass:`.

Still console only,
and in the unit's journal (B1114):
the refusal line itself,
a refusal before the run log opened,
and the build lines of the task.

## B1112: the wait for the run folder to answer is bounded

Status:
fixed on 2026-10-31 (UTC) in `6d57f27934`,
after the failing cases of `e34df38acf`.

`openRunLog` awaited the file sink's verification with no bound,
after the pass held its lock,
so a disk that never answered left a pass holding its lock and saying nothing.
The wait is bounded now by `RUN_LOG_VERIFY_MS`,
30 seconds,
through `settleWithin`.
The run log reports its `destination` as `writable`,
`unwritable` or `unanswered`,
and records nothing in the last two.
The pass refuses to start in words that name the folder and say which it was,
and the lock is released by the `await using` that holds it.
The console logger of an unanswered log leaves the unverified sink out,
because the logger would wait for it on its first line.
The case plants a verification that never settles and passes the bound under fake timers.

The dependency:
`@monochromatic-dev/async-time-core` is a `devDependency` of typography,
and the import in `run-log.ts` is accepted by the linter and bundled into the built package.
`package.json` and `pnpm-lock.yaml` are untouched,
since moving the entry to `dependencies` stales the lock and installs are not part of this round.
If the owner wants the published package to depend on it,
the move and a lock regeneration are owed.

## B1113: a stop signal ends a pass with its reason recorded

Status:
fixed on 2026-10-31 (UTC) in `6d57f27934`,
after the failing cases of `e34df38acf`.

A pass had no signal handler:
`systemctl --user stop`,
a reboot's SIGTERM and a Ctrl-C ended it at once,
with nothing in its folder to say why and its lock file left for the next pass to judge stale.
`handlingStopSignals` (`pass-stop-signals.ts`) is installed once the lock and the run log are held,
and on SIGINT or SIGTERM:
records `[corpus-pass] the pass stopped: <signal> arrived, ...` at level error,
flushes the run log,
releases the lock,
and exits with the conventional code,
130 for SIGINT and 143 for SIGTERM.
Each step is tried whatever the one before did,
and a second signal changes nothing.
The handlers come off when the pass returns.
The pass does not wait for the entry in flight:
an artifact is written to a temporary name and renamed,
so a stop mid-write leaves nothing half-written,
and the entry is pending for the next pass as after any kill.

The cases:
the order of record,
flush,
release and exit with a stand-in for the process's signals,
and a real child program that holds a lock and a run log as a pass does,
which the case starts and signals itself (`runUntilSignalled` of the shared child fixture,
which kills the child at the shared hang stop):
it exits 143 and 130,
its folder holds the record naming the signal,
and `pass.lock` is gone.

`process.exit` is called once,
in `exitProcess`,
behind a disable comment for `unicorn/no-process-exit`.
The rule has no option for a handler that must end a pass that is still running,
and ending it with a signal's conventional code needs an exit.

What holds of the flush is in B1090.

## B1114: a refusal before the run log opens leaves no record

Status:
runbooks fixed on 2026-10-31 (UTC).

A pass opens its run log after it holds the lock.
A refusal before that point is stated once on the console and recorded nowhere:
the task runner's guard,
the command line read,
an environment override that cannot be read (`corpus-pass.ts` reads them all before the pass starts),
the runs directory another pass holds,
a `MONOCHROMATIC_LOG_DIRECTORY` inside the run folder (B1115),
a run folder the pass cannot write,
and one that does not answer (B1112).
The runbook's detached form sent the console to `/dev/null`,
so such a refusal was lost,
and its step said a pass that cannot write the log folder "refuses to start and says so",
which is true only of the console.
The runbook (`doc/runbook/translation-repair-corpus-pass.md`) now launches the pass as its own unit with no output property,
so the console reaches the journal,
says where to read it (`journalctl --user --unit=<unit> --output=cat`,
which keeps the output after `--collect` removes the unit),
and lists these refusals as the journal's,
beside the build lines.
`mistake-prevention.md` says the same.
No code changed:
the refusal text is on the console,
which is where the owner reads a refusal.

## B1115: the default logger must not write into a run folder

Status:
fixed on 2026-10-31 (UTC) in `6d57f27934`,
after the failing case of `e34df38acf`.

The run log is built with no retention sweep (B1092),
but the default logger every other module writes through takes its directory from `MONOCHROMATIC_LOG_DIRECTORY`
and sweeps it:
day directories past its age bound go,
and the oldest files are trimmed by size and count.
An operator who set the variable to a run folder's `log` directory handed the sweep the run's evidence.
`refuseLogDirectoryInRunFolder` (`pass-log-directory-guard.ts`) runs after the lock is held and before the run log opens,
and refuses in words that name the variable,
the directory and the run folder when the variable names the run folder or a place in it,
links resolved and a directory that does not exist yet judged by its nearest parent that does.
The case tries the run folder,
its `log`,
a day below it and a folder not made yet,
and starts the pass with a directory outside as the control.
The unset case needs no guard:
the default directory is the user's state or the project's `node_modules/.monochromatic`,
the default runs directory sits inside it,
and the sweep reads names that begin with a date only (B1092).

## B1116: a record file exists only once a record is written

Status:
wording fixed on 2026-10-31 (UTC).

`mistake-prevention.md` and the runbook said that right after the launch `${RUNDIR}/log/<day>/` holds one
`.log.jsonl` file.
The sink makes the file with its first record,
so at the instant the unit starts there is none.
Both now say the file appears with the first record.

## B1117: debug and trace records reach the folder

Status:
fixed on 2026-10-31 (UTC) in `d865644c86`,
after the failing cases of `ab7b3d8389`.

The console sink drops debug and trace records unless a run is verbose,
and the file sink records them always,
so a folder holds records a console capture never held.
They stay in the folder:
more evidence.
The reader (`readRunLogLines`,
`readRunLogText`) hands the report parsers what the console showed,
which is every record but the debug and trace ones,
unless it is asked for them (`verbose`).
So every count a report makes is the count it made over a capture:
the `lines` of `spend-report` and `run-timing-report`,
the markers all five readers match,
and the lines `slice-cost-report` drops.
The case writes a record of every level whose message quotes a round marker,
and finds none of the debug and trace ones counted.
`run-records --verbose` prints them.

## B1118: a record is shown as the console showed it

Status:
fixed on 2026-10-31 (UTC) in `d865644c86`,
after the failing cases of `ab7b3d8389`.

The console sink writes control characters (C0 but the line feed and the tab,
DEL and C1) as `\uXXXX`;
the reader passed them raw,
and the runbooks read the files with `jq --raw-output '.message'`,
which prints a message raw,
so an escape sequence reaches the terminal,
and which jq 1.8.1 refuses with exit 5 on a record holding a lone surrogate.
`renderRunLogLine` writes the message through the logger's own function for it
(`_neutralizeControlCharacters` of `@monochromatic-dev/logger-core/ts`,
an underscore export that exists so the logger's tests reach it;
using it keeps one definition of the escaping instead of a copy that can drift).
The case writes all 160 characters from NUL to the end of C1 through the real console sink and through the reader,
and finds the same text for each.

`run-records` is the new built command and task of the run package:
`mise run //package/translation-repair/run:run-records -- <run folder or record file>...` prints one line for each record,
every line through the same function once more,
so a foreign line is escaped too.
The two runbooks read the records with it.
It refuses a log named twice (B1119).

## B1119: a folder named beside its own log was counted twice

Status:
fixed on 2026-10-31 (UTC) in `d865644c86`,
after the failing cases of `ab7b3d8389`.

`refuseRepeatedLogs` compared the spellings of the paths named,
so a run folder named with its `log` directory,
a day directory or one of its files,
or with a link to it,
read the same files twice and totalled every call twice.
It is asynchronous now and resolves each named log to its record files and each file to its real path
(`realpath`),
and refuses the first log that reaches a file an earlier one reached,
naming both.
The check by spelling stays first,
for a log named twice that does not exist yet.
The meter report collapses exact repeats of a reading already and needs no refusal.
The cases:
5 pairs of overlapping names,
and a folder with no record,
refused in the readers' words.

## B1120: a reader holds no file whole

Status:
fixed on 2026-10-31 (UTC) in `d865644c86`,
after the failing case of `ab7b3d8389`.

`readRunLogText` read every file whole,
rendered into a list and joined it:
the file's text,
the list and the joined text held at once.
`readRunLogLines` reads each file in chunks,
cuts lines on the line feed alone as before (a last line no line feed ends is a line),
and hands them one at a time,
so a consumer that stops stops the read.
The case reads a record of 500000 characters and stops after two lines with a later file that cannot be opened.
`spend-report` and `run-timing-report` read lines with it and no longer split a text.

One existing case pinned the old error text:
the case of `slice-cost-report-run.unit.test.ts` that names a directory for the log still sees the fault escape unwrapped.
The text of that read failure was `EISDIR: illegal operation on a directory, read '<path>'` from `readFile`,
and it is `EISDIR: illegal operation on a directory, read` from a stream,
which no longer quotes the path.
The case pins the new text.

The limit that remains,
said plainly:
the parsers of `meter-report`,
`slice-cost-report` and the cap census take a whole text,
and those of `spend-report` and `run-timing-report` take the whole list of lines,
so each report holds what it counts once,
not one line.
Only `run-records` holds one line.
The memory of a report is the size of the records it reads and no longer a multiple of it.

## B1121: corrections to this record, and what did not move

Status:
done on 2026-10-31 (UTC).

Corrections to B1090 to B1096,
made in place:
the flush claim and the signal (B1090,
B1113),
`typography's 11 became 6` (B1090),
the wording of the census walk's depth (B1090 and B1093),
the control characters and the debug and trace records in B1091 (B1117,
B1118),
and the launcher edits of B1096,
which the lead made.
The index lines of B1090,
B1091 and B1093 in `audit-ledger-entries-by-id.md` were over 120 columns and are shortened with their headings.

Commit messages that say something this record corrects:
`ab7b3d8389` says 5 new cases in the glossary reader suite;
there are 4.
`6d57f27934` also holds the lint spellings of `adjudication/src/child-environment.test-fixture.ts`,
a test fixture,
which its message does not list.

No cache version moved:
this round changed where records are written and how they are shown,
no key material,
no stored record shape,
no prompt,
no sheet and no decision rule.
The pipeline digest moves as it does for any commit that changes a built file.
The editor fidelity bench and the other runners that capture a `.log` (B1099) were not started.
