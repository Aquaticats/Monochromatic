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

The writer's order here is superseded on 2026-11-27 by
"The writer's members become the reviewer's", below.
This section stands as what was decided on 2026-11-26.

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
those members are the reviewer's since 2026-11-27
("The writer's members become the reviewer's", below);
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

## The writer's members become the reviewer's (owner, 2026-11-27)

Recorded 2026-11-27T16:15-05:00.
The owner typed to the coordinating session:
"Change writer's list to match the reviewer's list."

So the writer asks what the reviewer asks, in the reviewer's order:
`qwen-token-plan-individual/deepseek-v4.1-flash-1017`,
then `synthetic/hf:deepseek-ai/DeepSeek-V4.1-Flash-1017`,
then `opencode-go/deepseek-v4.1-flash`,
then `neuralwatt/deepseek-v4.1-flash-flex`.
The member the writer's list lacked was Synthetic's, and it now stands second, where the reviewer has it.
The order of 2026-11-26 ("Qwen, opencode-go, flex (Recommended)") is superseded for the writer
and stands in this record as history;
the reviewer's order is unchanged.
Standard Neuralwatt's DeepSeek lane stays closed to writers,
and the provider named `deepseek` stays no member of either pool.
`package/pi-plugin/rate-limit-fallback/src/model-pools.ts` carries one list, `BOTH_POOLS_MEMBERS`,
that both pools name, so the two orders cannot drift apart,
and `src/writer-order.unit.test.ts` fails if either pool stops asking it.

The limits Pi advertises for a selection are its smallest registered member's,
so this changes what a writer run may carry before it compacts.
Measured on 2026-11-27 over the members Pi 1.6.0 registers on this machine:
qwen's member reports 1,000,000 tokens of window and 384,000 of output,
Synthetic's 524,288 and 65,536,
opencode-go's 1,000,000 and 384,000,
and Neuralwatt's flex lane 1,048,560 and 393,216.
The writer advertises 524,288 tokens of window and 65,536 of output,
where it advertised 1,000,000 and 384,000 before this change,
a window smaller by 475,712 tokens and an output limit smaller by 318,464;
the reviewer advertises 524,288 and 65,536 both before and after,
because Synthetic's member was already the smallest one it asks.

Superseded the same day,
at about 16:46 -05:00:
neither pool advertises its smallest member's pair from then on,
and both advertise their head member's,
1,000,000 tokens of window and 384,000 of output over the four members above.
The next section records the owner's answer,
and the figures above stand as what the pair was between 16:15 and 16:46 on 2026-11-27.

The driver's table needs no change:
`VIRTUAL_WRITER.memberPools` in `package/pi-shared/subagent-driver/src/pool-limits.ts`
names the same three pools as `VIRTUAL_REVIEWER.memberPools`,
because Synthetic's member belongs to no pool of `PROVIDER_POOLS`
and that table's rule lists no member outside a pool.
A launcher's line for a run on the virtual writer names those three pools
and says the run holds no slot in any pool.

### Advisor runs on Neuralwatt's `nw-large` (owner, 2026-11-27)

The same minute,
a round's session held one result of the `advisor` tool
naming provider `neuralwatt` and model `nw-large`,
and told the owner of it, having asked nothing.
The owner wrote:
"nw-large is advisor, and that's fine."

So `neuralwatt/nw-large` is the model the `advisor` tool's reviews run on there,
a reader of another model's work rather than a writer or reviewer member,
and no pool of the driver's table lists it.
The sentence decides that this use of the model needs no change,
not that a writer or reviewer run may name it.

## Skip a member too small for the request, and advertise the head's window (owner, 2026-11-27)

Recorded 2026-11-27T16:46-05:00.
The owner answered with the limits of "The writer's members become the reviewer's" in view,
asked how a writer could keep asking Synthetic's member
without lowering what a writer run may carry.
Three options were put:

- "Skip too-small member (Recommended)".
- "Keep three members",
  which drops the change and leaves the writer asking three.
- "Live as built",
  which leaves every writer round on the smaller window.

The owner chose the first,
in its words:

> Keep the shared four-member list,
> but change the router so a member whose own window cannot hold the request is passed over,
> and the pool advertises the head member's window.
> Pro:
> writers get Synthetic as second choice
> and keep 1,000,000 tokens;
> the reviewer's window rises to 1,000,000 as well.
> Con:
> it is not built;
> it needs one more round on the router,
> which must estimate a request's size before sending,
> and a wrong estimate ends in the member's own rejection,
> after which Pi compacts and retries once (existing behaviour).
> Until that round lands the writer keeps its three members.

The option carried two examples,
which are the behaviour asked for:
Qwen refuses a 600,000-token request,
Synthetic (524,288) is passed over as too small,
and `opencode-go` (1,000,000) answers;
Qwen refuses a 200,000-token request,
and Synthetic answers.
The first of those examples is also the largest request either pool may now send,
1,000,000 tokens of window being what a writer or reviewer may fill before Pi compacts.

### What each pool advertises now

Both pools advertise the pair their head member reports,
read from the session registry whenever Pi asks for the catalog:
`rate-limit-fallback/writer` and `rate-limit-fallback/reviewer` carry 1,000,000 tokens of window
and 384,000 of output with the four members above registered,
which is qwen's member's pair.

The head is the first member of `BOTH_POOLS_MEMBERS` that the session's registry resolves
with a window and an output limit both above zero.
That is the first member a request is asked of,
because dispatch skips a member the registry does not resolve,
so the two readings agree by construction.
A session where qwen's member is not registered advertises Synthetic's 524,288 and 65,536,
and a member further down the list never lowers the pair on its own.

`src/model-pools.ts` computes it in `headMemberLimits`,
and `src/entry-and-status.unit.test.ts` and `src/virtual-models.unit.test.ts` prove the head's pair
with a smaller pair behind it,
so an advertisement of the smallest member's limits fails there.

### What the router knows of a request's size

Pi hands a provider the normalized transcript and no count of it,
so the router counts with Pi's own estimator,
`estimateContextTokens` from `@earendil-works/pi-ai` (`dist/utils/estimate.js`):
the last answer's reported usage plus 3.5 characters per token for every message after it,
and 3.5 characters per token for the whole transcript until an answer reports a usage.
Pi takes its compaction threshold and each request's output cap from the same function,
`clampMaxTokensToContext` (`dist/api/simple-options.js`),
so the count the pool compares with a member's window is the count Pi already acts on.
`src/request-size.ts` holds it,
and `src/request-size.unit.test.ts` pins it,
including that a failed answer is no anchor and a completed one is.

Measured on 2026-11-27 over 200 of the driver's runs,
each run's `summary.json` naming its `sessionStats.sessionFile`:
48,805 requests,
each request's estimate against the prompt tokens its member reported for it
(`input + cacheRead + cacheWrite`).
In the 35,773 requests of sessions Pi never compacted or edited,
98.0% of the estimates stood within 1% of the reported count
and 99.4% within 5%;
the fifth percentile was 0.999 of the reported count and the ninety-fifth 1.002.
The members that reported were qwen's `deepseek-v4.1-flash-1017` (13,158 requests),
NeuralWatt's flex lane (12,182),
NeuralWatt's standard lane (12,064),
`opencode-go/deepseek-v4.1-flash` (10,736),
Synthetic's `hf:deepseek-ai/DeepSeek-V4.1-Flash-1017` (662)
and `openai/gpt-6.2-sol` (3).
Of those untouched requests,
177 estimated 10% or more above their member's count and 9 that far below.

### No margin between the estimate and a member's window

The router keeps none,
and the measurement above is why:
a margin of the size the estimate errs by,
0.1% to 0.2% of the count,
is about a thousandth of a member's window,
so it would pass over members that can hold a request without buying protection.
A wrong estimate ends in the member's own rejection,
which the owner's option accepted
and which Pi already recovers from under a virtual selection:
it compacts and retries the turn once,
and `mise run //package/pi-plugin/rate-limit-fallback:verify:extension` shows that arm.

The one place a margin would matter is a request whose estimate is wrong by more than a member's
window:
among the untouched requests,
0.6% stood 5% or more away from the reported count,
and the largest and smallest of those are requests whose transcript the session's own history does
not reproduce.

### The edge where the input fits and the output limit does not

A member's window holds the request's input and the answer together,
and the router compares only the input:
Pi fits the answer into what the window leaves,
through `clampMaxTokensToContext` in the member's own adapter,
and a request whose input fits is asked of that member
even when the output limit it carries would not fit beside it.
At a 524,288-token member with its 65,536-token output limit,
an input of 509,317 tokens is asked there:
input plus that limit is 574,853,
above the window,
and Pi's clamp leaves 10,875 tokens of answer,
so the pair sent comes to 520,192.
`src/too-small-member.unit.test.ts` holds that edge with both controls,
and with Pi's own clamp fitting the pair.

### What a passed-over member costs

Nothing is remembered:
the decision is made per request,
from that request's own size and the window the member reports,
and the next request asks the member again.
A member held after an undelivered request is still asked last when nothing else can be tried,
because that hold is the pool's guess;
a member whose window cannot hold the request is not,
because asking it could only end in its own rejection.

### What stays decided

Standard Neuralwatt's DeepSeek lane stays closed to writers,
the provider named `deepseek` stays no member of either pool,
and a run on either virtual selection holds no slot in any member pool.

### The round that built it

Branch `feat/pi-plugin-rate-limit-fallback-the-router-passes-over-a-member-too-small-for-the-request-and-the-pool-advertises-its-heads-window`,
over the branch that carried "The writer's members become the reviewer's",
unapproved and non-final.
