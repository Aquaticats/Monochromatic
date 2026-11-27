# Writer rounds take Qwen first, through Pi's virtual writer

Recorded 2026-11-26T03:00-05:00.
The owner decided this on 2026-11-26,
answering two questions put through the question tool.

## Why it was asked

OpenCode Go refused every request with its monthly usage limit from about 02:35,
and the three writer rounds pinned to it ended mid-work.
The owner asked:
"What? Don't you have a virtual writer model in Pi? And what of my Qwen quota?"

Both were in the record and unused by writers.
Pi has a virtual `writer` model,
`rate-limit-fallback/writer` (`package/pi-plugin/rate-limit-fallback/README.md`),
which moves a request to its next member on a 429 that comes before any output;
the round launchers named one pool each instead,
and the virtual writer's members still began with standard Neuralwatt,
which the owner closed (`.claude/skills/pi-as-subagent/SKILL.md`, "Who writes and who reviews").
The owner had put Qwen in place of standard Neuralwatt on 2026-11-11
("Please replace the neuralwatt standard deepseek pool with qwen-token-plan-individual  deepseek-v4.1-flash-1017"),
removed it on 2026-11-13,
and added it back on 2026-11-18
("Add my qwen individual token plan deepseek-v4.1-flash-1017 to the deepseek pool." and
"Orderings: qwen individual token plan first."),
which the driver's pool table read as for short reviews only
(`package/pi-shared/subagent-driver/src/pool-limits.ts`, `QWEN_PLAN_DEEPSEEK`, `takesDefaultRuns: false`).

## The writers' order

Asked in what order writers take the pools,
the owner answered "Qwen, opencode-go, flex (Recommended)":
`qwen-token-plan-individual/deepseek-v4.1-flash-1017`,
then `opencode-go/deepseek-v4.1-flash`,
then `neuralwatt/deepseek-v4.1-flash-flex`.
Standard Neuralwatt stays closed,
and the provider `deepseek` is never used.
Qwen's room is 5 runs at once,
the figure the pool table took from another model on the same plan,
until a run measures this model's.

## The virtual writer

Asked whether rounds start on the virtual writer,
which moves a refused request on by itself,
the owner answered "Use the virtual writer (Recommended)".
So its writer members become the owner's order with standard Neuralwatt removed,
its README's open check (the live check of the quota read's deadline) runs before rounds use it,
the driver counts a run against the pool the router actually picked,
and the launchers start rounds on `--provider=rate-limit-fallback --model=writer`.

## Until the virtual writer is ready

The round launcher (`~/temp/agent/zfs-open-item-design-rounds-launch-2026-11-22/writer-pool-for-a-round.ts`;
scratch,
deleted on 2026-11-27;
not kept)
takes `qwen` by default from 2026-11-26T02:50-05:00,
and a launcher names `WRITER_POOL=opencode-go` or `WRITER_POOL=neuralwatt-flex` for the next pool,
so a refused round is restarted on the next pool by hand,
as three were on the night this was decided.

## The virtual writer is ready

Merged as `7c82b27ad0` (2026-11-26T07:09:30-05:00):
the virtual writer's members are the owner's order with standard Neuralwatt removed
(`package/pi-plugin/rate-limit-fallback/src/model-pools.ts`, guarded by `src/writer-order.unit.test.ts`);
the README's open check runs as the task `verify:quota-deadline`,
whose record is `package/pi-plugin/rate-limit-fallback/doc/quota-read-latency-records/`;
and the driver counts a run on the virtual writer in no pool at all
(`package/pi-shared/subagent-driver/src/pool-limits.ts`, `VIRTUAL_POOLS`).
The owner,
2026-11-26,
asked how the driver's pool gate counts such a run,
answered "No count (Recommended)",
because the router has no single pool to report and moves a request on by itself.
So a run on `rate-limit-fallback/writer`,
and on the reviewer selection the same way,
holds no slot in any member pool,
nothing about the members' room bounds how many run together,
and the gate counts only the runs pinned to one pool.
That is commit `7acfa7e5b8`
(2026-11-26T19:16:14-05:00,
unapproved and non-final).
Before it,
this record's merge `7c82b27ad0` had the driver count such a run in every member pool,
which a change of 2026-11-26T18:06:16-05:00 (commit `c512a9f9f4`) replaced
with a count once against the room the open members have together.
The section "The virtual writer is not capped at its smallest member" records all three states.
The plugin's built `dist/` in the main checkout,
which Pi loads,
was rebuilt after the merge.
From 2026-11-26T07:10-05:00 the round launcher takes `virtual`
(`--provider=rate-limit-fallback --model=writer`) by default,
and `WRITER_POOL=qwen`,
`opencode-go` or `neuralwatt-flex` pins one member.

## The virtual writer is not capped at its smallest member (owner, 2026-11-26)

At about 17:30 -05:00 the owner read that five rounds ran at once because Qwen's room is 5 and answered:
"Just because the 1st pool is 5 at a time doesn't mean you gotta limit yourself."
Two things had held the count at five:
the coordinator's round starters waited for fewer than five virtual-writer drivers,
and the driver counts a virtual-writer run as a slot in every member pool (`VIRTUAL_POOLS`),
so the smallest member bounds them all and each run also holds opencode-go's and flex's room it is not using.
The coordinator's starters had held that count themselves:
the round launcher calls `drive-pi-subagent.ts` directly,
which refuses only a closed pool and checks no pool's room (its own header, lines 5 and 6),
so the driver's count never reached those rounds.
From 2026-11-26T17:50-05:00 every coordinator round starts on the virtual writer,
pinned to no member,
under the owner's "Up to 64 rounds at once." alone.
The coordinator had also spaced starts three minutes apart behind a memory test,
which the owner never set; the owner, 2026-11-26: "I didn't tell you to apply the 3 minutes apart and memory test rules."
From 2026-11-26T17:45-05:00 every queued round was started at once, up to the 64.
A first try pinned the overflow to opencode-go,
and both rounds it started ended on their first request with opencode-go's
`429: {"type":"GoUsageLimitError"` refusal of its spent monthly quota,
which the virtual writer moves on to flex because it comes before any output
(`package/pi-plugin/rate-limit-fallback/README.md`).
Starts through the driver's pool gate
(`start-pi-run-when-a-pool-has-room.ts`,
the batch list and the retrying start)
counted a virtual-writer run as a slot in every member and so stopped at Qwen's 5
until commit `c512a9f9f4`
(2026-11-26T18:06:16-05:00,
unapproved and non-final),
which counted such a run once against the room its open members have together.
The owner then answered the count itself,
asked how the driver's pool gate counts a run on the virtual writer:
"No count (Recommended)".
Since commit `7acfa7e5b8`
(2026-11-26T19:16:14-05:00,
unapproved and non-final)
the gate counts only the runs pinned to one pool:
a run on the virtual writer,
and on the reviewer selection the same way,
holds no slot in any member pool,
so no pool's room decides whether such a run may start.
A pool's own room bounds only the runs pinned to it,
and a run on a virtual selection neither fills a pool nor keeps one from taking a pinned run.
The tallies the driver prints count the runs pinned to each pool and name no selection,
and the launcher's line for a run on a selection names its members and says it holds no slot in any pool.
A member a caller closes with `PI_RUNS_POOL_LIMITS` adds no room to any count and does not by itself stop such a run:
the provider's refusal at that member is what moves the request on.
(round `~/temp/agent/subagent-driver-counts-a-virtual-writer-run-once-against-its-members-combined-room-2026-11-26/`;
scratch,
deleted on 2026-11-27;
what this record uses from it is stated here).

## The pool gate counts no virtual run (owner, 2026-11-26)

Asked at about 19:10 -05:00 how the driver's pool gate counts a run on the virtual writer
(no count, once against the members' combined room, or a slot in every member as before),
the owner answered "No count (Recommended)":
the gate counts only runs pinned to one pool,
and a run on the virtual writer or the virtual reviewer holds no slot in any member,
because the router moves a refused request on by itself.
The driver round that was building the combined-room count was told so through its inbox the same minute,
since that count was the coordinator's design and not the owner's.
