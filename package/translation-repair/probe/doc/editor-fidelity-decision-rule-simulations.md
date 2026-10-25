# Editor fidelity bench: simulations behind the decision rule

The decision rule of the editor fidelity bench
(`decideEffort` in `src/corpus-run/editor-fidelity-decide.ts`,
quoted in the [probe README](../README.md))
states its bounds on the chance of choosing another effort when none is better,
and the read step warns where those bounds do not hold.
This file keeps the simulations those statements rest on:
the false pick bounds and the readings above a true fix rate of 0.9,
the exact test of the repeat warning with its measured false alarm and detection rates,
the high fix rate warning with the share of benches that print neither warning,
and the model the simulations play.
The figures of rounds 8 and 9 are also in the ledger entries of
[the multi-pass arms and repeat agreement ledger][arms-ledger],
and those of round 10 in [the exact repeat test ledger][round-10-ledger].

[arms-ledger]: ../../doc/audit-ledger-editor-bench-multi-pass-arms-and-repeat-agreement.md
[round-10-ledger]: ../../doc/audit-ledger-editor-bench-exact-repeat-test-and-self-contained-runbook.md

## The false pick bounds

Two efforts are each tested at 0.025,
so the chance of picking either when neither is better is under 0.05 where the test holds its level.
It holds it when every arm repeats itself alike and the true fix rate is no higher than 0.9:
the simulation read 0.055 at the worst of 192 such cells at 800 replicates,
which is inside the noise of that many replicates (0.008),
and the four cells over 0.05 read 0.035 at most when run again at 4000.
The grid stopped at a true fix rate of 0.9,
so the figures are not stated for higher fix rates.
It does not hold it when the arms repeat themselves unlike one another.
With the high and the low effort giving a trial their first answer again in every pass
and the default effort asked afresh each pass,
at a true fix rate of 0.9 over 264 units,
the chance of choosing another effort when none is better was 0.098 at the worst cell of the grid run at 4000
replicates and 0.107 by the committed fixture at 4000 replicates.
Both are readings at 0.9 and not a ceiling:
the chance rises with the fix rate.
Above 0.9 the committed fixture read this
(three passes of 264 units,
the arms repeating 1/1/0,
correlation 1):
0.108 at 0.91 and 0.123 at 0.92 (re-check 6,
seed prefix `edrc6`,
8000 replicates each),
0.1298 at 0.93 (`edr9-fx`,
4000 replicates) and 0.1318 at 0.93 (`edrc6`,
8000 replicates),
which pool to 0.131 over 12000 replicates,
and 0.1750 at 0.95 (`edr9-fx`,
4000 replicates).
The reading at 0.93 is therefore the fixture's 0.13 to 0.14.
At a correlation of 0.5 the fixture read 0.0975 at 0.93 and 0.1365 at 0.95 (`edr9-fx`),
and at 0.9 under the round 8 seed it read 0.1072,
which round 10 read again under that seed as the positive control of its run.
An independent re-implementation of the model (re-check 5,
seed prefix `edrc5`,
4000 replicates,
264 units,
correlation 1) is a second source,
and its readings are kept apart from the fixture's:
0.142 at 0.93,
0.173 at 0.95 and 0.241 at 0.97,
and 0.126 at 0.95 at a correlation of 0.5.
At 0.93 the two sources differ by 0.011,
about 1.6 standard deviations of a difference of two such readings,
and the two models are one law with different random streams.
Of the unequal-agreement cells that screened at 0.05 or more and were run again at 4000 replicates,
none of the 28 at a true fix rate of 0.3 or 0.5 read over 0.05,
6 of 32 at 0.7 did (0.066 at most),
and 47 of 59 at 0.9 did.
A single effort alone read 0.054 at the worst.
The excess was found by simulation and not derived.
The default effort repeating itself more than the others read 0.011 in the re-check's control,
so the read step prints the repeat agreement of each arm
and warns only when the default effort is the one that repeats itself less,
and a pick of high or low under that warning is less sure than the level of 0.025 says.

## The repeat warning

The warning is a one-sided Fisher exact test of the default effort's count of trials whose passes all read alike
against the count of the other arm with the highest share,
at 0.05 divided by the number of other arms
(0.025 for the usual three arms,
0.017 when a second medium arm is read).
It replaced a bare gap of 15 points that took no account of how many trials were asked again:
re-check 6 saw that gap warn on three alike arms at 100 shared units,
from 9 of 12 trials against 11 of 12.
The division of the level matters.
Computed exactly by enumerating the counts (no random draws,
the output is in the ledger entry for round 10),
the better of two tests at 0.05 warned on up to 0.073 of the benches of arms alike
when two other arms hold 88 trials each,
and dividing the level keeps the chance of a warning at or under 0.05 by the union bound,
counting the trials of an arm as independent draws.
Every bench gets the test and there is no floor on the trials asked again:
a bench with few of them simply needs a larger difference.
Round 10 measured the rates by simulation of the committed fixture
(one run,
seed prefix `edr10-sim`,
4000 replicates a cell,
a leading 100,
176 or 264 units of three passes over the pool of 88 trials,
which hold 12,
88 and 88 trials asked again):

- Three alike arms,
  at fix rates 0.5,
  0.7 and 0.9,
  with a repeat giving the first answer again with chance 0,
  0.5 and 1,
  and within-entry correlation 0 and 1 (18 cells at each size).
  The warning fired on 0.0000 to 0.0270 of the benches at 100 units,
  0.0000 to 0.0553 at 176 and 0.0000 to 0.0475 at 264.
  The mean over the 12 cells whose arms do not always repeat was 0.015,
  0.036 and 0.035.
  One of the 54 readings is over 0.05:
  0.0553 at a fix rate of 0.9 with chance 0 and correlation 1 at 176 units,
  where the trials of one entry read as alike as the model allows and are not independent draws.
  It is not noise.
  Re-check 7 (seed prefix `edrc7`) read the same cell at 0.0535 over 30000 replicates at 176 units
  (standard error 0.0013) and 0.0488 at 264 units (0.0012),
  and pooled with the 4000 of this run the cell at 176 units reads 0.0537 over 34000,
  about three standard deviations over 0.05.
  The false alarm rate of the warning is about 0.054 in that cell,
  and below 0.05 in the three other cells re-check 7 took beside it,
  at 176 units:
  0.0296 (fix rate 0.9,
  agreement 0,
  correlation 0),
  0.0430 (fix rate 0.7,
  agreement 0,
  correlation 1)
  and 0.0379 (fix rate 0.9,
  agreement 0.5,
  correlation 1).
  The frozen `RULE` line says the chance is held at or under 0.05 counting the trials of an arm as independent draws.
  That is a statement about independent draws,
  and the trials of this cell are not.
- The default effort repeating itself less than high and low,
  which repeat on every trial,
  by 15 and by 30 points of repeat agreement (fix rates 0.5,
  0.7 and 0.9 and correlation 0 and 1).
  A gap of 15 points was seen on 0.019 to 0.041 of the benches at 100 units
  (no better than a false alarm,
  as 12 trials asked again leave no room for it),
  on 0.984 to 0.996 at 176 units and 0.992 to 0.996 at 264.
  A gap of 30 points was seen on 0.272 to 0.288 at 100 units and on 1.0000 at 176 and at 264.
  At a fix rate of 0.9 a gap of 30 points cannot arise,
  since an arm asked afresh in each pass reads two passes alike on 82 percent of the trials and three passes on 73:
  the largest gaps,
  18 and 27 points,
  were seen on 0.041 and 0.062 at 100 units and on 0.998 to 1.0000 at 176 and 264.
  The other arms having no disagreement of their own,
  these are the figures for the clearest case;
  against arms that repeat less than always the test needs a larger difference.
- The positive control,
  the round 8 seed at a fix rate of 0.9 with the arms repeating 1/1/0,
  read 0.1072 by the fixture's own `pickShares` and by the script's pick,
  as in rounds 8 and 9.

## The high fix rate warning

The high fix rate warning is read on the share observed,
and it fires from 90 percent so that the region the rule states its bounds for (true fix rates up to 0.90)
and the warning meet.
At 93 percent a true rate of 0.91 or 0.92 had neither a stated bound nor a warning,
and at 150 units (a capped bench holding 62 trials asked twice) re-check 6 saw neither warning print on 22.6 percent of
the benches at 0.92.
Round 10 measured the benches that print neither warning,
with the final warnings (the fix rate warning at 90 percent of the shared units and the exact repeat test),
the arms repeating 1/1/0,
correlation 1,
4000 replicates at 264 units and 6000 at 150:

- At 264 units neither warning printed on 0 of 4000 benches at true fix rates 0.91,
  0.92 and 0.93 (and 0.90).
  The fix rate warning fired on 0.697,
  0.853 and 0.955 of them,
  and the repeat warning on 1.000,
  1.000 and 0.9998.
- At 150 units neither warning printed on 1,
  1 and 2 of 6000 benches at 0.91,
  0.92 and 0.93 (0.0002,
  0.0002 and 0.0003;
  2 of 6000 at 0.90).
  The fix rate warning fired on 0.697,
  0.810 and 0.908,
  and the repeat warning on 0.926,
  0.871 and 0.786.
- At correlation 0.5 neither printed on 0 of 4000 at 264 units,
  and on 3,
  0 and 0 of 6000 at 150 units at 0.91,
  0.92 and 0.93.
- Of the 11 benches in all that printed neither warning
  (11 of 80000 over 16 cells,
  four of them at the true rate 0.90 where the rule's own bound holds),
  one picked high or low,
  which is too few to state a chance.

So when the default effort cleanly fixes 90 percent or more of the units every arm holds,
the read step prints `WARN high fix rate`,
and a verdict of high or low stands under the caveat whether or not a `WARN repeat agreement` line printed.
The share is observed and the true rate is not:
at a true rate of 0.90 the line printed on 0.5208 of the benches at 264 units,
a prompt for the caveat and not a verdict.

## The smallest difference at 60 shared units

At 60 shared units the smallest difference detected with chance 0.8 is about 30 points when medium fixes half
and 24 points when it fixes seven in ten,
against 37 and 28 points at 40 units,
which is why 60 is the fewest the read step accepts.

## The passes a bench walks

At 88 units the smallest difference chosen with chance 0.8 is about 26 points when medium fixes half,
against 25 points at 176 units where a repeat always gives the arm's first answer again,
so a second pass buys the bench nothing when the arms repeat themselves so.
Where they do not repeat themselves so,
three passes over 264 units detect a difference of about 13.5 points where agreement is 0 (round 8 power table)
and one pass over 88 units detects about 25 points,
and the verdict is read over those 88 units whatever the high arm's 264,
so the bench money the high arm spent on its passes 2 and 3 buys the verdict nothing.

## The units a cap buys

The guard admits an ask only while the spend so far plus the ask's ceiling is at most the cap.
The ceiling is the `ceiling` line of the dry run over its asks,
0.377 USD an ask for the current pool.
An arm that spends s a unit therefore completes floor((cap - 0.377) / s) + 1 units,
which is fewer than cap / s:
about 21 fewer at 0.0167 USD a unit,
and one fewer at the margin of 60 units
(0.25 USD a unit at 15 USD gives 59,
where 15 / 0.25 says 60).

## The simulation

The simulation plays the bench over the committed pool in its order,
pass by pass:
three arms fix each trial with a stated chance,
share a trial's draw with chance 0 or one half,
read the trials of an entry alike with a correlation from 0 to 1,
and give a trial asked again the arm's first answer with a stated agreement from 0 to 1 and a fresh draw otherwise.
It is `src/corpus-run/editor-fidelity-paired-null.test-fixture.ts`
(with its table and units in sibling fixture files),
and its unit test pins that its table agrees with the rule's exact test for entries of any size,
that it walks the pool's 88 trials in order and then again for each pass,
that a repeat follows its agreement,
and that one cell picks nothing more often than five in a hundred.
The grid of arms with their own agreement ran once for each of three tests,
seeded,
with 800 replicates for each of 1344 cells and 4000 for the cells that screened at 0.05 or more:
the exact sign-flip test the rule uses,
a studentized test (a Student reference and a bootstrap reference),
and an exact sign-flip test on per-entry mean differences with one weight for each entry.
None held 0.05 over the whole grid
(worst confirmed 0.098 for the rule's test,
0.110 and 0.155 for the Student and bootstrap references,
and 0.094 for the per-entry means),
and the per-entry mean test lost no power against the rule's test
but gained no bound,
so the rule's test stays and its bound is stated where it holds.
The figures are in the ledger entry for round 8.