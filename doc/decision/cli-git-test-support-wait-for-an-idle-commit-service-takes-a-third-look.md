# cli-git test support: the wait for an idle commit service takes a third look

## Status

Answered in the question tool on 2026-11-23,
recorded 2026-11-23T04:17:37-05:00.
It concerns test support only:
no product code changes with it.
The build is in a fix round on issue 6345's branch,
`fix/git-policy-cli-worktree-removal-held-commit-control-failed-in-two-gates`,
and is not merged.

## What was asked

Issue 6345's branch fixes a test helper:
the wait that several hundred wrapper tests use to know the commit service is idle.
Its second integration pass
(`~/temp/agent/integration-second-pass-worktree-removal-held-commit-control-2026-11-22/keep/review.md`,
"The recovery pair in the wait")
found that the wait can still return early in one case no current test reaches:
a claim that expired is recovered,
which moves a record through two folders between the wait's two looks.
The coordinator had already started the fix round with the pass's first-ranked fix and asked whether to keep it.

## What was chosen

The project chose "A:
a third look (Recommended)":
"What the fix round is building now.
Pros:
closes the case found;
tiny cost.
Cons:
4 of 18,365 schedules of four moves are still missed,
which the helper's comment will say."

Not chosen:
"B:
document the gap only",
which left the wait with two looks;
and "C:
a signal from the service",
which had the commit service tell the tests when it is idle.

## Still open from the same pass

The pass put two more points under its heading "For the owner".
One,
where the corrected words go,
is the ordinary procedure and was followed:
a fix round on the branch,
then a third pass.
The other is not asked yet:
what a removal whose wake missed should do
(issue 6354).
The pass ranks saying in the warning which commands finish the removal first,
documenting it as a limit second,
and waking the service from commands a person reads third.
