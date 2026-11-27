# Pi driver: closed pools, the Qwen plan's limit, and the worktree launcher's default

## Status

Answered by the project in the question tool on 2026-11-23,
after the driver's closed-pool refusal merged (`65a1dc0e78`) and its follow-ups merged (`cdc21b3661`).
The options are the stopped pass's
(`~/temp/agent/integration-pass-pi-driver-refuses-a-closed-pool-2026-11-23/keep/review.md`
(scratch,
deleted on 2026-11-27;
what this record uses from it is stated here)),
copied into the follow-up round's report.
The package is `package/pi-shared/subagent-driver`.

## The Qwen token plan's DeepSeek pool

The project chose "Keep 5 (Recommended)":
"As landed.
Cost:
the two pools of 5 on one plan may together pass the provider's advice of six to eight."
Only a run that names that pool goes there;
a run naming no model goes to opencode-go,
then Neuralwatt flex.
Not chosen:
"Measure first",
and "Lower to 3".

## A worktree launch that names no provider

The project chose "Take the first open pool":
"It still starts.
Cost:
the run isn't counted against the pool's limit."
So `start-pi-run-in-a-worktree` with no provider takes the first open pool in the owner's order
(opencode-go,
then Neuralwatt flex),
and never a closed pool,
in place of the refusal that landed with `65a1dc0e78`.
Counting the run against that pool's limit removes the stated cost,
and is done where the launcher can do it.
Not chosen:
"Refuse and point at the gate (Recommended)",
and "Keep the old default,
warn".

## How a reopened pool reaches a run in its own unit

The project chose "Forward the setting (Recommended)":
"As landed:
one more --setenv word for the unit.
Cost:
the setting rules that run for its whole life."
Not chosen:
"A 'pool checked' flag",
and "Document the limit only".
