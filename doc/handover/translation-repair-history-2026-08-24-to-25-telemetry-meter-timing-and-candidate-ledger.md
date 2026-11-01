# Translation repair history 2026-08-24 to 2026-08-25: the telemetry the pipeline records and the readers built for it

The provider availability meter recorded,
readable and samplable,
then extended to carry its numbers,
the run timing lines that say where a run's wall-clock went and the report that reads them,
and the candidate ledger that keeps what each model wrote,
with its writer and its reader.

Part of [Translation repair history: 2026-08-24 to 2026-08-25](translation-repair-history-2026-08-24-to-25.md),
which lists every file of this record.

Historical evidence only:
no next-action list here is a current instruction,
and the stop condition in the index applies.

## `#201` landed: availability is recorded, readable, and samplable

Four commits on 2026-08-24,
all pushed.

### What shipped

-   `provider-budget.ts` logs `METERS synthetic=<state> hyper=<state>` at `info`
    on every meter reading,
    once per sixty-second freshness window.
    Since 8f774f34f the line also carries the numbers each state was read from;
    see the `#202` section.
    It was already computing this and saying it at `debug`,
    which runs do not record.
-   The meter now has three states.
    `meterRecordOf` (named `meterStateOf` until 8f774f34f) reports
    `wet`,
    `dry` or `unreadable`,
    and `routesAsDry` maps that back to the routing bit.
    Both are exported `@internal` and pinned by six cases.
-   `meter-sample-read.ts` parses those lines back into samples.
    No regex.
-   `meter-dry-span.ts` computes duty cycle and outage spans as bounded ranges.
-   `meter-report` reads any number of logs and reports both providers.
    Spends nothing.
-   `budget-sample` takes one reading between runs.
    Spends no generation.

### The defect the round-trip found

Reading a real sampler log back reported `unread=1` on an intact log.
The sampler's own summary said "the METERS line above is the record",
and the parser counted that sentence as a record it could not read.

A record is now recognised by its first field parsing.
Prose does not do that;
a record truncated part way through its second field still does.
Confirmed both ways:
the real log now reads `unread=0`,
and the fixture's planted undated record still reads `unread=1`,
so the gate did not simply switch the detection off.

### State

-   Lint 0 warnings 0 errors on 845 files,
    types clean,
    619 suites pass,
    0 fail.
-   One reading exists so far:
    2026-08-24T18:17:35.383Z,
    `synthetic=wet hyper=dry`.
    Charm Hyper has been dry all day;
    Synthetic is up.
-   The record only grows from here,
    so the numbers are worth reading again after the next few passes
    rather than now.

### What is left

-   `#200`,
    an editor-role calibration,
    is still open and still needs its shape decided.
-   `#94`,
    the slice rename,
    is still deliberately deferred.

## `#202`: the record carries the numbers, not only the verdict

One commit on 2026-08-24,
8f774f34f,
pushed.

### How it was found

Checking whether Charm Hyper was really out during the `#200` calibration,
the record answered `METERS synthetic=wet hyper=dry` and nothing else.
That is exactly what a wrong threshold in `budget-routing.ts` would also print,
so the only way to tell was a live `GET /v1/credits`,
which answered:

```json
{
  "balance": 0
}
```

The meter was right.
The point is that the record could not say so,
and for any moment already past there is no second call to make.
That is the same failure `#201` was opened on,
one level down:
a reading was being computed,
used,
and then dropped.

### What shipped

-   `syntheticMeterLevel` and `hyperMeterLevel` in `budget-routing.ts`
    render `key=value` tokens from the same snapshot
    the dryness verdict is read from.
    One read,
    so a verdict and its numbers cannot describe different moments.
-   `meterStateOf` became `meterRecordOf` and returns `{ state, fields }`.
    A meter that did not answer reports no fields;
    `state` already carries that it did not.
-   The record now reads:

    ```text
    METERS synthetic=wet hyper=dry syntheticWeekly=97% syntheticFiveHour=48/50 syntheticThrottled=no hyperBalance=0
    ```

-   `meter-sample-read.ts` defines a level as any field whose value
    is not a meter state,
    so a field added later reaches a human instead of being dropped.
    The report's dedupe key covers levels,
    so two genuinely different readings at one instant no longer collapse.
-   `meter-report` prints the level at both ends of the record.
    Two ends,
    because the last says what the budget is now
    and the first says which way it moved to get there,
    which separates a budget this run drained
    from one that was empty before it started.

### Why throttling earned its own field

Synthetic reads dry for three reasons that route identically:
the weekly budget empties,
the rolling window empties,
or the account is actively throttled while both still have room.
`syntheticThrottled=yes` beside a full window is the third,
and nothing in the record could previously show it.

### State

-   Committed before verification,
    per `AGENTS.md` `GCE`.
-   Build,
    lint,
    type-check and suite are OWED.
    None can run while the `#200` calibration holds `dist`:
    every mise task declares `depends = ["build"]`
    and would rewrite the bundle underneath a live run.

## `#215`: a run now says where its wall-clock went, and a CLI reads it back

The task's own text quoted the owner's standing instruction:
"If you found out we're not logging enough,
you should change the pipeline to log enough."
Following that instruction found a second gap the task had not named.

### What the log could not say, measured before changing anything

The live full-roster calibration log was surveyed for every line shape it carries.
Three tags appear in it and no others:
`reportStreamProgress` on 2405 lines,
`takeReading` on 161,
`exchangeWithRetry` on 4.
`takeReading` is the availability meter,
polled every minute or two.
There is no dispatch line,
and there is no round boundary line.

So the log records when each call ENDED and nothing else about time.
A call's start is not recoverable,
a round's extent is not recoverable,
and the question the audit `doc/audit/every-volume-guard-is-blind-to-one-model.md`
opened on has no answer in the data.

A lower bound was computable and was computed,
before the fix,
as a control.
Each completion line carries `firstByte` and `maxGap`,
and the largest gap falls
strictly after the first byte,
so their sum bounds the call's duration from below.
Intervals built from that sum are subsets of the true ones,
so overlaps counted on them
can only undercount:

```text
calls                        2405 (2349 completed, 56 cut)
log span                     6.15 h
summed duration FLOOR        2.37 h
mean concurrency FLOOR       0.39
peak concurrency FLOOR       9
```

Read that as a floor and nothing more.
It says at least nine calls were once in flight together,
and it cannot say what the figure actually is.

A method note worth keeping:
the first version of this sweep matched 1043 of 2405 lines and reported a peak of 5.
The label pattern required a colon,
so it silently dropped every non-`hf:` model.
An uncapped `grep --count` on the raw marker is what caught it,
which is the `QRY` rule paying for itself.

### The two lines that landed

`StreamProgress` gained `elapsedMs`,
computed as `Date.now() - state.armedAt` in
`armIdleGuard`'s `progress()`,
and `reportStreamProgress` prints it directly after
the outcome.
With the line's own timestamp that gives every call an interval,
which is what an
overlap count needs and what no completion line carried before.

`runGatherRound` now writes a round line,
which nothing did:

```text
editor round: 6/7 heard, 91402ms total, 61401ms to quorum, 30001ms in grace
```

The split is the point.
Time before quorum is the round doing its work;
time after it is the round waiting on
voices it may never hear.
Only the second is straggler cost,
and one round duration cannot tell them apart.
The audit could bound that cost only from above,
at the grace window times the number of
cut events,
and recorded that confirming it "needs the dispatch timestamps the run does
not currently record".

Both lines carry ids,
counts and durations only.
No corpus wording enters either.

### The reader, because a log nothing reads is not a measurement

`run-timing-parse.ts`,
`run-timing-read.ts` and `run-timing-report.ts`,
with the
`run-timing-report` mise task,
mirror the `spend-` and `ledger-` families.

Every read names what it found rather than returning an absence.
A completion line with no duration and a line that is not a completion at all are
different facts about a log:
the first says the run predates `#215`.
Folding them together would let a mixed archive's readable half be reported as the whole,
which is the shape of error this project has hit before.
The house `no-nullish-union` rule is what forced the discriminated union,
and it made the
reader better:
`readRunTiming` used to check `undefined` and then re-inspect the text to
count untimed lines,
and now one read decides all three outcomes.

Boundary verification,
all three states:

```text
new-format fixture   2 rounds, 29.0% of round time in grace, 1 voice lost,
                     mean 1.05 in flight, peak 2, 21.00s of calls across 20.00s of run
live pre-215 log     NO ROUND LINE, 2533 completion lines carry no elapsed field,
                     NO TIMED CALL
no argument          throws, naming the usage
```

Every figure in the fixture row is hand-computed from three intervals at
`[0,10]`,
`[2,8]` and `[15,20]` seconds,
not recorded from a run,
so a change in the sweep fails the case instead of moving the target.

### GFP

Both new guards were shown to fail with the guard removed and to pass with it restored.

```text
elapsedMs set to 0            stream-cut and stream-idle-guard both exit 1
                              "expected +0 to be at least 20"
round line deleted            stage-round exits 1, both cases
                              "expected exactly one round line, got 0"
```

The idle-guard failure is non-vacuous:
`firstByteMs` read 20 from a real wait,
so the assertion compared a measurement against a constant rather than zero against zero.

### What is owed

The measurement itself.
Achieved concurrency and the real straggler cost cannot be computed until a run emits the
new lines,
and the calibration now running was launched from the old build.
The audit's 1.45 hour figure stays an upper bound until then.
This is recorded as owed,
not predicted:
the point of the two lines is that the answer was unknown,
and it still is.

### Suite

676 PASS,
0 FAIL,
exit 0.
Lint 0 warnings 0 errors.
Build clean.

## The pipeline now keeps what each model wrote (`#212`, 2026-08-25)

### The gap, found by hitting it

Asked to confirm or reject that one seat was weak for this job,
the honest answer available from the archive was that nothing could answer it.
No run this project has ever made kept a single line any model produced.

The producer standing says a seat was preferred on 3.0% of disinterested ballots.
It cannot say whether that seat wrote something WRONG
or merely something nobody picked as the single best of ten.
Those are different findings with different remedies.
Every archived artifact predates model attribution,
neither calibration writes candidates to its run directory,
and the log names only the WINNING candidate's author,
so a losing candidate cannot be joined to the model that wrote it.

Re-deriving the evidence meant buying fresh calls for something a finished run had already paid for.

### What now records it

`candidate-ledger.ts` writes one JSON file per judged contest into
`${TRANSLATION_REPAIR_RUNS_DIR}/ledger/`,
holding every candidate's exact rendered text,
every model behind it with composites expanded,
every ballot with its reason verbatim,
and the winning position or that the round declined.

ONE HOOK COVERS EVERY CONTEST.
The translate lane,
both editor paths,
the refiner and the fidelity judge all route through
`selectBestCandidate`,
so nothing has to be remembered per caller.

### Two shapes this had to take, and why

A WRAPPER,
NOT A HOOK IN THE CASCADE.
The deciding function leaves by six returns,
five of them declines.
Threading a write through each would be five chances to miss the sixth.
`candidate-select.ts` now exports `decideBestCandidate` with its logic untouched,
and `candidate-select-record.ts` wraps it.
The wrapper lives in its own module because `candidate-select.ts` sits at 269 of its 300 permitted
code lines and restating the request type there would breach the cap;
`Parameters<typeof ...>` borrows the signature instead of copying it.

IT NEVER RAISES INTO THE SELECTION PATH.
A pipeline that failed a slice because its telemetry could not write would be worse than one with
no telemetry,
so every failure is caught,
named and swallowed.
The test for that matters more than the success cases:
it points the run directory at a path where a FILE sits where the directory belongs,
and asserts the caller is undisturbed.

WITH NO RUN DIRECTORY NAMED,
NOTHING IS WRITTEN.
That is the ordinary path for every unit run and every probe,
not an edge case.

### What it holds, and where it must not go

Candidate text is a rendering of a corpus passage,
so the ledger holds unlicensed corpus wording exactly as the settled artifacts already do.
It lands under the run directory,
outside this repository,
and must never be committed.

### The current run gets none of this

The calibration in flight started on a build that predates the ledger,
and rebuilding `dist/` mid-run would invalidate every cached slice and re-buy the whole run.
So the roster question that prompted this stays unanswerable from the archive,
and the next run answers it without a single extra call.

## The ledger has a reader, and writing it found a real gap (`#212`, 2026-08-25)

Written for the same reason the spend reader was:
`#210`'s writer looked finished until its reader was built,
and building the reader found two real defects in the writer.
A writer with no reader has never been checked against anything.

### The join is the whole point

A ballot names a POSITION,
not a model.
Nothing before this could say which model a judge was talking about
when it explained why it did not pick something,
which is exactly the evidence a roster question needs.
`candidates[best - 1].producers` in `src/corpus-run/ledger-read.ts` is that join,
one-based because the slate the judges saw was.

`summariseLedger` returns per-seat counts:
candidates written (composites credit both authors),
contests won,
votes from judges with no stake,
the denominator of ballots those judges could cast,
and self-votes counted apart.
`workOfModel` returns one seat's candidate text beside every disinterested judge's verbatim reason.

Two ballot faults are counted separately and neither is dropped:
an abstention names nothing,
and a ballot naming a position the slate does not hold is a fault in the judge.
Folding them together would report one count of two where a contest had one of each.

### The parser refuses rather than filling in

`src/corpus-run/ledger-parse.ts` turns a file into a shape the reader can trust,
raising `LedgerShapeError` on anything else.
A truncated file quietly read as a contest with no ballots
would report a seat as unjudged when the record was simply lost.

Model ids are read as plain strings,
deliberately,
not as the catalog union the writer held.
A ledger is read to ask questions ABOUT the roster,
including about a seat since dropped,
so narrowing a recorded id back into today's catalog would be a claim the reader cannot support.

`weight` and `selfVote` are not read at all.
The reader works out who had a stake from the producer lists,
because it needs that for EVERY candidate and `selfVote` speaks only about the one its ballot named.
Reading it as a cross-check would prove nothing either:
`candidate-select.ts` and `candidate-ledger.ts` both derive their answer
from `producerModelIds` on the same producer in one process,
so the two can never disagree.
That was checked before being skipped.

### GFP found a gap the first twenty cases missed

Five mutations,
and the first one survived:
changing the join to `candidates[best]` left every test green.
Every ballot naming a middle position resolves under either indexing,
and so does a ballot past the end;
only a ballot naming the LAST candidate on a slate tells the two apart,
and no case named one.

The added case reads the declined contest,
which holds exactly one candidate whose only judge names it.
All five mutations are red now:
the join off-by-one,
ignoring stake in the denominator,
folding the two ballot faults together,
keeping an author's remark about its own work,
and tolerating an absent array in the parser.

### Verified at the boundary, in three states

The production writer wrote a real ledger into a throwaway run directory,
and the CLI read it back:

-   A real ledger prints the per-seat summary,
    and `--model <id>` prints that seat's text with the judges' verbatim reasons.
    Every number was checked by hand against the fixture.
-   An absent ledger (`ENOENT`) prints `NOTHING RECORDED` and exits non-zero.
-   An unreadable ledger (`EACCES`) now RAISES instead of reporting an empty run.

That last one was a real defect found by running the control.
Any `readdir` failure previously read as "this run recorded nothing",
so a permissions problem would have been reported as an answered roster question.
The refusal names the filesystem code rather than the message,
because a code carries no path and a run directory path can name a person.

The task is `mise run //package/module/translation-repair:ledger-report` (moved on 2026-10-18 to
`//package/translation-repair/run:ledger-report`),
with the run directory taken from `TRANSLATION_REPAIR_RUNS_DIR`
through the same `resolveRunsDir` every other reader in this family uses.

### State

Built,
lint clean,
types clean,
663 suites passing,
zero failures.
Parked in `~/temp/agent/spend-telemetry-210.tar.gz` with the `#210` spend work,
thirty-one files,
repo-relative paths,
untarred over the repo root to apply.
