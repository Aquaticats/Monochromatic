# Pi agent concurrency: the limit of 20 and the trial of 2026-11-09

The owner raised the number of Pi subagents that may run at once from 16 to 20 after a trial on 2026-11-09.
This record holds the provider figures and the trial's measurements behind that limit,
so a later trial compares against them instead of re-deriving them.
`.agents/skills/pi-as-subagent/SKILL.md` states the limit itself.

## The provider figures the limit rests on

The owner found that `GET https://api.neuralwatt.com/v1/rate_limits` reports the account's limits.
At 2026-11-09T06:19:41Z, `deepseek-ai/DeepSeek-V4.1-Flash` read `effective_concurrency` 20,
`floor` 15, `flex_floor` 22, `burst_ceiling` 20 and `burst_active` true, on tier `max`.
No published description of those fields was found, so what they mean is read from their names.
The limit counts requests in flight, and an agent spends much of its time running tools.
Anything else using the same key counts against the same limit,
and this host may run short before the provider does.

## What the trial measured

Over the first 28 minutes, with up to twenty agents running,
the open model calls were a median of 7, a 99th percentile of 13 and a highest of 15.
They reached 15 for 0.12% of that time and never reached 20
(`${HOME}/temp/agent/pi-runs/measure-model-calls-in-flight.ts`).
Scaling those figures, which is an estimate, about 26 to 30 agents would reach 20 open calls.

Over 100 one-minute samples, 60 had at least 20 agents running, the fewest 16 and the most 20.
Those 100 minutes saw 4,919 model turns and no rate-limit error.
The errors were 2 Cloudflare 520 responses and 1 request timeout, each ending one run,
which is what the driver job is fixing.

## The baseline and what the count can see

Before the trial, at up to sixteen agents, 158 feeds held 25,631 model turns,
with no retry and no error shaped like a rate limit.
The eight errors they did hold (five aborted operations, three unconfigured providers) show the count can see an error.
A provider error reaches a run as an `assistant` record with `"stop":"error"`,
and Pi's own retry as an `auto_retry_start` record.
A provider that throttles by answering slowly reports no error,
so the watcher also prints the median model turn before and during a trial.
The median model turn was 3937 ms against 3153 ms in the two hours before,
and that difference has not been compared with the run-to-run spread, so it may be noise.

The coordinator's pick, with the veto open, was that the trial passes after sixty minutes with twenty running and no
error shaped like a rate limit; the first such error ends it
(`${HOME}/temp/agent/pi-runs/watch-trial-of-twenty-pi-agents.ts`).

## What the owner said

On how many run at once, the owner said
"Try 20,
and if it doesn't blow up (in terms of api rate limits),
we're permanently raising the practical limit to 20."
On going higher: "no, that's definitely going to blow up."

The trial met no rate-limit error, so the practical limit in the skill is 20.
