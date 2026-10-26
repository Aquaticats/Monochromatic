# Audit ledger: the pass's lifecycle line stamps (B961 to B964)

The file keeps the name it was created under,
`audit-ledger-lifecycle-line-stamps-round-two.md`,
since the record of B960 and the indexes cite it by that name;
it holds the entries B961 to B964 whatever the round that last touched them.

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

Round 4 answered the third independent re-check,
which found no major finding either.
Its minor finding and its nits extend the same entries and take no new id:
the message of the attempt-count refusal and the unpinned range text (B961),
two slips of round 3's commits (B962),
and,
in B963,
the logs the cap census counted as pass runs,
its unbounded line,
the order of the tags on a line,
the spellings the entry-logger scan missed,
the guards that read too little,
and the logger root that a lane could wrap.

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

Round 4 (re-check 3,
nits 1 and 5):
`AttemptCountError` is thrown in the queue's `attempt` callback,
between two entries and outside any `try`,
so the pass ends and the only catch is `reportingRefusals`,
which printed the message under "a fault in the command".
The message named the entry and the count and nothing else.
It now names the entry,
the file and the count,
says the pass ends before the entry runs,
says the next launch reads the count as 0 beside a warning naming the entry and the file
(so the entry sorts first and its attempts are counted again from 1),
and says the operator can launch again or set the count to a whole number smaller than 9007199254740991 first.
`countAttempt` takes the attempts path for it and the queue hands it the path it persists to.
Fixed on 2026-10-25 (UTC) in `c15e8b73ac` (glossary and corpus-pass),
with its case committed red first in `555f01970e` (it fails by assertion on the first piece of the message).
A case through `runPassQueue` expects the rejection to name the file the queue was handed.
The error stays without the `messageNamesOnly` marker for the reason round 3 gave,
so the label "a fault in the command" stays;
the message is what a person reads.
The range text of the warning ("no whole number from 0 to 9007199254740990") is pinned in `d4b5ae50ae`
(test only):
mutant G4 of re-check 3,
which named `MAX_SAFE_INTEGER` as the end of the range,
fails the case by assertion with glossary rebuilt from it,
and the case passes unmutated and again once the source is restored.

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
Two more slips of round 3's commits,
recorded by re-check 3 and not amendable:
the subject of `7257b0ffb4` is exactly 100 columns where the rule is under 100,
and the body of `0c5f71edbf` quotes the phrase that the position-reference scan had flagged in a source comment.
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
  The order `[entry] [attempt n] [lane slice i]` is the order the context writes its three tags in;
  the printed order of a line also holds the function tags the code adds,
  and round 4 records it in its paragraph of this entry on the order of the tags.
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
  `entry-log-lines.test-fixture.ts` collects the lines from `console.info` and `console.warn`
  (round 4 added `console.error` and `console.trace`),
  one line each,
  since the logger hands a run of lines of one level to the console as one call.
- The scan,
  `entry-logger-scan.unit.test.ts`,
  reads the family source and finds an entry logger built as `tagged({ tag: entryId })` or `tagged({ tag: entry.id })`
  that no `sliceTagged` wraps,
  modelled on the log root scan.
  It found sixteen on the parent and none now.
  It read two spellings of the tag and not a template literal;
  round 4 replaced the text match with a reading of the syntax tree,
  recorded in its paragraph of this entry on the scan.
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
  The second half of this bullet was wrong:
  `repairPreparedDocument` wraps one when its caller omits `parentLogger`,
  and round 4 fixes the doubled bracket that composition printed
  (round 4's paragraph of this entry on the root logger).

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

Round 4 (re-check 3,
minor 1):
the head of 2000 lines counted test-suite logs as pass runs.
A test suite runs fixture passes in its own process and prints their `START` lines into its log,
at line 267 to 1900 of it,
so once the head grew the marker alone made 515 of them pass runs and the report's tally of pass-run logs moved by 47 percent.
Measured over every `.log` under the agent root (124,509 files,
the first 8 MiB of each),
the old head of 4096 bytes accepted 450 logs,
37 of them holding test runner `[PASS]` lines,
and the head of 2000 lines accepted 1104,
552 of them holding `[PASS]` lines.
Every one of those 552 carries `START tip=` with forty `f` digits,
`files=1` and one pipeline digest:
the `START` line of a fixture pass.
107 more are the re-check's own probe logs,
with forty `a` digits,
and the other 445 carry a varied commit id and hold no `[PASS]` line
(308 of them the old head accepted and 137 it did not,
since a mise run prints about 254 build lines before `START`).
So the reader counts the first line that opens `START tip=` only when the field after it is a commit id a repository could give:
forty lowercase hexadecimal digits that are not one digit repeated.
Over the same 1104 logs the built reader refuses the 552 suite logs and the 107 probe logs and reads the 445.
The 37 suite logs the old head accepted are refused too.
Fixed on 2026-10-25 (UTC) in `65f93a3e60` (run),
with its cases committed red first in `8b7fa4e445`:
a real pass behind 254 build lines (read on both sides,
a control),
a suite log whose `START` line is the fixture's,
commit ids that are no commit id,
and the report case that leaves such a log out of the pass-run count.
The three suite-shaped cases fail on the parent by assertion.
The limit is recorded in the TSDoc:
a fixture that printed the repository's own head,
or invented a varied forty-digit id,
would pass the test.
The cases that wrote `tip=tabby` now carry a real-shaped id.

Round 4 (re-check 3,
nit 7):
the reader held each line whole,
so a log with no line break was read entire:
18 of the 2412 logs over 1 MiB under the agent root have none in their first MiB
(three are 13.4 MB browser store files),
and the built reader answered `not-a-pass-run` on one in 91 ms with 42 MiB of resident memory.
The head is now read in blocks of 65536 bytes,
one byte to a character since only ASCII is looked for,
and a line longer than 65536 bytes ends the search with the log counted as no pass run.
The longest line up to and including `START` among the 445 real pass logs is 378 bytes,
so the bound is about 170 times it,
and it is a choice.
The same log answers in 8 ms with 2 MiB.
Fixed in `80d45b389b` (run),
with its cases committed red first in `e48f8ac881`:
a log of one 13-megabyte line that opens with a `START` line,
and a `START` line of exactly 65536 bytes (read) and one of 65537 (not a pass run).
A lone carriage return still ends a line,
and a carriage return and line feed split between two blocks count one empty line more,
which moves the head of 2000 lines by that line.
A case pins a `START` line whose marker is cut by the end of a block;
it passes on the parent too.

Round 4 (re-check 3,
nit 2):
the order of the tags on a line.
The docs said the lines of an attempt read `[<entry>] [attempt <n>] [<lane> slice <i>]`,
"a bracket of its own after the entry".
That is the order the context writes its three tags in,
and the printed line holds the function tags the code adds among them.
Over the re-check's capture of eight settled entries,
four shapes cover the lines:
a module root logger writes `[<module>] [<entry>] [attempt <n>] [<lane> slice <i>]` and then its function tags;
an entry logger,
and a stage logger tagged over it,
write `[<entry>] [attempt <n>] [<lane> slice <i>]` and then the function tags
(`[Tabby] [attempt 2] [repair slice 3] [editor] stretches` in `log-context.unit.test.ts`);
a lane that tags the logger it was handed with its function and slice-tags that writes the function tags first
(`[Tabby] [runDocumentLanes] [repairPreparedDocument] [attempt 2] [repair slice 0] SLICE-START ...`,
pinned in the same file);
and the contest writes `[Tabby] [attempt 2] [contest slice 0] [contestDocumentLanes] [slice 0] [contestLaneSlice] ...`,
where `contest` is the lane name,
so `[contest slice 0]` is the lane-slice bracket and the later `[slice 0]` is a tag the contest code writes itself.
The lanes are `repair`,
`translate`,
`consolidation` and `contest`.
On every line the attempt bracket comes after the entry and names the attempt once,
and the lane-slice bracket,
where the line is inside a slice,
follows it at once;
only the function tags move.
A recipe that must find every line of an attempt anchors on the `[attempt <n>]` bracket and never on its place after the entry.
`configuration.md` and the pass runbook now say so.

Round 4 (re-check 3,
nit 3):
the scan of the entry loggers.
Re-check 3 planted one plain logger at a time in the pass's source:
`tagged({ tag: entryId, })` was found,
and a template literal,
an aliased import of `tagged`,
another key order,
`pair.id`,
a const alias of `tagged` and tight spacing were not.
The text match also turned off without a failure when a formatter changed the spacing.
The scan now parses each file of `corpus-pass` and `slice-cache` and finds every call of `tagged`,
under whatever name the file binds it,
whose `tag` names an entry
(the identifier `entryId`,
or a read of a member called `id` or `entryId`,
followed through the file's const names)
and which is not the `l` of a call of `sliceTagged`,
under its own name or an alias,
with any parenthesis or assertion between them.
The finding lives in `entry-logger-finding.test-fixture.ts`.
The six spellings are planted as fixtures,
with two more (a const set to `entry.id` and a call inside `String`),
and each is found;
the controls that must stay quiet are the wrapped forms and a string,
a function name and a slice index as the tag.
Run over the source of `644b50b094`,
before the entry loggers were wrapped,
the finding totals 16 with the per-file counts of the first scan,
and over the head it totals 0.
Fixed in `0c3b3ae14c` (source-scan),
with its cases committed red first in `f15d1f374a` (the planted spellings are found by none of them on the parent),
and the bookkeeping the scans asked for in `a23e9fba91`.
What it still cannot see:
a tag built from the entry by a name it does not know,
and a function that receives an `l: Logger` and never calls `tagged`,
which only the behavioural cases of the whole run,
the seams and the hooks reach.

Round 4 (re-check 3,
nit 4):
the guards were thinner than their names.
`entryLinesWritten` stubbed `console.info` and `console.warn` only,
so mutant F1 (the fixture ignores `console.warn`) left every case green and an error or fatal line was never read.
It now stubs `console.info`,
`console.warn`,
`console.error` and `console.trace`
(a fatal line goes out as an error;
debug lines go to the process's stderr and only a verbose run lets them through,
and trace and debug are filtered by the default level,
so no case can read them).
Fixed in `6be915cc48` (test-fixture),
with its own test file committed red first in `6e874eeff7`:
it writes a line at each level a run prints by default and fails on a copy of the parent's fixture by assertion
(the error and fatal lines are missing),
and fails for F1 with the fixture restored afterwards.
The whole-run case settled the front-matter entry.
Re-check 3 probed that entry with its own scripted client and counted 24 bracketed lines and no model call;
the case itself,
measured in round 4,
served 22 model calls and captured 52 lines,
3 of them lane lines (`SLICE-START`) and none a contest or consolidation line.
The body of `504ab3cd7c` repeats the re-check's figures and not these,
and cannot be amended;
this entry replaces them.
The case now settles `ENTRY` through the file's scripted client and asserts that models were asked and that the
captured lines include a lane line (`SLICE-START`) and a contest line (`[contest slice `);
`504ab3cd7c` changes the test only,
and mutant S7 (`settleEntry`'s entry logger left plain,
corpus-pass rebuilt from it) fails it by assertion on the missing contest line.

Round 4 (re-check 3,
nit 6):
the root logger that a lane could wrap.
`repairPreparedDocument` defaults its parent logger to a `contextRoot` logger and builds its own as
`sliceTagged(tagged({ l: parentLogger }))`,
so a caller that omits the parent logger got
`[translation-repair-pipeline] [Tabby] [attempt 2] [repair slice 3] [repairPreparedDocument] [attempt 2] [repair slice 3] kneads`.
The only caller that omits it is `repairTranslation` (`repair/src/repair-entry.ts`),
which has no production caller,
so the double print was latent and older than round 3;
the pass hands the lanes its entry logger.
`contextRoot` is now `sliceTagged` over a logger that writes the entry alone:
called first it writes the entry,
attempt and lane slice as before;
beneath a slice-tagged logger it writes the entry and leaves the brackets to the logger called first,
so the composition prints `[translation-repair-pipeline] [Tabby] [repairPreparedDocument] [attempt 2] [repair slice 3] kneads`.
Fixed in `d3a116fb39` (typography),
with its case committed red first in `eef95c313b`
(the lane line holds the attempt twice on the parent,
so no line has the expected shape);
the case also pins the root's own order for a stage logger tagged over it.
Typography is bundled into the provider,
house-rule,
repair,
provider-stream and corpus-pass builds,
which were rebuilt and run (the gates are in the round's report).

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
Round 4 changed no cache version either:
the same search over `efdf70f61f..HEAD` and over `5c40a5fcb9..HEAD` printed nothing,
and the control printed `c265e8df95` once more.
The cap census,
the attempt-count message,
the scan and the fixtures touch no prompt,
model call or cached step input,
and the root logger's change moves a log line only.

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
