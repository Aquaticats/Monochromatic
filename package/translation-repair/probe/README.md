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
  It served as the judge effort bench for Haiku 5.5,
  whose [outcome](doc/judge-effort-bench-outcome.md) is that every effort reached the ceiling,
  so Haiku 5.5 keeps the card without an effort parameter.
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
  (default 1000,
  more than the pool holds walked through several passes;
  `--cap 0` is a dry run that reads the corpus,
  prints the manifest,
  pool,
  planned,
  ceiling,
  guard and `RULE` lines,
  builds no client and spends nothing),
  `--passes n`
  (walk the pool n times,
  pass by pass:
  every trial in the pool's one seeded order,
  then every trial again;
  a trial of a later pass is a unit of its own,
  named `pass k` on its row line,
  and joins its entry's cluster in the test),
  `--trials n`
  (walk the first n units of the passes' one order,
  which is how a later arm of a bench walks the units the first arm completed;
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
  the arm is refused when it and `--max-usd` together pass 60).
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
  it prints a `CAPPED at unit k` line and a `TRIALS completed k of p units` line,
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
  A reply with no request behind it is counted nothing:
  the reuse layer of the run client logs a `PROMPT-REUSE` line and sends nothing,
  so the guard writes no `SPEND` line and aborts the arm with `RepliedWithoutCallError`,
  and no row is scored for it.
  Each row names its pass to that layer (`reuseScope`),
  so a later pass asks the model afresh.
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
  It also prints before the `CHECKS` line,
  even when a check failed,
  a `REPEAT` line for each arm,
  which it prints even for one log read alone
  (the counts and share of the trials the arm was asked in more than one pass whose passes all read alike,
  over the units every arm holds when several logs share units,
  and over the arm's own rows otherwise),
  a `WARN repeat agreement` line for each arm of the default effort that an exact one-sided test finds repeating itself
  less than another arm
  (printed for two or more logs and never for one log read alone),
  and a `WARN high fix rate` line for each arm of the default effort that cleanly fixed 90 percent
  or more of the units every arm holds
  (printed only when the logs share units,
  so never for one log read alone and never for two logs without a low arm).
  Pass the three or four arm logs after `--`.
  The exit status of an arm is read from `<arm>.exit` beside `<arm>.log`.
  OPEN (ledger B1099) as the arms still capture their console into `<arm>.log` and this command reads it.
  The corpus pass and the reports of its records moved to the run folder (ledger B1090 to B1098).
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
The owner ruled 60 USD for all the arms together on 2026-10-24 (it was 20);
the caps named here (24,
18 and 15) add to 57.
The pool holds at most 88 trials,
so the money buys evidence through more units:
each arm walks the pool three times with `--passes 3`,
and a trial asked again joins the cluster of its entry in the test.
The read step can also read a second medium arm,
given as a fourth log between the medium and the low,
as one more control (it joins the labels the test permutes).
The bench has run at `85e21653a2`,
and [its outcome](doc/editor-fidelity-bench-outcome.md) is that no effort was shown better than medium,
which stays the editor's effort.

### Decision rule

The rule is code (`decideEffort` in `src/corpus-run/editor-fidelity-decide.ts`),
its text is `decisionRuleText`,
and the same text is printed on the `RULE` line of every arm's log before its first call,
and the read step prints the verdict `decideEffort` reaches over the trials every arm completed:

> clean fix is exact plus equivalent,
> counted trial by trial.
> A trial in which an ask ended at the output-token cap scores as unusable for its arm,
> and a unit in which an ask of any arm threw is left out of every arm's comparison and counted,
> where a unit is a trial in one pass over the pool (an arm may walk the pool several times,
> pass by pass).
> The arms are compared over the units every arm completed,
> which are the leading units of one fixed seeded order,
> grouped by the entry each comes from,
> so a trial asked again joins the cluster of its entry.
> Each of the low and the high effort is tested against the default
> (medium) by a one-sided exact paired permutation test,
> permuting the arm labels within each entry,
> with entries as the units,
> at 0.025 for each effort (a second medium arm,
> if one is run,
> joins the labels and is read as one control).
> The paired difference is printed per shared unit with a 95 percent interval from a seeded bootstrap over entries
> (10000 resamples,
> seed editor-fidelity-bench/interval/v1) and the exact p-value.
> An effort other than the default is chosen only when its test is at or under 0.025.
> When both efforts are shown better,
> the one with the larger estimate is chosen;
> when the estimates are equal,
> the cheaper effort is chosen.
> A medium verdict means no effort was shown better than medium,
> not that the others are as good.
> The estimates for each kind of defect (insertion,
> alteration) are printed beside it and veto nothing.
> With fewer than 60 shared units the read step refuses to give a verdict.
> By simulation over the committed pool (editor-fidelity-paired-null.test-fixture.ts,
> seeds edr7-sim and edr8-sim),
> at true fix rates from 0.30 to 0.90,
> within-entry correlations from 0 to 1 and an agreement of a repeat with the arm's first answer from 0 to 1,
> the chance of choosing another effort when none is better is at most 0.05 when every arm repeats itself alike
> (the worst of 192 cells was 0.055 at 800 replicates,
> within the noise of 0.008,
> and the four cells over 0.05 were at most 0.035 at 4000),
> and 0.107 when the default effort repeats itself far less than the others (a true fix rate of 0.9,
> three passes,
> 4000 replicates of the committed fixture).
> Both figures hold for fix rates up to 0.90 and no higher.
> Above 0.90 the chance when the default effort repeats itself far less than the others was higher:
> the committed fixture read 0.108 at 0.91,
> 0.123 at 0.92,
> about 0.13 at 0.93 and 0.175 at 0.95
> (0.108 at 0.91 and 0.123 at 0.92 at reviewer seed edrc6 and 8000 replicates;
> 0.130 at 0.93 at seed edr9-fx and 4000 replicates,
> and 0.132 at 0.93 at seed edrc6 and 8000 replicates;
> 0.175 at 0.95 at seed edr9-fx and 4000 replicates;
> each in three passes of 264 units with correlation 1),
> and an independent re-implementation of the model read 0.142 at 0.93,
> 0.173 at 0.95 and 0.241 at 0.97
> (reviewer seed edrc5,
> 4000 replicates,
> 264 units).
> The read step prints each arm's repeat agreement over the units every arm holds
> and warns when an exact one-sided test of the default effort's count of trials that read alike in every pass
> against that of the other arm with the highest share is at or under 0.05 divided by the number of other arms,
> which holds the chance of that warning at or under 0.05 when every arm repeats itself alike,
> counting the trials of an arm as independent draws,
> a warning relied on for fix rates up to 0.90,
> and prints WARN high fix rate when the default effort fixes 90 percent or more of the shared units,
> since the repeat warning can miss unequal repeats at fix rates this high
> and a verdict of high or low then stands under the caveat.
> Over the 88 trials walked once the smallest difference chosen with chance 0.8 is about 26 points when medium fixes
> half,
> 20 points when it fixes seven in ten and 14 points when it fixes eighty-five in a hundred,
> and over 176 units (two passes) 25,
> 21 and 14 points where a repeat always gives the arm's first answer again,
> 21,
> 17 and 12 points where it does so half the time and 17,
> 14 and 10 points where it never does,
> and a smaller difference cannot be told apart.
> The insertion trials measure locating an added sentence among its neighbours and copying the neighbours verbatim,
> not the span-only view production gives an editor.

The read step answers with two warnings,
measured in round 10 by one simulation run of the committed fixture
(seed prefix `edr10-sim`,
4000 replicates a cell):

- The repeat warning is a one-sided Fisher exact test of the default effort's count of trials
  whose passes all read alike
  against the count of the other arm with the highest share,
  at 0.05 divided by the number of other arms,
  with no floor on the trials asked again.
  On three alike arms it fired on 0.0000 to 0.0270 of the benches at 100 shared units (12 trials asked again),
  0.0000 to 0.0553 at 176 and 0.0000 to 0.0475 at 264 (88 trials asked again),
  over 18 cells at each size,
  and one of the 54 readings is over 0.05:
  the cell of fix rate 0.9,
  agreement 0 and correlation 1 at 176 units,
  0.0537 over 34000 replicates with re-check 7 (about three standard deviations over 0.05),
  against 0.0296,
  0.0430 and 0.0379 in its three contrast cells.
- The high fix rate warning fires when the default effort cleanly fixes 90 percent or more of the shared units,
  so that the rule's stated region (true fix rates up to 0.90) and the warning meet.

The simulations behind these figures,
the false pick bounds and the readings above a true fix rate of 0.9
(the fixture's and the independent re-implementation's kept apart),
the contrast cells and the detection rates when the default effort repeats itself 15 and 30 points less,
the exact test of the repeat warning with its measured false alarm and detection rates,
the share of benches that print neither warning,
the smallest difference detected at 60 shared units,
and the model they play are in
[the simulations behind the decision rule](doc/editor-fidelity-decision-rule-simulations.md).

### Launching the arms

One arm at a time,
each as its own systemd user unit,
launched through the root `mise.toml` key path
(`mise exec`) with every provider key but the Claude key unset,
from a frozen copy of the built probe so that a later build cannot change what an arm ran.
Run from the repository root of the checkout that holds the sealed key file.
Every block of this section sets the variables it reads,
so each runs in a fresh shell,
and every block that reads the logs begins with the two lines that name the bench directory,
`SHA` and `BENCH`,
as the prepare block does.
`SHA` is the ten characters of the commit the bench runs at,
and the frozen copy and the bench directory are named for it.
The blocks read it from `HEAD`,
so moving `HEAD` between the prepare block and the last arm makes `SHA` name another directory:
when `HEAD` may move,
write the prepare block's `SHA` value into the later blocks in place of the `git rev-parse` line.

```bash
# README.md (editor fidelity bench): prepare, once, at the commit the bench runs
SHA=$(git rev-parse --short=10 HEAD)
mise run //package/translation-repair/probe:build
cp --recursive package/translation-repair/probe/dist/final/node \
  "package/translation-repair/probe/node_modules/.frozen-dist-$SHA"
BENCH="$HOME/temp/agent/editor-fidelity-bench-$SHA"
mkdir --parents "$BENCH"
```

#### Dry runs before the first arm

A dry run of each arm with `--cap 0` and no environment at all but `PATH` and `HOME`
prints the manifest,
pool,
planned,
ceiling,
guard and `RULE` lines and exits without building a client or calling a model.
The line to read is the `RULE` line,
which is the rule the verdict will apply.
Run it for each arm with that arm's card,
cap,
`--bench-spent-usd` and trials flag:
write `high`,
`medium` or `low` on the first line of the block and run it once for each.
The number of units n is not known before the high arm has run,
so the dry runs of the later arms take a stand-in for it,
any number from 60 to 264:
the ledger entry of round 7 used 200.
The stand-in changes only the `planned:` and `ceiling` lines of the dry run,
and every dry run takes the same `--passes`.

```bash
# README.md (editor fidelity bench): dry run of an arm, ARM is high, medium or low
ARM=high
case "$ARM" in
  high) CARD=claude-sonnet-5-5-effort-high; MAXUSD=24; SPENT=0; TRIALS=(--passes 3) ;;
  medium) CARD=claude-sonnet-5-5-effort-medium; MAXUSD=18; SPENT=24; TRIALS=(--passes 3 --trials 200) ;;
  low) CARD=claude-sonnet-5-5-effort-low; MAXUSD=15; SPENT=42; TRIALS=(--passes 3 --trials 200) ;;
esac
env --ignore-environment PATH="$PATH" HOME="$HOME" \
  mise run //package/translation-repair/probe:editor-fidelity-probe -- \
  --candidates "$CARD" --candidates-alone --max-usd "$MAXUSD" --bench-spent-usd "$SPENT" "${TRIALS[@]}" --cap 0
```

#### The high arm and n

The arms run in this order,
because the first arm decides how many units the bench has.
`high` runs first with the high card,
`--max-usd 24` and `--passes 3`,
which walks the whole pool three times,
pass by pass
(264 units,
the trials of the pool asked three times each).
If the guard ends it before the passes are done,
it prints a `CAPPED at unit k` line,
keeps the rows it finished and exits 0;
that is a shorter bench and not a failed arm.
Each arm is the same command with its own name,
card,
cap,
`--bench-spent-usd` and trials flag,
which the block sets from `ARM` on its first line.
For `medium` and `low` it reads n from the high log
(the section "Medium and low" says how to take one pass in place of three)
and sums the `GUARD total=` lines of the logs written so far,
rounded up to the cent,
for `--bench-spent-usd`,
before the arm's own log exists.
Do not run it again for an arm whose log was written:
the arm's own log would join the sum.
The block for the first arm:

```bash
# README.md (editor fidelity bench): launch an arm, ARM is high, then medium, then low
ARM=high
SHA=$(git rev-parse --short=10 HEAD)
BENCH="$HOME/temp/agent/editor-fidelity-bench-$SHA"
case "$ARM" in
  high) CARD=claude-sonnet-5-5-effort-high; MAXUSD=24 ;;
  medium) CARD=claude-sonnet-5-5-effort-medium; MAXUSD=18 ;;
  low) CARD=claude-sonnet-5-5-effort-low; MAXUSD=15 ;;
esac
if [ "$ARM" = high ]; then
  SPENT=0
  TRIALS=(--passes 3)
else
  N=$(rg --only-matching --replace '$1' 'TRIALS completed ([0-9]+)' "$BENCH/high.log")
  if [ -z "$N" ]; then echo "no TRIALS line in $BENCH/high.log, so the high arm has no n" >&2; exit 1; fi
  REUSED=$(rg --only-matching 'PROMPT-REUSE source=' "$BENCH/high.log" | wc --lines)
  SPENT=$(rg --only-matching --no-filename --replace '$1' 'GUARD total=([0-9.]+)' "$BENCH"/*.log \
    | awk '{ micro += int($1 * 1000000 + 0.5) } END { printf "%.2f\n", int((micro + 9999) / 10000) / 100 }')
  TRIALS=(--passes 3 --trials "$((N - REUSED))")
fi
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

Read n,
the number of units the high arm completed with a call,
from the `TRIALS completed n of p units` line of `$BENCH/high.log`
less the rows a reply of the reuse layer answered,
which a `PROMPT-REUSE` line stands for each.
A unit that made no call is no trial:
the build of this round writes no such row,
and the billed high log of `dd4b15b75c` says 150 where 88 made a call.
The command prints the number alone.
For the log of a dry run (no `TRIALS` line) it prints nothing and exits 1
with a message on standard error;
the launch block,
the stop rule and the pre-launch block do the same,
so no arm gets `--trials 0`.
Each ends the shell it runs in,
so it runs in a shell of its own:

```bash
# README.md (editor fidelity bench): n, the units the high arm completed with a call
SHA=$(git rev-parse --short=10 HEAD)
BENCH="$HOME/temp/agent/editor-fidelity-bench-$SHA"
N=$(rg --only-matching --replace '$1' 'TRIALS completed ([0-9]+)' "$BENCH/high.log")
if [ -z "$N" ]; then echo "no TRIALS line in $BENCH/high.log, so the high arm has no n" >&2; exit 1; fi
REUSED=$(rg --only-matching 'PROMPT-REUSE source=' "$BENCH/high.log" | wc --lines)
echo "$((N - REUSED))"
```

#### Stop rule

When n less the rows of the high arm in which an ask threw is under 60,
do not launch medium or low.
Only rows that made a call count:
a row answered by the reuse layer (a `PROMPT-REUSE` line stands for it) is no trial and is taken off n.
A row in which an ask threw is left out of every arm,
so it cannot be shared,
and n counts it.
The command counts the high arm's reuse lines and its rows whose stop words include `threw` and prints what is left:

```bash
# README.md (editor fidelity bench): n less the rows without a call and the rows in which an ask threw
SHA=$(git rev-parse --short=10 HEAD)
BENCH="$HOME/temp/agent/editor-fidelity-bench-$SHA"
N=$(rg --only-matching --replace '$1' 'TRIALS completed ([0-9]+)' "$BENCH/high.log")
if [ -z "$N" ]; then echo "no TRIALS line in $BENCH/high.log, so the high arm has no n" >&2; exit 1; fi
REUSED=$(rg --only-matching 'PROMPT-REUSE source=' "$BENCH/high.log" | wc --lines)
THREW=$(rg --only-matching '; stopped [a-z+-]*threw[a-z+-]*$' "$BENCH/high.log" | wc --lines)
echo "$((N - REUSED - THREW))"
```

`rg` exits 1 and prints nothing when no row threw or no reply was a reuse,
so `wc` counts 0 and the pipeline exits 0 (do not set `pipefail` here).
Under 60,
record n,
the count and the `CAPPED at unit k` line of the high arm in the ledger and stop:
the read step refuses a verdict under 60 shared units,
and a later arm shares at most the units the high arm completed less those rows.

#### Before medium and low

Two reads decide whether and how to launch them.
First,
the spend of the high arm a unit,
and the units each later cap buys at that spend.
A lower effort asks for fewer output tokens,
so the high arm's spend a unit is taken as the most a later arm spends a unit;
this is an assumption the bench has not measured,
and the guard still ends an arm that spends faster.
An arm that spends s a unit completes floor((cap - 0.377) / s) + 1 units,
fewer than cap / s,
for reasons the simulations doc gives in its section on the units a cap buys.
The spend a unit is over the units that made a call.
For a log with no `PROMPT-REUSE` line it is the `GUARD total`,
which books no reuse,
and the block refuses a log that has no `GUARD total=` line.
A log of the billed build books each reuse at its ceiling,
so its spend is the `SPEND` lines without `estimated=`
(0.0071 USD a unit over 88 calls where the `GUARD total` reads 0.27),
which holds only while the `estimated=` lines are the reuses and no others;
the block refuses a log where their count differs.
The block counts that reserve and also checks n,
less the rows without a call and the rows in which an ask threw,
against the floor of 60,
which the cap alone does not:

```bash
# README.md (editor fidelity bench): the units each later cap completes, the guard's reserve counted
SHA=$(git rev-parse --short=10 HEAD)
BENCH="$HOME/temp/agent/editor-fidelity-bench-$SHA"
N=$(rg --only-matching --replace '$1' 'TRIALS completed ([0-9]+)' "$BENCH/high.log")
if [ -z "$N" ]; then echo "no TRIALS line in $BENCH/high.log, so the high arm has no n" >&2; exit 1; fi
REUSED=$(rg --only-matching 'PROMPT-REUSE source=' "$BENCH/high.log" | wc --lines)
THREW=$(rg --only-matching '; stopped [a-z+-]*threw[a-z+-]*$' "$BENCH/high.log" | wc --lines)
if [ "$REUSED" -eq 0 ]; then
  HIGH=$(rg --only-matching --replace '$1' 'GUARD total=([0-9.]+)' "$BENCH/high.log")
  if [ -z "$HIGH" ]; then echo "no GUARD total= line in $BENCH/high.log, so the spend is unknown" >&2; exit 1; fi
else
  CEILINGS=$(rg --only-matching ' SPEND provider=.* estimated=' "$BENCH/high.log" | wc --lines)
  if [ "$CEILINGS" -ne "$REUSED" ]; then
    echo "$CEILINGS SPEND lines at a ceiling for $REUSED reuses: the log does not tell the spend" >&2; exit 1
  fi
  HIGH=$(awk '/ SPEND provider=/ && !/ estimated=/ && match($0, /cost=[0-9.]+/) {
    micro += int(substr($0, RSTART + 5, RLENGTH - 5) * 1000000 + 0.5) }
    END { printf "%.6f\n", micro / 1000000 }' "$BENCH/high.log")
fi
echo "$HIGH $N $THREW $REUSED" | awk '{ calls = $2 - $4; per = $1 / calls; left = calls - $3; reserve = 0.377;
  m = int((18 - reserve) / per) + 1; l = int((15 - reserve) / per) + 1;
  printf "units left after thrown rows %d; spend a unit %.4f USD; ", left, per;
  printf "medium completes %d units, low completes %d units; %s\n", m, l,
  (left >= 60 && m >= 60 && l >= 60) ? "all reach the floor of 60" : "under the floor of 60: do not launch" }'
```

When either cap buys fewer than 60 units,
do not launch at that cap:
raise that arm's `--max-usd` only as far as the bench total allows
(the measured `GUARD total=` of the arms already run,
plus this arm's cap,
plus the cap of any arm after it,
under 60 USD),
and run the pre-launch read again with the new cap in place of 18 or 15;
when no cap within the total buys 60 units,
record the figures and stop.

Second,
the repeat agreement of the high arm,
which the read step prints for one log as it does for three
(the bench checks fail for want of three logs,
which is expected here,
and the `REPEAT` line is printed all the same):

```bash
# README.md (editor fidelity bench): the repeat agreement of the high arm
SHA=$(git rev-parse --short=10 HEAD)
BENCH="$HOME/temp/agent/editor-fidelity-bench-$SHA"
mise run //package/translation-repair/probe:editor-fidelity-read -- "$BENCH/high.log" | rg "^REPEAT "
```

The headers of `mise` and the refusal go to the error stream,
which the filter does not read.

When the line says `(100 percent)`,
the high arm gave every trial it was asked again the same reading in each pass,
and the later arms may take one pass,
which costs a third,
since a repeat that gives the first answer again adds no power:
`--passes 3 --trials 88` in place of `--passes 3 --trials "$N"` on the `TRIALS=` line of the launch block,
the first 88 units being the pool once.
That assumes the later arms repeat themselves as the high arm does,
which one pass cannot show:
the read step then prints `no trial was asked in more than one pass` for all three arms,
the high arm included,
because every arm is read over the 88 shared units,
so the 100 percent read from the high log alone is not repeated in that output.
If they do not repeat themselves so,
the one pass costs power,
and the simulations doc gives how much in its section on the passes a bench walks.
When the line says less than 100 percent,
the later arms take `--trials "$N"` as the launch block has it,
and the read step compares the arms' agreements after the last arm.

#### Medium and low

Then `medium` runs with the medium card and `--max-usd 18`,
and `low` with the low card and `--max-usd 15`,
each with `--passes 3 --trials n`
(or `--trials 88` where the high arm repeats itself on every trial),
so that each walks the same first n units in the same order.
The launch block of the section "The high arm and n" does it:
write `medium` on its first line and run it,
then write `low` and run it after the medium arm has ended.

After the last arm,
check all three together:

```bash
# README.md (editor fidelity bench): the read step
SHA=$(git rev-parse --short=10 HEAD)
BENCH="$HOME/temp/agent/editor-fidelity-bench-$SHA"
mise run //package/translation-repair/probe:editor-fidelity-read -- \
  "$BENCH/high.log" "$BENCH/medium.log" "$BENCH/low.log"
```

The read step prints a `REPEAT` line for each arm,
the counts and share of the trials the arm was asked in more than one pass whose passes all read the same way
(a clean fix in each or in none),
over the units every arm holds,
and a `WARN repeat agreement` line for each arm of the default effort that an exact one-sided test finds repeating
itself less than another arm,
naming that arm and its counts,
the arm it is under and its counts,
the p-value and the level.
The rule's bound on choosing another effort when none is better holds when the arms repeat themselves alike,
and a verdict of high or low printed under that warning is less sure than the level of 0.025 says.

On `WARN repeat agreement`,
the lead keeps the arms as evidence
(the line is a caveat and not a failed check,
and the exit status does not change)
and reports a verdict of high or low as shown better under the caveat
that the chance of a false pick may be above 0.05
(0.107 at a true fix rate of 0.9 by the committed fixture,
higher above it),
quoting the warning beside the verdict.
The lead weighs that verdict against its cost before changing the default effort.
A verdict of medium is not under the caveat of either warning,
since the chance concerns choosing another effort when none is better.
Rerunning an arm needs a bench total the owner sets,
so it is not a step of this runbook.

When the default effort cleanly fixes 90 percent or more of the units every arm holds,
it also prints `WARN high fix rate: medium cleanly fixed k of u shared units (p percent), ...`.
On that line,
do not read the absence of a `WARN repeat agreement` line as clearing the verdict,
because the repeat warning can miss unequal repeats at that fix rate.
Report a verdict of high or low as shown better under the caveat that the chance of a false pick may be 0.11 to 0.18
(0.108 at 0.91 to 0.175 at 0.95 by the committed fixture,
and up to 0.241 at 0.97 by the independent re-implementation,
which the simulations doc keeps apart),
and weigh it against its cost before changing the default effort.

An arm with a `FAIL` line is not evidence,
whatever its clean fix count says.
The checks are the exit status,
the manifest line,
the pool line,
the planned line,
the ceiling line,
the guard line (with the `--bench-spent-usd` it states),
the arm's `EDITOR` line,
the row count against the units it completed
(with a `CAPPED` line saying why an arm completed fewer than it planned,
and a failure for a row a `PROMPT-REUSE` line stands in front of,
which made no call and is no trial),
no `[error]` line,
the `SEAT` line (`threw` stated,
nothing unreachable,
no withheld or dark seat),
the identity of the `SPEND` lines' sum with the `GUARD total=`,
and that every row line states how its asks ended.
A unit in which an ask ended at the output-token cap or threw fails no arm:
the first scores as unusable for its arm
and the second is left out of every arm and counted.
A `WARN` line says that more than a tenth of the units of an arm were unusable or refused
(`WARN low: unusable plus refused is 5 of 40 units, over 10 percent`),
or that some were while the log states no count of units
(`WARN low: unusable plus refused is 5 of an unknown number of units (the log states none)`).
On that line the verdict stands as the rule computes it,
since a unit that yielded no edit scores as not fixed for its arm.
The lead reads the `ARM` and `STOPS` lines of that arm for how many units ended at the output-token cap,
were refused or were unusable otherwise,
and reports the count beside the verdict:
a verdict for an effort is not weakened by units its own arm lost,
and a verdict against an effort whose arm lost them
(medium kept while high or low lost units,
or high or low picked while medium lost units)
is reported as owed in part to the lost units and not wholly to worse edits.
Across the arms the checks are
three or four distinct logs at the cards high,
medium,
(a second medium if run)
and low in the order run,
each arm's stated `--bench-spent-usd` at least the `GUARD total=` of the arms before it,
each later arm having walked the first arm's units in the first arm's order,
the same manifest digest in every one,
a bench total under 60 USD,
and at least 60 units shared by every arm
(a unit is a trial in one pass over the pool).

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
