# Opencode go pool answer times against neuralwatt flex 2026 11 16

## Status

Moved on 2026-11-23 from the old handover,
verbatim;
not yet edited into a record of its own.

## Moved from the old handover

From lines 46364 to 46366 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

- **First timing of the `opencode-go` pool against the flex pool,
  same window,
  from the going runs' feeds (numbers alone;
  window 15:07:31 to 15:22:55):
  the model's time for one answer is about a twelfth to a twentieth of flex's.**
  Two runs on `opencode-go` (`deepseek-v4.1-flash`):
  medians 3,034 ms and 5,492 ms over 36 and 99 answers,
  55 to 58 output tokens a second by median.
  Eleven runs on flex (`deepseek-v4.1-flash-flex`):
  medians 57,612 ms to 71,746 ms over 3 to 18 answers each,
  4 to 15 output tokens a second;
  the flex medians sit near a minute whatever the answer's length,
  which reads as a wait before the answer and not as slow writing.
  The spread among the flex runs (57.6 s to 71.7 s) is far smaller than the difference.
  The pool's wall factor stays 2 for now,
  and that is a choice with a reason:
  this reading has one and then two runs on a pool of twenty places,
  and says nothing of its speed with thirteen going or near its weekly limit.
  It is taken again when five or more runs go there,
  and the factor is set from that reading.
  Script and output:
  `~/temp/agent/opencode-go-pool-reopened-on-the-owners-word-2026-11-16/` (`print-the-model-time-per-turn-of-each-run-whose-feed-was-written-lately.ts`,
  `keep/model-time-per-answer-of-the-going-runs-first-reading.txt`).
  The second run there is the checkout-migration author's sixth,
  a session that ran on flex before:
  a resumed session moved to the new pool and takes turns.

From lines 46428 to 46429 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

- **Second timing of the two pools,
  now with eight runs on `opencode-go` (window 16:35:00 to 17:05:33,
  feeds of the going runs,
  numbers alone):
  `opencode-go` medians 4,232 ms to 7,306 ms an answer over eight runs (23 to 172 answers each;
  47 to 89 output tokens a second),
  flex medians 39,811 ms to 62,554 ms over six runs.**
  The first reading had one and two runs there (3,034 ms and 5,492 ms),
  so the pool's time for an answer did not grow with eight going.
  On these two readings the pool needs no doubled wall;
  its factor goes from 2 to 1,
  with both readings written beside the value,
  as a follow-up on top of the pool-limits answer once that is reviewed and in `main`,
  since both change `pool-limits.ts`.
  Output:
  `~/temp/agent/opencode-go-pool-reopened-on-the-owners-word-2026-11-16/keep/model-time-per-answer-of-the-going-runs-second-reading-with-eight-runs-on-the-pool.txt`.
  At 17:05:32,
  13 Pi units were active:
  eight on `opencode-go` and,
  by the counts line,
  six on flex,
  one of which had just ended.
