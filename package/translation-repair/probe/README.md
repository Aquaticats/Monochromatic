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
  (with its drawn parts `editor-fidelity-pool-drawn-1.ts` to `editor-fidelity-pool-drawn-4.ts`),
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
  the three source-reviewed references and heading sections drawn from the same corpus pin by a stated rule,
  forty trials in all,
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
  (default 64,
  more than the pool holds;
  `--cap 0` is a dry run that reads the corpus,
  prints the manifest,
  pool,
  ceiling,
  guard and `RULE` lines,
  builds no client and spends nothing),
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
  the per-arm cost ceiling,
  the guard and the decision rule before any call,
  and refuses each call,
  and each attempt the Claude client makes inside a call,
  ending the arm,
  when the spend so far plus its ceiling would pass `--max-usd`.
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
  default 40),
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
  threw and abandoned counts)
  and a `STOPS` line (how the asks of each trial ended),
  then the verdict of the decision rule.
  Pass the four arm logs after `--`.
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
by running the one pool of forty trials through four arms:
`claude-sonnet-5-5-effort-low`,
`claude-sonnet-5-5-effort-medium` twice
(the second run measures how far two identical arms differ),
and `claude-sonnet-5-5-effort-high`.
The owner ruled 20 USD for all four arms together;
the caps named here add to 19.

### Decision rule

The rule is code (`decideEffort` in `src/corpus-run/editor-fidelity-decide.ts`),
its text is `decisionRuleText`,
and the same text is printed on the `RULE` line of every arm's log before its first call,
and the read step prints the verdict `decideEffort` reaches over the four arms:

> clean fix is exact plus equivalent,
> counted over the trials of both kinds of defect.
> An effort other than the default (medium) is chosen only when its clean fix count is above the mean of the two
> medium arms by at least 12 trials,
> or by at least the two medium arms' own difference when that is larger,
> and neither kind of defect (insertion,
> alteration) is more than 3 trials below the mean of the two medium arms for that kind.
> A smaller gap,
> a tie,
> an effort that fixes fewer,
> and a gap a kind contradicts all keep the default effort.
> With no true difference,
> a gap of 12 trials is the least that keeps the chance of choosing another effort under 0.05
> at every true fix rate from 0.30 to 0.95,
> counting the two trials of an entry as one draw at the worst
> (editor-fidelity-decide-null.test-fixture.ts),
> so the bench cannot separate efforts that differ by less.
> The insertion trials measure locating an added sentence among its neighbours and copying the neighbours verbatim,
> not the span-only view production gives an editor.

The floor of 12 comes from an exact enumeration over the pool's 24 entries
(16 with both kinds of defect,
6 with an insertion only,
2 with an alteration only),
at fix rates from 0.30 to 0.95 and within-entry correlations from 0 to 1,
in `src/corpus-run/editor-fidelity-decide-null.test-fixture.ts`,
whose unit test pins it.
A floor of 3 would call a difference in 0.58 of benches with none;
11 would in 0.07;
12 does in 0.043 at the worst.
The arms are treated as independent,
which overstates how far they stray when they share trials,
and the kind clause is left out,
which can only keep a pick,
so the 0.043 is an upper bound.

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
cp --recursive package/translation-repair/probe/dist/final/node "package/translation-repair/probe/node_modules/.frozen-dist-$SHA"
BENCH="$HOME/temp/agent/editor-fidelity-bench-$SHA"
mkdir --parents "$BENCH"
```

Each arm is the same command with its own name,
card,
cap and `--bench-spent-usd`
(the sum of the `GUARD total=` of the arms already run,
rounded up to the cent;
0 for the first).
The arms in order are `low` with the low card and `--max-usd 3`,
`medium-a` with the medium card and `--max-usd 4`,
`medium-b` with the medium card and `--max-usd 4`,
and `high` with the high card and `--max-usd 8`.
The first:

```bash
# README.md (editor fidelity bench): arm 1 of 4, low
ARM=low; CARD=claude-sonnet-5-5-effort-low; MAXUSD=3; SPENT=0
mkdir --parents "$BENCH/$ARM"
systemd-run --user --wait --collect "--unit=tr-edfid-$ARM" --property=MemoryMax=2G --property=CPUQuota=100% \
  "--working-directory=$PWD" "--setenv=PATH=$PATH" "--setenv=HOME=$HOME" \
  --setenv=TRANSLATION_REPAIR_STARTED_BY=editor-fidelity-probe "--setenv=TRANSLATION_REPAIR_RUNS_DIR=$BENCH/$ARM" \
  "--property=StandardOutput=truncate:$BENCH/$ARM.log" \
  mise exec -- env --unset=TRANSLATION_REPAIR_NEURALWATT_API_KEY --unset=TRANSLATION_REPAIR_SYNTHETIC_API_KEY \
  --unset=TRANSLATION_REPAIR_CHARM_HYPER_API_KEY --unset=TRANSLATION_REPAIR_AMAZON_BEDROCK_API_KEY \
  --unset=DEEPSEEK_API_KEY --unset=TRANSLATION_REPAIR_XIAOMI_MIMO_API_KEY --unset=TRANSLATION_REPAIR_OPENROUTER_API_KEY \
  node "package/translation-repair/probe/node_modules/.frozen-dist-$SHA/editor-fidelity-probe.mjs" \
  --candidates "$CARD" --candidates-alone --max-usd "$MAXUSD" --bench-spent-usd "$SPENT"
echo $? > "$BENCH/$ARM.exit"
```

The other three differ in the first line only:

```bash
# README.md (editor fidelity bench): the first line of arms 2 to 4
ARM=medium-a; CARD=claude-sonnet-5-5-effort-medium; MAXUSD=4; SPENT=<measured so far>
ARM=medium-b; CARD=claude-sonnet-5-5-effort-medium; MAXUSD=4; SPENT=<measured so far>
ARM=high; CARD=claude-sonnet-5-5-effort-high; MAXUSD=8; SPENT=<measured so far>
```

Before each later arm,
read the earlier arm's `GUARD total=` line from its log.
After the last arm,
check all four together:

```bash
# README.md (editor fidelity bench): the read step
mise run //package/translation-repair/probe:editor-fidelity-read -- \
  "$BENCH/low.log" "$BENCH/medium-a.log" "$BENCH/medium-b.log" "$BENCH/high.log"
```

Before the first arm,
a dry run of it with `--cap 0` and no key in the environment prints the manifest,
pool,
ceiling,
guard and `RULE` lines and exits without building a client;
the line to read is the `RULE` line,
which is the rule the verdict will apply.

An arm with a `FAIL` line is not evidence,
whatever its clean fix count says.
The checks are the exit status,
the manifest line,
the pool line,
the ceiling line,
the guard line (with the `--bench-spent-usd` it states),
the arm's `EDITOR` line,
the row count,
no `[error]` line,
the `SEAT` line (`threw` stated,
nothing unreachable,
no withheld or dark seat),
the identity of the `SPEND` lines' sum with the `GUARD total=`,
that every row line states how its asks ended,
and that no ask ended at the output-token cap or threw
(a trial that ended at the cap measured the cap and not the effort);
a `WARN` line says more than 4 trials of an arm were unusable or refused;
and across the arms,
four distinct logs at the cards low,
medium,
medium and high,
each arm's stated `--bench-spent-usd` at least the `GUARD total=` of the arms before it,
the same manifest digest in every one and a bench total under 20 USD.

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
