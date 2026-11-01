# Grading round three of the translation-repair precision gate

Round three asks TWO questions and hands you TWO files to answer them in.
This is the first round with a second sheet,
so this runbook's ordering is not a formality:
grading them out of order changes what the first number measures.

## Before you start

Both sheets quote UNLICENSED corpus text.
They live outside the repo,
under `node_modules/.monochromatic/translation-repair-runs/`,
and must never be committed.

The files are named after the draw seed:

```text
grading-sheet-milestone-three-precision-round-three.md
repair-sheet-milestone-three-precision-round-three.md
sample-manifest-milestone-three-precision-round-three.json
```

A `-preliminary` suffix means a scratch draw taken before coverage filled.
Those are meant to be redrawn and are not the gate.

Both scoring commands now print a NOTE about the DRAW DIGEST before their
numbers,
and for round three it will say the binding is the weaker one.
That is expected and is not a problem with your grading.
Sheets drawn from now on carry a fingerprint of the exact item list,
and the
scorers refuse a sheet whose fingerprint disagrees with its manifest;
round three was drawn before that existed,
so it is checked on seed and corpus
pin as it always was.
Nothing about the numbers changes.

The manifest is not for you to read.
It records which issue sat at which sheet position,
which is the only way a grade you write can be matched to what the pipeline
thought about the same item.
It is written in the same instant the sheets are,
because re-running the draw does not recreate it:
the draw is deterministic in its seed but not in its pool,
and the pool grows with every entry that settles.
Keep it beside the sheets;
without it the repair grades cannot be scored against anything.

## Step 1: grade the detection sheet, and only that

Open `grading-sheet-...-round-three.md`.

Each item asks one question:
is this accepted issue a REAL translation defect,
or a false positive?
Replace `[ ]` with `Y` or `N`.

Add rationale freely after the letter.
Both earlier rounds carry rationale that nothing else reproduces,
and it is read:
round two's notes are what raised the repair-quality question in the first
place.
Either form parses:

```text
### 3. grade: [Y]  (Y = real defect · N = false positive)
### 4. grade: [Y, but the warmer word would be "naps"]  (Y = ...)
### 5. grade: N. The original does quote this.  (Y = ...)
```

If you cannot decide,
say so instead of guessing:

```text
### 6. grade: [Not enough context to grade]  (Y = ...)
```

Answers that name no verdict are counted as DECLINED,
kept out of both the numerator and the denominator,
and reported separately.
A declined item is more useful than a coerced one.

DO NOT OPEN THE REPAIR SHEET YET.
Seeing a proposed correction makes an alleged defect look more real,
which moves these grades.
Round one and round two were both graded without any correction visible,
so reading the repair sheet first would compare round three against them
through a different instrument,
and the change of instrument would show up as if the pipeline had improved.

## Step 2: grade the repair sheet

Open `repair-sheet-...-round-three.md` only once step one is finished.

Item numbers match the detection sheet exactly.
For every item you graded `Y` there,
answer whether the text the pipeline produced actually fixes it:

```text
- repair grade: [ ]  (Y = fully fixes this defect and breaks nothing nearby · N = it does not)
```

`Y` means the returned wording resolves the defect AND introduces no new error
nearby.
A better phrasing existing does not make it `N`;
if you want to say so,
write it as rationale after the letter.

Leave items blank where you graded `N` for detection:
there was no defect to fix,
so the question has no answer.

Some items carry no grade box at all.
That is deliberate,
and the sheet says which case it is:

-   `not-selected`:
    a repair was written,
    but the unchanged text won its slice.
-   `withdrawn`:
    a repair was written,
    but the whole page was blocked as non-translation.
-   `no-region`:
    no targeted repair exists for that issue at all.

Those count against COVERAGE rather than against repair quality,
and the sheet shows what was attempted anyway so the attempt is visible.

Two things worth knowing while you read:

-   A `SHARED` line means one edit was written for several accepted issues at
    once,
    and names the other sheet items it repeats under.
    You will meet the same before-and-after text there.
    Judge it against the claim of the item you are on.
-   A `NOTE` about a naturalness pass means a later stage rewrote the slice,
    so the edit shown is not the final wording.
    The sheet then prints the slice as actually returned;
    grade THAT.

## Step 3: get the numbers

```bash
mise run //package/translation-repair/probe:score-agreement -- \
  --sheet /ABSOLUTE/path/to/grading-sheet-milestone-three-precision-round-three.md
```

Pass an ABSOLUTE path.
The task runs with the package directory as its working directory,
so a repo-relative path resolves somewhere unintended.

Blind pre-grades for this draw ARE recorded,
in
`pre-grades-milestone-three-precision-round-three.json` beside the sheets:
50 items,
49 scored and 1 handed over as genuinely contested.
They were written without the agent seeing your grades and are deliberately not
reproduced anywhere you would read before grading,
because naming them would
anchor you toward agreeing and this same sheet produces the gate number.
So grade the sheet without looking at that file,
and the agreement rate falls
out of the `score-agreement` command.

One asymmetry to know when you read that rate.
The sheet shows no source anchor for addition-class claims,
because an addition
points at nothing in the original,
so those cannot be graded from the sheet
alone.
The agent read the corpus directly at the pinned commit for them,
and marked
those grades `VERIFIED AGAINST SOURCE` in their notes.
Disagreement on those items may be that asymmetry rather than judgement.

It prints three precision readings and,
when blind pre-grades were recorded for
the draw,
the agreement rate against them:

```text
PRECISION items=50 scored=47 realDefects=37 strict=0.740 excluded=0.787 lenient=0.800 unscored=10,12,17
```

-   `strict` counts a declined item as a false positive.
-   `excluded` drops declined items from the denominator.
-   `lenient` counts a declined item as a real defect.

The gate bar is 0.9.
Earlier rounds by the same tool:
round one 0.560 / 0.636 / 0.680,
round two 0.740 / 0.787 / 0.800.

Output carries counts and sheet positions only,
never a quote or your rationale,
so it is safe to paste anywhere the sheets themselves are not.

## Step 4: score the new-defect probe against your repair grades

This round added a probe that asks whether a repair BROKE something nobody had
raised.
It runs in shadow mode:
it recorded verdicts and changed nothing about what shipped,
because until your grades exist there is no way to know how often it is wrong.

```bash
mise run //package/translation-repair/probe:score-probe -- \
  --repair-sheet /ABSOLUTE/path/to/repair-sheet-milestone-three-precision-round-three.md \
  --manifest /ABSOLUTE/path/to/sample-manifest-milestone-three-precision-round-three.json
```

Absolute paths again,
for the same reason.
Run without the two flags to see the probe's own counts and no comparison.

The line that matters is the second one:

```text
AGREEMENT joined=41 probeFlagged=6 refutedByHuman=5 sharedWithHuman=1 flaggedUnscored=0 unflaggedFailures=9 refinedJoined=10
```

READ `refinedJoined` FIRST,
and subtract it before reading anything else.
The probe runs inside the accuracy stage and the naturalness lane runs after it,
so on a slice the lane rewrote,
the probe judged wording that never shipped
while the repair sheet asked you to grade the wording that did.
Those positions compare two different texts and belong in neither column.
For this draw it is 10 of the 50:
positions 19,
22,
24,
26,
29,
31,
33,
35,
40,
and 45.
Read every other count over the remaining 40 and say so in the verdict.

This limit is specific to judging the PROBE.
It does not touch the gate number from step 3,
and it does not mean a gate would
have judged the wrong text either:
a gate would act during candidate selection,
which is also before the lane runs.

`refutedByHuman` is the only clean number there.
Those are items where the probe claimed the repair introduced damage and you
graded `Y`,
which says the repair breaks nothing nearby.
You read the same wording and disagreed,
so each one is a correct repair that a gate would have thrown away.

`sharedWithHuman` is NOT the opposite of that and must not be read as
confirmation.
Your `N` fires both for a repair that did not fix its target and for one that
broke something,
and the sheet has no way to separate those,
so agreement there might be about a different defect entirely.
`unflaggedFailures` is an upper bound on what the probe missed,
for the same
reason.

If `refutedByHuman` is a large share of `probeFlagged`,
the probe is not fit to block anything and the honest move is to leave it
recording.
If it is near zero,
the probe is finding damage the pipeline currently ships.

The gating question already has a recorded answer,
and this measurement is what
reopens it rather than what decides it.
You chose to keep the probe in shadow mode on 2026-08-07;
the four options and the reason each was ranked where it was are in
`doc/decision/introduced-defect-probe-gating.md`,
along with the two outcomes
worth naming in advance.
That document is the one to revise once these numbers exist,
not this runbook.

A second probe now runs on the naturalness lane itself,
reported on a
`REFINEMENT` line by the same command:

```text
REFINEMENT rewrittenSlices=0 majorityIntroduced=0 minorityIntroduced=0 noneIntroduced=0 ...
```

`rewrittenSlices=0` does NOT mean the lane broke nothing.
It means no artifact in the runs directory carries that audit,
which is true of
everything settled before run 012.
The command prints a note saying so whenever the count is zero.

A `LANE` line sits beside it and answers a different question:
whether the lane was able to run at all.

```text
LANE slicesOffered=101 slicesSilent=6 entriesWithRewrites=15/47
```

`slicesSilent` counts slices where NO refiner answered.
One model refines,
so a roster of one has no quorum to lose and its failure
moves no other number.
Run 013 is where the six come from:
its only refiner lost every voice to schema-mismatch,
so five settled entries
carry no rewrite at all.
Read a rising `slicesSilent` as the lane going quiet,
not as a run with nothing
worth rewriting;
`doc/handover/translation-repair-history.md` has the evidence.
The decision this left open,
whether the lane's schema-mismatch was a provider window or a standing condition,
closed on 2026-08-12:
the lost voices were a provider's channel marker left in front of Kimi-K3's JSON,
and a parser fix reads past it
([the marker came back one character longer](../handover/translation-repair-history-2026-08-12-to-15-channel-marker-recurrence-and-lost-run-log.md#the-marker-came-back-one-character-longer)
records the fix and its later widening).

## What this round cannot tell you

Round three changed many things at once:
the roster,
the editor ensemble,
the checker set,
the quorum rule,
the
adjudication policy,
the house policy,
the naturalness lane,
and the
resolution-credit rule.
A precision delta will not be attributable to any single one of them.
Say so in the verdict rather than implying otherwise.
