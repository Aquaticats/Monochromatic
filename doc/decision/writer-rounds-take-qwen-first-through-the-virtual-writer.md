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

The round launcher (`~/temp/agent/zfs-open-item-design-rounds-launch-2026-11-22/writer-pool-for-a-round.ts`)
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
and the driver counted a run on the virtual writer in every pool its router may take
(`package/pi-shared/subagent-driver/src/pool-limits.ts`, `VIRTUAL_POOLS`),
because the router has no single pool to report and moves a request on by itself.
So a virtual writer run then took a slot in each member at once,
and the smallest member's room (Qwen's 5) bounded how many ran together.
Since 2026-11-26T18:06:16-05:00 the driver counts such a run once against the room its open members have together
(commit `c512a9f9f4`,
unapproved and non-final;
the section "The virtual writer is not capped at its smallest member" records the change):
the owner,
reading that five rounds ran at once,
said "Just because the 1st pool is 5 at a time doesn't mean you gotta limit yourself.",
so that room together (45 with the limits recorded here) bounds these runs,
and a member's own room bounds only the runs pinned to it.
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
which counts such a run once against the room its open members have together
(round `~/temp/agent/subagent-driver-counts-a-virtual-writer-run-once-against-its-members-combined-room-2026-11-26/`).
The count that decides now is the runs on a selection's members,
which are the runs on the selection itself and the runs pinned to one member,
against that room together:
45 with the limits recorded here (5 + 20 + 20),
while a closed member adds no room and the runs already on it still count.
A member's own room bounds only the runs pinned to it,
and a run pinned to one pool both needs that room and leaves the members' room together a slot,
so a run that names no model of its own may be refused once the members together are full,
although its own pool has room left.
The reviewer selection follows the same rule.
The tallies the driver prints carry one entry per pool,
counting the runs pinned to it,
and one per selection,
as `rate-limit-fallback/writer 2/45 together`,
and a refusal says how full the members are together as well as what each member reads.
