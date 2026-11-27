# Synthetic infinite small vision as a reviewer 2026 11 19

## Status

Moved on 2026-11-23 from the old handover,
verbatim;
not yet edited into a record of its own.

## Moved from the old handover

From lines 45051 to 45062 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:


**Owner,
2026-11-19,
on a quota-free reviewer:**
"Also check Synthetic's infinite:small:vision for reviewer.
It's the
same model as deepseek v4.1 flash 1017,
just much slower,
but it doesn't drain my quota.
You might need to benchmark it
on a speed basis."
Pi lists it as `synthetic/infinite:small:vision` (context 524.3K,
max output 65.5K,
the same limits
as Synthetic's `hf:deepseek-ai/DeepSeek-V4.1-Flash-1017`).
Benchmark going (unit
`reviewer-speed-infinite-small-vision-against-qwen-20261119`,
script and results in
`~/temp/agent/reviewer-speed-infinite-small-vision-against-qwen-deepseek-2026-11-19/keep/`)
(scratch,
deleted on 2026-11-27;
what this record uses from it is stated here):
one fixed read-only review
task (two `git show` reads at `7d10dc5f88`,
three questions),
four rounds in rotated order of the Qwen-plan DeepSeek arm,
the `infinite:small:vision` arm and a second Qwen arm as the same-model control;
each run's wall time,
turns and output
tokens,
read as ratios to the control arm of its own round.
**Round 1 (2026-11-19,
from the feeds' own offsets):**
the Qwen arm answered in 14.5 s (prompt sent at 3.6 s,
settled
at 18.1 s) and `infinite:small:vision` in 322.7 s,
both in three turns:
about 22 times slower,
one sample,
no spread yet.

From lines 45378 to 45385 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:


**Reviewer speed result (2026-11-19,
four interleaved rounds,
model time from the feeds' prompt-sent to settled,
`keep/runs.jsonl`):**
(scratch,
deleted on 2026-11-27;
what this record uses from it is stated here).
Qwen-plan DeepSeek 9.4,
10.8,
6.3,
17.7 s;
its same-model control 18.9,
13.5,
11.9,
8.3 s (Qwen
over control 0.50 to 2.13,
the run-to-run spread);
`infinite:small:vision` 186.5,
496.4,
413.9,
488.7 s,
which is 13,
41,
46 and 38 times the mean of the two Qwen runs of its round,
well outside that spread.
All twelve answers named the 60 s
bound and `SYNC_DEADLINE_MILLISECONDS` correctly,
so the answers matched and only the speed differed.
At the median
ratio a 150-turn review that takes about 17 minutes on Qwen would take several hours (extrapolation,
not measured).
For the owner:
where,
if anywhere,
it sits in the reviewer pool.
