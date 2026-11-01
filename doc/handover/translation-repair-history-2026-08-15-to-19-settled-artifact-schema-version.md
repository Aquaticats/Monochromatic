# Translation repair history 2026-08-15 to 2026-08-19: the settled artifact gets a version

The schema version and slice count added to the settled artifact,
the reader that tells unrecorded,
uncounted and counted artifacts apart,
the measurement that no stored artifact can reach a new refusal,
and the six further findings the same reader returned.

Part of [Translation repair history: 2026-08-15 to 2026-08-19](translation-repair-history-2026-08-15-to-19.md),
which lists every file of this record.

Historical evidence only:
no next-action list here is a current instruction,
and the stop condition in the index applies.

## The settled artifact gets a version, and the review of that work

### The settled artifact has a version, and its absences stay absences

`#96`'s schema half is built,
in `f2b8c4e39`.
It was the highest-rated finding
of the whole-day review and it was blocked by nothing,
which is why it went
first rather than the parts Question 5 holds.

WHAT THE ARTIFACT NOW CARRIES:
`artifactSchemaVersion`,
and the `sliceCount`
that both index sets are out of.
Fields had been added three times with no
version marker,
so every reader told the generations apart by guessing from
which fields happened to be present,
which works exactly until two generations
differ in something other than presence.

WHAT READS IT BACK:
`readArtifactChangeSets`,
wired into `parseSettledArtifact`
so every consumer of the parser gets it.
It answers one of three kinds,
and the
three are the point:

-   `unrecorded`,
    when neither index set was written.
    NOT an empty set.
    A run
    nobody wrote index sets for must never read as a run that changed nothing,
    and an empty array is exactly how those two become indistinguishable.
-   `uncounted`,
    when both sets are there without a `sliceCount` to bound them.
    Everything else about them is still checked.
-   `counted`,
    when the version promises all three,
    which is fully validated.

It REFUSES one index set without the other,
a versioned artifact missing either
of them,
and a version this reader does not know.
That last one matters most:
meeting a generation written after you were compiled tells you exactly one
thing,
which is that you do not know the shape,
and carrying on is how an
instrument reports a wrong number rather than a missing one.

MEASURED BEFORE DESIGNING IT,
at zero quota:
164 artifact files are on disk and
NOT ONE carries `pipelineDigest`,
`sourceBytes` or the index sets.
So every
artifact that exists is the oldest generation,
and the two generations between
it and version 1 are empty populations.
They are named in the version history
anyway,
because a reader that meets one must not read it as the generation
before.
The measurement is also what makes wiring the reader into
`parseSettledArtifact` safe:
no artifact on disk can reach any new refusal.

`orderedChangeSets` SPLIT to make this possible:
`checkedChangeSets` holds every
rule that needs no slice count,
and `orderedChangeSets` adds the range check on
top.
The `uncounted` reading needs the first without the second.
Assembly is
unaffected,
and the split is deliberately not exported through a barrel,
so the
public surface did not grow:
its unbounded behaviour is pinned through the
reader and its negative-index refusal through the assembly tests that already
existed.

BOTH GUARDS WERE SHOWN TO FAIL WITHOUT THEMSELVES,
after the commit,
by removing
them,
rebuilding,
running and restoring.
Removing the unknown-version refusal
fails that one test;
removing both change-set refusals fails both of theirs.

A HARNESS FACT WORTH ADDING to the one recorded earlier tonight:
two failures
inside ONE suite both report,
as an `AggregateError` naming how many children
failed.
It is only across SUITES that a failure hides what comes after,
because
each `await describe(...)` runs at module scope.
So a probe's failure list is a
floor at suite granularity,
not at test granularity.

WHAT THIS CLOSED ELSEWHERE:
`#94`'s enforcement half is now entirely done.
Both
index sets are checked and ordered at every return of both lanes and the blocked
exit,
`sliceCount` is on the repair result and in the artifact,
and what remains
in `#94` is only the rename from `chunk` to `slice`,
which is held with `#99`
because renaming before slice identity is settled means renaming twice.

#### And that work went to the same reader, which found six more things

All six were taken,
in `edf269a67`.
Two are worth carrying forward as lessons
rather than as changelog:

THE REFUSAL I HAD NOT THOUGHT OF is an artifact that carries `sliceCount` with
no version.
No writer ever produced it,
which is exactly why it slipped past:
the shape that produces it is a CURRENT artifact whose version field was lost to
an edit or a merge,
and reading that as a generation predating the count throws
away a denominator the run recorded.
The general lesson is that a version field
makes absence meaningful in BOTH directions,
and only one of them is obvious.

I WAS WRONG ABOUT WHO TO BLAME.
I had let `AssemblyContractError` escape the
reader on purpose,
on the reasoning that a repeat or an overlap is a broken lane
contract wherever it is found.
The reader cannot know that:
a run,
an edit,
a
truncation and a merge all look identical from inside a file.
It now reports
what the artifact contains and where,
naming the entry,
and carries the contract
message through without asserting how it got there.
Worth remembering when the
next reader is tempted to describe its input's history.

THE REST,
in one line each.
`KNOWN_ARTIFACT_SCHEMA_VERSIONS` is separate from
the constant the pass writes,
so a later bump cannot orphan the generation
before it.
Which failure a multiply-invalid set reports is now stated and pinned
rather than falling out of the split.
`requireCount` refuses whole numbers past
what JSON carries exactly,
since `Number.isInteger` says yes well beyond the
point where round trips stop being exact.
And a round-trip test drives the real
writer through JSON into the parser,
because every other test in that file
hand-builds the record and would keep passing with the writer misspelling every
field it writes.

BOTH NEW GUARDS FROM THAT COMMIT were shown to fail without themselves as well,
in one probe:
removing the count-without-version refusal and reverting the
safe-integer check fails three tests across two files.
