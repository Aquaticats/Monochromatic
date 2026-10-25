# Judge effort bench for Haiku 5.5: the outcome

This file records what the judge effort bench found for Haiku 5.5,
so that a reader of the [probe README](../README.md) can see the result without the logs,
which stayed outside the repository.
The bench asked whether Haiku 5.5 judges better or worse at the efforts low,
medium and high than the card that sends no effort,
which takes the model's own default.
The card notes in `shared-contract/src/claude-card-sides.ts` name that default as `medium`.
It ran on 2026-10-23 from 13:05 to 13:09 UTC.

## What the bench ran

Each arm ran `judge-fidelity-probe` with `--cap 28`,
`--candidates` naming the arm's card alone and `--candidates-alone`,
from one frozen bundle built at `98395c8555`.
Every arm printed the same pipeline digest.
Only the Claude key was set,
so the unconfigured lines name DeepSeek and Xiaomi and nothing else.
There were four arms:
two of `claude-haiku-5-5-effort-medium`
(medium a and medium b,
the second as a control for the spread),
one of `claude-haiku-5-5-effort-low` and one of `claude-haiku-5-5-effort-high`.

## Validity

Every arm passed the checks the bench plan set before any arm ran:

- The unit ended with `status=0/SUCCESS`,
  named one judge (the arm's card)
  and printed no `partial reviewed matrix` line and no `[error]` line.
- It logged 28 rows,
  none without a ballot,
  with 14 `SPEND provider=anthropic` lines for the arm's card and 14 `PROMPT-REUSE` lines.
- Its seat line read `asked=28 usable=28 unusable=0 threw=0 silent=0 abandoned=0 unreachable=0`.

## Scores and spend

- Medium a:
  clean 14 of 14,
  0.005330 USD,
  completion tokens 230 to 567 per call.
- Medium b:
  clean 14 of 14,
  0.004418 USD,
  completion tokens 184 to 576 per call.
- Low:
  clean 14 of 14,
  0.003906 USD,
  completion tokens 156 to 422 per call.
- High:
  clean 14 of 14,
  0.005045 USD,
  completion tokens 325 to 709 per call.

The two medium arms differed on 0 of 14 judgements,
so the noise threshold was 1 flip
(the larger of the measured spread and 1).
The four arms spent 0.018699 USD together.

## What the outcome means

Every effort reached the ceiling of 14 of 14,
so this sample cannot separate low,
medium and high.
That is a ceiling effect:
it does not show the efforts judge alike on harder work.
Low cost 0.000968 USD less than the mean of the two medium arms over the 14 calls.

The lead's ruling,
open to the owner's veto,
keeps Haiku 5.5 on the card without an effort parameter.
The bench could not separate the efforts,
and under the owner's rule that quality comes before cost,
a lower price alone does not justify a level no measurement has backed.
The three effort cards stay in the catalogue for a harder instrument.

The bench did not measure Sonnet 5.5's effort as an editor.
The [editor fidelity bench](editor-fidelity-bench-outcome.md) measured that later,
and no effort was shown better than medium.
