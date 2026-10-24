# @monochromatic-dev/translation-repair-probe

Probes and their reports:
the rendering-audit probes,
the introduced-defect probe and its relabel and verification runs,
the recall benchmark,
damage sampling,
the judge-fidelity probe,
and the score reports that read graded sheets and artifacts.
It is its own package because the probes measure the pipeline from outside it,
and only `bench` and `coverage-census` depend on it.

## What it provides

`src/index.ts` re-exports each public name from the module that declares it.
The groups that follow name the main ones.

- Introduced-defect probe runs:
  probe the sensitivity of the defect probe,
  re-probe regions a human read as damaged,
  and verify flagged edits.
  Names:
  `runSensitivity`,
  `runProbeRelabel`,
  `runProbeVerify`,
  `singleRegionTally`,
  `reportProbeTelemetry`,
  `persistProbeRun`.
  Declared in `corpus-run/probe-sensitivity-run.ts`,
  `corpus-run/probe-relabel-run.ts`,
  `corpus-run/probe-verify-run.ts`,
  `corpus-run/probe-single-tally.ts`,
  `corpus-run/probe-telemetry-report.ts`,
  and `corpus-run/probe-store.ts`.
- Rendering-audit probes:
  run the rendering audit over settled artifacts and over planted defects.
  Names:
  `runAuditSensitivity`,
  `runSettledAudit`,
  `runSettledReport`,
  `auditRepeatsAcross`,
  `auditRelocationPairs`.
  Declared in `corpus-run/audit-sensitivity-run.ts`,
  `corpus-run/rendering-audit-settled-drive.ts`,
  `corpus-run/rendering-audit-settled-report-run.ts`,
  `corpus-run/rendering-audit-settled-repeat.ts`,
  and `corpus-run/rendering-audit-settled-relocation.ts`.
- Recall benchmark:
  plant known omissions into clean translations and grade whether the repair loop restores them.
  Names:
  `runRecallBenchmark`,
  `chooseRecallEntries`,
  `seedRecallEntry`,
  `persistRecallScorecard`,
  `recallScorecardLines`.
  Declared in `corpus-run/recall-benchmark-run.ts`,
  `corpus-run/recall-benchmark-choose.ts`,
  `corpus-run/recall-benchmark-entry.ts`,
  `corpus-run/recall-scorecard-store.ts`,
  and `corpus-run/recall-benchmark-report.ts`.
- Damage sampling and grading draws:
  draw shipped regions or a stratified sample for human grading.
  Names:
  `sampleDamage`,
  `collectTwoLaneShippedRegions`,
  `drawRegions`,
  `drawGradingSample`,
  `collectShippedRegions`.
  Declared in `corpus-run/damage-sample-run.ts`,
  `corpus-run/damage-region-v2.ts`,
  `corpus-run/damage-sample-draw.ts`,
  `corpus-run/draw-sample-run.ts`,
  and `corpus-run/damage-sample-pool.ts`.
- Score reports:
  print precision,
  agreement,
  attribution,
  crosscheck and verification reports from graded sheets and artifacts.
  Names:
  `printGradeReport`,
  `printAttribution`,
  `printCrosscheck`,
  `printProbeScore`,
  `printVerifyScore`,
  `buildAttributionReport`,
  `buildCrosscheckCensus`.
  Declared in `corpus-run/score-agreement-run.ts`,
  `corpus-run/score-attribution-run.ts`,
  `corpus-run/score-crosscheck-run.ts`,
  `corpus-run/score-probe-run.ts`,
  `corpus-run/score-verify-run.ts`,
  `corpus-run/attribution-report.ts`,
  and `corpus-run/judge-crosscheck.ts`.
- Judge-fidelity probe:
  judge fixed source-reviewed references against hash-verified damage.
  Names:
  `runFidelityProbe`,
  `fidelityPlan`,
  `boundFidelityMatrix`,
  `REVIEWED_FIDELITY_REFERENCES`,
  `readReviewedFidelityReferences`,
  `buildReviewedFidelityReference`,
  `seatJudges`.
  Declared in `corpus-run/judge-fidelity-probe-run.ts`,
  `corpus-run/judge-fidelity-probe-plan.ts`,
  `fidelity-reference-manifest.ts`,
  `fidelity-reference-read.ts`,
  `fidelity-reference-build.ts`,
  and `corpus-run/judge-independence.ts`.
- Editor-fidelity probe:
  seat one Claude card alone as the editor of a hash-verified seeded defect and read what it did to the text,
  with no critic,
  panel,
  judge
  or checker.
  Names:
  `editorFidelityTrials`,
  `EDITOR_FIDELITY_POOL`,
  `poolLine`,
  `scoreEditedText`,
  `runEditorFidelityProbe`,
  `runEditorTrial`,
  `editorCeilings`,
  `spendGuardedClient`,
  `guardTotalLine`,
  `cardsOnlyClient`,
  `editorProbeEnv`.
  Declared in `editor-fidelity-trials.ts`,
  `editor-fidelity-pool-manifest.ts`
  (with its drawn parts `editor-fidelity-pool-drawn-1.ts` to `editor-fidelity-pool-drawn-10.ts`),
  `corpus-run/editor-fidelity-pool-line.ts`,
  `editor-fidelity-score.ts`,
  `corpus-run/editor-fidelity-probe-run.ts`,
  `corpus-run/editor-fidelity-trial.ts`,
  `corpus-run/editor-fidelity-ceiling.ts`,
  `corpus-run/editor-fidelity-spend-guard.ts`,
  and `corpus-run/editor-fidelity-client.ts`.

## Runners

`src/build-entries.ts` lists the library index and each runner as a build entry,
and each runner is built to `dist/final/node/<runner>.mjs`.
Every runner has a task of the same name in `mise.toml` that builds first.
Start one with `mise run //package/translation-repair/probe:<task>`,
passing arguments after `--`.
A built command refuses to run in a process that holds provider keys unless `TRANSLATION_REPAIR_STARTED_BY` names
that command,
and each task sets it to its own name.

- `draw-sample`:
  draws the stratified precision sample from settled artifacts into a grading sheet outside the repository.
  Append `-- --final` for the gate sheet.
- `score-attribution`:
  reports per-critic calibration from a run's settled artifacts:
  what each critic was asked,
  what it raised,
  and how often an accepted issue rested on it alone.
- `score-agreement`:
  reports precision off a graded sheet,
  plus agreement against blind pre-grades when they exist.
  Override with `-- --sheet PATH --manifest PATH --pre-grades PATH`.
- `score-crosscheck`:
  reports the population a judge crosscheck would run over,
  per author and per arm.
- `score-probe`:
  reports what the shadow-mode introduced-defect probe found across a run's settled artifacts,
  over distinct shipped regions.
  Pass `--repair-sheet PATH --manifest PATH` after `--` to score it against the human grades;
  both or neither.
- `recall-benchmark`:
  measures recall:
  plants known omissions into clean translations,
  runs the repair loop,
  and grades restoration against the deletions.
  Append `-- --plan` for a zero-quota setup check.
- `probe-sensitivity`:
  asks whether the introduced-defect probe can detect damage at all,
  each region under three prior-issue lists;
  cat-themed fixtures only.
  Spends quota.
- `audit-sensitivity`:
  asks whether the rendering audit can detect a planted defect at all,
  and whether it invents one:
  one flipped rendering and one faithful;
  cat-themed fixtures only.
  Spends quota.
- `rendering-audit-settled`:
  runs the rendering audit over every decided slice of every settled version 2 artifact in an archive,
  reading retained and replaced slices apart.
  Telemetry only:
  nothing it says gates what ships.
  Pass `--archive dir`,
  `--clone dir`,
  `--only ids` and `--cap n` after `--`.
  Spends quota unless `--cap 0`.
- `rendering-audit-settled-report`:
  reads a persisted settled-audit run:
  the archive and fresh halves apart,
  what each auditor thought worth a claim,
  and the relocation candidates.
  Pass `--run path` after `--` to read a run other than the newest,
  and `--against path` to print the across-run band against an earlier run.
  Spends no quota.
- `judge-fidelity-probe`:
  judges fixed source-reviewed references against hash-verified damage.
  Options after `--` are `--only` with reviewed entry ids,
  `--cap n`,
  `--damage deletion|insertion|alteration`,
  `--candidates` with seatable ids,
  `--candidates-alone` and `--context`;
  `--cap 0` is a metadata-only preflight.
  Spends quota.
- `editor-fidelity-probe`:
  seats each card `--candidates` names alone as the editor of an inserted sentence or an altered number,
  and scores the patched text without a model.
  The sheet is the one the corpus pass sends an editor,
  built by `buildEditorMessages` through `gatherEditorProposals`,
  with the sections either side of the passage,
  the declared names and the whole original page.
  The trials are the frozen pool's:
  the source-reviewed references and heading sections drawn from the same corpus pin by a stated rule,
  at most two trials to an entry,
  88 trials from 53 entries in all
  (the draw wants 150 and the rule's limits stop it there),
  held as ids,
  offsets and hashes with no corpus text.
  The headline is the **clean fix**:
  a patched text equal to the clean text,
  or equal to it once whitespace runs are collapsed and trailing whitespace is ignored.
  A reply that removes the defect and changes something else is **defect only**,
  has its own line and is never counted in the headline.
  Collateral is a change to text outside the defect.
  Refused and unusable replies are counted apart.
  Every EDITOR line is printed pooled and for each kind of defect.
  Options after `--` are `--only` with entry ids of the pool,
  `--cap n`
  (default 200,
  more than the pool holds;
  `--cap 0` is a dry run that reads the corpus,
  prints the manifest,
  pool,
  planned,
  ceiling,
  guard and `RULE` lines,
  builds no client and spends nothing),
  `--trials n`
  (walk the first n trials of the pool's one seeded order,
  which is how a later arm of a bench walks the trials the first arm completed;
  refused beside a `--cap` above 0,
  allowed beside `--cap 0` for the dry run of that arm),
  `--damage insertion|alteration`
  (a deletion is refused,
  because its fix is a sentence the editor must write),
  `--candidates` with seatable Claude card ids,
  `--candidates-alone` (required),
  `--max-usd n` (default 2)
  and `--bench-spent-usd n`
  (required,
  with no default:
  what the arms run before this one measured,
  0 for the first arm;
  printed on the guard line;
  the arm is refused when it and `--max-usd` together pass 20).
  Prints the manifest digest,
  the pool,
  the trials planned,
  the per-arm cost ceiling,
  the guard and the decision rule before any call,
  and refuses each call,
  and each attempt the Claude client makes inside a call,
  ending the arm,
  when the spend so far plus its ceiling would pass `--max-usd`.
  An arm the guard ends that way is capped,
  not failed:
  it prints a `CAPPED at trial k` line and a `TRIALS completed k of p` line,
  keeps the rows it finished,
  and exits 0.
  The guard admits one call at a time and throws if a second is asked while one is in flight.
  Neither the typed-decision exchange nor the quota read of the client the probe builds sends a request:
  the client is built over the Claude key alone,
  so the first is refused for want of a decisions client and the second answers an exhausted snapshot.
  The spend so far is every `SPEND` line the Claude client wrote,
  the attempts it abandoned and retried included;
  a call whose last attempt wrote no line and threw,
  or a reply that reports no usage,
  is counted at its ceiling by a `SPEND` line of the guard's own,
  and a call that never left,
  or an HTTP error status that carried no body,
  is counted nothing,
  and the arm ends with a `GUARD total=` line.
  Only the Claude key is read.
  Spends quota.
- `editor-fidelity-pool-draw`:
  draws the pool from the pinned corpus clone and writes `src/editor-fidelity-pool-drawn*.ts`,
  ids,
  offsets and hashes only.
  Options after `--` are `--seed` (default `round-4`),
  `--want` (trials the pool holds,
  reviewed ones counted;
  default 150,
  which the rule's limits stop short of),
  `--out` (the directory the files go to)
  and `--check`
  (compares the files in `--out` with a fresh draw,
  then reads every committed reference through the hash-verifying reader and runs the scoreability check on its trials).
  Reads the corpus clone,
  calls no provider.
- `editor-fidelity-read`:
  checks the log of every arm of the bench and prints one `PASS`,
  `WARN` or `FAIL` line for each check,
  then `CHECKS n pass, n fail`,
  and exits 6 when any check failed.
  When every check passed it also prints,
  for each arm,
  an `ARM` line
  (the clean fix count pooled and for each kind of defect,
  the defect-only count beside it as a secondary count,
  and the unusable,
  refused,
  threw and abandoned counts),
  a `SHARED` line
  (the trials every arm completed,
  and how many were left out as thrown or as not completed by every arm),
  and for each arm a `STOPS` line
  (a count for each stop reason of its asks,
  and the rows that did not end `ok`),
  then for each of the high and the low effort a `PAIRED` line
  (the paired difference against medium,
  its 95 percent interval from a seeded bootstrap over entries,
  the exact p-value and whether the effort is shown better)
  and a `KINDS` line
  (the paired estimate for each kind of defect,
  secondary and vetoing nothing),
  and last the `VERDICT` of the decision rule.
  Pass the three or four arm logs after `--`.
  The exit status of an arm is read from `<arm>.exit` beside `<arm>.log`.
  Spends nothing.
- `damage-sample`:
  draws shipped regions at random and asks a human the same source-anchored damage question the probe asks,
  with the probe's verdict kept in the manifest.
- `probe-verify`:
  probes damaged and unread regions with the accepted issues withheld,
  and writes one blind sheet asking whether each flagged edit really damaged the text.
- `score-verify`:
  joins the graded blind verification sheet to its manifest and reports what the unlabelled probe's flags are worth
  on regions nobody had read.
- `probe-relabel`:
  re-probes the regions a human read as damaged under three arms (the production disclosure,
  the other prompt,
  no list at all),
  to separate a suppressing prompt from a dismissing screen from a hard judgement.
  Spends quota.

## Editor fidelity bench

The bench asks which effort of the Sonnet 5.5 card should edit,
by running the one seeded pool through three arms,
each once:
`claude-sonnet-5-5-effort-high`,
then `claude-sonnet-5-5-effort-medium`,
then `claude-sonnet-5-5-effort-low`.
The owner ruled 20 USD for all the arms together;
the caps named here (8,
6 and 5) add to 19.
The read step can also read a second medium arm,
given as a fourth log between the medium and the low,
as one more control (it joins the labels the test permutes).

### Decision rule

The rule is code (`decideEffort` in `src/corpus-run/editor-fidelity-decide.ts`),
its text is `decisionRuleText`,
and the same text is printed on the `RULE` line of every arm's log before its first call,
and the read step prints the verdict `decideEffort` reaches over the trials every arm completed:

> clean fix is exact plus equivalent,
> counted trial by trial.
> A trial in which an ask ended at the output-token cap scores as unusable for its arm,
> and a trial in which an ask of any arm threw is left out of every arm's comparison and counted.
> The arms are compared over the trials every arm completed,
> which are the leading trials of one fixed seeded order,
> grouped by the entry each comes from.
> Each of the low and the high effort is tested against the default (medium)
> by a one-sided exact paired permutation test,
> permuting the arm labels within each entry,
> with entries as the units,
> at 0.025 for each effort
> (a second medium arm,
> if one is run,
> joins the labels and is read as one control).
> The paired difference is printed per shared trial
> with a 95 percent interval from a seeded bootstrap over entries
> (10000 resamples,
> seed editor-fidelity-bench/interval/v1)
> and the exact p-value.
> An effort other than the default is chosen only when its test is at or under 0.025.
> When both efforts are shown better,
> the one with the larger estimate is chosen;
> when the estimates are equal,
> the cheaper effort is chosen.
> A medium verdict means no effort was shown better than medium,
> not that the others are as good.
> The estimates for each kind of defect (insertion,
> alteration) are printed beside it and veto nothing.
> With fewer than 40 shared trials the read step refuses to give a verdict.
> By simulation over the committed pool
> (editor-fidelity-paired-null.test-fixture.ts,
> seed edr6-sim),
> at true fix rates from 0.30 to 0.95 and within-entry correlations from 0 to 1,
> the chance of choosing another effort when none is better is at most 0.047;
> over all 88 trials the smallest difference chosen with chance 0.8 is about 25 points when medium fixes half,
> 21 points when it fixes seven in ten and 14 points when it fixes eighty-five in a hundred,
> and a smaller difference cannot be told apart.
> The insertion trials measure locating an added sentence among its neighbours and copying the neighbours verbatim,
> not the span-only view production gives an editor.

Two efforts are each tested at 0.025,
so the chance of picking either when neither is better is under 0.05,
and the simulation measured 0.047 at the worst.
The simulation plays the bench over the committed pool in its order:
three arms fix each trial with a stated chance,
share a trial's draw with chance 0 or one half,
and read the two trials of an entry alike with a correlation from 0 to 1.
It is `src/corpus-run/editor-fidelity-paired-null.test-fixture.ts`,
and its unit test pins that its table agrees with the rule's exact test,
that it walks the pool's 88 trials in order,
and that one cell picks nothing more often than five in a hundred.
The full grid ran once,
seeded,
with 1000 to 2000 replicates for each cell,
and its figures are in the ledger entry for this round.
At 40 shared trials the smallest difference detected with chance 0.8 is about 37 points when medium fixes half,
which is why 40 is the fewest the read step accepts.

### Launching the arms

One arm at a time,
each as its own systemd user unit,
launched through the root `mise.toml` key path
(`mise exec`) with every provider key but the Claude key unset,
from a frozen copy of the built probe so that a later build cannot change what an arm ran.
Run from the repository root of the checkout that holds the sealed key file.

```bash
# README.md (editor fidelity bench): prepare, once, at the commit the bench runs
SHA=$(git rev-parse --short=10 HEAD)
mise run //package/translation-repair/probe:build
cp --recursive package/translation-repair/probe/dist/final/node \
  "package/translation-repair/probe/node_modules/.frozen-dist-$SHA"
BENCH="$HOME/temp/agent/editor-fidelity-bench-$SHA"
mkdir --parents "$BENCH"
```

The arms run in this order,
because the first arm decides how many trials the bench has.
`high` runs first with the high card and `--max-usd 8`,
over the whole pool.
If the guard ends it before the pool is done,
it prints a `CAPPED at trial k` line,
keeps the rows it finished and exits 0;
that is a shorter bench and not a failed arm.
Read n,
the number of trials it completed,
from the `TRIALS completed n of p` line of `$BENCH/high.log`:

```bash
# README.md (editor fidelity bench): n, the trials the high arm completed
rg --only-matching "TRIALS completed [0-9]+" "$BENCH/high.log"
```

Then `medium` runs with the medium card and `--max-usd 6`,
and `low` with the low card and `--max-usd 5`,
each with `--trials n`,
so that each walks the same first n trials in the same order.
Each arm is the same command with its own name,
card,
cap,
`--bench-spent-usd`
(the sum of the `GUARD total=` of the arms already run,
rounded up to the cent;
0 for the first)
and trials flag.
The first:

```bash
# README.md (editor fidelity bench): arm 1 of 3, high, over the whole pool
ARM=high; CARD=claude-sonnet-5-5-effort-high; MAXUSD=8; SPENT=0; TRIALS=()
mkdir --parents "$BENCH/$ARM"
systemd-run --user --wait --collect "--unit=tr-edfid-$ARM" --property=MemoryMax=2G --property=CPUQuota=100% \
  "--working-directory=$PWD" "--setenv=PATH=$PATH" "--setenv=HOME=$HOME" \
  --setenv=TRANSLATION_REPAIR_STARTED_BY=editor-fidelity-probe "--setenv=TRANSLATION_REPAIR_RUNS_DIR=$BENCH/$ARM" \
  "--property=StandardOutput=truncate:$BENCH/$ARM.log" \
  mise exec -- env --unset=TRANSLATION_REPAIR_NEURALWATT_API_KEY --unset=TRANSLATION_REPAIR_SYNTHETIC_API_KEY \
  --unset=TRANSLATION_REPAIR_CHARM_HYPER_API_KEY --unset=TRANSLATION_REPAIR_AMAZON_BEDROCK_API_KEY \
  --unset=DEEPSEEK_API_KEY --unset=TRANSLATION_REPAIR_XIAOMI_MIMO_API_KEY \
  --unset=TRANSLATION_REPAIR_OPENROUTER_API_KEY \
  node "package/translation-repair/probe/node_modules/.frozen-dist-$SHA/editor-fidelity-probe.mjs" \
  --candidates "$CARD" --candidates-alone --max-usd "$MAXUSD" --bench-spent-usd "$SPENT" "${TRIALS[@]}"
echo $? > "$BENCH/$ARM.exit"
```

The other two differ in the first line only,
with n and the measured spend written in:

```bash
# README.md (editor fidelity bench): the first line of arms 2 and 3
ARM=medium; CARD=claude-sonnet-5-5-effort-medium; MAXUSD=6; SPENT=<measured so far>; TRIALS=(--trials <n>)
ARM=low; CARD=claude-sonnet-5-5-effort-low; MAXUSD=5; SPENT=<measured so far>; TRIALS=(--trials <n>)
```

Before each later arm,
read the earlier arms' `GUARD total=` lines from their logs.
After the last arm,
check all three together:

```bash
# README.md (editor fidelity bench): the read step
mise run //package/translation-repair/probe:editor-fidelity-read -- \
  "$BENCH/high.log" "$BENCH/medium.log" "$BENCH/low.log"
```

Before the first arm,
a dry run of each with `--cap 0` and no key in the environment prints the manifest,
pool,
planned,
ceiling,
guard and `RULE` lines and exits without building a client;
the dry run of a later arm takes `--trials n` beside the `--cap 0`.
The line to read is the `RULE` line,
which is the rule the verdict will apply.

An arm with a `FAIL` line is not evidence,
whatever its clean fix count says.
The checks are the exit status,
the manifest line,
the pool line,
the planned line,
the ceiling line,
the guard line (with the `--bench-spent-usd` it states),
the arm's `EDITOR` line,
the row count against the trials it completed
(with a `CAPPED` line saying why an arm completed fewer than it planned),
no `[error]` line,
the `SEAT` line (`threw` stated,
nothing unreachable,
no withheld or dark seat),
the identity of the `SPEND` lines' sum with the `GUARD total=`,
and that every row line states how its asks ended.
A trial in which an ask ended at the output-token cap or threw fails no arm:
the first scores as unusable for its arm
and the second is left out of every arm and counted.
A `WARN` line says more than 4 trials of an arm were unusable or refused.
Across the arms the checks are
three or four distinct logs at the cards high,
medium,
(a second medium if run)
and low in the order run,
each arm's stated `--bench-spent-usd` at least the `GUARD total=` of the arms before it,
each later arm having walked the first arm's trials in the first arm's order,
the same manifest digest in every one,
a bench total under 20 USD,
and at least 40 trials shared by every arm.

## Family dependencies

Short names are directory names under `package/translation-repair`;
a package is named `@monochromatic-dev/translation-repair-<short name>`.

### Depends on

Production dependencies (`dependencies` in `package.json`):

- `adjudication`
- `archive`
- `artifact`
- `artifact-pool`
- `assembly`
- `corpus-pass`
- `corpus-source`
- `document-structure`
- `glossary`
- `house-rule`
- `insertion`
- `lane-contest`
- `markup`
- `page-check`
- `provider`
- `provider-stream`
- `reference`
- `rendering-audit`
- `repair`
- `run`
- `shared-contract`
- `slice-reading`
- `stage`
- `typography`

Development dependencies other than `test-fixture` (`devDependencies`,
used by tests):

- `translate-floor`
- `translate-lane`

### Used by

Packages that list it in `dependencies`:

- `bench`
- `coverage-census`

## Tests

Tests sit beside the sources as `*.unit.test.ts` and import built output:
this package through its own build,
`../dist/final/node/index.mjs`,
and the siblings they need through their package names,
which resolve to each sibling's built entry.

Build before testing:
`mise run //package/translation-repair/probe:build` builds this package,
and the library build keeps family siblings external,
so this package's built entry imports the siblings' built entries.
Its `build:js:node` task therefore declares `depends` on the `build:js:node` of every sibling listed in
`package.json`,
and the siblings build first.

Run the tests with `mise run //package/translation-repair/probe:test:unit`,
or build and test together with `mise run //package/translation-repair/probe:buildAndTest`.
Pass test file paths to run only those files.

Test helpers used across the family,
such as scratch directories and capturing loggers,
come from `test-fixture`,
imported as `@monochromatic-dev/translation-repair-test-fixture/ts/<file>`.

## Family docs

- [Family README](../README.md):
  the design,
  the package list and where the shared docs live.
- [Package split plan](../../../doc/planning/translation-repair-package-split.md):
  the split plan,
  with the package map and the rules the move keeps.
