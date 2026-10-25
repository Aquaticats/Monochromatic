# Audit ledger: the editor bench's exact repeat test, the high fix rate warning and the self-contained runbook

The follow-ups of B899 from round 10 on:
the high fix rate warning that meets the rule's stated region,
the exact test of the repeat warning with its measured rates,
the lead's actions on two warnings,
the self-contained runbook blocks,
and the units wording of the frozen lines.

Part of [Audit ledger](audit-ledger.md),
which lists every file of this record.
It is a new file of 2026-10-25,
because [the multi-pass arms file](audit-ledger-editor-bench-multi-pass-arms-and-repeat-agreement.md) stood at 788 lines
and this round's entry ran to 396 lines when written (403 after round 11).
The follow-ups of B899 of rounds 8 and 9 are in that file,
and those of round 11 are in [the prompt reuse file](audit-ledger-editor-bench-prompt-reuse-in-repeat-passes.md).

## B899, round 10: the gap between the rule's region and the warnings, an exact repeat test, and a self-contained runbook

Status:
fixed on 2026-10-25 (UTC) in the commits the section "The round 10 commits" names,
open to the owner's veto.

No new id:
these are follow-ups to B899,
and nothing in the round needed one.

The independent re-check of round 9 (`editor-recheck6.md`) found no major,
two minor and four nit;
the lead ruled on them on 2026-10-25 (the owner's standing rule is that no mistake is too small to fix)
and the rulings are applied as written,
red first for every change of behaviour.
`origin/main` at `b975d68aaf33cf267536ef1db27a1342cd17b0d1` was merged first (one doc file of main,
no conflict).

### The gap between 0.90 and the warning (finding 1, minor)

The rule text states its bounds for true fix rates 0.30 to 0.90 and names the first reading past them at 0.93,
and the high fix rate warning fired from an observed share of 0.93:
a true rate of 0.91 or 0.92 had neither a bound nor a warning,
where the fixture reads the false pick chance 0.108 and 0.123.

- `DEFAULT_FIX_RATE_WARNED_AT_PERCENT` is 90.
  The constant,
  its TSDoc (the threshold is read on an observed share),
  the WARN line (the repeat warning can miss unequal repeats at fix rates this high,
  and the line names the fixture's readings),
  the rule text,
  the README citations and the pinning cases changed together:
  red `92493edc5c` (four assertion failures),
  fix `f25619fa5f`.
- The rule text names the fixture's readings above 0.90 with their sources:
  0.108 at 0.91 and 0.123 at 0.92 (re-check 6,
  seed prefix `edrc6`,
  8000 replicates each),
  0.130 at 0.93 (seed `edr9-fx`,
  4000 replicates) and 0.132 at 0.93 (`edrc6`,
  8000 replicates),
  about 0.13,
  and 0.175 at 0.95 (`edr9-fx`,
  4000 replicates),
  each in three passes of 264 units with correlation 1.
  The independent re-implementation's 0.142 at 0.93,
  0.173 at 0.95 and 0.241 at 0.97 (`edrc5`) are named apart,
  in `FALSE_PICK_REIMPLEMENTATION_READINGS`.
- Cases:
  90 of 100 units warns and 89 of 100 does not;
  80 of 88 (90.9 percent) warns,
  79 of 88 (89.8) does not and 82 of 88 still does;
  133 of the 150 shared units (88.7 percent) does not warn though the arm fixes 93.6 percent of the 264 it walked;
  a second medium arm at 89 of 100 is not named;
  the built read step prints the line after the REPEAT lines.

The share of benches that print neither warning,
measured by one run of the committed fixture's own functions with the final warnings
(unit `edr10-sim`,
seed prefix `edr10-sim`,
arms high,
low and default repeating 1/1/0,
three passes,
`sim.ts` and `sim.txt` of the round's directory;
4000 replicates for 264 units and 6000 for 150):

- Positive control:
  the round 8 seed at a fix rate of 0.9 and correlation 1 read any pick 0.1072 (high 0.0503,
  low 0.0570) by the fixture's `pickShares` and 0.1072 by the script's pick.
- At 264 units neither warning printed on 0 of 4000 benches at 0.90,
  0.91,
  0.92 and 0.93.
  The fix rate warning fired on 0.5208,
  0.6965,
  0.8528 and 0.9545 of them,
  and the repeat warning on 1.0000,
  1.0000,
  1.0000 and 0.9998.
  The chance of any pick was 0.1042,
  0.1158,
  0.1115 and 0.1293.
- At 150 units (62 trials asked twice) neither warning printed on 2,
  1,
  1 and 2 of 6000 benches at 0.90,
  0.91,
  0.92 and 0.93.
  The fix rate warning fired on 0.5642,
  0.6972,
  0.8098 and 0.9075,
  the repeat warning on 0.9570,
  0.9263,
  0.8708 and 0.7860.
  The chance of any pick was 0.0718,
  0.0710,
  0.0757 and 0.0842.
- At correlation 0.5 neither printed on 0 of 4000 at 264 units at each rate,
  and on 2,
  3,
  0 and 0 of 6000 at 150 units.
- Of the 11 benches in all that printed neither warning at 150 units,
  one picked high or low (1 of 3 at 0.91 and correlation 0.5);
  that is too few to state a chance.
- Re-check 6 had measured the pair as built in round 9 at 150 units and 0.92:
  22.6 percent of benches printed neither warning,
  the repeat warning on 41.9 percent and the fix rate warning at 93 on 37.8.
  The exact repeat test of finding 6 is what closes most of that,
  with the line at 90 the rest.
- The share is observed and the true rate is not:
  at a true rate of 0.90 the fix rate line printed on 0.5208 of the benches at 264 units,
  a prompt for the caveat and not a verdict.

### The fixture's reading at 0.93 (finding 2, nit)

The README said the caveat was 0.13 to 0.24,
which set the fixture's 0.13 at 0.93 beside the re-implementation's 0.24 at 0.97.
It now gives the fixture's readings alone (0.108 at 0.91 to 0.175 at 0.95) in the lead's caveat,
the 0.93 reading as the fixture's pooled 0.131 over 12000 replicates (0.13 to 0.14),
and the re-implementation's readings apart in `probe/doc/editor-fidelity-decision-rule-simulations.md`.
At 0.93 the two sources differ by 0.011,
about 1.6 standard deviations of a difference of two such readings;
no second run of the other implementation was made.

### The lead's action on two warnings and a self-contained runbook (finding 3, minor)

- `WARN repeat agreement`:
  the lead keeps the arms as evidence,
  reports a verdict of high or low under the caveat that the chance of a false pick may be above 0.05
  (0.107 at a true fix rate of 0.9 by the fixture,
  higher above it) with the warning quoted beside it,
  weighs it against the cost before changing the default effort,
  and does not read a verdict of medium as under the caveat.
  A rerun needs a bench total the owner sets.
- `WARN <arm>: unusable plus refused is k of n units, over 10 percent`:
  the verdict stands as the rule computes it,
  since a unit that yielded no edit scores as not fixed for its arm;
  the lead reads the `ARM` and `STOPS` lines of that arm and reports the count beside the verdict,
  and reports a verdict against an effort whose arm lost the units as owed in part to them.
- Every block of the runbook sets the variables it reads
  (`SHA` and `BENCH` by the two lines of the prepare block,
  `N`,
  `THREW`,
  `HIGH`,
  and `ARM` with its card,
  cap,
  spend and trials flag through a `case` on the first line of the dry run and launch blocks).
  Each block was extracted from the README (`extract-block.ts`) and run as written in a fresh shell
  (`bash --noprofile --norc -c`) under the keyless environment,
  on logs of the probe's own writers (`make-readme-logs.ts`;
  invented cats) placed at the bench directory the block names:
  - dry run of the high,
    medium and low arms:
    `planned: 264 of 264`,
    `200 of 264` and `200 of 264`,
    `ceiling ...: 264 units,
    one ask each,
    at most 99.4330 USD`,
    `--max-usd 24`,
    `18` and `15`;
  - n:
    62 for the capped log with three thrown rows,
    and an empty line with exit 0 for the output of a dry run;
  - stop rule:
    59;
  - pre-launch:
    `units left after thrown rows 59; spend a unit 0.3823 USD; medium completes 47 units,
    low completes 39 units; under the floor of 60: do not launch`;
  - repeat agreement of the high arm:
    `REPEAT high: no trial was asked in more than one pass` on the 62-unit log and
    `REPEAT high: 88 of 88 trials gave the same reading in every pass (100 percent)` on the whole log;
  - read step on three whole logs:
    three REPEAT lines,
    `WARN high fix rate ... 264 of 264 shared units (100 percent)`,
    `CHECKS 49 pass,
    0 fail`,
    `SHARED 264 units over 44 entries`,
    and the verdict medium;
  - launch block for the three arms with a stub of `systemd-run` that prints its arguments
    (`stubs/systemd-run`,
    which starts nothing):
    `--max-usd 24 --bench-spent-usd 0 --passes 3` for high,
    `--max-usd 18 --bench-spent-usd 2.50 --passes 3 --trials 264` for medium
    and `--max-usd 15 --bench-spent-usd 4.00 --passes 3 --trials 264` for low
    (2.50 and 1.50 from the logs of the arms before it).
  The prepare block was not run as written (it builds and copies the package into `node_modules`),
  and the real launch was not run (it calls the provider);
  the launch block was run to the call of `systemd-run` only.

### Four wording points of the runbook (finding 4, nit)

- The n command prints an empty line and exits 0 for a log with no `TRIALS` line
  (run on a dry run's output:
  one newline,
  exit 0),
  and the README says so.
- `WARN high fix rate` is printed only when the logs share units,
  never for one log read alone and never for two logs without a low arm;
  the README says so and the `REPEAT` line is the one printed for a log read alone.
- The one-pass option prints `no trial was asked in more than one pass` for all three arms,
  the high arm included,
  because every arm is read over the 88 shared units;
  the README says so.
- The dry run block and the launch block take the arm on their first line,
  where the README held two assignment lines of which the second won.

### Units in two more lines (finding 5, nit)

The ceiling line the probe writes says `264 units,
one ask each` (and `1 unit`),
and the two checks are named `units in the first arm's order` and `units at the output cap or thrown`.
These lines are frozen into each arm's log,
so they changed before the first arm.
The ceiling check reads the number after the colon,
so it reads a line with either noun.
Red `1af1a7e98c` (17 FAIL lines in four files),
fix `4dbed26ba0`.
The full run of the package then found the case of the ceiling check that edits the line by hand,
which still replaced `trials`;
it was repaired in `1eafa6527c`.

### The exact test of the repeat warning (finding 6, nit)

The warning no longer tests a bare gap of 15 points.
It sets the default effort's count of trials whose passes all read alike against the count of the other arm with the highest
share by a one-sided Fisher exact test,
the hypergeometric tail summed in whole numbers and decided by cross-multiplying,
so a p-value of exactly one in twenty (0 of 3 against 3 of 3) is at the level of 0.05.
The level is 0.05 divided by the number of other arms.

- Why the division:
  computed exactly by enumerating the counts (`design-check.ts`,
  `design-check.txt`,
  no random draws,
  arms of 88 trials with one common agreement),
  one test at 0.05 against one other arm warns on 0.0413 at the most (0.0291 at agreement 0.9),
  and the better of two tests at 0.05 warns on 0.0734 at agreement 0.5,
  0.0655 at 0.7,
  0.0616 at 0.8 and 0.0528 at 0.9
  (at 12 trials 0.0576 at agreement 0.5).
  The division keeps the chance of a warning at or under 0.05 by the union bound when the trials of an arm are independent draws.
- Every bench gets the test and there is no floor on the trials asked again:
  9 of 12 against 12 of 12 (p 0.1087,
  re-check 6's three alike arms at 100 units) does not warn,
  7 of 12 (0.0186) does.
- The warning line prints both counts and the p-value and the level.
- Cases:
  the exact tails (15 of 20 against 20 of 20 is 0.0236,
  16 of 20 is 0.0530,
  38 of 44 against 44 of 44 is 0.0130,
  30 of 44 against 38 of 44 is 0.0367),
  computed twice (big integers and log-gamma floats) and equal to six decimals,
  the level in whole numbers,
  the division by the other arms,
  the arm named among equal shares,
  one-sided,
  and the line.
  Red `6a6a40b230` (7 repeat cases,
  the rule text and the new file failing to load),
  fix `a8eb43d3a0`.
  The fix changed the red commit's new test file in one respect:
  the lint rule `no-nullish-union` refused a return type with `undefined`,
  so `repeatShortfallOf` became `repeatShortfallsOf` and returns none or one entry of an array.
- `REPEAT_WARNED_ABOVE_POINTS` is gone,
  and with it the calibration TSDoc of the 15 points (4.5 percent over the grid,
  0.046 at most among the benches it did not fire on);
  those figures are history of round 8.

Measured by one run of the committed fixture's model against the built test (unit `edr10-sim`,
4000 replicates a cell,
a pass of 88 trials,
leading 100,
176 and 264 units of three passes,
holding 12,
88 and 88 trials asked again):

- Three alike arms at fix rates 0.5,
  0.7 and 0.9,
  a repeat giving the first answer again with chance 0,
  0.5 and 1,
  and within-entry correlation 0 and 1 (18 cells at each size,
  54 readings).
  The warning fired on 0.0000 to 0.0270 at 100 units,
  0.0000 to 0.0553 at 176 and 0.0000 to 0.0475 at 264.
  The mean over the 12 cells whose arms do not always repeat was 0.0150,
  0.0360 and 0.0354.
  One reading is over 0.05:
  0.0553 at a fix rate of 0.9,
  chance 0,
  correlation 1 and 176 units,
  where the two trials of an entry read as alike as the model allows
  and the independence the bound assumes fails most.
  The round 10 text called it 1.5 standard deviations of that many replicates (0.0036) over 0.05 and within noise.
  Re-check 7 showed that wrong,
  and the round 11 file corrects it in its section on the 0.0553 reading:
  the cell reads 0.0537 over 34000 replicates.
- The default effort repeating itself less than high and low (which repeat on every trial) by 15 and 30 points of repeat
  agreement,
  the gap set for the passes of each size (two passes at 100 and 176 units,
  three at 264;
  realised mean gaps 14.7 to 15.2 and 29.9 to 30.3 points),
  fix rates 0.5,
  0.7 and 0.9,
  correlation 0 and 1:
  15 points was seen on 0.0187 to 0.0410 at 100 units,
  0.9840 to 0.9955 at 176 and 0.9918 to 0.9960 at 264;
  30 points on 0.2717 to 0.2875 at 100 units and on 1.0000 at 176 and at 264.
  At 0.9 a gap of 30 points cannot arise (an arm asked afresh each pass reads two passes alike on 82 percent and three on 73),
  and the largest gaps,
  18 and 27 points,
  were seen on 0.0413 and 0.0622 at 100 units,
  0.9998 and 0.9978 at 176 and 1.0000 at 264.
- The other arms in that run have no disagreement of their own,
  so these are the figures for the clearest case:
  against arms that repeat less than always the test needs a larger difference.
  Whether the real Sonnet editor repeats itself unequally was not measured.

### The README and a new doc

The README held 944 lines and would have held 1064.
The prose after the rule quote moved to `probe/doc/editor-fidelity-decision-rule-simulations.md`
(204 lines when written,
212 when re-check 7 read it,
223 after round 11),
the README keeps the figures of this round in a short list and links the doc,
and stands at 972 lines (999 after round 11).
The README quote of the rule equals `decisionRuleText()` of the built package (4249 characters each,
whitespace normalised,
`quote-check2.ts`);
the check against the `RULE` line of a dry run of the final head is in the round's report.

### The round 10 commits

- Merge of `origin/main` at `b975d68aaf33cf267536ef1db27a1342cd17b0d1`:
  `cb582cad94e67e2706510c163bfcbc0f86ea3536`.
- Finding 5:
  red `1af1a7e98cfa3ce1745413ed2094cbc14a69b5e5`,
  fix `4dbed26ba037ce81dcf635d503bb7fadf9158aa6`,
  and the repair of the ceiling check case `1eafa6527ceaac895146e9e6ba29e5fac5b1aa05`.
- Finding 6:
  red `6a6a40b230160606774f5d87388c04c4b4dd457f`
  (its body says "8 cases of the repeat suite and the rule text":
  7 cases of the repeat suite and the rule text,
  8 in all),
  fix `a8eb43d3a0b678682b593210a929697532b41209`
  (its body says "the six assertions" that follow the new return shape:
  seven assertion lines changed).
- Finding 1:
  red `92493edc5cbcb503332878b6aaa852a196cd110a`,
  fix `f25619fa5fb734e192db59245eb237d5ea23dd75`.
- The README and the new doc:
  `94f0725cb386096fc686a8b20db4deba4082c96a`,
  and `e377af832341785ebc9c614c99fbe1021f713f02` for two references by position and three lines broken inside a clause.
- The number reads scan lists the exact test's calls of `Number` and `BigInt`:
  `466c1155bb9d77af77f950de52c818cc51d5e276`.

### The round 10 cache statement

`git log --extended-regexp -G '_CACHE_VERSION = [0-9]+' 763cf869627c41827ed11614587246252511e3fa..HEAD -- '*.ts'`
lists no commit (the range holds the merge of main).

The same search over `c265e8df95~1..c265e8df95` lists `c265e8df9525f4513b8229482d3e8d932ea630df`,
so the search can show a hit.

This round changes the probe's rule text,
read step lines,
check names and the ceiling line,
and no request body,
reply or key.

No cache version moves.

### The round 10 slips

- Commands outside the keyless wrapper at the start of the session (read-only listings and reads of files of the brief
  and of the worktree);
  one chain of variable assignments before an `env` call;
  three `; echo` suffixes to a command.
- Two `mise run` calls (the dry runs of medium and low) in one tool block.
- Two `git commit` calls in one tool block (`466c1155bb` and `e377af8323`).
- The red commit `6a6a40b230` and the fix `a8eb43d3a0` state counts that are imprecise (see the commits).
- The fix of finding 5 was run against the four test files it touched,
  and left the ceiling check case of `editor-fidelity-arm-check.unit.test.ts` failing until the full run found it.
- The new test file of the red commit of finding 6 changed in the fix commit (the array return),
  where rounds 8 and 9 kept red and fix tests byte-identical.
- One `Edit` call on a path that does not exist.
- The reasoning of the design check (the exact enumeration) ran in a capped unit as `edr10-design`;
  a smoke run of the simulation script with 5 replicates ran as `edr10-smoke` before the one run `edr10-sim`.
