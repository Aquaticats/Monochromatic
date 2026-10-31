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

Part of [Audit ledger](audit-ledger.md),
which lists every file of this record.

## B1090: a pass wrote nothing of its own, and its console capture was the record every reader parsed

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

A throw out of the pass after the log is open is logged as `the pass stopped: <refusal text>` while the log stands,
and thrown on unchanged.
The console gets the refusal only after the run log is closed,
so without this line a pass refused by a generation drift left no word of why in its folder.

What the run folder holds after a pass:
`log/<UTC day>/<timestamp>.<uuid>.log.jsonl`,
one file for each process that opened the run log,
named by the sink and never rotated,
one JSON object per line holding exactly `level`,
`message` and `timestamp`.
The day directory is the sink's own and cannot be turned off.
A pass and a `budget-sample` taken into the same folder each leave a file of their own beside the other's.

A correction to the commit messages:
`3679320e90` says it turns "the 40 failing cases of the previous commit" green.
The failing cases of `7c51effbe0` are 24
(typography 11,
run 10,
slice cache 2,
corpus pass 1),
and typography's 11 became 5 in `3679320e90` when its three reader suites moved to glossary with the reader.
`a8c99f75be` says the cap census walk "goes five levels down a named directory and not three":
the bound `MAX_DEPTH` went from 3 to 5,
and the case that holds it went from reaching directories four levels below the root to six.

## B1091: how a reader finds the records of one run, and what it says when there are none

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
The one difference from a console capture is that the console sink writes control characters other than the line feed
and the tab as `\uXXXX` and this does not:
a message reaches the parser as it was logged.

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

## B1093: the five readers that parsed a pass's console read its records, with the same reports

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
  The walk goes five levels down a named directory,
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

## B1096: what is console only, and what the launcher has to change

Status:
recorded on 2026-10-31 (UTC),
the launcher is outside the repository and was read,
not edited.

Console only,
with nothing that parses it:
the seat report and the run end reports that `reportingRefusals` prints to stderr after the pass body returns,
the refusal line itself (the `the pass stopped` record of B1090 carries its text),
the build lines of the mise task (`rolldown v...`),
and the lines a probe prints through `artifact-pool.ts` (`POOL ...`),
which the pass never calls.
The readers match only these markers:
` METERS `,
` SPEND `,
the stream completion marker,
` round: `,
`SLICE-COST` and `START tip=`.
None of the console-only lines carries one.

The launcher `/var/home/user/temp/agent/tr-fan/s245/split/launch-rotation.ts` has to change in these places:

- Drop `--property=StandardOutput=truncate:${log}`,
  and the `existsSync(log)` guard against an earlier capture.
- Read START from the records:
  poll `${runsDir}/log/*/*.log.jsonl` and parse each line as JSON,
  taking the record whose `message` starts with `START `,
  not a line that starts with `START` in a capture.
  The digest check on `pipeline=${FROZEN} ` then reads that message.
- The `rolldown v` lines come from the mise build the unit runs.
  They reach the unit's journal and no record,
  so the bundler version is read from `journalctl --user --unit=<unit> --output=cat`.
- The `/\[error\]/` early exit reads `level === "error"` of the records.
  A stderr line of the pass is no longer in any file the launcher reads.

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
