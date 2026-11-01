# Translation repair history 2026-08-19 to 2026-08-21: the window wired into the repair lane and the instruments for its verify

The neighbouring-source window wired into every repair-lane stage that needs it,
the test that proves the sheets carry it,
what the five-entry verify had to show,
the decision not to window the checker,
the gate reader and the false pass it produced first,
and the probe that proves the window reaches production.

Part of [Translation repair history: 2026-08-19 to 2026-08-21](translation-repair-history-2026-08-19-to-21.md),
which lists every file of this record.

Historical evidence only:
no next-action list here is a current instruction,
and the stop condition in the index applies.

## `#107`'s window is wired into the repair lane, and the 5-entry verify is RUNNING

### What landed in the repair lane and why each stage is in it

Landed in `711497365`.
What it changed,
and the shape of it matters more than the diff:

```text
driver   repair-translation.ts   window computed per slice, BY POSITION
key      repair-slice-key.ts     folded in; version stays 27, deliberately
fan-out  repair-chunk.ts         one windowFragment spread into all three stages
critic   critic-prompt.ts        + chunk-critic-phase.ts + repair-stages.ts
panel    adjudicate-prompt.ts    + repair-stages.ts
editor   edit-prompt.ts          + repair-editor-stage.ts
```

THE CRITIC IS IN IT BECAUSE OF CLAIM FLOW,
not because it seemed thorough.
A slice
raising zero claims skips every downstream stage,
so a surplus passage can only ever be
removed if a CLAIM names it,
and only a critic that can see next door will name it.
The
panel and editor follow necessarily:
a panel that cannot see the neighbour must reject a
claim about text outside the slice as unfounded,
so widening the critic alone would have
manufactured exactly the claims the panel is guaranteed to throw away.

ONE FRAGMENT,
THREE SPREADS.
`windowFragment` is built once in `repairChunk` because the
three stages must see the SAME window or they contradict each other.
Three separate
arguments can drift;
three spreads of one value cannot.

THE VERSION CONSTANT STAYS AT 27 and the reasoning is recorded in the key's own history.
The window is folded into the KEY,
so a slice that has a neighbour keys anew and
recomputes,
which is correct because it is being asked a different question,
while a lone
slice keys identically and resumes,
which is also correct because there was no window to
show it.
A bump would have discarded both,
including the lone slices whose question never
changed.

### The trap that was avoided, and it is the one `#99` named

`neighbouringSource` takes a POSITION and throws on anything else.
The repair loop's
`chunkIndex` is a STAMPED index,
and passing it would either throw or,
worse,
address some
other slice's neighbours.
The loop now runs `for (const [sliceIndex, slice,] of
slices.entries())` so the position is correct by construction rather than by lookup.

### Why a test file exists for something the type checker should catch

TypeScript does NOT excess-property-check a spread.
Every one of the five call sites passes
the window as an optional property spread into an object literal,
so a stage that silently
dropped the parameter would compile,
lint,
and pass every existing test while sending the
models exactly the sheet they got before.
The failure mode is a change that looks landed
and does nothing,
so `nearby-window-reaches-the-models.unit.test.ts` asserts against the
RENDERED TEXT of all three sheets.

SHOWN TO FAIL BEFORE BEING TRUSTED,
per `GFP`,
and after being committed so restoring could
not discard it:
dropping `${nearbyBlock}` from the critic template alone turns the suite
red at exactly that case.
Restored,
and 430 pass with 0 failures.

### What is running and what it has to show

Five flagged entries into `~/temp/agent/win107-verify-20260819`,
detached,
about five hours.
The `~/translation-repair-runs-flagged-20260818/artifacts` pool is the BASELINE and must not
be overwritten:
the damage is already recorded there,
so it is the positive control and
re-spending to reproduce it would be waste.

Gates,
none of them a feeling:

-   `lintong` repair lane:
    each distinguishing noun phrase appears EXACTLY ONCE in the
    assembled document.
    Not at-most-once.
    Removal here is only safe if the content really
    does ship next door,
    and zero occurrences is a worse failure than two
-   `saurikissa` slice 7 no longer ships the severed sentence
-   the contested-cut replacement rate at UNFLAGGED slices does not fall away from about 0.95
-   claim counts at unflagged slices do not inflate materially.
    `#92` found cost tracks
    claim count,
    so this is the trigger that would send the window from always-on to gated
    at flagged slices plus or minus one

Instruments already exist:
`~/temp/agent/join-107.mjs` and
`~/temp/agent/severed-sentence-census.mjs`.

SOURCE EDITS ARE OFF AGAIN WHILE THIS RUNS,
for the reason recorded above:
the process in
flight is safe,
but any interruption plus a rebuild stamps a new digest and the guard
refuses to resume into this pool.

### The checker is deliberately NOT windowed, and that is a decision rather than an omission

Recorded because the code and the tracker would otherwise disagree,
and a later session
would read the gap as unfinished work and thread it for nothing.

The first sketch of this change listed the checker beside the panel,
on the reasoning that a
stage verifying a removal cannot tell a fix from a deletion without seeing the neighbour.
That is overstated for what the checker actually does.
A removal claim quotes the surplus
AS IT APPEARS IN THIS SLICE,
and the checker verifies that the quoted text is gone from this
slice.
That question is answerable within the slice,
and the same holds for additions on the
deficit side.
Handing it the window would spend context on evidence it has no question for.

WHAT WOULD REOPEN IT,
stated so the decision is falsifiable:
if the verify artifacts show
`resolved` failing to fire on relocation-class issues,
the checker is being asked something
it cannot answer and the window follows.
That is a thing to read in the artifacts when they
land,
not a thing to assume either way now.

### Three effects to expect in the verify results, none of which is damage

Named in advance so they are not misread as regressions when the run lands.

CRITICS MAY QUOTE FROM THE NEARBY BLOCKS despite the rule forbidding it.
Those claims die at
anchoring,
because a quote taken from next door cannot anchor in this slice,
so the outcome
is safe.
They are still spent quota,
so COUNT THE DROPS:
a high count is the signal that the
rule needs strengthening or that the blocks need a stronger separator.

THE INTRODUCED-DEFECT PROBE IS WINDOWLESS and will therefore read a legitimate surplus
removal as dropped content.
Expect `removal`-class findings to rise on relocation slices,
and do not book them as damage without reading them.

`ISSUE_CATEGORIES` IS A CLOSED VOCABULARY WITH NO RELOCATION KIND,
so these claims must ride
an existing category.
Note which one they land in.
That is evidence about whether the
vocabulary wants a kind;
it is not a decision to take from this run.

### One defect already found in what just landed, fix scheduled rather than rushed

`repairSliceKey` spreads the two window sides as BARE values into a positional array,
so a
source-only window and an incumbent-only window carrying the same text hash identically.
`translateSliceKey` avoids this by spreading NAMED properties.
Tracked as task `#126` with
the labelled-entry fix.
It re-keys every windowed slice,
so it lands AFTER this verify and
BEFORE the 92-entry relaunch,
while no cache is worth keeping.

### The gate reader, and the false pass it produced first

`~/temp/agent/window-gates.mjs --new <dir> [--baseline <dir>]`.
Zero quota,
reads settled
artifacts only,
and turns the verify's completion into one command:

```text
GATE A  duplicated phrases, derived from the baseline, counted in the new repair document
GATE E  critic claims per slice, flagged against unflagged, baseline against new
GATE E2 claims per HEARD critic, with the roster each reading rests on
GATE B  delegated to severed-sentence-census.mjs
GATE C  delegated to join-107.mjs, both cuts
```

GATE A DERIVES ITS PHRASES INSTEAD OF HOLDING THEM,
which is what lets it live in a file at
all:
no corpus text is written into the repo,
the tracker or the instrument.
`#107` recorded
that the distinguishing phrases occur TWICE in the baseline repair document and ONCE in the
baseline translate one,
and that is a machine-checkable signature,
so the phrases are
recovered by searching for it and then counted in the new document.
Output names a phrase by
word count and a short hash,
never by its wording.

EXACTLY ONCE IS THE TEST,
not at most once,
and the reader says `LOST` rather than `ok` at
zero.
Removal at slice n is only correct if the content really does ship at n plus or minus
one,
and a document that quietly dropped the passage is worse off than one that says it
twice.

GATE E2 EXISTS BECAUSE CLAIMS PER SLICE BECAME UNREADABLE.
A stage that heard three of six
raises fewer claims than one that heard six whatever it was shown,
so while heard-counts are
moving,
claims per slice cannot be attributed to the window at all.
Claims per HEARD CRITIC
survives that,
and the reader now prints the average and minimum roster beside every rate so
no gate verdict is read without the roster it rests on.
The baseline reference is 1.70 claims
per heard critic at flagged slices and 1.74 at unflagged,
average roster 5.9,
minimum 5.

IT REPORTED A CLEAN PASS ON ITS FIRST RUN,
and both halves of that were wrong.
The two lanes
name their assembled text differently,
`repairedText` against `translatedText`,
so reading
one field for both compared every phrase against an empty string and found nothing.
The
reader then printed that nothing as PASS.
Two fixes,
and the second matters more than the
first:
the field is now read per lane and a missing one THROWS rather than defaulting to
empty,
and a derivation that finds no phrase is no longer a pass.
In `lintong` it is an
INSTRUMENT FAULT,
because that duplication was found by hand and an instrument that cannot
see it is broken;
elsewhere it is inconclusive,
which is also not a pass.

VALIDATED BY POSITIVE CONTROL,
which is the only reason to trust it:
run with the baseline
as its own `--new`,
it reports STILL DUPLICATED for every phrase it derived and Gate A reads
FAULT.
An instrument that cannot fail on known damage cannot clear anything either.
Gate C
reproduces `#107`'s recorded contested-unflagged rate of 0.9524 on the same pool.

This is the third null-that-was-a-bug in two days,
after the test file that registered
nothing and the census that read 93 entries and 0 assets.
All three were caught by the same
question,
asked before believing the answer:
could this probe have shown a positive at all.

### The window is proven to reach production, not just the prompt builders

`~/temp/agent/window-reaches-production.mjs`,
zero quota,
no network.

THE UNIT TEST COULD NOT ANSWER THIS.
It asserts that `buildCriticMessages` renders a window
it is HANDED.
Whether `repairPreparedDocument` computes one and hands it over is a different
question,
and it is exactly the one `#107` records going wrong before:
the translate lane's
window sat unused for weeks because the call site never passed what the builder accepted.

The probe drives the real `prepareDocumentPair` and `repairPreparedDocument` with a stub
client that records every sheet and answers "no issues",
so each slice raises zero claims and
skips every later stage.
Three invented cat-themed sections,
each carrying a marker no other
section uses,
so a sheet can be attributed to the slice it was asked about.

```text
slices prepared 3, sheets asked 6, all 6 carrying a NEARBY fence

middle slice   window carries the preceding neighbour   2 of 2
               window carries the following neighbour   2 of 2
first slice    window carries the following neighbour   2 of 2
               window wrongly carries the last slice    0 of 2
last slice     window carries the preceding neighbour   2 of 2
               window wrongly carries the first slice   0 of 2
```

THE TWO ZEROES ARE THE POINT,
more than the twos.
They are the negative control:
the window
reaches exactly one slice each way and not the whole document,
which is what
`neighbouringSource` promises in its TSDoc and what keeps the cost bounded.
A window that
quietly widened would show here as the first slice seeing the last.

IT REPORTED A FAILURE FIRST,
and the probe was wrong rather than the code.
The first version
filtered sheets by "mentions the middle marker" and read 4 of 6,
which looks like a partial
forward.
It is not:
slice 0's sheet mentions the middle marker because the middle slice is
ITS neighbour.
What is under review and what is context sit on opposite sides of the NEARBY
fence,
so the sheet has to be split there before it can be attributed.
Same lesson as the
gate reader an hour earlier,
in the opposite direction:
that one called a broken probe a
pass,
this one called working code a failure.

WORTH PROMOTING INTO THE PACKAGE as a driver-level test,
since it covers the threading no
builder test can.
Source work,
so it waits for the run.
