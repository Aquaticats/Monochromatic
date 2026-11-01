# Translation repair history 2026-08-24 to 2026-08-25: the editor calibration built, the naturalness lane it never ran and the writer calibration's missing silent-model list

The provenance note of 2026-08-25 that counts six sections aged out of the working handover and moved unedited,
and that names the three sections kept there,
then the three aged-out sections filed here:
the editor calibration built once its blocker proved imaginary,
the naturalness lane the editor calibration never ran,
and the writer calibration that had no silent-model list,
found by auditing it against its own module note.
The other three aged-out sections are filed by subject in other files of this group:
the availability record and the record that carries the numbers are in `translation-repair-history-2026-08-24-to-25-telemetry-meter-timing-and-candidate-ledger.md`,
and the first reading of the zero-content anomaly on the deepseek seat is in `translation-repair-history-2026-08-24-to-25-provider-wire-boundaries.md`.
The three kept sections,
and the reseat evidence,
are in `translation-repair-history-2026-08-24-to-25-editor-calibration-reseat-evidence-the-14-slice-run-and-the-second-calibration.md`.

Part of [Translation repair history: 2026-08-24 to 2026-08-25](translation-repair-history-2026-08-24-to-25.md),
which lists every file of this record.

Historical evidence only:
no next-action list here is a current instruction,
and the stop condition in the index applies.

## Six sections aged out of the working handover on 2026-08-25 (second round)

Moved verbatim under the rule the working handover preamble sets:
history takes closed work whose conclusion is already encoded in the code,
and superseded reasoning kept only for its evidence.

KEPT IN THE WORKING FILE DELIBERATELY:
the sections a reader needs to INTERPRET the
standing when the calibration exits,
namely "Zero editor rounds does not mean nothing
was repaired",
"How many slices an editor calibration needs",
and "The 14-slice editor
calibration finished,
and it settles no seat".

## `#200`: the editor calibration is built, and its blocker was imaginary

### The finding that unblocked it

The task was held because a settled artifact exposes neither the envelopes nor
the issues an editor worked from,
so the claims looked unreplayable,
and the
choice looked like an artifact schema change or inventing claims as fixtures.

Neither is needed.
`ChunkRepairOutcome.rounds` already carries,
per round,
the
slate judges saw with each candidate's producer attached,
and every ballot cast
over it,
and both sides name the same `CandidateProducer` and `SelectionBallot`
out of `candidate-select-model.ts`.
That is `SelectionRound` one re-shape away.

So the calibration drives the lane live and reads what it records.

### What shipped

-   `repair-selection-rounds.ts` projects recorded rounds into the standing's
    shape,
    sorting by the position judges were shown and REFUSING a slate whose
    positions are not one to its length.
    A ballot names a candidate by number,
    so that is the assumption which cannot be checked afterwards.
-   `producer-standing-report.ts` holds the rendering both calibrations share.
-   `editor-calibrate` drives the whole repair lane,
    all ten editing and all ten
    judging every slice,
    and reports the EDITOR and REFINER standings off one
    spend.

### The one deliberate divergence from production

Checkers self-certify in this runner and nowhere else.
Production forbids a
checker proving its own repair,
and a full editor roster leaves nobody
independent when the roster is ten;
rotating editors out would reintroduce the
survivorship the shape exists to avoid.
Safe here because checking runs after
selection:
the ballots a standing reads are cast before any checker is asked.

### What is still owed

The MEASUREMENT,
which needs a full roster.
Charm Hyper has been dry all day,
so
a run now seats five of ten and produces exactly the survivorship the shape
prevents.
`budget-sample` makes the recovery observable;
run the calibration
once a sample shows `hyper=wet`.

A one-slice smoke run was made against the dry roster to prove the runner
executes.
It ran 7m18s and exited 0:
the checker assertions passed with the full
roster seated,
the five Hyper-only models were refused as lost voices,
and the
lane continued on the five that answered.

IT REPORTED ZERO ROUNDS,
AND THAT IS THE LANE WORKING.
Critics raised two claims
on that slice,
the panel adjudicated two issues,
and the lane then said
`chunk 5: nothing to edit, unchanged`,
because neither issue was ACCEPTED.
No
editor is asked to write on a slice like that,
so no round exists to count.

ADJUDICATED IS NOT ACCEPTED,
and that governs how the real measurement must be
sized.
A slice can buy ten critics and a ten-model panel and contribute nothing
to an editor standing.
The default of six slices may well yield very few rounds;
draw generously and read the new "from N of M slices" line before trusting a
standing.

Because a null from a probe never shown able to produce a non-null says nothing,
there is now a positive control in the unit suite:
real-shaped rounds driven
through the projection and the tally together,
asserting a standing falls out
with its counts.
That is what makes the live zero readable as "this slice had
nothing to repair" rather than "the instrument produces nothing".

## The editor calibration never ran the naturalness lane at all (`#200`)

2026-08-24,
found by watching the partial run rather than by reading code.

### The signal

Nine slices,
every one reporting `0 refiner rounds`,
and `grep -i refine` over the whole run log returning
nothing but my own progress lines.
A stage that declines still logs;
a stage that never runs does not.

### The cause

`repairChunk` does not reach refinement.
It accepts `refinerModelIds` only so `repair-contract.ts` can compute
the union of models the slice must seat,
sets `refined: false`,
and returns.
`runRefineStage` is called from `refine-slice-settle.ts`,
which the DOCUMENT driver runs afterwards,
per slice,
in `refine-phase.ts`.

So the module note in `editor-calibrate.ts` claiming

> IT REPORTS THE REFINER STANDING TOO,
> off the same spend

was false from the day it was written.
The refiner standing it printed was always going to be empty,
and an empty standing there would have read as
"the rewriters answered nothing",
which is a different and much worse claim than
"no rewriter was ever asked".

### The fix, in `f49cde75b`

`settleRefinedSlice` now runs on each slice off the same client,
and its rounds land on the outcome beside the accuracy lane's.
That call was already shaped for this:
it takes the `ChunkRepairOutcome`,
the source,
the incumbent and the models,
and derives eligible paragraphs itself.
Definitions are passed empty,
which is honest rather than a sentinel:
a drawn slice carries no document glossary behind it.

It also reports how many slices reached a rewriter at all.
A paragraph under the eligibility floor is never offered to one,
so a slice can buy the whole accuracy lane and reach no refiner,
and without that denominator the standing cannot be read.

### Why this mattered enough to fix mid-run

The refiner seat was reseated on 2026-08-24
on the same 40-round WRITING calibration the editor seat was.
It is exactly as unmeasured.
Measuring one seat while leaving the other
would have answered half the question `#200` exists for,
and would have done it while printing an empty table
that looked like an answer.

### What the partial run in flight will report

Its editor standing stands.
Its refiner standing will be empty,
and that emptiness means the lane was never run,
not that the rewriters were silent.
The full-roster re-run is the one that measures the refiner seat.

## The writer calibration had no silent-model list at all (`#200`)

Found 2026-08-24 by auditing `producer-calibrate.ts` against its own module note,
the same method that had just falsified `editor-calibrate.ts`'s note twice.

### What the audit confirmed and what it refuted

Three claims in the note hold:
`translatorModelIds: RUN_ROSTER` and `judgeModelIds: RUN_ROSTER`
really do seat every model as writer and judge,
self-votes really are discounted,
and every rate really does carry its count.

The suspected consensus blind spot is NOT there.
`editor-ensemble.ts:387` ships any sole candidate unjudged;
`translate-judge.ts:314` short-circuits only when the sole survivor IS the incumbent,
and its own comment gives the reason a sole FRESH candidate is still judged:
the repair pipeline's later safeguards,
the resolution checkers and the unchanged-versus-patched selection,
do not exist on the translate path.
On the editor path they do,
which is why the editor's short-circuit is a measurement gap and not a correctness one.

### The gap that was real

`producerStandings` carries a row only for a model somebody voted on.
A model its provider refused for budget writes nothing,
so it vanishes from the table,
and an absent row reads exactly like a model that wrote and lost.

`editor-calibrate.ts` named those models.
`producer-calibrate.ts` did not,
and that is the instrument that seated the current three writers,
on a day Charm Hyper was empty.

### What landed, in `eb60eac09`

`producer-silence.ts` splits a seated roster three ways:
judged,
wrote but was never voted on,
and wrote nothing at all.
The two silent groups are named separately because they call for opposite readings:
one is evidence already paid for,
the other is evidence nobody has bought yet.
The silent line carries both denominators,
as `covers N of M seats`.

A standing or a slate naming a model the run never seated raises `UnseatedStandingError`,
since coverage of one roster cannot be read off another.
Both inputs are checked,
because either one naming an unseated model
means the table and the roster came from different runs.

It is read AFTER the standings are printed in both callers,
so a run whose evidence disagrees with its own roster
still leaves every standing it paid for on stdout before the refusal.

`editor-calibrate.ts` drops its inline version and uses the shared one.

### A defect this found in the same file, introduced earlier the same day

`SliceRounds.shipped` was read off the REFINED outcome.
`issue-authors.ts:354` unions the editors with any refiner whose rewrite won,
so that field named both seats in one list,
printed under a heading a reader takes as editor credit.
With both rosters at all ten the mixing is invisible in the ids and wrong in the attribution.

The editor column now comes from the accuracy lane's own outcome,
before refinement.
The refiner column comes from a new `refinedBy` on `RefinedSliceOutcome`,
which names the models whose rewrite is actually in the text that ships:
empty on a non-translation slice,
on a rewriter that changed nothing,
and on a rewrite the recheck rolled back.
It is kept off the cached `RefinedSliceSettlement` for the reason `asked` is:
a slice resumed from disk bought no rewrite.

### Two tests were importing a path that does not exist

`artifact-rounds-read.unit.test.ts` and `digest-group.unit.test.ts`,
both written earlier the same day,
imported `../dist/final/node/index.mjs` from `src/corpus-run/`,
which resolves to `src/dist`.
The 72 sibling tests in that directory use `../../dist/`.
Both would have thrown at import.
Corrected before the suite ran,
so the suite never recorded a green run over them.

### State

Code and tests landed in `eb60eac09` and `8ed5da052`.
`producer-silence.ts` was exercised directly against source ahead of the build,
which is what caught the silent line reading `compares 1 models and not 4`.

All four checks are green as of `fc5dca624`:
build exit 0,
unit suite 628 suites passing and 0 failing at exit 0,
oxlint 0 warnings and 0 errors,
and `tsc --build` clean.
The lint pass went from 164 warnings to 0,
and it caught an auto-fix that had changed behaviour:
rewriting `error instanceof Error` inside a template literal
moved the ternary outside the template,
leaving a non-empty string as the condition
so the branch was always taken on a value typed `unknown`.

Type-check found two source defects nothing else had:
`requireJudgedRound` declared `RepairJudgedRound` and returned six of its fields,
and `editor-standing-read.ts` referenced a `DirectoryReading` type it never declared.

Both new CLIs are verified at the user boundary.
`budget-sample` now prints:

```text
METERS synthetic=wet hyper=dry syntheticWeekly=95.98% syntheticFiveHour=2750/2750 syntheticThrottled=no hyperBalance=0
```

and `meter-report` reads those levels back and prints them beside the verdict,
which closes `#202`.
`editor-standing-read` accounts for all 41 archived artifacts,
none of them malformed,
which closes `#203`.
