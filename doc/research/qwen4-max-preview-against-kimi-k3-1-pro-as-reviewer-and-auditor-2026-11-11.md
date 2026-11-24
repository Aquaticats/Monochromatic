# Qwen4 max preview against kimi k3 1 pro as reviewer and auditor 2026 11 11

## Status

Moved on 2026-11-23 from the old handover,
verbatim;
not yet edited into a record of its own.

## Moved from the old handover

From lines 30217 to 30232 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

The qwen review trial,
scored at 19:17 by a Claude Code native subagent
(a second pass by the same vendor as the coordinator,
not an independent review;
`~/temp/agent/qwen-trials/review-trial-scoring.md`).
Three changes,
each reviewed by both models,
each finding held against what the round 227 fix agents confirmed or refuted:
Kimi made 6 findings,
4 confirmed,
1 refuted on a later base than the reviewed commit,
1 mixed;
qwen made 8,
3 confirmed,
none refuted,
5 with no verdict (the fix agents were given Kimi's review only).
Each model had one confirmed finding the other missed
(Kimi:
`push.default=matching` ignores the pinned commit;
qwen:
a relative program path started from the worktree,
fixed in `70b105bf1c`).
Two qwen findings nobody ran look real by reading and are unfixed on the fix branch:
`git commit -m msg --end-of-options -m x` drops the file named `-m` from the commit and exits 0,
and an abbreviated `--only` (`--on`) reaches Git without its paths and fails.
Round 239 (`fix/git-policy-cli-commit-options-end-of-options-and-abbreviated-only`,
on `fix/git-policy-cli-kimi-findings-commit-message-options-round-210-findings`) reproduces and fixes them.
Limits:
three reviews;
the qwen blocked-commits review wrote no `review.md` and was scored from its last message;
two fix agents worked on later bases than the reviewed commits.
On reviews,
unlike on audits,
qwen is not behind Kimi in this sample;
the owner's "Trial beside Kimi" stands,
and more paired reviews would firm it up
(not started:
each qwen review brief is written by hand,
and the earlier ones name an owner,
which briefs no longer do).
