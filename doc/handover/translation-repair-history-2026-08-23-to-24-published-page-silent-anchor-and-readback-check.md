# Translation repair history 2026-08-23 to 2026-08-24: the silent anchor that cost a run its page, and reading the published tree back

The run that finished and delivered nothing,
the first diagnosis that was wrong and the one-expression publisher defect,
the silence column that could not tell a gap from an emptying,
the read-back of the published tree and the first check that passed a page with text cut out of it,
and the same check moved inside the pass with its guard proofs.

Part of [Translation repair history: 2026-08-23 to 2026-08-24](translation-repair-history-2026-08-23-to-24.md),
which lists every file of this record.

Historical evidence only:
no next-action list here is a current instruction,
and the stop condition in the index applies.

## XIEPT2 lost 4 h 48 m at its last step, and the first diagnosis was wrong (`#194`)

THE `#189` VERIFICATION RUN FINISHED AND DELIVERED NOTHING.
`~/temp/agent/xiept2-verify-2026-08-23`
ran 17,295,337 ms with zero error lines,
wrote no artifact and no fixed page,
and the background
command reported exit code 0.
The only trace of the failure is the last line of its log:

```text
TALLY XIEPT2 status=ERROR ms=17295337 aborted=false
error=slice 12 has no translation and writes none: an anchor is where a rendering belongs, ...
```

`aborted=false`,
so this was not the per-entry deadline.
It threw at the very end,
in `spliceSlices`.

### The root cause named here first was WRONG, and the wrong one is the natural reading

RECORDED RATHER THAN QUIETLY REPLACED,
because it took a run log to refute and would be
re-derived by the next reader of `consolidate-settle.ts`.

The claim was:
`settleConsolidation` calls `judgeTranslateSlate` with `incumbentKind: 'present'`
hardcoded,
so at an anchor it judges and gates against a standing text that does not exist.
That line is unreachable with a blank standing text.
`settleConsolidation` returns `no-standing-text` about ninety lines above it,
at `if (standingText === '')`,
before any judge or gate is bought.
What the judges fall back on at that stage is `standingText`,
not the archive's own wording,
so where the literal is reached there really is a text to keep,
and it is true rather than assumed.
Threading the slice's own `incumbentKind` there would be actively wrong:
an anchor whose lanes both produced wording does have a standing text.
`9daa6e728` puts that reasoning in the file beside the literal.

### What actually happened, traced through the log

```text
pass.log:753   00:40  [translateDocument] slice 12: no translation in the archive
                      and none produced (no-candidate-backed); the passage stays missing
pass.log:754   00:40  SLICE-COST lane=translate chunk=12 sourceChars=344 exit=unfilled
pass.log:1962  04:12  TALLY XIEPT2 status=ERROR
                      error=slice 12 has no translation and writes none
```

THE ABSENCE MACHINERY WORKED.
`TranslateAbsenceError` was raised,
caught,
and the slice recorded unfilled,
which is exactly what `translate-absence.ts` promises:
one refused anchor costs its own slice rather than the entry.
The entry then ran the contest and the consolidation for another three and a half hours
and died at its last step,
in `publishFixedPage`.
`pass-entry.ts` calls that BEFORE the artifact write,
deliberately,
so that an artifact existing means a page was published;
the cost of the ordering is that a publish refusal keeps neither.

### The defect was one expression in the publisher

```ts
// corpus-run/publish-fixed.ts, before 5dabe9c92
replacementText: (reading.kind === 'wording') ? reading.text : '',
```

`WouldShipReading` exists to make that unrepresentable.
Its own comment:
"NEVER REPRESENTED AS AN EMPTY STRING,
which is the trap this whole shape exists
to close ... A reading that carries no `text` key at all makes that unrepresentable rather than
warned against."
Thirty lines later the builder turned the named absence back into the empty string,
and handed the splice a blank rendering at an anchor.

THE EMPTY STRING IS NOT SIMPLY WRONG,
which is why the fix is narrower than deleting that branch.
At a CONTENT span it is a real decision:
both lanes may have removed wording the archive held,
and the splice allows that.
Dropping the row instead would republish text the deciders agreed to drop.

THE TEST PINNED IT,
and its fixture is what gives the game away.
`publish-fixed-replacements.unit.test.ts` asserted a silent slice gets the empty string,
reasoning that "republishing the archive underneath it would undo it",
over a fixture with `incumbentKind: 'absent'` and `repairDelivery: 'gap-remains'`.
There is no archive wording at an anchor to republish.
The rationale was about content spans and the fixture proving it was an anchor.

### What shipped

`5dabe9c92`:
a silent slice contributes no replacement AT AN ANCHOR and keeps `''` at a content
span,
branching on `incumbentKind` rather than on an empty incumbent,
per `translate-absence.ts`:
testing the text conflates an anchor with a span whose archive wording
genuinely is blank.

THE SPLICE GUARD IS UNCHANGED AND STILL PROVEN.
`splice-slices.ts` refuses blank text at an anchor because it "leaves the passage missing while the
run reports it delivered",
and it is the only thing that stopped a silent hole reaching a memorial
page.
What changed is upstream:
nothing feeds it a rendering nobody wrote.
Its case moved into `publish-fixed.unit.test.ts` beside the case that now publishes the gap.

GFP-PROVEN BOTH DIRECTIONS.
Removing the anchor branch fails two cases in 38 ms:
`shippableReplacements` returns a row where it must return none,
and `publishFixedPage` raises the very `SliceSpliceError` that cost XIEPT2 its run.
Two control cases stay green through the removal,
the content-span silence and the ordinary decided wording,
so the failures are about anchors rather than about silence.

THE POLICY REVERSAL IS DELIBERATE.
The publisher used to refuse the entry outright,
on the argument that a half-document is worse than a failed entry because only the failure gets
retried.
XIEPT2 settled that:
the refusal kept neither page nor artifact,
and a retry meets the same passage and the same judges.
The archive already carries that gap,
so the page loses nothing that was ever there
and keeps every slice the run did buy.
The run still says so out loud:
`pageSilent` has been on the TALLY line since `#175`,
and the artifact records the slice as `unfilled` and `gap-remains`.

BLAST RADIUS ZERO on what already exists.
Across 44 artifacts on disk,
37 parsed,
199 slices,
every slice reads `wording`
and none reads `nothing-ships`,
so no published page deleted or refused anything.
The probe is `~/temp/agent/silent-slice-blast.mjs`;
that null rests on the anchor test case as its positive control rather than on itself.

### The silence column could not tell a gap from an emptying either (`#195`)

Every member of `WouldShipSilence` names which stage left a slice with no wording
and none of them says whether there was anything there to lose.
`lanesAgreedOn` admits it in its own comment:
an agreed empty string "covers a gap neither lane wrote into and text both lanes removed".
The two contest reasons share the fault at one remove,
since a content span whose archive wording is blank reaches them as readily as an anchor.

`a86cbb034` and `932e0b280`:
the silent reading carries `incumbentKind`,
filled from the comparison
row at both producers,
and `pageRelationLabel` reads `gap:<reason>`,
`emptied:<reason>`,
or
`silent:<reason>` for a row written before the field existed.
Three words rather than a name split,
so all three reasons are covered and no name is added.

IT ALSO CORRECTED `5dabe9c92`,
which had put `incumbentKind` on `WouldShipSlice`.
Two fields that must always agree with nothing enforcing it is worse than one where it is needed,
and the distinction only decides anything where nothing ships.

Nothing of the failed run is recoverable but the log.
The verification re-run is `~/temp/agent/xiept2-anchorfix-2026-08-24`,
launched 2026-08-24 04:47
from the fixed build.

## The published tree is read back now, and the first check that did it was too weak (`#197`)

FOUND 2026-08-24,
while looking for what else could fail the way `#194` did.

`#175` made the mirrored tree of fixed `page.en.md` files the thing this pipeline produces.
Every check built since reads artifacts.
An artifact is what the deciders said;
a page is what a reader gets.
`#194` is the price of the gap:
the publisher handed the assembler a blank rendering,
and the only thing that noticed was a guard inside the splice,
four hours and forty-eight minutes into the entry.

`verify-published` closes that.
It spends no quota and touches no model.
Run it with `TRANSLATION_REPAIR_RUNS_DIR=<dir> mise run //package/module/translation-repair:verify-published` (moved on
2026-10-18 to `//package/translation-repair/run:verify-published`).

### The first version passed a page with two hundred characters cut out of it

The check as first built was an ordered occurrence scan:
every wording the artifact says would ship must appear in the page,
in slice order,
without overlapping.
Its module note claimed
"a correct page always passes it,
and a page that lost or reordered a rendering cannot".

Run against four real run directories it reported everything clean.
Per QPC that is worth nothing until the probe is shown able to fail,
so the control was a throwaway copy of `163b-verify`
with two hundred characters cut from the middle of `dogesir_/page.en.md`.

    dogesir_: wordings=10 silent=0 pageChars=3640 missing=0
    CONTROL exit=0

`pageChars` fell by exactly two hundred and `missing` stayed at zero.
The check did not notice.

THE REASON IS STRUCTURAL,
not a coding slip.
The wordings cover the slices.
A page is the archive with the slice spans replaced,
so most of it is text no reading describes,
and a deletion there is invisible to any per-slice check.
The claim in the module note was true only of losses INSIDE a slice wording.

### What replaced it: arithmetic that covers the text no slice names

Splicing replaces each span with its replacement,
so the document grows by exactly what each slice added
and shrinks by what it removed,
and everything between the spans is carried through untouched.
That gives a prediction that needs no offsets:

    expected = archiveChars + sum over slices of (shippedChars - incumbentChars)

The archive text is on the artifact already,
stored by `#128` and decided in `doc/decision/artifact-stores-the-archive-text.md`,
which is what makes this reachable without adding a field.

MEASURED BEFORE IT WAS TRUSTED,
on six real published pages across four runs.
The prediction was exact on all six:
`gaoyanger` 349 to 370,
`dogesir_` 3716 to 3840,
`wangzihao980` 2133 to 2295,
`Acheron` 1269 to 1172,
`AmbeR_the_anpa` 2516 to 2576,
`Anilovr` 2236 to 2343.
Five grew and one shrank,
so the arithmetic is not a restatement of the archive length in either direction.

Both controls then failed the page and the negative control stayed clean:

    untouched:      chars=3840=expected            missing=0   exit=0
    minus 200:      chars=3640/expected 3840       missing=0   exit=1
    plus 200:       chars=4040/expected 3840       missing=0   exit=1

`missing=0` on both mutations is the point.
The occurrence scan still reports nothing wrong;
only the arithmetic refutes them.

### Where the arithmetic stops being an equality

A FILLED ANCHOR MAKES IT A FLOOR.
`spliceSlices` composes the separators around an inserted rendering
rather than carrying them in any row,
which `delivery-invariants.ts` states in the same words:
a concatenation of row texts differs from the document while nothing is wrong.
Those separators are real characters nobody counted,
so a page with one is longer than the sum predicts.
`pageWeightRefutes` is one-sided accordingly:
exact weighings must match,
inexact ones only set a floor,
and a page under its floor lost text either way.

An artifact written before `archiveText` was stored reports `UNWEIGHED` rather than agreeing,
so a run of old artifacts cannot read as a run that was checked.

### Both halves are kept, because neither subsumes the other

The order scan catches wording swapped between two slices of equal size,
which the length invariant cannot see.
The length invariant catches the two hundred character cut,
which the order scan cannot see.
Each has a test naming the other as the case it does not cover,
so removing one will fail a test that says why it was there.

Landed in `093b53001`,
572 PASS,
0 FAIL,
0 lint warnings,
types clean.

## The published-tree check now runs inside the pass, and its guard proofs

### The check now runs inside the pass, not only after it

A check nobody runs is not a check.
`verify-published` reads a run back after the fact;
`refusePageThatDisagrees` asks the same two questions inside `publishFixedPage`,
between the splice and the write.

That position is the whole design.
`pass-entry.ts` already reads its tally line before publishing,
and states why in the same terms:
a question that can refuse an entry has to be asked while refusing is still free.
Raised after the write it would leave a page on disk that no artifact accounts for,
inverting the ordering a resumed pass depends on.
Asked here,
a disagreement publishes nothing and settles nothing,
the pass reports the entry failed,
and the stage caches still hold every answer,
so a re-run reproduces the contradiction rather than losing it.

The archive handed to the guard is the text actually being spliced
rather than the copy the artifact stores,
so the weighing is an equality on every entry
instead of reporting an older artifact as unweighable.

REFUSING AN ENTRY AT ALL IS ONLY DEFENSIBLE BECAUSE BOTH CHECKS ARE ONE-SIDED.
The scan asks only that each wording occur in order,
and greedy leftmost matching finds an assignment whenever any exists,
so it cannot call a correct page wrong.
The arithmetic is an equality only where no anchor was filled and a floor otherwise.
A test pins the filled-anchor case,
because refusing a page longer than the sum
would stop every entry that fills a gap,
which is the work this pipeline exists for.

`corpus-barrel.ts` was at 299 code lines against a 300 cap,
so the publish chain moved to `publish-barrel.ts`,
following the seam `control-barrel.ts` and `generation-barrel.ts` used before it:
by audience.
The barrel falls to 271.

### GFP found a bad test rather than a bad guard

Four mutations,
each rebuilt and run,
then restored.

    delete the guard CALL from publishFixedPage    exit 1, 2 cases fail
    delete the LENGTH branch                       exit 1, 4 cases fail
    delete the WORDING branch                      exit 0, NOTHING FAILED
    restored                                       exit 0, 574 PASS

The third one is the finding.
The case meant to pin the wording branch used a page missing a rendering,
which is ALSO the wrong length,
so the arithmetic refused it and the assertions could not tell which branch had spoken.
`toContain('slice')` did not discriminate either,
because the length refusal says "every slice change".

Repaired two ways.
The assertion now looks for "in slice order",
which only the scan says.
And a new case swaps two settled renderings between their slices,
so the page holds both wordings,
is exactly as long as it should be,
and is wrong only in order.
It asserts both that the two pages are the same length
and that the arithmetic does NOT refute the swapped one,
so if that premise ever stops holding
the case fails rather than quietly measuring the other branch.

Re-run after the repair,
the wording mutation fails two cases and restores clean.

### The wiring had no test either

Deleting the guard call from `publishFixedPage` would have left every existing case green.
That is the same shape of gap that let `#194` ship.

The new case builds an artifact whose comparison row claims more archive wording at a slice
than the archive holds there,
so the artifact and the publisher disagree about what the slice covers.
The wording still ships and still lands in order,
so only the arithmetic notices.
It asserts the throw AND that no page exists afterwards,
since a guard raising after `writeFileAtomic`
would satisfy the first and still leave the page behind.
