# Seating the editor and refiner columns on the full-roster calibration

Decided 2026-08-25 from the finished 40-slice full-roster calibration,
launched 2026-08-25T01:30Z and finished in 42518 seconds,
zero `[error]` lines.
Read with `~/temp/agent/standing-from-log.mjs`,
whose self-test reproduces
a prior run exactly and which derives the Bonferroni critical value from the row count
rather than hardcoding one.

The owner delegated this:
"You have authorization to drop models that are exceptionally bad."

## What the run measured

Two columns,
scored separately,
both against a ten-seat Bonferroni threshold of z = 2.81.

```text
EDITOR   114 judged rounds from 31 of 40 slices, pooled 1177/7438 = 15.82%
         3.68 rounds per contributing slice, so deflation is sqrt of that = 1.92x

  29.6%  z  9.83  deflated  5.12  CLEARS   hf:zai-org/GLM-5.2       (200/675, 112 candidates)
  22.7%  z  3.98  deflated  2.07  raw only qwen3.8-max              (102/450,  81 candidates)
  21.2%  z  4.13  deflated  2.15  raw only hf:moonshotai/Kimi-K3    (169/799, 132 candidates)
  21.0%  z  3.78  deflated  1.97  raw only hf:Qwen/Qwen3.8-27B      (151/720, 117 candidates)
  14.2%  z -1.23  deflated -0.64  no       deepseek-v4-pro-0813     (108/761, 128 candidates)
  12.4%  z -2.67  deflated -1.39  no       minimax-m3               ( 98/793, 124 candidates)
  12.3%  z -2.73  deflated -1.42  no       deepseek-v4-flash-0731   ( 99/804, 128 candidates)
  12.1%  z -2.96  deflated -1.55  raw only hf:openai/gpt-oss-120b   (102/843, 127 candidates)
  11.3%  z -3.50  deflated -1.83  raw only nvidia Nemotron-3        ( 91/804, 124 candidates)
   7.2%  z -6.62  deflated -3.45  CLEARS   gemma-4-26b-a4b-it       ( 57/789, 117 candidates)

REFINER   22 judged rounds from 22 of 40 slices, pooled 126/853 = 14.77%
          1.00 rounds per contributing slice, so deflation is 1.00x

  30.2%  z  3.16  CLEARS   hf:zai-org/GLM-5.2       (16/53,   6 candidates)
  29.6%  z  4.35  CLEARS   hf:Qwen/Qwen3.8-27B      (32/108, 13 candidates)
  24.5%  z  2.77  no       qwen3.8-max              (25/102, 12 candidates)
  14.0%  z -0.22  no       hf:moonshotai/Kimi-K3    (16/114, 14 candidates)
   9.8%  z -1.28  no       nvidia Nemotron-3        ( 8/82,  10 candidates)
   9.4%  z -2.00  no       hf:openai/gpt-oss-120b   (16/171, 20 candidates)
   7.7%  z -1.02  no       minimax-m3               ( 2/26,   3 candidates)
   6.8%  z -1.73  no       deepseek-v4-pro-0813     ( 4/59,   7 candidates)
   5.4%  z -2.99  CLEARS   gemma-4-26b-a4b-it       ( 7/129, 15 candidates)
   0.0%  z -1.25  no       deepseek-v4-flash-0731   ( 0/9,    1 candidates)
```

## The refiner column carries no deflation, which is why it decides more than its size suggests

Its 22 rounds came from 22 distinct slices,
exactly one each,
so the within-slice correlation that deflates the editor column by 1.92x does not exist here at all.
Twenty-two independent rounds outrank 114 correlated ones for this purpose,
because the effective sample is the count of independent slices rather than of rounds.

This corrects an alarm raised mid-run and recorded in `doc/handover/translation-repair.md`:
the refiner column looked thin and turned out to be the cleaner of the two instruments.

## Decision one: `hf:zai-org/GLM-5.2` is the first-choice writer in both columns

Best in both,
and the only seat clearing on the strong side of both after deflation.
Its editor margin is not close:
z 9.83 raw,
5.12 deflated,
against a threshold of 2.81.

## Decision two: `hf:Qwen/Qwen3.8-27B` is a refiner before it is an editor

It clears decisively as a refiner (z 4.35,
undeflated) and fails as an editor (1.97 deflated).
Where the two roles compete for one seat,
it takes the refiner one.

## Decision three: the roster is NOT narrowed, and `gemma-4-26b-a4b-it` stays

It is measurably last in both columns and significantly so,
z -3.45 and -2.99 after deflation.
The owner's authorization to drop exceptionally bad models would cover it on those numbers alone.
It stays anyway,
for a reason the numbers do not show and the pipeline's shape does:

-   A losing candidate does not degrade the output.
    Judges choose among candidates;
    one that never wins costs a call and changes nothing that ships.
    "Worst" here means "least often preferred",
    not "harmful".
-   The cost argument is the only argument for dropping it,
    and the owner's standing guideline makes cost a non-constraint.
-   It answers cleanly:
    zero cut voices,
    zero zero-content streams,
    zero reasoning volume.
    It is not misbehaving,
    which is what "exceptionally bad" was authorized against.
-   `#136` set the precedent on `GLM-4.7-Flash` and it holds:
    a seat is not dropped for placing last on a preference measure.

WHAT WOULD CHANGE THIS:
evidence that its candidates dilute judging rather than merely lose,
or that its schema mismatches (4 of the 6 in an earlier run) cost rounds the roster needed.
Neither is measured.
Both are measurable,
and neither is measured yet.

## What this standing is NOT

It is preference evidence only.
`doc/audit/every-volume-guard-is-blind-to-one-model.md` carries the availability evidence,
and the two must not be read as one number.
In particular `qwen3.8-max` places second here while recording 281 zero-content streams,
which is an accounting artifact of its tool-call channel and not a delivery failure.

Neither column is a verdict about model quality in general.
It measures which seat writes text these judges prefer,
on this corpus,
under these house rules,
at this date.
