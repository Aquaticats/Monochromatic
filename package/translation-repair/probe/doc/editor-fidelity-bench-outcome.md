# Editor fidelity bench: the outcome

This file records what the editor fidelity bench found when its three arms had run and the read step had read them,
so that a reader of the [probe README](../README.md) can see the result without opening the logs.
Every figure is copied from the read step's own output over the three arm logs.
The bench ran at `85e21653a2`,
and its three logs (`high.log`,
`medium.log` and `low.log`) are in the bench directory `editor-fidelity-bench-85e21653a2`.
The rule that judged the arms is quoted in the README section "Decision rule",
and the simulations behind its bounds are in
[the simulations behind the decision rule](editor-fidelity-decision-rule-simulations.md).
To check a figure here,
run the README's read step (the block "the read step" under "Medium and low") over the three logs,
with `SHA=85e21653a2` set by hand in place of the block's `git rev-parse` line,
since a later checkout names a bench directory that does not exist,
and compare the `ARM`,
`PAIRED`,
`KINDS`,
`REPEAT` and `VERDICT` lines.

## What the bench ran

The pool held 88 trials over 53 entries (insertion 50,
alteration 38),
and each arm walked it in 3 passes,
which is 264 units.
The three logs carry the same manifest digest,
`09ee3857d15b4c244e66e07e88a720327e82907ebe2093773f7914f133ecfb62`,
and the arms ran in the order high,
medium,
low.
All 264 units were shared by every arm:
264 units over 53 entries,
with none left out as thrown and none not completed by every arm.
Every arm ended with all 264 rows logged and every ask stopped as `ok`,
and none was unusable,
refused,
thrown,
abandoned or at the output cap.
The read step reported 49 checks passed and 0 failed.

## The three arms

Clean fix is exact plus equivalent,
and every clean fix of this bench was exact (equivalent 0 in each arm).

- High:
  clean fix 125 of 264 (insertion 52 of 150,
  alteration 73 of 114),
  defect only 133 (insertion 92,
  alteration 41),
  collateral 139,
  missed 6.
- Medium:
  clean fix 112 of 264 (insertion 42 of 150,
  alteration 70 of 114),
  defect only 143 (insertion 99,
  alteration 44),
  collateral 152,
  missed 9.
- Low:
  clean fix 112 of 264 (insertion 41 of 150,
  alteration 71 of 114),
  defect only 144 (insertion 101,
  alteration 43),
  collateral 152,
  missed 8.

## The two paired tests

Each of high and low was tested against medium,
the default,
by the one-sided exact paired permutation test of the rule at 0.025,
over 264 shared units from 53 entries.
The interval is the 95 percent interval of the seeded bootstrap over those 53 entries.

- High against medium:
  +4.9 points,
  interval +0.7 to +9.8 points,
  p 0.0288,
  shown better at 0.025:
  no.
  By kind,
  which is secondary and vetoes nothing:
  insertion +6.7 points over 150 units,
  alteration +2.6 points over 114 units.
- Low against medium:
  +0.0 points,
  interval -2.8 to +2.7 points,
  p 0.6016,
  shown better at 0.025:
  no.
  By kind:
  insertion -0.7 points over 150 units,
  alteration +0.9 points over 114 units.

## Repeat agreement

The read step prints,
for each arm,
the trials that gave the same reading in every pass:

- High:
  80 of 88 trials (91 percent).
- Medium:
  81 of 88 trials (92 percent).
- Low:
  83 of 88 trials (94 percent).

The read step printed no warning line.

## Spend

The bench amount is 60 USD for all the arms together,
and the caps named in the README (24,
18 and 15) add to 57.
The SPEND lines and the guard of each arm agree:

- High:
  1.860067 USD,
  held to `--max-usd 24` after 0 USD spent by the arms before it.
- Medium:
  1.634552 USD,
  held to `--max-usd 18` after 1.87 USD declared as spent before it (the arms before it counted 1.860067 USD).
- Low:
  1.619022 USD,
  held to `--max-usd 15` after 3.5 USD declared as spent before it (the arms before it counted 3.494619 USD).

The total measured is 5.113641 USD,
under the 60 USD allowance.

## The verdict

The read step printed this line (broken here at its punctuation):

> VERDICT medium (none-shown):
> no effort was shown better than medium at 0.025,
> so the default effort stays;
> that does not show the others are as good

## What the verdict means

- Medium stays the editor's effort.
  No code changes come of the bench,
  and the card and the default effort stay as they were.
- High was not shown better.
  Its estimate of +4.9 points came with p 0.0288,
  which missed the level of 0.025 that the rule fixed before any arm ran.
  That does not show high is not better:
  the interval runs from +0.7 to +9.8 points,
  and this bench does not settle it.
- Low was not shown better either,
  with an estimate of +0.0 points and p 0.6016.
  The rule's own words apply to both efforts:
  a medium verdict does not show the others are as good.

## No second medium arm

The read step can read a second medium arm as one more control,
and the rule places it between the medium arm and the low arm in run order.
None was run.
The low arm had already run when the read step read the logs,
so a second medium run after the read would have been chosen after the results were seen,
and it would not stand in the order the rule gives it.

## Where the figures are

The read step's output over the three logs is the source of every figure in this file.
The section "Round 15" of
[the editor bench ledger](../../doc/audit-ledger-editor-bench-prompt-reuse-in-repeat-passes.md)
records the same outcome beside the fixes of that round.
