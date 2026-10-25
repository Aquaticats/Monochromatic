# Audit ledger: the editor bench's multi-pass arms, the false-pick bound and the repeat agreement

The follow-ups of B899 from round 8 on:
the planned line of an arm that walks the pool in passes,
the false-pick bound under unequal repeat agreement and the lines that print each arm's repeat agreement,
the runbook with its stop rule and its pre-launch read,
and the findings of the independent re-check of round 8, with what round 9 changed.

Part of [Audit ledger](audit-ledger.md),
which lists every file of this record.
It is a new file of 2026-10-25,
because [the paired rule file](audit-ledger-editor-bench-paired-rule-and-trials.md) stood at 1010 lines.
B894 to B899 and the sections of B899 before round 8 are in that file.

## B899, round 8: multi-pass arms, the false-pick bound under unequal repeat agreement, and the runbook

Status:
fixed on 2026-10-24 (UTC) in the commits the section "The round 8 commits" names,
open to the owner's veto.
No new id:
these are follow-ups to B899.

### The planned line of an arm that walks the pool in passes

The read step refused every arm that walked the pool in passes.
The planned line counts units (a trial in one pass),
and the check held it against the 88 trials of the pool,
so `264 trials planned of a pool of 88` failed all three arms of the bench the runbook launches.
No case had built a planned line and a pool line with the probe's own writers,
because the arm-log fixture wrote both by hand with the same count.

- The red cases (`071be645685d07b6c6fe7c498e860c43d3507f77`) build the planned,
  pool,
  ceiling,
  row and tally lines with
  the real writers,
  through a new fixture,
  and run them through the arm checks and the whole read step to the
  `PAIRED` and `VERDICT` lines.
  Nine cases failed by assertion,
  seven for the passes and two for the warning on unusable and refused trials.
- The fix (`e31574e91130ca6841a78b6752d620ff3decf4dc`) reads the passes the planned line names,
  and holds the planned units against the trials of the pool times those passes.
  A single-pass log keeps its wording (`41 trials planned of a pool of 88`);
  a multi-pass log says `264 units planned in 3 passes over a pool of 88 trials`,
  and a count over the passes says so.
- Every other comparison in the read step against a pool size or a single-pass count was searched for:
  the ceiling line against the planned units,
  the `EDITOR` line and the rows against the units completed,
  and the floor of 60 shared units.
  Each already counted units,
  and each now has a case over one pass,
  three whole passes,
  a first arm the guard capped at 150 of 264 units and a later arm given `--trials 150`.
- The warning on unusable and refused trials was a fixed count of 4,
  which is a tenth of 40 trials and under a sixtieth of 264 units.
  It is a tenth of the units now:
  `WARN low: unusable plus refused is 5 of 40 units, over 10 percent`,
  with a case at 26 and 27 of 264 and at 8 and 9 of 88.
  The exported constant `UNUSABLE_PLUS_REFUSED_WARNED_ABOVE` is replaced by
  `UNUSABLE_PLUS_REFUSED_WARNED_SHARE_PERCENT`;
  the old name is not added to a forbidden-strings file,
  because no other file names it
  (a search of `package` and `doc` found it only in this entry).

### The false-pick bound under unequal repeat agreement

The round 7 bound of 0.055 was simulated with one agreement for all three arms.
An arm that gives a trial its first answer again in every pass,
read against an arm asked afresh each pass,
is a case the grid did not hold.

The model has a difficulty shared by the arms with chance 0 or 0.7,
within-entry correlations 0,
0.5 and 1,
true fix rates 0.3,
0.5,
0.7 and 0.9,
the same fix rate for every arm,
and unit sizes 60,
88,
176 and 264.
Each arm has its own agreement of a repeat with its first answer,
0,
0.5 or 1,
which is 27 triples at 176 and 264 units and one at 60 and 88,
where no trial is asked twice.
That is 1344 cells,
800 replicates a cell,
and 4000 fresh replicates for each cell that screened at 0.05 or more.
Seeds name the cell and not the design,
so each design played the same benches.
The binomial noise at 0.05 is 0.008 at 800 replicates and 0.003 at 4000.
Each design ran once as a capped systemd user unit
(`edr8-sim-1`,
`edr8-sim-2` and `edr8-sim-3`,
MemoryMax 4G,
CPUQuota 200 percent,
one after the other),
with the output in `sim-sign.txt`,
`sim-studentized.txt` and `sim-means.txt` of the round's directory.

A sign-flip reference for a one-sample t gives the same p-value as the sum,
because t is monotone in the sum for a fixed sum of squares;
the run checks this (worst difference 0).
So the studentized candidates take a Student reference and a bootstrap-t reference.

The false pick chance of another effort when none is better,
by design:

- The rule's test,
  the exact sign-flip on entry sums:
  mean 0.036,
  worst screened 0.095,
  worst confirmed 0.098 (53 of 123 re-run cells over 0.05).
  With equal agreement the worst screened was 0.055 of 192 cells,
  and the four cells over 0.05 read 0.035 at most at 4000.
  The worst single effort was 0.054.
- A studentized test with a Student reference:
  mean 0.046,
  worst screened 0.114,
  worst confirmed 0.110.
- A studentized test with a bootstrap-t reference (500 resamples):
  mean 0.048,
  worst screened 0.130,
  worst confirmed 0.155 at 60 units with equal agreement.
- An exact sign-flip on per-entry mean differences with one weight for each entry:
  mean 0.038,
  worst screened 0.090,
  worst confirmed 0.094 (73 of 160 re-run cells over 0.05).
  With equal agreement the worst screened was 0.055 and the worst confirmed 0.040.

The smallest difference chosen with chance 0.8,
at a medium fix rate of one half,
seven in ten and eighty-five in a hundred,
for the rule's test,
the means test,
the Student reference and the bootstrap reference:

- At 60 units:
  30.4 and 24.0 and not reached by 15;
  30.0 and 23.6 and not reached;
  29.0 and 22.2 and not reached;
  29.4,
  22.1 and 13.3.
- At 88 units:
  25.3,
  20.5 and 13.7;
  25.0,
  20.4 and 13.6;
  23.6,
  19.7 and 13.1;
  25.0,
  19.7 and 12.6.
- At 264 units and agreement 1:
  25.6,
  20.6 and 13.6;
  25.1,
  20.4 and 13.5;
  24.7,
  19.6 and 12.9;
  25.2,
  19.6 and 12.5.
- At 264 units and agreement 0:
  13.5,
  11.5 and 8.6;
  13.6,
  11.9 and 8.7;
  13.1,
  11.3 and 8.1;
  13.3,
  11.3 and 8.0.

The rule's test reproduces the round 7 table within 0.7 point
(176 units at agreement 0,
one half and 1:
17.4,
14.6 and 10.1;
21.0,
17.7 and 11.9;
25.2,
20.7 and 13.7).

None of the three held 0.05 over the whole grid,
so the choice is the brief's fallback:
the rule's test stays,
and the bound is stated where it holds.
The per-entry means test is the nearest (0.094 against 0.098),
which is within one standard deviation of the rule's figure at 4000 replicates,
with the same power,
so changing the test would give up the committed exact decision for no bound.
The studentized tests are worse on both counts.

Where the excess sits:
it needs a true fix rate of 0.9 (none of the 28 re-run cells at 0.3 or 0.5 is over 0.05,
6 of 32 at 0.7 with 0.066 at most,
47 of 59 at 0.9),
and,
in all ten patterns of agreement (high,
low,
medium) that went over,
the default effort repeats itself less than another arm:
1/1/0 read 0.098,
0.5/1/0 0.095,
1/0.5/0 0.094,
1/0/0 0.070,
0.5/0.5/0 0.069,
1/1/0.5 0.065,
0/1/0 0.065,
0.5/0/0 0.056,
1/0.5/0.5 0.054 and 0.5/1/0.5 0.053.
It was found by simulation and not derived.

The committed fixture takes an optional `armAgreements` in `BenchScenario`
(red `194ec49c962a7c9b707abe6400c32819646cfdf6`,
fix `f6a37645c90c75e5746fce7b2778a70624a3bec1`).
Run once through it at 4000 replicates for 264 units at a true fix rate of 0.9
(seed prefix `edr8-sim/`,
output `fx-confirm.txt`):
agreement 1/1/0 and correlation 1 read 0.107,
the same agreement at correlation 0.5 read 0.088,
1/1/1 read 0.025,
0/0/0 read 0.034 and 0.5/0.5/0.5 at a rate of 0.5 read 0.043.
The scratch drivers (`edr8-sim.ts`,
`edr8-warn.ts`) are in the round's `scratch` directory and are not committed;
a unit test pins that the fixture goes over 0.07 at 1/1/0 and stays at or under 0.05 at 1/1/1.

The figures of the rule text come from two sources,
which are two implementations of one model and are not interchangeable.
The 192 cells,
the 0.055 and the 0.035 of the equal-agreement claim,
and every figure of the grid of 1344 cells,
are from the scratch driver `edr8-sim.ts` (seed `edr8-sim`,
a seed string of its own for each cell),
which takes only the order of the pool's trials from the fixture.
The 0.107 is from the committed fixture
(`pickShares` in `editor-fidelity-paired-null.test-fixture.ts`,
the units of three passes over 264 asks,
fix rates 0.9 for the three arms,
correlation 1,
share 0,
`armAgreements` `[1, 1, 0]`,
4000 replicates,
seed `edr8-sim/unequal 1/1/0 rate 0.9 corr 1`),
and the same cell read 0.098 in the driver.
The two readings differ by 0.009,
which is about 1.3 standard deviations of the difference of two readings at 4000 replicates,
so noise alone may explain it,
and the two implementations were not compared further.
The rule text cites the larger as what the committed fixture reaches
and the grid as what the driver reached.
The committed fixture was not run over the whole grid.

The rule text now says the chance is at most 0.05 when every arm repeats itself alike
(the worst of 192 cells was 0.055 at 800 replicates,
within the noise of 0.008,
and the four cells over 0.05 were at most 0.035 at 4000),
up to 0.107 when the default effort repeats itself far less than the others,
and that the read step prints each arm's repeat agreement and warns above 15 points.
The README quotes it,
checked as equal after whitespace is normalized.

### The repeat agreement and its warning

The read step prints,
after its check lines and before the count of them,
`REPEAT <arm>: 40 of 44 trials gave the same reading in every pass (91 percent)` for each arm,
or `no trial was asked in more than one pass`.
The share is of the trials an arm was asked in more than one pass
whose passes all read the same way,
a clean fix in each or in none,
with an ask that ended at the output-token cap not a clean fix and a row in which an ask threw left out.
It prints for one log as for three,
because the runbook reads the high arm's before the other arms exist
(the bench checks fail for want of three logs,
as expected,
and the line is printed all the same).
A `WARN repeat agreement:` line follows when the arms that have a figure differ by more than 15 points,
compared by cross-multiplying the counts so that exactly 15 does not warn.

The measured agreement is on another scale than the simulation's parameter,
which is the chance a repeat gives the first answer again:
two draws that agree with chance p agree about p squared plus one minus p squared of the time.
So the amount was calibrated on the measured figure,
by a separate seeded run (`edr8-sim-4`,
3000 replicates a cell,
176 and 264 units,
true fix rates 0.5,
0.7 and 0.9,
correlations 0.5 and 1,
share 0,
all 27 triples):

- At 15 points the warning fired on 4.5 percent of the benches whose arms repeat alike,
  on 75.9 percent where the parameters differ by one half and on 96.1 percent where they differ by 1.
  The worst cell over the benches it did not fire on read 0.046.
- At 20 points the figures were 0.9,
  63.3 and 88.8 percent,
  and 0.049.
- At 25 points the worst unwarned cell read 0.063,
  at 30 0.078,
  at 35 0.096 and at 40 0.106.
- At 10 points the warning fired on 21.4 percent of the benches whose arms repeat alike.

So 15 is the amount:
it keeps the unwarned chance under 0.05 with a margin of the noise of that run (0.004),
and fires on a twentieth of the benches that need no warning.
The limits are the grid's:
60 and 88 units and a true fix rate of 0.3 are not in it,
an arm on one pass prints no figure,
and the agreement is measured on the trials an arm was asked again,
which at three passes is all of them.

### The round 8 runbook

The README section "Launching the arms" (`probe/README.md`) now has:

- A dry-run block that runs as written
  (`mise run //package/translation-repair/probe:editor-fidelity-probe -- ... --cap 0`
  under `env --ignore-environment`,
  so that no key is in the environment),
  with the `--trials 200` of the later arms named a stand-in for n,
  which is not known before the high arm has run
  (`dryrun-r8.txt` of the round's directory holds the four dry runs).
- A command for n that prints the number alone
  (`rg --only-matching --replace '$1' 'TRIALS completed ([0-9]+)' "$BENCH/high.log"`),
  tried on a log of three whole passes (264),
  on a log the guard capped (150),
  and on a dry run's output,
  which prints nothing and exits 1 because it has a `planned:` line and no `TRIALS` line.
- A stop rule:
  with n under 60 the later arms are not launched,
  and n and the `CAPPED` line are recorded.
- The command that reads `GUARD total=` and sums the earlier arms for `--bench-spent-usd`,
  rounded up to the cent
  (4.00 for 1.5 and 2.5;
  3.82 for 2.500001 and 1.3125).
- A pre-launch check of medium and low:
  the high arm's `GUARD total=` over n is the spend a unit,
  taken as the most a later arm spends a unit,
  which is an assumption and not measured,
  and 18 and 15 USD over it are the units each later cap buys.
  Under 60 units for either,
  the arm is not launched at that cap,
  and its cap rises only within the 60 USD bench total or the bench stops.
  For the current pool the dry run's ceiling is 0.377 USD an ask,
  so at the ceiling the caps buy 47 and 39 asks,
  under the floor.
- When the high arm's repeat agreement is 100 percent,
  read from the `REPEAT high:` line of the read step run on its log alone,
  the later arms may take `--passes 3 --trials 88`,
  one pass,
  since at agreement 1 a second pass buys nothing
  (25 points at 88 units against 25 at 176 where a repeat always gives the first answer).
  This assumes the later arms repeat themselves as the high arm does,
  which one pass cannot show.
  The dry run of that form prices 88 asks at 33.1443 USD at the ceiling.

### Corrections to the round 7 slips

The round 7 entry records four of the slips of `editor-fidelity-r7/ledger-notes.md` in its section "Slips".
The rest are corrected here.

- The commit body of `100280270928a48e4727c4a850a9c23d52e98385` names the red as
  `0c94291e56ff6c1ff6c96bf66ac6f5e4f6b0eef9`,
  which is not a commit.
  The red is `0c94291e5603a41d0bf4d2c05f1248852c96e8d3`.
- The red `0c94291e5603a41d0bf4d2c05f1248852c96e8d3` carries stubs with the old behaviour
  (a table that knows ones and twos,
  one pass,
  no fixes),
  so its failing lines are wrong values and missing behaviour and not a load failure.
- `0da0a7831ed7186aae9e73386f614244f489c538` says 11 failing cases and the run shows 12
  (3 bench checks,
  5 report and 4 decision rule);
  `79839a033ba6a9ef316dcadad9c53856a70a0a7b` states 12.
- `326bb54c1a132ad46e7314604b01362bcf72c9e1` moved the plan-line cases into their own test file with their fix
  and had no red run.
- The red commits `06c0122568fa621965709bdabf80e0ffc3836bf0`,
  `a75383cb7e16862576f44b157b8ecf627c37a612`,
  `226ff26981ffc42872c1828ed2c00317cb90bd6c` and `abab3b1b30300cd044ab9f13a0c346f30b4ef088`
  fail the type check of their test files until their fixes land,
  as their bodies say.
- The three simulations of round 7 ran side by side as three capped units,
  once each and not repeated.
- The README failed the Markdown line-break lint at four places and the ledger at about thirty;
  both were broken after each clause and the source scans then passed.

### The round 8 commits

- Merge of `origin/main` at `ba772ca717028bb0811ea8d0178d2cb327c925da`:
  `a15f895f8d63060149dfeda66163686d2ab24b83`.
- Planned line and warning:
  red `071be645685d07b6c6fe7c498e860c43d3507f77`,
  fix `e31574e91130ca6841a78b6752d620ff3decf4dc`.
- Repeat agreement,
  per-arm agreement in the fixture and the rule's bound:
  red `194ec49c962a7c9b707abe6400c32819646cfdf6`
  (the new test file fails to load until the fix exports `repeatLinesOf`,
  and the type check reports it,
  as the body says),
  fix `f6a37645c90c75e5746fce7b2778a70624a3bec1`.
- The source scans failed on the new test files and on the new source after those commits.
  A test commit that moves the built-command helper of the passes and repeats cases into one fixture,
  merges the two top-level describes of the repeats file into one and stops exporting a name only its fixture reads:
  `0f7766cb07643e94fb9b2e898db260822116e080`.
- Bare counts of passes,
  units and trials after the two fixes,
  found by the count-noun scan:
  red `d4c1e2ea97d862426a8cfe640a1ff9f26a8be88c`
  (the over-count wording of the planned line names its units,
  and one trial asked again is counted in the singular),
  fix `61b31bd63a91bdbd5998e19bacb0c4e19c62232d`,
  which also groups the rows of an arm with `Map.groupBy` where the fold-copy scan found a copying fold.
- A type that only its fixture read,
  found by the scan of declarations nothing reads:
  `aa9bc6e498a48c274d845eff4953bcadd278e284`.

### The round 8 cache statement

`git log --extended-regexp -G '_CACHE_VERSION = [0-9]+' fd179058041b677f075f3656dee1a135427b0c7e..HEAD -- '*.ts'`
lists no commit of this round (the start of the round is that hash).
The same search over `c265e8df95~1..c265e8df95` lists `c265e8df9525f4513b8229482d3e8d932ea630df`,
so the search can show a hit.
This round changes the probe's read step,
rule text and tests and no request body,
reply or key.
No cache version moves.

### The round 8 slips

- `d4c1e2ea97d862426a8cfe640a1ff9f26a8be88c` says it fails 3 cases;
  the run shows 2 (the over-count case of the passes file,
  which holds two expectations,
  and the singular case).
- The red `071be645685d07b6c6fe7c498e860c43d3507f77` and the fix `e31574e91130ca6841a78b6752d620ff3decf4dc`
  left a bare count of passes in the planned line,
  and the red `194ec49c962a7c9b707abe6400c32819646cfdf6` and the fix `f6a37645c90c75e5746fce7b2778a70624a3bec1`
  left a bare count of trials,
  a copying fold,
  a second top-level awaited describe,
  a duplicated helper body and an exported name nothing read.
  The source scans caught them after the docs were written,
  and the four commits that follow the fix commit repaired them.
- The README and the ledger entry failed the scan of references by position and the Markdown line-break lint
  in the first draft.
  Phrases that pointed up or down the page were replaced by the name of the section they pointed to,
  and the lines were broken after every comma,
  semicolon,
  colon and full stop.
- The test file of `194ec49c962a7c9b707abe6400c32819646cfdf6` was changed in
  `0f7766cb07643e94fb9b2e898db260822116e080` and `d4c1e2ea97d862426a8cfe640a1ff9f26a8be88c`,
  after its fix commit,
  so the rule that a red commit's test files stay byte-identical in the fix holds for each pair
  and not for the file across the round.
- The simulation scripts and the warning calibration are scratch scripts of the round's directory and are not committed.
