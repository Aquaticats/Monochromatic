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
  The mutant now fails two cases.
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

Cache versions:
none moved.
`git log --extended-regexp -G '_CACHE_VERSION = [0-9]+' 5c40a5fcb9..HEAD` printed nothing,
and its control over `c265e8df95~1..c265e8df95` printed `c265e8df95`.
No prompt,
model call or cached step input changed.

## B964: docs that claimed what the code did not

Fixed on 2026-10-25 (UTC) in `80a8a4cf57`.

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
