# Translation repair history 2026-08-23: discounting a checker's verdict on text it helped write

The half-weight discount for self-certifying checkers and why the roster half did not follow,
the correction that authorship must come from the shipped producer,
the finding that the checker-side discount is unreachable by design under the independence assertion,
and the arithmetic checked before that assertion is relaxed.

Part of [Translation repair history: 2026-08-23, the work recorded for that day alone](translation-repair-history-2026-08-23-single-day.md),
which lists every file of this record.

Historical evidence only:
no next-action list here is a current instruction,
and the stop condition in the index applies.

## Task 91: the self-certification discount, and why the roster half did not follow

Question 4 answer A is implemented.
A checker's verdict on text it helped write is heard at `SELF_VOTE_WEIGHT`,
the same one-half the selection stage already applies,
imported rather than re-declared.

AUTHORSHIP IS PER ISSUE,
which the task note did not anticipate.
It said "the winning editor's identity",
singular.
An `EditableEnvelope` carries `issueIds` plural,
and each envelope's selected round has its own winning producer,
so one patched text has several authors and the discount has to resolve per issue.
`producerModelIds` already maps a producer to every model with a stake in it,
covering composites and collapsed incumbents,
so nothing new had to decide what a stake is.

AUTHORSHIP EXISTS ONLY THROUGH APPLIED OPERATIONS.
An envelope the gate refused put no text into the candidate,
so the issues it named keep whole votes.
A round scoped to `CHUNK_SCOPE_ENVELOPE` authors every issue in the chunk,
including ones it was never told about,
which is why `IssueAuthorship` keeps `perIssue` and `everyIssue` apart
instead of flattening the chunk-wide case across an issue list it would have to be handed.

THE REFINE RECHECK DISCOUNTS BOTH STAGES.
Refined text is the editor's repair rewritten for naturalness,
so `retainsResolvedIssues` now receives the outcome carrying the rounds of both,
and takes its envelope map from `repairRegions`,
the regions the accuracy stage actually replaced.

`checker-sensitivity` passes `UNATTRIBUTED_TEXT` at all three of its calls.
Its sheets are hand-written fixtures no roster model wrote,
and discounting there would measure the discount rather than the checkers.

### A defect the suite caught, and one the probe caught

`selectedIndex` is the ONE-BASED number of the winner on the slate the judges were shown,
and its own doc warns the caller may rotate the slate first.
Reading `slate[selectedIndex]` therefore named the neighbour,
and named nothing at all when the winner sat last.
`refine-phase.unit.test.ts` threw `Expected non-nullish value` on three cases immediately.
Fixed by matching `entry.index` against `selectedIndex`,
the only join a rotated slate survives.

THE FIRST VERSION OF THE GUARD FOR THAT FIX PROVED NOTHING.
It used a rotated slate,
winner numbered 1 at position 1,
where `slate[selectedIndex]` lands on the winner by luck;
the case passed with the defect restored.
Natural order is the discriminating shape:
winner numbered 1 at position 0,
so indexing names the LOSER.
This is the second time in this work a guard has looked green while guarding nothing,
after the barrel-export false null recorded under task 181.

### GFP

Removing the discount fails five weight-dependent cases,
node exits 1,
PASS suites 2 to 1,
with zero import errors,
so the bundle loaded and the failures are the assertions.
The positive control beside the discriminating case,
identical ballots with nobody named as an author,
still passes,
which is what shows the assertion reads the discount rather than the ballots.

Restoring the array index fails the join case with a value mismatch rather than a throw,
plus the composite case with a throw,
covering both failure modes.

### What #84's numbers say about each half

CORROBORATES THE HALF.
#84 measured self-preference paired across two draws sharing no slices:
own 0.367 against 0.220 for judges holding no stake,
excess 0.147,
lift 1.67 times.
A discount dividing by two is the right order and slightly more aggressive than the measured effect,
so the owner's "consistency is its only argument" now has a second argument.

DOES NOT SUPPORT THE ROSTER HALF,
which is why that half is not in these commits.
Question 1 answer D said the producing roster stays at three
"until #84 measures judge quality on preserve-or-replace,
then widens on those numbers".
#84 reported,
and its numbers do not carry that decision:

- Its width series is `NO WIDTH TREND`,
  and it is a series about SELF-PREFERENCE by width,
  0.083 then 0.182,
  non-monotone and mostly inside its own band.
  It says nothing about whether wider PRODUCTION yields better candidates.
- Its one width-correlated quality-adjacent number is replacement rate,
  0.83 at widths two to four against 0.94 at widths five and six.
  Replacing the incumbent more often is churn,
  not quality;
  reading it as quality assumes the thing being asked.
- Its abstention finding cuts the other way.
  `gpt-oss-120b` and `Nemotron-3-Super` decline most slices carrying any archive imperfection,
  so on exactly the slices that matter the effective roster is about four voices,
  not six.

The cost side was already measured:
two producers to six multiplies calls by 1.7 and tokens by 1.8.
Cost is not the constraint here;
the missing measurement is.
Widening now would act on a number that does not measure what the decision needs.
Carried on its own task with the probe it would take.

FINAL GATE:
suite 546 PASS / 0 FAIL / exit 0,
lint 0 warnings 0 errors,
`lint:types` exit 0.
Commits:
bd972fbc2,
664cbc42f,
de3e8c51f,
583005335,
23f1432e4.

## Task 187: the discount was reading the wrong source, and missed the author on a reachable path

Task 91 shipped the self-certification discount and its arithmetic is unchanged by this.
What moved is where authorship comes from.

### What was wrong

`collectIssueAuthors` reconstructed "who wrote the shipped text" from the judged rounds.
Two directions of error,
one of which defeated the feature outright:

- UNDER-DISCOUNTED THE AUTHOR.
  When the whole-chunk judges decline with `indecision`,
  `editor-ensemble.ts` ships `pickFallbackPatch({ candidates: repairing, },)`,
  a real editor's applied operations.
  The round is recorded as `declined`,
  and a declined round has ballots and no winner,
  so `roundWinnerAuthors` credited nobody.
  Creditable issues existed and the checkers ran,
  and the model that wrote the shipped text certified its own work at full weight.
  That is exactly what task 91 promises to stop.
- OVER-DISCOUNTED NON-AUTHORS.
  Envelope-round winners were credited for their envelope's issues.
  The composite is assembled from those winners,
  but a single model's whole-chunk proposal competes against it and can win,
  and when it does their text did not ship.
  Direction-blind,
  like the discount itself,
  so not a conservative error.

Neither was reconstructable.
The information is not in `authorsFromRounds`' input,
so no patch to that function could have been correct.

### What was verified NOT broken, rather than assumed

- The unconditional chunk-scope credit is CORRECT BY CONSTRUCTION.
  The apply gate runs BEFORE selection (`editor-ensemble.ts` takes `Candidate<PatchOutcome>[]`),
  and zero-applied candidates are excluded twice:
  `repairing` filters on `applied.length > 0` in `repair-editor-stage.ts`,
  and the composite joins the slate only when it applied something in `editor-candidates.ts`.
  A selected chunk round therefore cannot have a winner that wrote nothing.
- Refine's declined branches are clean.
  Both dispositions ship `unchanged` in `refine-stage.ts`,
  so no candidate's text ships there without a winner.
- No settled artifact is affected.
  Rounds are ephemeral and no artifact field carries them,
  confirmed by scanning both run directories.

### The shape it landed in

The stage knows the shipped producer at every exit,
so it states it instead of leaving it to be inferred.
`ChunkPatchSelection` and `EditorStageResult` carry `shippedProducer`,
`pickFallbackCandidate` returns the candidate rather than a bare patch so the declined path has one,
and `chunkCandidateOf` builds slate and fallback the same way so the two cannot drift.

Authorship then reads that producer.
A `model` producer wrote the whole chunk and answers for every creditable issue.
A `composite` defers to the envelope rounds that assembled it,
which is the one case where parts of the shipped text have different authors.
`NOBODY_WROTE_IT` means the untouched translation shipped.

Absence is a NAMED STATE rather than `CandidateProducer | undefined`.
The repo forbids nullish unions and was right to here:
shipping the translation as it stood is a decision two exits reach on purpose.

`ChunkRepairOutcome` carries `authorship` so the naturalness recheck can still name the editor after a resume,
which cost `SLICE_CACHE_VERSION` 30.
`collectRefinedAuthors` unions the refiners onto it from `RefineStageResult.contributors`,
a field whose own documentation claimed callers barred its models and which had no readers at all.

### GFP, both directions, zero import errors on each

- Probe A restored the rounds reconstruction.
  Three cases failed:
  the lone winner,
  the indecision fallback,
  and the envelope winner whose composite lost.
  The positive control,
  `NAMES NOBODY WHEN THE UNTOUCHED TRANSLATION SHIPS`,
  still passed.
- Probe B dropped the refiners from the union.
  Two cases failed.
  The positive control,
  a lost refinement adding nobody,
  still passed.

### Also fixed on the way

`splice-slices.unit.test.ts` had hand-copied the whole `ChunkRepairOutcome` shape as an inline return type,
which is why it drifted the moment the record grew.
It names the contract now.
`unchangedChunkOutcome` moved to `repair-unchanged-outcome.ts`,
the max-lines remediation for `repair-chunk.ts` reaching 301,
which also stops three exits restating one record shape.

FINAL GATE:
suite 546 PASS / 0 FAIL / exit 0,
lint 0 warnings 0 errors,
`lint:types` exit 0.
Commits:
9b433ea4f,
e796d2956.

## Task 187, corrections found while covering the wire

Three corrections to the section above.
None of them change what shipped;
two change what a later session should believe about it.

### "Reachable path" was true of the code and false of production

The heading above says the discount missed the author "on a reachable path".
Reachable in code,
yes.
Reachable in a corpus pass,
no,
and the distinction matters because it decides whether the machinery earns its complexity.

`wroteTextForIssue` weights a checker's verdict at half when that checker appears in `IssueAuthorship`.
Authorship can only ever name an editor
(`producer.modelId`,
`producer.matched`,
`producer.contributors` at `candidate-select-model.ts:136-139`)
or a refiner,
added by `collectRefinedAuthors`.
`assertCheckerIndependence` at `repair-contract.ts:359` refuses any roster where a checker is also an editor
or a refiner,
and it runs at `repair-chunk.ts:126` and `refine-phase.ts:153`,
so every chunk and every refine phase is gated on it.
The production roster at `corpus-run/run-config.ts:192`,
the only `RepairModels` literal outside TSDoc examples,
is disjoint:
editors and refiners are Kimi-K3,
GLM-5.2 and GLM-4.7-Flash,
checkers are Qwen3.8-27B,
Nemotron-3-Super-120B and gpt-oss-120b.

So the checker-side discount has changed no tally that ever shipped.
Task 187's corrections are right and covered;
they are also currently inert,
and so is the half of task 91 that built them.

SCOPE THIS PRECISELY.
`SELF_VOTE_WEIGHT` has two independent readers and only one is unreachable.
The SELECTION side at `candidate-select.ts:276` halves a JUDGE's vote for its own candidate,
`judgeModelIds` is the whole six-model roster,
and the quorum threshold at `repair-contract.ts:288` is written around it.
That copy fires on every chunk;
the suite log shows it live
(`hf:zai-org/GLM-5.2 chose candidate 1 at weight 0.5`).
Do not delete it.

Whether to drop the checker-side apparatus or instead relax the assert and let the strongest editors check too,
with their self-votes discounted,
is a design question with a measurement behind it.
It is filed,
not decided here.

### The stored authorship went stale the moment a refinement was kept

`repairedText` became the refined text while `authorship` stayed editor-only,
so every `refined: true` record misstated its own contract.
The recheck's union lived in an argument and died with the call.
Fixed at the shipping return only:
both rollback paths hand back the editor's text,
so the editor's authorship is already true of them.

No cache bump was needed.
`SLICE_CACHE_VERSION` 30 is baked into the key,
and no pass has run since it landed,
so there is no v30 entry to go stale.

GFP:
removing the stored union fails `STORES BOTH STAGES ON A RECORD WHOSE REWRITE SHIPPED`
with `expected [ GLM-4.7-Flash ] to deeply equal [ GLM-4.7-Flash, GLM-5.2 ]`,
while the control `NAMES NO REFINER ON A SLICE IT ROLLED BACK` still passes.
Restored byte-identical,
zero import errors on the probe.

### Test files are exempt from max-lines

The section above splits a test file at the 300-line budget.
That split is fine to keep,
but it was self-imposed:
`package/config/oxlint/src/overrides.ts:161` turns `eslint/max-lines` off for `**/*.{test,bench}.ts`.
Only source files carry the budget.

FINAL GATE:
suite 546 PASS / 0 FAIL / exit 0,
lint 0 warnings 0 errors,
`lint:types` exit 0.
Commits:
3922cba3b,
f3c444458.

### Correction to the section above: the unreachability was already recorded

`doc/decision/translation-repair-question-answers.md` lines 130 to 140 already say it,
and say it as a deliberate state rather than a defect:

> `assertCheckerIndependence` in `repair-contract.ts` THROWS when any checker id also appears among the editors
> or refiners.
> A roster where a model could certify its own text is refused before any work happens,
> so no
> checker is ever a self-certifier and a half-weight branch would never run.
>
> Relaxing that assertion is the "widen the producing roles" half of `#91`,
> and question 1 answer D blocks
> roster widening until `#84` reports.
> So the ordering is forced:
> `#84`,
> then the roles widen,
> then the half
> becomes reachable and is applied.

So the checker-side discount is unreachable BY DESIGN,
pending task 186,
not by oversight.
Task 188 is the "and is applied" step,
blocked by 186,
not a discovery.

What the 2026-08-23 reading did add,
and what is worth keeping:
the two readers of `SELF_VOTE_WEIGHT` must not be conflated.
The selection-side copy at `candidate-select.ts:276` is live on every chunk and must survive any cleanup
aimed at the checker-side one.

The lesson for a later session is the search,
not the fact:
`grep -rn 'SELF_VOTE_WEIGHT' src/` found the code
and `grep -rn 'assertCheckerIndependence' doc/` would have found the decision.
Searching the source and not the decisions is how a recorded choice gets rediscovered as a bug.

## Task 188, the arithmetic checked before the relaxation is written

Read on 2026-08-23,
while task 186's probe was in flight,
so the relaxation could be written the moment
the width question resolves rather than after it.

The worry worth settling first was whether relaxing `assertCheckerIndependence` would leave the discount
doing nothing.
If every checker is also an editor,
and every checker were therefore halved,
the discount would cancel out of `tallyResolutionChecks`:
its rule is `resolved: fixed > (notFixed + worse)`,
a comparison of two weighted sums,
and scaling both sides by one half leaves every verdict exactly where it was.
A uniform discount over a fully overlapping roster is not a safeguard,
it is a no-op.

It is not uniform.
The weight is chosen per issue at `package/module/translation-repair/src/tally-resolution.ts:265` (moved on 2026-10-18
to `package/translation-repair/adjudication/src/tally-resolution.ts`),
by `wroteTextForIssue({ authorship, issueId, modelId, },)`,
so a checker is halved only on the issues whose shipped text it actually helped write
and keeps full weight on every other issue in the same slice.
A six-seat roster checking its own slice therefore still produces a mixed tally
rather than a uniformly scaled one.

This is what makes task 187 a prerequisite rather than a neighbour.
The discount reads `authorship`,
and until 187 the record could carry an author list describing pre-refinement text,
which would have halved the wrong checkers on the wrong issues once the roster overlapped.

What task 188 still owes when it runs,
none of which this reading settles:
the assertion's duplicate-checker refusal must survive,
because a repeated id is a different fault from an overlapping one
and `gatherStageVoices` still counts a repeat twice toward quorum
while `runCheckerStage` collapses it to one ballot;
`assertJudgeableProducerRoster`'s capacity arithmetic at `repair-contract.ts:288` must be re-read
at whatever width 186 lands on;
and the selection-side reader of `SELF_VOTE_WEIGHT` must not be touched.
