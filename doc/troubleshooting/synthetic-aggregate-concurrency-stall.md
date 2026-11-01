# Synthetic aggregate-concurrency stall

Fact base for a support report to [Synthetic][].
All observations were collected on 2026-07-16 and 2026-07-17 (UTC)
against `https://api.synthetic.new/openai/v1/chat/completions`.
We are grateful for the service and for the flat-rate plan;
this document exists because we would like help understanding one behavior,
not because we believe anything is owed.
Everything below is stated as measured,
with client-side factors we could not rule out listed separately
so that nothing here overclaims.

## Resolution update on 2026-08-30

The stall did not reproduce against the current account and active model roster
with short structured requests.
Two zero-retry arms each completed 20 of 20 concurrent calls,
5 on every active model,
in 4,113 ms and 3,943 ms.
Every response was HTTP 200 and carried an exact schema-valid answer.

A width-25 arm kept three models at 5 and raised `gpt-oss-120b` to 10.
The gpt-oss backend returned 7 HTTP 200 responses and 3 HTTP 429 responses,
while every width-5 model completed.
The current safe shared value is therefore 5 per model,
not the production default of 1 and not a model-size-based 10.

The original July evidence remains valid for its account state,
roster,
and real review workload.
The new result resolves the immediate assumption that aggregate load above roughly 7 always collapses.
It does not prove that 20 corpus-sized generations have the same behavior.
Method,
current results,
source impact,
and evidence digests are recorded in
`doc/troubleshooting/translation-repair-provider-concurrency.md`.

## Summary

With more than one concurrent request per model
(aggregate load above roughly seven streams from one client),
requests to most model backends stall:
they are accepted,
return no error status,
and produce no completion within a 5 minute client cap.
The same requests complete in seconds to about a minute
when sent one-per-model.
Both `zai-org` models keep completing under load (slowed 2x to 4x)
while every other vendor's backend stalls,
which is why this looks like prioritization or queuing rather than
client-side networking.
We would like to know whether this is expected plan behavior,
and if so,
how the documented per-pack concurrency is intended to be used.

## Account and plan context

- Subscription:
  5.5 pack-equivalents throughout the observation window
  (2026-07-16 through 2026-07-17 04:15 UTC):
  one founder's pack (worth 1.5 normal packs)
  plus 4 normal packs purchased on 2026-07-16.
  The live `/v2/quotas` five-hour ceiling moved from 750 to 2750 with
  that purchase,
  consistent with 1.5 plus 4 pack-equivalents at 500 each.
- Quota pressure is not a factor:
  immediately before the largest test the five-hour block read
  2748.5 of 2750 remaining,
  and weekly credits were similarly untouched.
- Documented concurrency expectation
  (from the public plan description as we understand it):
  one concurrent request per model per subscribed pack,
  different models fully parallel,
  same-model excess queued server-side.
  With 5.5 pack-equivalents we expected about 5 concurrent requests per
  model to be servable.

## Client environment

- Node.js 26.5.0 on Linux (single host,
  single process per test run).
- Requests issued through the global `fetch`;
  the negotiated transport to this origin is HTTP/2
  (evidenced by `ERR_HTTP2_STREAM_ERROR` from `node:internal/http2`
  during one mass teardown,
  noted under client-side factors).
- All chat calls stream (`stream: true`) and are drained to completion;
  transient statuses (408,
  429,
  5xx) are retried with jittered
  exponential backoff.
- No proxy or VPN on our side;
  one source IP.

## Request shape

Every observation below uses the same request template:

```jsonc
// POST /openai/v1/chat/completions
{
  "model": "<one of the seven model ids below>",
  "messages": [ /* fixed instruction prompt plus two documents */ ],
  "stream": true,
  "stream_options": { "include_usage": true },
  "response_format": { "type": "json_schema", "json_schema": { /* ... */ } }
  // no temperature, no reasoning effort, no max_tokens
}
```

Models exercised:

- `hf:zai-org/GLM-5.2`
- `hf:zai-org/GLM-4.7-Flash`
- `hf:Qwen/Qwen3.6-27B`
- `hf:moonshotai/Kimi-K2.7-Code`
- `hf:MiniMaxAI/MiniMax-M3`
- `hf:nvidia/NVIDIA-Nemotron-3-Super-120B-A12B-NVFP4`
- `hf:openai/gpt-oss-120b`

The two documents in each prompt are a Chinese source text and its
English translation,
0.7 KB to 2.6 KB per side for the controlled sweep below.

## Healthy baselines (same account, same client)

- 2026-07-16,
  three concurrent tiny calls to one model
  (`hf:zai-org/GLM-4.7-Flash`):
  all three fully overlapped and completed in 2.0 s to 2.4 s each,
  so same-model concurrency was genuinely served at small scale.
- 2026-07-16,
  fourteen concurrent streams
  (two per model across all seven models,
  real review workload):
  completed normally over several minutes with zero 5xx.
- 2026-07-17 around 03:20 UTC,
  seven concurrent streams
  (one per model,
  one 9.3 KB document pair):
  all seven completed in 308 s wall,
  the fastest such pass we had recorded.

## Observation 1: burst of 35 dispatches draws instant 502s

2026-07-17,
about 03:52 UTC.
35 tiny prompts (5 per model,
answer expected to be one character)
dispatched simultaneously:

- 27 of 35 first attempts returned HTTP 502 within roughly 3 seconds
  of dispatch (03:52:29 to 03:52:32).
- Backoff retries recovered 21 of the 35 calls overall.
- Completions were extremely slow for one-character answers:
  78 s to 119 s.
- 14 of 35 calls produced nothing within a 120 s cap.

The instant,
clustered 502s look like an edge/burst gate;
they are absorbed fine by retries and are not our main question.

## Observation 2: controlled sweep, silent stalls above one-per-model

2026-07-17,
roughly 03:56 to 04:12 UTC.
Identical review calls over the same five small document pairs,
swept across per-model concurrency levels 1,
2,
3,
5,
run sequentially with a fresh client per level.
Per-exchange client cap:
300 s.
Crucially,
the sweep produced zero 5xx and zero transport errors at every level:
stalled calls were accepted and simply never completed.

Level 1 (7 calls,
one per model):
7 of 7 completed,
71 s wall.

- `MiniMax-M3` 3 s,
  `gpt-oss-120b` 6 s,
  `Qwen3.6-27B` 35 s,
  `Kimi-K2.7-Code` 39 s,
  `GLM-4.7-Flash` 50 s,
  `GLM-5.2` 71 s,
  `Nemotron` 71 s.

Level 2 (14 calls,
two per model):
4 of 14 completed.

- Completed:
  `GLM-5.2` 102 s and 160 s,
  `GLM-4.7-Flash` 216 s and 278 s.
- All ten calls to the other five vendors hit the 300 s cap,
  including `MiniMax-M3` on the exact document it had answered in 3 s
  five minutes earlier.

Level 3 (21 calls,
three per model):
5 of 21 completed.

- Completed:
  `GLM-5.2` 131 s,
  161 s,
  189 s;
  `GLM-4.7-Flash` 245 s,
  293 s.
- The other sixteen calls hit the 300 s cap.

Level 5 (35 calls,
five per model):
3 of 35 completed.

- Completed:
  `GLM-5.2` 83 s,
  120 s,
  202 s.
- Even `GLM-4.7-Flash` no longer completed;
  the other thirty-two calls hit the 300 s cap.

Effective throughput by level:
5.9 completions per minute at level 1,
0.8 at level 2,
1.0 at level 3,
0.6 at level 5.

## The pattern we cannot explain

- Above one concurrent request per model,
  most vendors' backends stop responding entirely for this account:
  no error status,
  no stream data,
  no completion in 300 s.
- The degradation is vendor-asymmetric:
  `zai-org` models keep completing at every level (progressively slower),
  `GLM-4.7-Flash` drops out at level 5,
  and no other vendor completes anything at level 2 or above.
- The collapse is immediate and reversible:
  level 1 ran perfectly in the same process minutes before level 2
  collapsed,
  and level 1 performance had also been normal earlier in the day.
- Documented per-pack concurrency suggests 5 concurrent per model should
  queue at worst,
  not stall to zero output.

## Client-side factors we considered

Listed so the report does not overclaim.

- One earlier large run (2026-07-17 around 03:39 to 03:47 UTC,
  126 dispatches) failed almost entirely on a client-side bug of ours:
  fan-out deadlines armed at dispatch time starved queued calls.
  That run is excluded as evidence;
  its dispatches did reach the API,
  so it may be visible on your side around those timestamps.
- All streams from one process share what appears to be one HTTP/2
  session.
  We considered client-side session limits or flow-control exhaustion,
  but during the stalled levels the `zai-org` streams kept flowing on
  the same connection while other vendors' streams stayed silent,
  and level 1 on the identical stack was fast,
  which does not match a client-side head-of-line explanation.
- During one mass teardown of aborted streams the process observed
  `ERR_HTTP2_STREAM_ERROR` (`NGHTTP2_PROTOCOL_ERROR`) from
  `node:internal/http2`.
  Later levels of the sweep may additionally have been affected by
  earlier levels' aborted calls still generating server-side;
  this cannot explain level 2,
  which followed a fully clean level 1.
- Client caps in the sweep were generous relative to healthy latencies:
  300 s per exchange versus 3 s to 71 s healthy completions on the same
  documents.

## Questions for Synthetic

1. Is per-account aggregate concurrency (across different models)
   limited or deprioritized in a way that is separate from the
   documented per-model-per-pack concurrency?
2. Is the vendor asymmetry expected,
   for example different queuing policies per upstream GPU provider?
3. Are the instant clustered 502s on large simultaneous dispatch an
   intentional burst gate,
   and if so,
   what dispatch pacing do you recommend?
4. Do client-aborted streaming requests keep consuming account
   concurrency server-side after the abort,
   and for how long?
5. Is there anything about this account's state on 2026-07-17 between
   03:39 and 04:15 UTC (for example an automatic penalty triggered by
   the earlier burst) that would explain the collapse?

## What we can provide on request

- Exact per-call dispatch and completion timestamps for every
  observation above.
- Full request bodies (prompts are non-sensitive review instructions
  plus public memorial-site document pairs).
- Raw per-level results of the controlled sweep as JSON.
- A minimal reproduction script
  (plain `fetch` against the OpenAI-compatible endpoint).

Thank you for reading this far,
and thank you for running a service that makes seven-model ensembles
affordable in the first place.
We are happy to rerun any of these measurements with different
parameters if that helps.

[Synthetic]: https://synthetic.new
