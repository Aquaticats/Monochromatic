# Translation repair history 2026-08-05 to 2026-08-07: exported functions that no test names

The count of exported functions named by no test,
how it was measured and its traps,
the groups left uncovered,
and the gap closed afterwards with the defects the new tests found.

Part of [Translation repair history: 2026-08-05 to 2026-08-07](translation-repair-history-2026-08-05-to-07.md),
which lists every file of this record.

Historical evidence only:
no next-action list here is a current instruction,
and the stop condition in the index applies.

## Exported functions that no test names, and the gap closed

### Coverage gap CLOSED (2026-08-06)

Re-measured after the work below:
of 192 exported functions,
2 are named by no
test,
down from 33.
Both are deliberate and are listed here so nobody reopens
them as oversights.

`applyCandidate` is a pure delegation to `applyPatchOperations`,
which has
its own suite.
Its one plausible misuse,
swapping the arguments,
is a type
error because `EditableEnvelope[]` and `PatchOperation[]` are distinct.
A
test there asserts nothing the compiler does not.

`runCriticStage` runs on every case of the `runChunkCriticPhase` suite,
which scripts critic replies and asserts wire-level vote counting,
resolution
failures reaching findings,
and heard-critic accounting.
Its branches execute;
only its name is absent.

TWO LIVE DEFECTS came out of writing these,
both in code that looked fine:

`requireRecord` delegated to `isJsonRecord`,
whose test is
`typeof value === object && value !== null`,
so an ARRAY satisfied a guard
whose entire doctrine is throwing loudly.
Fixed locally rather than in
`isJsonRecord`,
which nineteen modules share for values where arrays are fine.

The refiner prompt fenced both the original chunk and every paragraph with a
fixed `=====`.
Enclosed text carrying that line closes its own block early,
so
the rest of the paragraph reads to the model as instructions.
`=====` is a
setext heading underline,
a shape real documents contain.
Not currently
triggered (no corpus file carries a five-or-longer equals run,
checked first),
fixed because `candidate-select-wire.ts` and the probe already settled this
with `selectFence`.

TWO LATENT FRAGILITIES are documented rather than changed:
`bandOf` duplicates
`classifyBand` with the same two cuts,
now pinned by an agreement test;
and
`ensembleRecall` never intersects hits with the seed universe,
safe only
because `prepare-entry.ts` and `gradeHits` derive both from one list.

METHOD NOTE WORTH KEEPING:
four of my own fixtures were wrong before the
toolchain or a reviewer caught them.
Two asserted shapes production cannot
produce (a duplicate seed hit;
two records sharing one model and entry),
one
asserted bug-shaped output as expected (`ensembleRecall > 1`),
and one
compiled only because a cast stopped TypeScript checking the literal
(`refusal-shaped` carries `marker`,
not `detail`).
Before writing a case,
check the shape can occur;
before trusting a passing case,
check nothing cast
the fixture into silence.

### Coverage gap: 33 exported functions no test names

Measured 2026-08-06 while acting on the user instruction "Fix even pre-existing
issues."
Of 192 exported functions,
33 are never named in ANY test file.
Two
groups were closed the same day (the `artifact-guard.ts` guards,
and
`spliceSlices`);
this records the rest so the next session does not have to
re-derive it.

HOW TO REPRODUCE.
Extract every `^export (async )?function NAME` from
`src/*.ts` and `src/corpus-run/*.ts`,
then for each name grep `--word-regexp`
across every `*.unit.test.ts`.
Names with zero hits are the gap.

TWO MEASUREMENT TRAPS,
both of which caught me.

Sibling-file absence is NOT the measure.
`align-blocks-walk.ts` has no
`align-blocks-walk.unit.test.ts`,
yet `alignBlocks` is thoroughly tested from a
neighbouring suite.
Counting modules without sibling tests reported 34 modules;
counting functions no test names reported 33 functions,
and they are different
sets.
Indirect coverage through a tested caller is real coverage (TC2),
so the
function-level count is the honest one.

Matching import blocks by indentation is NOT the measure either.
A first pass
read names out of import statements with a two-space-indent pattern and
silently missed every single-line and differently-indented import,
reporting
`normalizePunctuation` as untested when five tests name it.
Use `--word-regexp`
across the whole test file.

WHAT IS STILL UNCOVERED,
grouped by what a defect would cost.

Selection and measurement,
where a defect moves the milestone number:
`compareCandidates`,
`computeScorecard`,
`selectCreditableIssues`,
`classifySourceAnchor`,
`corroboratedCount`,
`downgradeCount`,
`applyCandidate`.
Note on `compareCandidates`:
its caller
`selectRepairCandidate` IS tested,
so the branches run,
but the comparator's
own ordering is never asserted directly.
That is the function issue #53 is
about.

Sampling,
where a defect biases the sheet the gate is graded on:
`bandOf`,
`countSettledPerBand`,
`rankWithinBands`,
`smallBandIds`,
all in
`corpus-run/band-order.ts`,
which has no test at all.

Wire guards,
where a defect admits a malformed model reply:
`isCandidateBallotWire`,
`isRefineReportWire`,
`resolveRefineRewrites`,
`collectDefinitions`,
`groupNodesAligned`,
`buildRefineMessages`,
`buildEditorCandidates`,
`assertJudgeableProducerRoster`.

Network stage runners,
testable only with an injected scripted client the way
the `createSyntheticClient` suite already does:
`runCriticStage`,
`runPanelStage`,
`runCheckerStage`,
`runEditorStage`,
`runChunkCriticPhase`,
`attemptStageCall`,
`exchangeWithRetry`.

Run-tooling IO,
lowest value since a failure there is loud and immediate:
`createRunClient`,
`readHeadSha`,
`resolveRunsDir`,
`readAttemptMap`,
`openSliceCache`,
`discardSliceCache`,
`listResumableEntries`.

WHY THIS IS WORTH DOING rather than noting.
Writing the `artifact-guard.ts`
suite took one pass and immediately found a real defect:
`requireRecord`
delegated to `isJsonRecord`,
whose test is `typeof value === 'object' && value
!== null`,
so an ARRAY satisfied a guard whose entire stated doctrine is
throwing loudly rather than returning a fallback.
That hole sat in the layer
feeding the precision measurement.
The gap list is not bookkeeping;
it is where
the next defect of that kind is.

PKG makes this a completeness condition,
so the package is not finished while
the list is non-empty.
It is also entirely zero-quota work,
which makes it the
right thing to reach for whenever a corpus pass is holding the provider budget.
