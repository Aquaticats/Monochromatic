# Translation repair history 2026-08-23 to 2026-08-24: keeping the ballots of the refinement recheck and the accept gate

The refinement recheck and the accept gate each recording their ballots on the settled artifact,
why a resumed run cannot be crashed by the new fields,
how much evidence was being discarded,
a false verification traced to a shell trap,
and the finding that the first live artifacts lacked the fields only because their builds predated them,
confirmed by a later artifact.

Part of [Translation repair history: 2026-08-23 to 2026-08-24](translation-repair-history-2026-08-23-to-24.md),
which lists every file of this record.

Historical evidence only:
no next-action list here is a current instruction,
and the stop condition in the index applies.

## The refinement recheck keeps its ballots too (`#192`)

WHAT WAS MISSING.
The naturalness lane asks the checkers a second time,
about the text a refiner
rewrote,
and until this landed that round left one finding and nothing else:
`refine-recheck-passed`
or `refine-rolled-back (<issue ids>)`.
A rolled-back slice therefore named the issues it lost and
never named who called them lost.
That is the same gap the deciding round had one stage over,
and it
was scoped out of that landing deliberately rather than missed.

WHAT LANDED.
`ChunkRepairOutcome` gains `recheckReadings` (required,
so every construction site says
whether a round ran) and `RepairIssueRecord` gains `recheckReading` (optional,
so an issue nobody
rechecked is absent rather than falsely unanimous).
`retainsResolvedIssues` now returns the readings
it already had in hand,
and both of its callers attach them:
the rollback path as well as the
keep path,
because the rollback is the case whose evidence matters most.

WHY IT IS A SECOND FIELD RATHER THAN AN UPDATE TO THE FIRST.
Both rounds rule on the same issue ids,
but `resolved` rests on the first alone.
The recheck is a rollback gate:
it keeps or discards a
rewrite whole and never revises `resolvedIssueIds`.
A reader that merged them would present a
verdict about text that may have been thrown away as the verdict behind what shipped,
and would hide
a checker that answered differently between the two rounds.

GFP,
BOTH DIRECTIONS,
each mutation applied to a committed guard and then reverted:

-   Making `retainsResolvedIssues` return `readings: {}` failed both new `runRefinePhase` cases and
    left the third one passing,
    which is the control:
    a slice with no confirmed issue buys no round,
    so an empty reading there is correct rather than a loss.
-   Removing the `recheckReading` spread in `repair-record.ts` failed the new disk-boundary case in
    `repair-provenance.unit.test.ts` and nothing else.

THE ANALYSER READS BOTH ROUNDS NOW,
labelled rather than merged,
and was positive-controlled on a
synthetic artifact carrying one deciding round that cannot flip and one recheck round built to flip.
It reports `FLIPS: 1 of 3` on that file with the flipping row labelled `recheck`,
so a null from it
is a null about the data rather than about the reader.

### Interim reading at 71 rounds

Four wide passes now,
422 ballots over 71 checker rounds:

```text
rounds carrying at least one self-vote                     68 of 71
rounds where the narrow three split                         2 of 71   <- ceiling on flips
rounds where any of the six dissented                       5 of 71
rounds where a writer said something no narrow checker did  3 of 71
FLIPS                                                       0 of 71
```

TWO OF THE FOUR PASSES ARE THE SAME ENTRY,
`lintong`,
run twice:
`checker-width-wide-2026-08-24` and
`checker-reading-vub-2026-08-24`.
Those two contribute 38 of the 71 rounds and rule on the same
underlying issues,
so they are correlated and the effective sample is smaller than 71.
Recorded here
because a later reader summing the four directories would otherwise treat them as independent.

The picture has not changed shape since 39 rounds:
the ceiling is still 2,
and the reason no flip
has been seen remains that this panel almost never splits.
One round in the batch pass lost a voice
and was decided by five rather than six,
which is the only panel-size variation observed.

### Why adding a required outcome field cannot crash a resumed run

THE WORRY,
WHICH IS REAL AND ALREADY ANSWERED.
`isChunkRepairOutcome` in
`package/module/translation-repair/src/corpus-run/slice-cache-store.ts` (moved on 2026-10-18 to
`package/translation-repair/slice-cache/src/corpus-run/slice-cache-store.ts`) validates a cached outcome
field by field,
and it checks neither `checkerReadings` nor `recheckReadings`.
A cache file written
before those fields existed would therefore pass the guard,
and `buildIssueRecords` would then index
`undefined` and throw.
Two required fields were added to that type today,
so this is the exact shape
of failure to check for.

IT CANNOT HAPPEN,
because the cache is namespaced by the built pipeline's digest.
Each lane keeps its
own marker file next to its slices (`generation.txt`,
`refine-generation.txt`,
`translate-generation.txt` and so on),
`digestPipeline` computes that marker by walking the build
output directory recursively,
and a lane whose marker has moved deletes its own files rather than
reading them.
Measured rather than reasoned:
a run started under the pre-`checkerReadings` bundle
carries `sha256-tree-v1:b...` and one started after carries `sha256-tree-v1:9...`,
so the two never
share a namespace.

THE COST SIDE OF THAT SAME PROPERTY.
Any rebuild invalidates every cache,
so a pass resumed after a
source change re-buys everything it had already settled.
That is the reason not to restart a
long-running pass merely to pick up a field:
the in-flight passes finish under the bundle they
started with,
and a fresh run directory is what gets the new one.

### How much evidence the recheck was discarding, measured on settled artifacts

Across the 11 settled version 2 artifacts on disk (`~/translation-repair-runs-20260817/artifacts` and
`~/translation-repair-runs-flagged-20260818/artifacts`),
all of which predate both reading fields:

```text
issue rows                                          489
rows sitting on a slice the naturalness lane rewrote 208
of those, rows recorded resolved                     141
refine-recheck-passed findings                        23
refine-rolled-back findings                            0
rows carrying any checker reading                      0
```

So the recheck round has been ruling on 141 issue-level verdicts across 11 entries and leaving 23
one-line findings behind.
At the production roster of three that is several hundred ballots bought
and discarded in this population alone,
and none of it can be recovered without re-running the stage.
Nothing has ever rolled back,
which is worth knowing before reading a future rollback as routine.

A CAUTION FOR WHOEVER READS THESE ARTIFACTS NEXT.
There is no `slices` array under
`lanes.repair.result`;
the per-slice facts ride on the issue rows,
and `refined` is a field of
`RepairIssueRecord`.
A reader that walks a `slices` path gets zero from every artifact and the zero
looks like a measurement.

### The deciding round reaches a real artifact, with a same-entry negative control

VERIFIED AT THE USER BOUNDARY on 2026-08-24,
which is what `#188`'s instrument was owed.
The pass at
`~/temp/agent/checker-reading-vub-2026-08-24` settled `lintong` under the bundle that carries
`checkerReading`,
and its artifact holds:

```text
issue records                                     38
records carrying a reading                        29   all shipped / resolved=true
records carrying none                              9   all no-region / resolved=false
ballots per round                                  6
seated roster recorded                             6
rounds whose stored tally disagrees with a recompute   0
```

THE CONTROL IS THE SAME ENTRY ONE BUNDLE EARLIER.
`~/temp/agent/checker-width-wide-2026-08-24` ran
`lintong` with the same settings before the field existed,
and its artifact carries 30 issue records
and zero readings.
Same entry,
same switch,
one build apart:
the field is being written rather than
defaulted.

TWO INDEPENDENT READERS AGREE ON THE SAME RUN.
The log reader recovers 29 rounds and 174 ballots from
`checker-ballot` lines;
the artifact reader recovers 29 rounds of 6 ballots from
`RepairIssueRecord.checkerReading`.
They share no code and read different files,
so this cross-checks
both instruments at once,
and it also settles that a reading is present for exactly the issues a
checker ruled on and absent everywhere else.

### The split rate is a property of the ENTRY, which changes how to sample this

At 171 rounds and 1022 ballots across five wide passes:

```text
rounds where the narrow three split                         7 of 171   <- ceiling on flips
rounds where any of the six dissented                      15 of 171
rounds where a writer said something no narrow checker did  8 of 171
FLIPS                                                       0 of 171
```

WHERE THOSE SEVEN SPLITS LIVE MATTERS MORE THAN THE TOTAL.
`lintong` contributes 50 of the 171 rounds
across two separate passes and zero splits in either.
`Arita` and `GLaDOSister` contribute 20 rounds
so far and 4 splits,
which is a fifth of their rounds.
The panel's disagreement rate is therefore an
entry property rather than a constant,
and half the rounds bought so far came from the entry least
able to produce evidence.

CONSEQUENCE FOR ANYONE EXTENDING THIS.
Buying more `lintong` rounds raises the round count and leaves
the ceiling where it is.
Entries whose panels actually split are the only ones that can answer the
width question,
and the settled artifacts already say which those are:
read `refine-recheck-passed`
counts and per-entry split rates before choosing what to run.

## The accept gate keeps its ballots too (`#193`)

FOUND BY AUDITING WHAT A SETTLED ARTIFACT ACTUALLY NAMES,
rather than by reading code.
Walking one
artifact for every path whose value looks like a catalog id gives the whole inventory of per-model
evidence in a settled run:

```text
.lanes.repair.result.issues[].checkerReading.ballots[].modelId       174
.lanes.translate.result.sliceSelections[].round.ballots[].modelId     12
.lanes.translate.result.slices[].stageResult.ballots[].modelId        12
.consolidation.slices[].verdicts[].modelId                            12
.lanes.translate.result.sliceSelections[].round.producers[].modelId   11
.lanes.repair.result.chunks[].heardCriticIds                           3
```

The adjudication panel is absent from that list,
and it is the stage that decides whether a claim
becomes an issue the pipeline repairs at all.
`AdjudicatedIssue.tallies[claimId]` held five weighted
numbers and nothing else.
One measured entry carried 60 claims,
none naming any model,
with an
observed tally reading `supported=1, unsupported=5` and no way to know which panelist said which.

WEIGHTED,
SO THE SUMS ARE NOT RE-DERIVABLE.
`voteWeight` folds `AdjudicationConfig.weights` into the
totals before they are stored,
and no artifact records that table.
Even with the votes,
a run under a
non-default table could not be re-tallied.
The ballot therefore stores its own weight.

WHAT ELSE THE AUDIT CLEARED,
so nobody re-opens it:
`claimAttributions[].proposers` already names who
FILED each claim (`#76`),
and the editor and judge `rounds[]` already carry `ballots`.
Only the
panel's own votes were summed away.

### Why this one is optional where `checkerReadings` is required

`AdjudicatedIssue.readings` is optional,
and the reason is a real difference rather than convenience.
About thirty files construct an `AdjudicatedIssue`,
and almost all of them are readers and tools
rebuilding one from a settled artifact rather than adjudicating anything.
Requiring the field would
make every one of them write an empty record,
which would say "no panel voted" where the truth is "I
am not the panel".
Absence here has exactly one meaning:
this issue did not come from `tallyVotes`.

`configuredPanelists` IS required on `tallyVotes`,
threaded from the roster the caller already counts
for its log line.
A lost voice leaves no ballot while an abstention leaves one,
and only the seated
count separates them.

### Size, measured rather than assumed

Injecting a realistic six-panelist reading per claim into all twelve settled artifacts on disk:

```text
growth per artifact                    9% to 17%
largest artifact after injection      681 KiB
ceiling (#168)                       7168 KiB
headroom                                   91%
```

### GFP, three mutations

-   `readClaim` returning no ballots failed the three content cases and left the keying case passing,
    which is the control:
    that case is about pairing,
    not about what a ballot says.
-   Dropping `readings` from the merged-cluster branch failed the keying case and nothing else.
-   Rebuilding `RepairIssueRecord.issue` field by field,
    which is how the tally came to be the only
    thing stored,
    failed the new disk-boundary case.
    `buildIssueRecords` passes the issue through
    whole today,
    so that case exists to keep it that way.

Suite 569 PASS,
0 FAIL,
exit 0.
Lint 0/0.

### Two files split, neither for cosmetics

`tally-votes.ts` was exactly at the 300-line cap,
so `voteWeight` and `tallyClaim` moved to
`tally-claim.ts` beside the new `readClaim`.
`pipeline-barrel.ts` went to 303 lines,
so the ballot and
reading exports moved to a new `ballot-barrel.ts`,
which now holds every export belonging to one
question:
who voted,
what they said,
and what it summed to.
Three stages of this pipeline decide by
weighted vote and all three now record their ballots.

### A false verification, and the shell trap that produced it

RECORDED BECAUSE THE CLAIM WAS PUBLISHED BEFORE IT WAS CHECKED.
This session reported that the
settled rendering audit read both a pre-change archive and a new-schema archive cleanly,
"exit 0",
and that passing `--clone` was what made the difference.
Both halves were wrong,
and one shell line
caused it:

```sh
# WRONG: $? is basename's, not the audit's
printf 'exit=%s\n' "$(basename "$arch")" "$?"
```

Command substitutions in an argument list run left to right,
so `$(basename ...)` executes first and
resets `$?` to its own status.
Every reading came back `0`.
Capture the status into a variable on the
line after the command,
before anything else runs:

```sh
mise run ... > "$OUT" 2>&1
code=$?
```

WHAT WAS ACTUALLY HAPPENING,
once the status was read correctly.
`--clone` never mattered:
its
default is `RUN_CORPUS_PIN.cloneDir`,
which is `join(homedir(), 'one-among-us', 'data')` and already
correct.
The refusal was `readArchiveSubjects` doing exactly what it is built and tested to do,
refusing a directory that carries artifacts at its root AND in subdirectories.
A corpus RUN directory
is such a directory:
`artifacts/` beside `attempts.json` and `pass.lock`.
The audit wants the
`artifacts/` directory itself.

```sh
# from the worktree; note the trailing artifacts/
mise run //package/module/translation-repair:rendering-audit-settled -- \
  --archive "${HOME}/<run dir>/artifacts" --cap 0
```

Moved on 2026-10-18 from `//package/module/translation-repair:rendering-audit-settled`
to `//package/translation-repair/probe:rendering-audit-settled`.

THE REAL RESULT,
measured that way:
exit 0 on both,
zero error lines,
160 and 155 lines of report.
A full artifact consumer reads the two new reading fields without complaint,
which is the end-to-end
half of `#192` and `#193` that a parser call alone does not cover.

ALSO FIXED WHILE IN THERE.
`rendering-audit-settled-args.ts` used `NO_CAP` as the sentinel for
`indexOf` returning nothing found,
so one constant meant both "buy everything" and "not in the array".
Split into `NO_CAP` and `FLAG_ABSENT`,
which is the `#170` rule applied to a file that had escaped it.

## The recheck ballots have still never reached a live artifact, and the reason is not a defect (`#192`)

MEASURED 2026-08-24 over every settled artifact on disk.

`#192` landed in `f65b727b7` and is GFP-proven in unit tests.
The live-artifact VUB was still owed,
and reading the artifacts says it is not merely owed but failing:

    issue records total = 1215
      refined AND resolved = 160      <- these buy a checker round
      carrying recheckReading = 0

The conjunction is what matters.
`retainsResolvedIssues` returns early with `readings: {}`
when the slice has no confirmed-resolved issue,
so `refined` alone proves nothing.
A record that is BOTH `refined` and `resolved` went past that early return,
bought a round,
and must carry its ballots.
There are 160 of them across twenty entries and not one has a reading.

### The cause is launch ordering, not a dropped field

Two readings were considered and the first one was wrong.

REFUTED:
an explicit artifact projection that lists fields and missed this one.
There is no such projection.
`rg 'checkerReadings' src --glob '!*test*'` finds seven hits,
none under `corpus-run/`,
so chunk outcomes are serialized whole
and a required field set to `{}` would appear as `{}`.

STANDS:
every one of those artifacts was written by a pass
whose build predates the field.
The tell is `checker-reading-vub/lintong`,
which carries `chunks[n].checkerReadings` and `issues[n].checkerReading`
and no `recheckReadings` at all,
while `AmbeR_the_anpa` and `Anilovr` carry none of the three.
Three different builds,
three different field sets,
all settling within two hours of each other.
This is the trap already recorded here:
a rebuild does not change a running pass,
and a pass launched before a field exists never writes it,
however long it runs.

Every construction site does set the field.
`repair-chunk.ts`,
`repair-unchanged-outcome.ts`,
`repair-not-applicable.ts`,
and both exits of `refine-slice-settle.ts` all name `recheckReadings`,
and `ChunkRepairOutcome` makes it required so a new site cannot forget it.

### A search trap that produced the wrong reading first

The refuted reading came from `rg 'recheckReadings' . | head -20`,
which returned exactly twenty lines and omitted `repair-not-applicable.ts:79`.
The omission looked like a construction site that forgot the field.
QRY names this exact failure:
a `head` cap makes a search report fewer matches than exist and says nothing about it.
Re-run uncapped before concluding a symbol is absent from a file.

### What is now in flight

`recheck-ballot-vub-2026-08-24` on `093b53001`,
running `keyword233,saurikissa`.
Both entries produced the `refined AND resolved` conjunction on every earlier run that settled them,
`keyword233` at 7 and 5 records and `saurikissa` at 5,
5,
5 and 4,
so both are expected to buy a recheck round rather than merely maybe.

The same run answers `#193`:
the accept gate's panel readings have never reached a live artifact either,
for the same reason and on the same schedule.

### Two batches were stopped rather than left running

`checker-width-wide-batch-2026-08-24` and `checker-width-wide-batch2-2026-08-24`
had failed at their originally launched task,
been relaunched with their output going to a closed descriptor,
and were holding `pass.lock` files naming pids that no longer existed.
They had produced no log line in fifty-two minutes.

They were stopped by pid and their stale locks removed.
Nothing was lost:
their slice caches were pinned to a pipeline digest
superseded by five commits since,
so a resume would have re-bought every call regardless.
The question they were opened for,
checker width,
was answered and closed by `#188`,
and the switch they set no longer exists.

## Both ballot readings reach a live artifact, and the launch-ordering diagnosis was right (`#192`, `#193`)

VERIFIED 2026-08-24 on `ballot-reading-vub-2026-08-24/GLaDOSister`,
settled by a pass launched on a build carrying both fields.

    issue records            120
    refined AND resolved      26
    of those carrying recheckReading   26      <- all of them

    chunks[n].recheckReadings                  10
    chunks[n].issues[n].readings.issue/<id>    240 panel readings
    configuredPanelists                        264
    checkerReading / checkerReadings           91 / 10

Every record that bought a recheck round kept its ballots,
and the accept gate's panel readings are on the artifact per claim.
Both live-artifact VUBs are met and both tasks close.

This also confirms the reading recorded above rather than merely being consistent with it.
The 160 records with no reading came from passes whose builds predate the fields;
the first entry settled by a pass launched after them carries every field on every eligible record.
No code changed between the two observations.

The same entry's page passes the standalone verifier at `chars=3502=expected`,
so `#197` has its live boundary check too.
