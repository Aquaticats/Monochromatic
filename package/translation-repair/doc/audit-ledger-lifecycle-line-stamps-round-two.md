# Audit ledger: the pass's lifecycle line stamps, round 2

Round 2 of the work recorded in
[runner reports and probes](audit-ledger-runner-reports-and-probes.md) at B960,
which holds B961 to B964:
the findings of the independent re-check of round 1,
answered on `tr-w354-stamps` after the merge of main at `87409a1e2b`.
The re-check found no major finding.
Its minor findings and nits are B961 (a hand-edited attempt count),
B962 (tests that could not fail),
B963 (the lines of the pass itself,
the logger's lines and the cap wording)
and B964 (docs that claimed what the code did not).
The corrections in place are in the entries they correct,
in the file of B960:
its census citation and its list of what stayed open.

Round 3 answered the second independent re-check,
which also found no major finding.
Its three minor findings and its nits extend the entries they belong to rather than take new ids:
the residue of B961 (the largest safe integer as a count),
the nits of B962,
and two findings of B963 (the logger's lines that read no context,
and the window the cap census searches for `START`).

Part of [Audit ledger](audit-ledger.md),
which lists every file of this record.

## B961: a hand-edited attempt count ended the pass at its first entry, at every launch

Fixed on 2026-10-25 (UTC) in `39ae8f2608` (glossary),
with its cases committed red first in `dc6c221c15`.

Found by the re-check of round 1,
which wrote an attempts file holding `1.5`,
`-3`,
`1e999` and `1e20` and ran the built reader and counter over it.
`readAttemptMap` read any JSON number through,
`countAttempt` added one,
and `lifecycleStamp` refuses an attempt that is not a whole number from 1.
The counts became 2.5,
-2,
Infinity and 1e20,
and each was refused with `LifecycleAttemptError` at the `OVERLAP` line of `settleEntry`,
which is outside the pipeline's `try`,
so the pass ended at the first entry the file touched.
`writeAttemptMap` had already persisted the bad count,
so every relaunch repeated it.
A real pass reaches this only through a hand-edited `attempts.json`,
since the pass is the file's only writer and the runs-directory lock keeps passes single.

The fix is at the reader:
a count that is not a non-negative safe integer reads as 0,
which sorts the entry first as a non-number already did,
and the warning that already named the entry now also names the attempts file and the value
(`1.5`,
`-3`,
`Infinity`,
`100000000000000000000`).
The cases write the four values as text,
because `JSON.stringify` writes the `1e999` that parses to Infinity as `null`,
and a control keeps 0 and the largest safe integer quiet.

Round 3 (re-check 2,
minor 2 and a nit):
that control kept one count the stamp refuses.
`9007199254740991` is a safe integer,
and one more is `9007199254740992`,
which is not,
so `countAttempt` wrote an unsafe count and `lifecycleStamp` refused it at the same `OVERLAP` line,
again at every launch.
Fixed on 2026-10-25 (UTC) in `a56ca8f70a` (glossary),
with its cases committed red first in `7257b0ffb4`:
a count is usable only when it and its successor are safe integers,
so 0 to 9007199254740990,
and the warning says that range,
since "no non-negative whole number" called the largest safe integer one.
`countAttempt` throws the new `AttemptCountError`,
exported from the package,
rather than write such a count,
and leaves the count it was given.
The control moves to `9007199254740990`,
the largest count a file can hold.
`AttemptCountError` stays without the `messageNamesOnly` marker,
because it names an entry id the corpus chose,
and the inventory in `message-names-only.unit.test.ts` records why.
The nit that the warning's quoting of a text count was unpinned (mutant G4 printed every value with `String`)
is closed in `2a99097f3b`:
the cases expect `counts Mittens as "5",` beside `counts Mittens as 1.5,`,
and G4 fails the case by assertion,
which passes unmutated.

Cache versions:
none moved (B963 holds the search).

## B962: three tests that could not fail

Fixed on 2026-10-25 (UTC) in `dbc4198da0` (markup),
`5eaf96f435` and `ce95582627` (corpus-pass).
These commits change tests only,
so no case fails on its parent.
Each was shown to kill the mutant the re-check ran against round 1:
the mutated source was built,
the new case failed,
and the source was restored from the commit.

- Mutant M03 replaced `wallClockNow` with `new Date(performance.now())`,
  an instant in 1970,
  and survived the whole file of `lifecycle-stamp.unit.test.ts`:
  the case that asked for the default clock compared the stamp with `wallClockNow()`,
  the function the stamp calls,
  so any default agreed with itself.
  That is the substitution B78 forbids for a stamp.
  The case now asks that the default stamp end in Z,
  sort after `2026-01-01T00:00:00.000Z` and round-trip through `Date`.
  The only reads of the default clock are the ones the function under test makes,
  and none is compared with a second read,
  which also removes the tolerance of 60 seconds the case had.
  The mutant failed two cases in `dbc4198da0` and fails three at the head (re-check 2 counted
  `lifecycle-stamp.unit.test.ts:77`,
  `:152` and `:163`).
- Mutant M15 removed `wallClock` from the `settle({ entry, attempt, wallClock })` call of `runPassQueue`,
  so the queue's clock never reached the settlement,
  and every file of the package passed:
  no case gave the queue a clock and read what `settle` received.
  The new case reads the instant off the clock the settlement is handed.
  The mutant fails it with `TypeError: wallClock is not a function`.
- Mutant M21 hard-coded `attempt: 1` in the `defectsLine` call of `printSettledLines`.
  `defectsLine` was covered alone,
  and no case printed a page with defects at an attempt other than 1.
  `printSettledLines` is now exported through the package API so a built-artifact test reaches it (a small helper,
  as the rules allow),
  and two cases print a clean page and a page with two defects at a fourth attempt.
  The mutant fails the case with defects,
  by assertion on the `DEFECTS` line.

The re-check also said `CAP OVERRIDDEN` was pinned in four test files.
A search for the wording found three,
`corpus-pass-lines`,
`corpus-pass` and `corpus-pass-refusals`,
with six assertions between them (B963 changed all six).

Three slips of the commits themselves,
the first two recorded as the second re-check found them and the third by the fixer of round 3,
none amendable:
`ce95582627` is typed `test` while it changes `corpus-pass/src/index.ts` to export `printSettledLines`
(allowed,
so a built-artifact test can reach the helper,
but the type should have said so),
and the red commit `6450c31d15` is not lint clean,
because its built-pass fixture held a regex literal that oxlint
refuses (the fix commit rewrote it),
and `80ed32c810` of round 3 is typed `style` while it replaces a module-level flag and a `try ... finally` with an
`AsyncLocalStorage`,
a change of mechanism that its body names but its type does not.
The raised count of 12 in `real-clock-bounds.unit.test.ts`,
the dropped allow-list entry of `pass-entry-persist.ts`
and the `attemptText` line of `blank-text-tests.unit.test.ts` are scan bookkeeping,
not a loosened bound.

## B963: the lines of the pass itself and the logger's lines carried no time or no attempt

Fixed on 2026-10-25 (UTC) in `c789a50c6e` (markup),
`edbeb549da` (typography and corpus-pass),
`400e444f1b` (slice-cache)
and `73d02da2e8` (corpus-pass).
Their cases were committed red first,
in `a10629e5f1`,
`c963d383f6`,
`aefa9241b0` and `6450c31d15`,
and each failed on its parent by assertion on the missing field,
or on the missing export for the new stamp writer.

B960 stamped the lines of an attempt and left open the lines that stop the pass between entries,
the logger's lines,
and the wording of `CAP OVERRIDDEN`.
The re-check read the census again and judged each:

- `SOFT budget reached`,
  `SPEND CEILING reached` and `STOP every provider is out of budget` are the lines that say when the pass stopped
  starting entries.
  They carry `at=<time>` as their last field and no attempt,
  since they belong to the pass.
  `stopBeforeNextEntry` takes the wall clock,
  and `runPassQueue` appends the time to the gate's stop note when it prints it,
  because the gate builds the note's text without a clock.
- `START` and `DONE` carry `at=<time>` as their last field.
  It is the last field rather than the second because `cap-census-pass-run.ts` and `cap-census-report.ts` find a pass by
  the prefix `START tip=`,
  and the cache-version comments of the artifact,
  repair and lane packages cite the `START tip=` lines as evidence.
- The `INCOMPLETE <id>: <side> page absent at the pin` line carries `at=<time>` straight after the entry and never
  `attempt=`:
  no attempt has begun for an entry whose page is absent.
  It shares its first two words with the `INCOMPLETE <id> at=... attempt=<n>:` line of an attempt,
  so the absent field is the one thing that tells them apart,
  and a case pins the difference.
  `lifecycleClockStamp` writes the time alone,
  and `lifecycleStamp` builds on it.
- The logger's lines inside an attempt name it.
  `LogContext` gains `attempt` as text,
  `inEntryLogContext` takes it as a required number so a caller cannot forget it,
  `inSliceLogContext` copies it,
  and the root logger and the slice logger write `[entry] [attempt n] [lane slice i]`.
  The tag is a bracket of its own after the entry:
  the two readers that split a logged line on `] [` read the level and the time,
  which stay first and second.
  A candidate-ledger round records the same context,
  so its `context` gains the field,
  which no reader of that record parses.
- `the pass resumes` and the parked warning are tagged-logger lines and always carried the logger's time,
  so the census list B960 left open was wrong about them.
  They carry no attempt either way.
- `CAP OVERRIDDEN` reads "each attempt of an entry runs under N minutes",
  as `START` (`hard-scope=per-attempt`) and `CAP TOO TIGHT` already say.
- `CorpusPassInput` takes an optional `wallClock`,
  so the pass-run cases fix the time of every line.
  The cases of the built command run in a child process whose clock they cannot fix,
  so the fixture takes the times out of the output and hands them back,
  and the cases compare the rest of each line whole.

Round 3 (re-check 2,
minor 1):
the attempt reached only the loggers built on the log context.
The pass's own per-entry loggers were plain `tagged({ tag: entryId })` calls on the default logger,
which never reads the context:
eight in `pass-entry.ts`,
three in `pass-reseat.ts`,
two in `pass-decline.ts`,
and one each in `pass-prepare-reseat.ts`,
`pass-lanes.ts` and `pass-insertion-reseat.ts`.
So `ARCHIVE ORIGINAL`,
`FOOTNOTES` (written through the logger `preparePassEntry` is handed),
`JUDGE SEATS` and the pass-level warnings printed no `[attempt n]`,
though the docs,
the logger bullet of this entry and the prevention rule said they did.
The round 1 census had named exactly these lines as the ones wanting the attempt.
Fixed on 2026-10-25 (UTC):
every one is now `sliceTagged({ l: tagged({ tag: entry }) })`,
and `runEntryPipeline` builds one such logger and hands it to its stages.
The commits are `eeac4078fd` (typography),
`9770c92b9d` (slice-cache) and `33b5ed4855` (corpus-pass and the test fixture),
with their cases committed red first in `644b50b094`,
`8cb7513e88` and `2fe404c947`,
and the scan in `09e50ec7d0` (answered in `0c5f71edbf`).
The first typography version drew eight errors from oxlint (a module-level `let`,
a `try ... finally`,
arrow functions),
so the flag became an `AsyncLocalStorage` in `80ed32c810`.
The red commit `644b50b094` fails on its parent by assertion on the doubled bracket.

- Decision:
  a line gets the attempt,
  lane and slice tags once.
  `repairPreparedDocument` and `translateDocument` slice-tag the logger they are handed,
  and `pass-lanes.ts` hands them the entry logger,
  so on the parent's `sliceTagged` a lane line would have read
  `[Tabby] [attempt 2] [repair slice 3] [repairPreparedDocument] [attempt 2] [repair slice 3] ...`.
  The logger called first now writes the tags and runs the rest of the line's journey under an `AsyncLocalStorage`,
  and a slice-tagged logger beneath it writes the line as it is,
  so lane lines keep the order they have always had.
  Two other shapes were weighed and not taken:
  dropping the `sliceTagged` of the two lanes,
  which would take the lane and slice tags from every bench and probe caller that hands the lanes a plain logger,
  and handing the lanes a plain entry logger while the pass writes its own lines through a slice-tagged one,
  which would leave a plain `tagged({ tag: entryId })` that the scan must allow.
- Cases,
  one per family.
  A whole `settleEntry` run inside attempt 2 expects the bracket on every line that names the entry (26 lacked it on
  the parent) and expects none to hold it twice.
  The seams of `pass-seams-reseat.unit.test.ts` (preparation,
  insertion admission,
  lanes) and the hooks of
  `pass-reseat.unit.test.ts` (`lanesHooksFor`,
  `translateReseatFor`) do the same for the loggers they build,
  and the decline case in `pass-decline.unit.test.ts` for `ARCHIVE ORIGINAL`.
  `entry-log-lines.test-fixture.ts` collects the lines from `console.info` and `console.warn`,
  one line each,
  since the logger hands a run of lines of one level to the console as one call.
- The scan,
  `entry-logger-scan.unit.test.ts`,
  reads the family source and finds an entry logger built as `tagged({ tag: entryId })` or `tagged({ tag: entry.id })`
  that no `sliceTagged` wraps,
  modelled on the log root scan.
  It found sixteen on the parent and none now.
  It reads two spellings of the tag and not a template literal.
  It covers the one logger the cases cannot reach,
  the debug line of `pass-lanes.ts`,
  which the logger writes to stderr.
- Mutant D1 makes `sliceTagged` write its tags whether or not a logger called first has written them.
  With typography and corpus-pass rebuilt from it,
  the typography case,
  the lanes seam case and the whole-run case fail by assertion,
  and all three pass unmutated (`202c97d8e1`).
- One copy of the logger's state in the command that runs a pass.
  The built command `corpus-pass/dist/final/node/corpus-pass.mjs` bundles typography with the provider,
  repair and translate-lane code,
  and `rg --only-matching` over it finds one `sliceTagsWritten = new AsyncLocalStorage`,
  one `contextRoot`,
  one `inEntryLogContext`,
  and one `repairPreparedDocument` and one `translateDocument`,
  so the lanes and the entry loggers of a real pass share the store that holds the once-per-line mark.
  The library entry `index.mjs` imports typography from the package instead,
  so the unit tests and the typography package share the other copy.
  No built file holds two.
- Readers of the newly tagged lines.
  `JUDGE SEATS`,
  `ARCHIVE ORIGINAL`,
  `FOOTNOTES`,
  `CORPUS NAMES`,
  `FRONT MATTER` and the re-seat lines were searched for in the non-test `.ts` of `run`,
  `bench` and `probe` and in `doc/runbook`.
  Only emitters and comments in `run/src/corpus-run/run-seats*.ts` name them,
  and nothing anchors on the position of the entry tag before them.
  The reader survey of re-check 2 (the timing,
  cap census and spend readers) covers the stream,
  spend and round lines.
- Not covered by a case:
  the `contextRoot` logger neither sets nor reads the once-per-line mark,
  and nothing wraps a `contextRoot` logger in `sliceTagged` today.

Round 3 (re-check 2,
minor 3):
the cap census found a pass by `START tip=` in the first 4096 bytes of its log.
A pass prints one `INCOMPLETE` line for each entry whose page is absent at the pin before `START`,
and each grew from 66 to 94 bytes with `at=`,
so a pin with 44 to 60 such entries moved the marker out of the window with the marker itself unmoved.
Fixed on 2026-10-25 (UTC) in `c6f996904f` (run),
with its case committed red first in `c515cbe961`:
`capCensusPassRunReading` reads the log line by line and stops at the marker or after 2000 lines,
a bound in lines that no field added to a line changes,
and a log without the marker costs 2000 lines of reading and not its whole text.
The number 2000 is a choice and not a measure of any pin:
a pin with more page-absent entries than that is read as no pass run.
The cases write 200 page-absent lines before `START` (red on the parent),
a marker as line 2000 (read) and as line 2001 (not),
and a line wider than the old window before the marker.
A bare carriage return also ends a line for the reader,
which the old split on line feeds did not;
a log with `START tip=` right after a lone carriage return would now count.
The lint fix is `7176ec3e25`.

Cache versions:
none moved.
`git log --extended-regexp -G '_CACHE_VERSION = [0-9]+' 5c40a5fcb9..HEAD` printed nothing,
and its control over `c265e8df95~1..c265e8df95` printed `c265e8df95`.
No prompt,
model call or cached step input changed.
Round 3 changed no cache version either:
the same search over `a60375aeb9..HEAD` and over `5c40a5fcb9..HEAD` printed nothing,
and the control printed `c265e8df95` again.
The logger's tags are no part of any cache key or of the pipeline digest's inputs but the built bytes.

## B964: docs that claimed what the code did not

Fixed on 2026-10-25 (UTC) in `80a8a4cf57` (the first three claims)
and `70e839ad3f` (the commit of this file,
which also replaces the fourth).

The re-check found four claims in the docs of round 1 that the code did not bear out:

- The census,
  B960's list of what stayed open,
  `configuration.md` and the pass runbook said `the pass resumes` carries no clock time.
  It is a tagged-logger line and carries the logger's.
  The docs now name the console lines that carry none before B963 and say the logger's lines carry the logger's time.
- The census said the after-run-exits runbook reads `TALLY ms=` only.
  It quotes the `DESTINATIONS` and `DEFECTS` shapes,
  which moved:
  both now carry `at=<time> attempt=<n>` after the entry.
  The runbook quotes the new shapes.
  Its `grep '^DESTINATIONS '` and `grep '^DEFECTS '` recipes were not affected.
- The docs said a stamp's `at=` is the time the line was printed.
  A settled `TALLY` is rendered before its artifact and page are persisted and printed after,
  so its `at=` is a little earlier than the `DESTINATIONS` line beside it.
  The docs say so.
  Moving the read to the print would change one line's code to match a doc,
  and the time of rendering is the one `settledTallyLine`'s own TSDoc already states.
- B960 cited a scratch file for its census,
  outside the repository,
  which nobody else has.
  The entry now describes the census in its own words.

The rule the entry B960 added is widened in `mistake-prevention-messages-labels-and-counts.md`,
"A lifecycle line a reader windows by time":
a line a reader windows by time carries a clock time whether it belongs to an attempt,
to the pass or to an entry no attempt has begun for,
and a line of an attempt also carries the attempt.

Recurrence:
`mistake-prevention-messages-labels-and-counts.md`,
"A lifecycle line a reader windows by time".
