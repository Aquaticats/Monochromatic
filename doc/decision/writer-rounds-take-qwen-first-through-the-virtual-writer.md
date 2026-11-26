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
and the driver counts a run on the virtual writer in every pool its router may take
(`package/pi-shared/subagent-driver/src/pool-limits.ts`, `VIRTUAL_POOLS`),
because the router has no single pool to report and moves a request on by itself.
So a virtual writer run takes a slot in each member at once,
and the smallest member's room (Qwen's 5) bounds how many run together.
The plugin's built `dist/` in the main checkout,
which Pi loads,
was rebuilt after the merge.
From 2026-11-26T07:10-05:00 the round launcher takes `virtual`
(`--provider=rate-limit-fallback --model=writer`) by default,
and `WRITER_POOL=qwen`,
`opencode-go` or `neuralwatt-flex` pins one member.
