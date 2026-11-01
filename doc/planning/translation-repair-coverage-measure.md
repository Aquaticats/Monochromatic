# Measuring test coverage in `module-translation-repair`

Recorded 2026-08-25,
after the measure that scoped `#231` turned out to be measuring something else,
and after three further attempts each answered a different question than the one
asked.

`PKG` says a package is unfinished until its tests cover every exported code
path,
and `TCV` says that means enumerating implementation branches rather than
counting green suites.
This package is large enough that the enumeration has to be mechanical,
so the question becomes which mechanical measure actually answers it.

## Four attempts, and what each one really measured

### Module reachability with barrel edges counted

`#208` and `#209` built the import graph,
marked a module reached when a test named one of its exports,
propagated along import edges,
and reported what was left.

It reported `unreached that export something: 0` on 2026-08-25,
which reads as complete coverage and is not.

`#231` had just added eleven `corpus-run/` helpers to barrels,
and a barrel re-export is an import edge like any other.
Naming any one helper in a test marked the barrel reached,
and the barrel then reached everything it re-exported.
The measure improved because the graph got denser.

It is also too coarse independently:
a module counts as reached when ONE of its exports is named,
so `gatherRelabelCases` stayed invisible because `locateSlice` beside it in
`probe-relabel-case.ts` had a test.

### Per-function naming

Ask instead whether any test names each exported function,
skipping barrels because a barrel's re-export list is not its own API,
and stripping comments from the test sources
because a function named only in an `@example` block is not exercised by it.

That reports 118 exported functions no test names.
The number is correct and it is not a gap list:
a helper reached through a caller whose own cases drive its branches is covered
in the sense `TCV` means.

### One-hop caller split

Splitting those 118 by whether their DIRECT callers are tested reported 25
gaps.
That number is wrong twice over.

It matched call syntax,
`fn(`,
which cannot see a function handed to a stage as a value.
That form is the normal one for wire guards here,
so `isTranslateReportWire` looked dead while three modules pass it as
`validate:`.

And one hop is the wrong depth.
Six helpers hang off `repair-chunk.ts`,
which no test names;
but `repair-translation.ts` calls it and IS tested,
so a two-hop chain carries tests to all six.

### Transitive reachability with barrels excluded

The measure that answers the question:
seed on modules whose own exported function a test names in CODE,
walk import edges,
and drop barrel modules from the graph entirely,
since a re-export is not a use and counting it is exactly what broke the first
attempt.

Run on 2026-08-25:

```text
non-barrel modules            : 509
  seeded (own export in tests): 410
  reached transitively        : 469
  UNREACHED                   :  40
unreached modules that export a function: 1
```

The other 39 are operator entry scripts that export nothing and run on import,
so no test can reach one by importing it,
and `PKG`'s "every exported code path" is vacuously satisfied for them.

## What is actually left

Two items,
both small,
and neither is what the earlier numbers suggested.

-   `writeWidthReport` in `corpus-run/editor-width-report.ts`
    is the one exported function whose module no test path reaches.
    It is live production code:
    `corpus-run/editor-width-probe.ts` calls it at line 227,
    and that probe is an entry script no test imports.

-   `unitSourceChars` in `group-source-first.ts`
    is referenced nowhere in the monorepo
    beyond its own declaration and the `document-barrel.ts` re-export.
    It is dead,
    and the ruling `#80` set applies:
    wire it up or delete it.
    Writing a test for it would be worse than leaving it,
    because a test makes dead code look load-bearing.

The remaining 118 directly-unnamed functions need reading rather than sweeping.
Each is reached through a caller,
and whether `TCV` is met depends on whether that caller's cases drive the
branches,
which no import-graph measure can see.

## Reading whether one branch is covered

Recorded 2026-08-25,
while working the functions no test names directly.

The four attempts recorded here all answer one question,
"is this module reached".
None answers what `TCV` asks,
which is whether a case drives a particular BRANCH,
and reading the caller's cases by eye answers it badly:
a case can execute a branch while asserting nothing that branch decides.

Mutate the branch and run the suite.
If nothing fails,
no case asserts what that branch decides,
whatever the import graph says.
This is `GFP` pointed at code that already exists rather than at a new guard,
and it costs one build plus one suite run per branch,
about two minutes in this package.

It overturned two readings on the day it was written.
`blankAgainst` looked uncovered,
since its only caller documents its own arm as unreachable;
removing the guard failed `translate-judge.unit.test.ts` too,
which hand-builds a slate the producer would never assemble.
And two new cases looked like they pinned the shape
`unchangedChunkOutcome` builds;
mutating that shape left them green,
because a run whose checkers refuse to confirm passes through the settlement
in `repair-chunk.ts` instead,
which builds its own outcome.

Measured this way on 2026-08-25:

-   Covered,
    so no case was written:
    `reanchorInsertions` on both arms,
    `anchorOffsets`' placed-nothing guard,
    `leavesOriginalUnplaced`'s second conjunct,
    `notApplicableRepair`'s finding,
    `placeInsertions`' shortfall label,
    `unchangedChunkOutcome`'s heard-critic count,
    `assembleRepair`'s shipped-slice count,
    and `applyCandidate`'s composite.

-   Uncovered,
    and closed by a case since:
    `assembleTranslation`'s two refusal dispositions beside alignment,
    the settlement's critic telemetry where nothing shipped,
    and `unchangedChunkOutcome`'s roster.

-   Uncovered and still open:
    `unchangedChunkOutcome`'s attributions,
    non-translation votes,
    and their standing.
    Reaching the exits that build those with a non-empty attribution list
    needs a scripted state this work has not found.

## What the sweep found, tier by tier

Recorded 2026-08-25,
after nineteen further branches were measured the same way.

Rounds run in batches of four or five,
each one mutating a branch,
rebuilding,
running the package suite and restoring.
A round costs about forty-five seconds here,
so a batch of five fits inside a single wait.

ONE BRANCH PER FUNCTION,
which bounds what a covered verdict means:
it says that branch is asserted,
not that the function is.
Breadth first was chosen deliberately,
since it finds the largest gaps soonest,
and a function whose representative branch came back uncovered is worth
deepening before one whose came back covered.

Covered,
so nothing was written:
`codePointCount` excluding the surrogate range,
`wordsOf` pushing its final word,
`collapseSoftLineBreaks` requiring both neighbours,
`sharedNumber` on both its source check and its digit floor,
`sourceBytesOf` counting bytes rather than characters,
the slice cache key and its file name,
`isTranslateReportWire` refusing a blank,
and `extensionOf` lowercasing.

Uncovered,
and closed by a case in `2302cf549`:
`reflowOrphans` clearing what it holds at an anchor,
`digitRuns` closing a run that ends the passage,
`unsupportedVariant` wrapping through ten,
all three decisions inside `buildLicensedQuotes`,
`pluralEntries` agreeing its noun,
`generationLines` printing the count,
and `unheardCacheDiscardFinding` naming its slice.

Tiers nine and ten added ten more branches,
of which nine were already asserted:
both stream terminators,
the finish reason read,
`requireFinite`,
`carriesPicture`,
`pairScore`'s trusted gate,
`addScore`'s gap term,
`introducedFootnoteFindings` accounting for inherited defects,
and `checkedChangeSets` refusing a negative slice index.

The tenth was `countCandidateWeights` counting only ballots that carried
weight,
closed in `6c7eaaef0` along with the row every candidate gets even when nobody
named it.
That one is worth reading before deleting:
today's producer sets `best` and its weight together,
so an in-range index always arrives above zero and the conjunct looks
redundant.
It stops being redundant the moment `SELF_VOTE_WEIGHT` is tuned to zero,
which is a knob rather than a constant,
and the counts would then credit a self-vote as a ballot contributing nothing.
A comment now says so beside the code.

The same commit closed the last item tier one left open,
`unchangedChunkOutcome`'s carried critic telemetry,
by calling the builder directly rather than driving the lane.
The earlier attempt failed because a run whose checkers refuse to confirm
settles in `repair-chunk.ts` and builds its own outcome,
so the scripted state it needed does not exist;
`XPT` allows the direct call,
and the builder is now exported with `@internal`.

THE UNCOVERED NINE CLUSTER,
which is worth noticing:
seven of them are the ends of things.
The last digit run in a passage,
the wrap from nine to zero,
the empty quote,
the repeated quote,
the noun at one entry.
Cases get written against the middle of a function,
where the interesting behaviour is,
and the ends go unasserted while still being executed on every run.

## The optional-property spread is a coverage blind spot

Recorded 2026-08-25,
after tiers sixteen and seventeen and the start of a sixty-round sweep.

Of the seven branches those rounds reported uncovered,
FOUR are the same shape:
a value threaded from one layer to the next as an optional property spread into
an object literal.

-   `refineSettledSlices` forwarding `identityContext` to the refinement phase.
-   `repairChunk` forwarding `neighbouringSourceText` into the fragment its four
    stages spread.
-   `sliceNeighbourContexts` putting the original on the source side and the
    archive English on the incumbent side,
    which is the same window read the
    wrong way round.
-   `subjectsOf` reading the translate lane's delivery ledger rather than the
    repair lane's.

TypeScript does not excess-property-check a spread,
so a layer that forwards nothing compiles and lints,
and every builder-level test still passes because each hands its builder the
value by hand.
The result is a change that looks landed and does nothing,
which is exactly what `#68`,
`#107` and `#126` each recorded separately without anyone naming the class.

WHAT A TEST FOR THIS HAS TO DO:
drive the layer that FORWARDS,
not the builder that renders,
and read the assertion off the recorded request rather than off the return
value.
`nearby-window-reaches-the-models.unit.test.ts` says so in its own header and
still cannot see `repairChunk`,
because it calls the builders directly.

AND IT NEEDS THE ABSENT CASE.
A layer that pasted the block unconditionally satisfies "the sheet carries it";
only "a document declaring nothing gets no block at all" separates forwarding
from hardcoding.

## Mutating an argument, when there is no comparison to flip

Recorded 2026-08-25,
after the sixty-round sweep and a nineteenth tier built differently.

The round generator that drove tiers one to eighteen mutates a COMPARISON:
it flips an operator,
rebuilds,
and reads the suite.
It could produce no round at all for 43 of 98 targets,
which is itself a measurement:
those functions decide nothing by comparing,
and their only decisions are which value goes where.

That is exactly the class the sweep had just proved unguarded,
so tier nineteen mutates ARGUMENTS instead:
swap two operands,
read one lane's ledger where the other was meant,
drop a leading character from a derived identifier,
seat the same arm twice where two were meant.

Eleven such rounds ran.
Nine came back covered,
which is a better rate than the comparison tiers managed,
and the two that did not are both worth the reading:

-   `collectTwoLaneShippedRegions` derives an entry id from its artifact's own
    file name.
    Starting that slice one character in changes every id the damage census
    reports and failed no test.

-   `bothOrders` exists to cancel position bias by running the same pair twice
    with the seats swapped.
    Seating the NARROW arm in both places failed no test,
    so nothing asserted the second order is a different question from the first.

BOTH ARE THE SAME SHAPE AS THE SPREAD BLIND SPOT.
Nothing is missing and nothing throws;
a value is simply the wrong one,
and every layer downstream carries it faithfully.

## The measuring instrument had its own blind spot

The generator matched `export function <name>(` and
`export async function <name>(` only,
so every GENERIC signature was skipped without a word.
Fixed by trying `<` as well as `(`.
The cost turned out to be two functions,
which is small,
but a measure that silently declines to measure part of its population is the
failure mode this whole document is about.

## Batch two, and a race the cases found

Recorded 2026-08-25,
landing the second group of sweep gaps in `76feb54d7`.

Four closed:
`gatherWidthInput` skipping a slice its critics filed nothing about,
`widthControlHolds` refusing a draw whose every slice is one sentence,
`resolvePool` refusing a filtered pool and an unfiltered one at once,
and `discardNamespace` counting what it removed while staying quiet when it
removed nothing.

THE DISCARD CASES FAILED FIRST,
and the reason is worth keeping.
`console.log` is one global binding.
The house capture helper swaps it,
runs,
and swaps it back,
which is atomic only while the critical section stays SYNCHRONOUS.
`discardNamespace` is async,
so three cases each held the binding across an await and diverted each other's
lines:
one case saw none of its own output and another saw two lines where one was
printed.

`concurrency: 1` on the suite fixes it,
and the module doc says why.
The existing console-capturing suite,
`corpus-run/probe-telemetry-report.unit.test.ts`,
is safe without it because its captured call is synchronous;
that is a property of the code under test,
not of the helper,
so it stops holding the moment anything it captures gains an await.

## The sweep is finished, and what it cost to finish it

Recorded 2026-08-25,
closing the last two branches in `b5ab11e7c`.

TALLY.
Seventy-three branches measured by mutation across nineteen tiers.
Nineteen came back uncovered and all nineteen are now closed by a case,
each one GFP-proven against the exact mutation that had failed nothing.
A twenty-six percent miss rate over a package whose suite was already green.

THE LAST TWO WERE THE EXPENSIVE ONES,
and both for the same reason:
neither branch could be reached from a value a test could construct.

`coverageControlHolds`' cap is counted on ROWS rather than on cases,
so reaching it needs five cases that each survive three scripted rounds of the
whole roster,
with a standing verdict of `carried` whose evidence a cut can actually locate.
One scripted reply serves all of it:
full coverage,
quoting a sentence really present in the translation.

`sampleBenchSlices`' empty-corpus guard could not be reached at all,
because the draw read the pinned corpus through a module constant.
The fix was not a testability hack:
`censusEntry` in `corpus-run/slice-census-entry.ts` already takes
`pin = RUN_CORPUS_PIN` as a defaulted parameter,
and its own TSDoc gives the reason,
"passed rather than read so this is testable against a throwaway clone instead
of the unlicensed one".
The draw now does the same,
and threads it to its private slicer as well.

## A git fixture in this repo obeys the repo's git policy

Worth knowing before writing another one.

The bench-draw fixture builds a real repository in a temporary directory,
and `git add --all` came back exit 1 with a JSON policy refusal rather than a
git error.
`git-policy-cli` is on `PATH`,
so it guards a throwaway clone exactly as it guards this one:
bulk staging and pathspec-less commits are rejected wherever they are run.

Name the paths.
The fixture wrote them,
so it knows them,
and `git add -- <paths>` plus `git commit --message ... -- <paths>` is what the
guard asks for.

## What not to do

Do not re-run module reachability with barrel edges counted.
Every barrel addition makes its answer better without making the package better.

Do not match call syntax when looking for callers.
A guard passed as `validate: isFooWire` is a caller,
and a scan for `isFooWire(` reports it as dead.

Do not read a first-try pass as evidence.
Every guard added under `#231` was proved by removing it,
rebuilding,
running,
and restoring,
per `GFP`.

Do not read a steady `] PASS ` count as evidence nothing broke.
The runner prints ONE `PASS` line per suite,
naming every child that passed,
and a separate `FAIL` line per child that did not.
A suite with one failing case still prints its `PASS` line naming the rest,
so the count held at 711 across a `GFP` round in which a case failed and the run
exited 1.
`TLY` already rules that the exit code decides;
this is what a count disagreeing with it looks like in this package.

Do not put two suites a single GFP round needs to read in one file.
`await describe` throws,
so a failure in the first suite aborts the file before the second runs,
and the round then proves nothing about the second.
`#231`'s second round hit exactly this.

## Tier twenty closed, and the rule that scoped it held again

The five gaps depth two found are all closed and all GFP-proven,
each against the exact mutation that had failed nothing:

```text
refinesettledslices-b2   COVERED (exit 1, 2 FAIL, 750 PASS)
assemblerepair-b2        COVERED (exit 1, 3 FAIL, 749 PASS)
gatherwidthinput-b2      COVERED (exit 1, 2 FAIL, 749 PASS)
resolvepool-b2           COVERED (exit 1, 2 FAIL, 749 PASS)
coveragecontrolholds-b2  COVERED (exit 1, 2 FAIL, 749 PASS)
```

The baseline before them was exit 0 with 750 passing cases.
Note again that the passing count does not fall when a case fails,
which is why the exit code is the verdict and the count is only context.

## The runner lost eight rounds to a buffer, in silence

Tier twenty-b ran for over an hour in the background and then vanished,
leaving a zero-byte output file and no verdicts at all.

Nothing crashed.
`python3` block-buffers stdout when it is not a terminal,
so every line the runner printed sat in a 4 KiB buffer waiting to be filled,
and the buffer went with the process.
A run that prints one short line per round,
where each round takes five minutes,
can run to completion without ever filling one buffer.

Two changes,
both cheap:

-   Launch with `python3 -u`,
    which makes every print reach the file as it happens.
-   Redirect to a real file under the scratch directory rather than relying on the task output,
    so a killed run still leaves everything that was flushed.

The general form is worth keeping:
a long measurement whose only output arrives at the end
has no partial credit,
and the failure mode is indistinguishable from a run that produced nothing to say.

## A waiting loop can answer for the process it is waiting on

While that run was dying,
the wait loop kept reporting it alive.

The check was `pgrep -f 'rounds-tier20b.json'`,
and the loop's own shell command line contains that string,
so `pgrep` matched the waiter rather than the runner.
The same shape had already bitten once in this session,
where `pgrep -af 'mutate-run.py'` matched the `grep` reading its output.

The reliable check reads what the process actually is:

```sh
for p in $(pgrep -f mutate-run); do
  [ "$(cat /proc/$p/comm)" = python3 ] && echo "alive: $p"
done
```

This belongs beside `QRY` rather than beside the mutation work.
A search that matches its own invocation is a search that always succeeds,
and the answer it gives is the one the caller was hoping for.
