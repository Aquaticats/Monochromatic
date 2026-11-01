# Translation repair history 2026-08-05 to 2026-08-07: the introduced-defect probe, its telemetry and the chain that joins it to human grades

Building the shadow-mode probe that looks for defects a repair introduced:
the misread of the user's answer,
the omission direction added to the screen,
the forged-sheet fix,
the readable telemetry,
and the measurement chain that joins probe and human grades.

Part of [Translation repair history: 2026-08-05 to 2026-08-07](translation-repair-history-2026-08-05-to-07.md),
which lists every file of this record.

Historical evidence only:
no next-action list here is a current instruction,
and the stop condition in the index applies.

## The introduced-defect probe built, its telemetry and the measurement chain to human grades

### Run 004: the introduced-defect probe, and a misread of the user I had to undo

Commits `21f5e9092` (probe) and `e3ba6d325` (integration tests).
The pass restarted as `pass8-run-004`.

#### What I got wrong first, because it will recur

The user was asked two questions.
Whether regressions should gate or rank,
offered as four options,
and whether to build a check for defects nobody had raised.
They answered the second with "Build it now"
and the first with "I believe there is a better option than those 4 you
listed",
naming none.

I then told them my reading was "build the check,
let it gate",
and called that their better option.
That is supplying an answer to the one question they withheld.
My own hedge in the same message,
"if that's not the better option you had in
mind,
tell me",
was the tell that I knew I was guessing.

The rule this earns:
a declined menu is not a delegation.
When a user rejects every option and names no replacement,
the question is still theirs,
and the only honest move is to do the part that WAS authorized and leave the
rest open.

#### Shadow mode, and why it is also the better engineering

The probe records and decides nothing.
`compareCandidates` is untouched.

That is not only deference.
The probe's failure mode is known before its first call:
every region it inspects contains a defect BY CONSTRUCTION,
since that is why the region was edited,
so a model asked whether anything is wrong will find something.
Its false-positive rate is unmeasured.
Wiring an unmeasured stage into a blocking position would let one bad verdict
discard a whole chunk's repair including fixes in other envelopes.
The 98.1 percent checker confirmation rate already on record is the reason to
measure a new prompt rather than assume a differently worded one discriminates.

Round three's artifacts plus the human repair-sheet grades
("fully fixes this defect AND breaks nothing nearby") are the measurement.
Re-open the gate question only with those numbers in hand.

#### Three defenses against the known failure mode

The verdict vocabulary offers NO `clean`.
It is `introduced-defect`,
`no-introduced-defect-found`,
`uncertain`.
`clean` would be false of a region whose original defect survives,
and forcing a prober to choose between `clean` and a defect verdict would push
every such region into the defect bucket.
The long negative name says what a negative verdict actually proves.

The pre-existing accepted issues are rendered into the sheet under
"PRE-EXISTING DEFECTS THIS EDIT TARGETED (these are NOT your findings)".

Every claim must quote the damaged wording from the AFTER text,
and `introduced-defect-screen.ts` then judges the quote with no model involved:
present in AFTER and absent from BEFORE is `corroborated`,
already present in BEFORE is `contradicted`,
missing or unfindable is `unanchored`.
Whitespace is collapsed on both sides first so a rewrapped quote still resolves.
That follows `screenNonTranslationVotes`:
deterministic evidence DISMISSES an impossible claim and never has to prove a
possible one.
There is no mechanical test for mistranslation,
so demanding positive proof would blind the probe to its own subject.

#### Where the telemetry lands

`ChunkRepairOutcome.introducedDefects` carries the whole report;
`RepairIssueRecord.introducedDefects` carries the tallies for the regions
serving THAT issue,
so a sheet item and the probe's opinion of the same item
join up inside one artifact record.
Both optional,
absent meaning unprobed.
The corpus artifact writer serializes `result.issues` wholesale,
so no writer change was needed.

`SLICE_CACHE_VERSION` 3 -> 4.

#### Also landed, found by sol while reviewing the probe

`assertCheckerIndependence` now refuses a checker roster listing one model
twice.
`gatherStageVoices` counts a repeated id's replies separately toward quorum
while `runCheckerStage` keys ballots by model id and collapses them,
so a three-model roster with a repeat could report quorum on what is really one
independent voice.
`assertJudgeableProducerRoster` already refused repeats for producers.
Latent,
not live:
the configured roster has no duplicates.
Verified.

Sol also flagged that `repair-chunk.ts` calls `assertCheckerIndependence`
without `refinerModelIds`.
CHECKED,
and it is not a defect:
`refine-phase.ts:111` makes the same call WITH refiners before any refinement
runs,
and returns early when the lane is off.
The chunk-level call happens before any refiner has written anything.

#### Gaps left open on purpose

The probe inspects the ACCURACY patch only.
The refiner lane is on (Kimi-K3,
which also edits),
so a defect the naturalness
rewrite introduces is invisible to it.
Sol's recommendation was to probe after every text-producing stage.
Not built:
it doubles the probe's cost and round three needs the accuracy
measurement first.
Watch for it when reading grades where `refined` is set,
since a human grading the final wording can mark N for damage the probe never
saw.

Sol's Q1 preference was a writer-disjoint DISCOVERY roster separate from a
confirming one,
so no model confirms its own claim.
The probe uses the checker roster for both.
Recorded because it is the first thing to try if the measured precision is poor.

#### File-budget moves

The critic stage plus its vote screening moved to `chunk-critic-phase.ts`,
and the probe exports to `probe-barrel.ts`,
both because the additions pushed `repair-chunk.ts` to 310 lines and
`pipeline-barrel.ts` to 324.
Split,
never raised (MXL).
`refine-barrel.ts` was the existing precedent for a second barrel.

#### The integration gap the warnings exposed

The pipeline stub in `repair-translation.unit.test.ts` had no script for
`introduced_defect_report`,
so every end-to-end run lost all prober voices and
the probe returned an empty report:
the wiring was never exercised and the suite would have stayed green if the
tally never reached the records.
Three cases now cover it,
and the middle one is the load-bearing assertion:
a defect EVERY prober corroborates must still ship.
That test fails the moment anything downstream starts reading the report,
which is what pins shadow mode against a future accidental gate.

### Run 005: the probe could not prove an omission

Commit `ec92567a5`.
Found by the advisor review immediately after run 004
started,
so the pass restarted again as `pass8-run-005` with zero artifacts lost.

The screen ran the differential in ONE direction only:
a quote had to be present in the AFTER text and absent from BEFORE.
The prompt said as much,
"quote the exact damaged wording FROM THE AFTER TEXT".

Omission damage has nothing in the AFTER text to quote.
Its absence IS the defect.
So every claim of the form "this edit dropped a clause" landed in `unanchored`
however true it was,
and a region the editors emptied outright could not be claimed against at all.

That is worse than a crash for a measurement instrument.
Dropping a clause while rewriting is among the likeliest ways an editor causes
collateral damage,
so `unanchored` would have filled with exactly the class that matters most,
and I would have read the round-three telemetry as "probers gave unusable
quotes" rather than "the screen cannot express this claim".
Sol's earlier review had flagged it in advance,
"zero-width insertions and empty replacements need explicit boundary-anchor
support",
and I shipped without it.

The fix is symmetric and was always available.
Wording present in BEFORE and absent from AFTER proves the edit removed it,
exactly as mechanically as the forward check proves it added something.
Claims now carry `evidence` for added damage or `omittedText` for dropped
content,
and `removal-corroborated` is its own count so the two never blur.

EXACTLY ONE anchor per claim.
Both at once is a wire fault,
not a stronger claim:
screening each and taking the better answer would let a prober launder a
contradicted quote by attaching a second one.

#### Two aggregation traps, recorded on task 53

Neither is guarded in code and both silently mix populations.

FILTER TO SHIPPED.
The probe runs wherever an operation applied,
including candidates selection
later rejected,
so `introducedDefects` sits on records whose disposition is
`not-selected` or `withdrawn`.
The human repair sheet grades only `shipped` items.
Without the filter the denominator includes regions nobody graded.

DE-DUPLICATE BY ENVELOPE ID.
Every issue sharing a merged envelope carries the SAME tally,
so aggregating over records counts one region once per issue it served.

#### Lint debt cleared while here

Commit `9183e3128`,
on the user's "Fix even pre-existing issues".
Nineteen warnings to zero:
Unicode blocks named in `protected-atom.ts`,
a real type guard replacing `verdict as GradeVerdict` in `grade-agreement.ts`,
the JSON round trip in `repair-provenance.unit.test.ts` split into a named
serialized form,
and the rate precision named in `score-agreement.ts`.

The `structuredClone` suggestion was REFUSED with reason rather than applied:
that test exists to cross the disk boundary,
and JSON drops what a clone keeps,
which is precisely what every optional field on a repair record depends on.

The remaining 150 errors are all `prefer-readonly-parameter-types`,
ignored on this branch by user decision under issue #414.

### Runs 006 and 007: the telemetry becomes readable

Commits `acdcd39ee`,
`b111fc376`,
`9533b0ba8`,
`a13ea1acf`,
plus tests in
`01a46e265` and README in `0bcb1328f`.

#### A quiet probe line said nothing

The first live line read
`3/3 heard over 1 regions, 0 corroborated, 0 contradicted, 0 unanchored`,
which is equally consistent with every prober finding nothing,
every prober declining,
and every ballot being dropped as a wire fault.
Those are three different states and the line could not tell them apart.
The negative verdicts now print beside the positive ones.

NOT restarted for this one:
artifacts already carried the full breakdown,
so the change affects what a run is readable as WHILE it runs,
not what it
records.
The two wire changes below did force restarts,
because telemetry has to come
from the shipped prompt.
On run 007 the same line reads `3 found nothing, 0 declined`,
which is the verification the terse line could not give.

#### Corpus prose could forge the probe sheet

The sheet fenced with a fixed `=====` and marked regions with bare `REGION n`,
`BEFORE:` and `AFTER:` lines.
A setext heading underline IS a row of equals signs,
so a translation containing one could close its own block and have the rest
read as sheet structure.

The package had already solved this TWICE and said so out loud:
`markdown-fence.ts` chooses a fence against enclosed content,
and its own
comment points at `candidate-select-wire.ts` doing the same for a prompt.
The probe shipped without either.
`selectFence` now lives in `prompt-fence.ts` and both use it.
Extracted rather than copied:
two implementations of a boundary deciding whether model-facing text can
impersonate instructions is one too many.

The adversarial test caught ME rather than the code.
The first assertion checked `sheet.includes('===== END =====')` was false,
but `====== END ======` CONTAINS that string,
so it would have passed at any fence width while proving nothing.
Line comparison,
not substring,
whenever a delimiter is the subject.

#### The records could not answer the question they existed for

Each issue record carried its regions' tallies but not the roster size.
Heard voices are recoverable by summing a tally's verdicts;
the CONFIGURED roster is recoverable from nothing.
So no artifact could answer whether a MAJORITY agreed,
which is the only thing the gate decision turns on.
Records now carry `IssueProbeReading`:
tallies plus both roster counts.

This is the second time review caught a defect the probe's own passing tests
did not.
Both times the tests checked what the code did rather than what the measurement
would need.

#### Reading it back

`summarizeProbeTelemetry` holds the two joins no type enforces:

-   DISTINCT ENVELOPES.
    Every issue of a merged envelope carries the same tally,
    so summing over
    records counts one region once per issue it served.
-   SHIPPED ONLY.
    The probe runs on candidates selection later rejected;
    the repair sheet grades only what shipped.

Majority is measured against the CONFIGURED roster,
never the heard one:
retry-to-quorum lets six models settle with three heard,
and a majority of those would be two probers speaking for six.
Unheard voices count as non-confirming,
the conservative direction for a probe whose false positives discard correct
repairs.
Contradicted claims count as NO evidence,
not weak evidence.

`readArtifactProbe` deliberately breaks with `artifact-read.ts` doctrine.
That reader feeds the precision gate and throws on everything malformed.
Here absence and malformation differ:
a record with no probe field is ordinary,
a field PRESENT and malformed means writer and reader disagree.
First is counted,
second throws.
Claims are dropped rather than parsed,
since they carry corpus quotes and the
CLI output must stay safe to paste where artifacts are not.

Run it with
`mise run //package/module/translation-repair:score-probe` (moved on 2026-10-18 to
`//package/translation-repair/probe:score-probe`).

Verified against a THROWAWAY fixture,
not an empty directory,
which proves
nothing:
two shipped records sharing one merged envelope collapsed to `regions=1`,
and a `not-selected` record carrying three corroborated claims was excluded,
so `majorityIntroduced` read 1 and not 2.

#### Restart ledger

Runs 004 through 007,
four restarts,
ZERO artifacts lost:
every one happened inside the first entry.
Wire or record-shape changes restart;
log wording and read-side code do not.

### The measurement chain, completed while the pass ran

Commits `f1737d6b4`,
`9d1dec09c`,
`51960a35c`,
`f93d86617`,
`aa8b9b1ce`.
Read-side only,
so `pass8-run-007` kept running throughout.

Three holes,
each of which would have made the round-three telemetry
unusable,
and one of them unrecoverably so.

#### Nothing could read a graded repair sheet

`formatRepairSheet` had existed since the two-sheet split and the runbook told
the user to fill it in,
but `parseGradedSheet` reads the DETECTION format only.
So the repair-quality number the second sheet exists to produce was
unobtainable,
and so was anything needing it.

`parseGradedRepairSheet` TRACKS FENCES,
which is why it cannot be a line filter.
The sheet quotes corpus prose that may contain a literal
`- repair grade: [Y]`,
and `repair-sheet.ts` fences that text precisely because it might.
A parser ignoring the fence would let quoted text fabricate a human verdict.
That is strictly worse than dropping one:
a missing grade shows up as an unscored item,
an invented grade shows up as
evidence.

`opensWithVerdict` and `trimLeadingDelimiters` moved to `verdict-letter.ts`.
Both sheets need the identical one-character rule separating a verdict from a
word starting with the same letter,
and two copies would drift into two
denominators.

#### The draw recorded nothing about what it drew

THIS IS THE UNRECOVERABLE ONE,
and it was caught with hours to spare.

The draw wrote two sheets and nothing else.
Sheets are numbered positions;
every machine verdict is keyed by issue id;
and the sheets deliberately print no issue id because a 64-hex string is noise
a grader reads past.
Re-running the draw does not recover the mapping:
the draw is deterministic in its SEED but not its POOL,
and the pool grows with every entry that settles,
so a draw taken at fifteen entries stops reproducing when the sixteenth lands.

Had round three been drawn before this landed,
the probe would have spent the whole round recording evidence for a question
nothing could ever ask.
`sample-manifest.json` is now written in the same breath as the sheets,
seed-named and overwrite-protected exactly as they are,
identifiers only.

#### The two instruments could not be asked about the same item

`probe-agreement.ts` joins them at ISSUE level,
not region level,
because that is the level they share:
a merged envelope serves several issues and the human grades issues,
so
"what did the human say about this region" has no answer when those issues were
graded differently.

ONE CELL IS EVIDENCE AND THE REST ARE NOT.
A repair grade of `Y` means "fully fixes this defect AND breaks nothing nearby",
so `Y` beside a probe finding is a direct human refutation:
each `refutedByHuman` is a correct repair a gate would have discarded,
and that
is the number a gate proposal has to answer for.
`N` is ambiguous BY CONSTRUCTION,
firing both for a repair that did not fix its
target and for one that damaged something,
so `sharedWithHuman` is reported as suggestive and never as confirmation,
and
`unflaggedFailures` is an upper bound on misses rather than a count.

Sheet and manifest lengths must agree or the run throws.
Joining across a disagreement would not lose a verdict,
it would MISLABEL every verdict after the divergence,
which nothing downstream
could detect.

#### Run it

```bash
mise run //package/module/translation-repair:score-probe -- \
  --repair-sheet /ABSOLUTE/path/to/repair-sheet-<seed>.md \
  --manifest /ABSOLUTE/path/to/sample-manifest-<seed>.json
```

Moved on 2026-10-18 from `//package/module/translation-repair:score-probe`
to `//package/translation-repair/probe:score-probe`.

Verified end to end on a throwaway fixture,
never on the run's own directory:
a probe-flagged issue graded `Y` scored `refutedByHuman=1`,
a probe-clean issue graded `N` scored `unflaggedFailures=1`,
two shipped records sharing one merged envelope collapsed to `regions=1`,
a `not-selected` record was excluded entirely,
and a truncated sheet threw rather than joining.

#### What is left, and what it waits on

Everything buildable without round-three data now exists.
The remaining items wait on the pass reaching fifteen settled entries:
task 53's gate decision (needs the grades),
task 48's blind pre-grade calibration (needs the sheet;
rounds one and two are NOT usable for it,
because those graded sheets may have
been read this session and blindness cannot be claimed),
task 51's recall re-measure (would contend for quota with the pass),
and task 58's refinement probe (needs the accuracy probe's measured
false-positive rate first).
