# Translation repair provider concurrency on 2026-08-30

## Resolution

The owner accepted the measured settings as production safe and authorized their application.
Live probes on Node.js 26.7.0 established the measured basis:

- Synthetic completed width 5 for each active model.
  Two aggregate arms completed all 20 requests,
  with 5 requests on each model and no retry,
  non-200 status,
  invalid answer,
  refusal,
  or transport error.
- Synthetic did not demonstrate a general width of 10.
  An aggregate arm with `gpt-oss-120b` at width 10 returned 7 HTTP 200 responses and 3 HTTP 429 responses.
  A Qwen width-10 arm also encountered HTTP 429.
- Hyper served 64 concurrent requests to `deepseek-v4-flash-0731`.
  All 64 returned exact schema-valid answers in 2,147 ms,
  with no retry,
  non-200 status,
  refusal,
  or transport error.

Synthetic production concurrency is 5 per active model,
with an aggregate width of 20 across the current four-model roster.
Hyper has no provider concurrency ceiling by owner confirmation.
Its width-64 result for short structured calls to `deepseek-v4-flash-0731`
provides a live lower-bound check rather than defining an artificial local cap.
Hyper request rate remains a separate limit from in-flight concurrency.
The owner-supplied account limit is 1,000 requests per hour;
the probe did not drive the account to that boundary.

Production used 5 Synthetic slots per model from 2026-08-30 to 2026-10-19
and leaves Hyper per-model concurrency unbounded locally.
The results do not authorize running dependency-bound pipeline nodes together.

## Synthetic removed its five-hour and concurrency limits (2026-10-19)

The owner reported on 2026-10-19 that Synthetic removed its five-hour limit and its parallelism limits.
Synthetic's [rate-limit description][synthetic-rate-limits],
read the same day,
describes a weekly credit limit alone,
of which 2% comes back every 202 minutes,
and no five-hour request limit or concurrency limit.
A meter reading at 14:35 UTC (`run:budget-sample`) found the `/quotas` body holding
`subscription`,
`search`,
`freeToolCalls`
and `weeklyTokenLimit`,
and no `rollingFiveHourLimit`.

So the measurements of 2026-08-30 in this document describe limits that no longer exist,
and they stay here as that record.
Since `859954aa47` Synthetic has no per-model ceiling in the client or the router,
as Hyper has none,
and its meter reads the weekly budget alone when the reply carries no five-hour window.
An HTTP 429 from Synthetic is still read as a refusal and holds the provider out;
no width beyond 5 has been measured since the change.

## Affected surface

The applied production limits live at these source boundaries:

- `SYNTHETIC_PER_MODEL_CONCURRENCY` in
  `package/translation-repair/provider/src/synthetic-client.ts` is positive infinity
  (5 until 2026-10-19).
- `createRoutingClient` in `package/translation-repair/provider/src/provider-router.ts`
  imports that same value rather than duplicating it.
- `createRunClient` in `package/translation-repair/run/src/corpus-run/run-config.ts`
  inherits client defaults.
- `HYPER_PER_MODEL_CONCURRENCY` in
  `package/translation-repair/provider/src/hyper-client.ts` is positive infinity.
- `package/translation-repair/corpus-pass/src/corpus-run/pass-overlap.ts:9` keeps entry overlap at 1.
- `package/translation-repair/slice-cache/src/corpus-run/entry-attempt-queue.ts:90` deliberately serializes entries.

The provider clients are reusable transport boundaries.
The scheduler must separately model:

- dependency readiness
- per-model in-flight slots
- provider aggregate in-flight slots
- provider request-rate budget
- account quota or credit availability

A single `p-limit` value cannot represent all of those constraints.

## Probe method

Prototype commits `557e9fd8e`,
`c1c9aaa78`,
and `a40d33375` contain the probe implementations used across the measurement sequence.
Records were emitted from 10:56 to 11:14 UTC on 2026-08-30.
A budget sample at 10:51 UTC reported Synthetic wet,
not throttled,
and at 2,750 of 2,750 five-hour requests;
Hyper was wet.
The active Synthetic roster at main commit `2caa18893` was:

- `hf:zai-org/GLM-5.3-Flash`
- `hf:Qwen/Qwen3.8-27B`
- `hf:moonshotai/Kimi-K3`
- `hf:openai/gpt-oss-120b`

The probes used the same streaming clients,
body parsers,
abort signals,
and transport used by translation repair.
Local client limits were raised to 128 only inside the probe.

Each logical call received a unique arithmetic task from a manually selected disjoint numeric range.
Structured arms required exactly one safe-integer `answer` property and checked the arithmetic result.
No prompt was repeated.
No raw provider response was retained in this document.

Each aggregate Synthetic run:

- ran one successful width-1 positive control for every model before the aggregate arm
- disabled retries so a transient status could not masquerade as admitted concurrency
- recorded provider status by model
- recorded local transport overlap by model and in aggregate
- stopped subsequent arms after HTTP 402 or 429

`localTransportPeak` proves that this client had the requests in flight together.
HTTP 200 plus complete schema-valid responses proves that the provider completed the workload.
Neither metric exposes the provider's internal GPU admission or queue.

The short response shape isolates request concurrency.
It does not reproduce long input processing,
large generated output,
or corpus stream-drain pressure.
Those remain user-boundary validation requirements for a replacement pipeline.

## Synthetic results

### Qwen3.8-27B

Interleaved controls took 1,003 ms to 1,598 ms.
Width 5 completed 5 of 5 in 1,486 ms.
Width 6 completed 6 of 6,
but one request took 19,143 ms.
A separate width-10 arm needed retries after HTTP 429 and is not a safe production bound.

### Kimi-K3

Interleaved controls took 1,442 ms to 2,123 ms.
Two width-5 arms completed 5 of 5 in 1,912 ms and 2,961 ms.
Width 6 completed 6 of 6 in 3,458 ms.
Width 5 is the documented-account-aligned candidate value;
width 6 shows one successful arm but does not justify a larger shared setting.

### GLM-5.3-Flash

Interleaved controls took 1,776 ms to 1,982 ms.
Two width-5 arms completed 5 of 5 in 2,276 ms and 3,493 ms.
Width 6 completed 6 of 6 in 2,787 ms.
Width 5 remains the conservative short-call candidate value.

### gpt-oss-120b

Interleaved controls took 1,217 ms to 1,417 ms.
Width 5 completed 5 of 5 in 1,386 ms.
Width 10 completed 10 logical calls in 2,054 ms only after 3 HTTP 429 retries.

The zero-retry aggregate arm reproduced the boundary:
7 of 10 gpt-oss calls returned HTTP 200 and 3 returned HTTP 429.
The other models at width 5 completed normally in that same arm.
The HTTP 429 responses were attached to gpt-oss calls.
Because all gpt-oss calls were dispatched at end of one aggregate burst,
this single arm does not distinguish model admission from burst or account-rate behavior.

### Aggregate width

Two zero-retry arms each dispatched 20 calls,
5 on every active Synthetic model.
Every call returned HTTP 200 with an exact schema-valid answer.
The arms completed in 4,113 ms and 3,943 ms.
The recorded local transport peaks were 20 in aggregate and 5 on each model.

This contradicts the July aggregate-stall observation for the current account,
roster,
and short request shape.
It does not invalidate that historical trace.
The provider behavior,
models,
and probe shape all changed.

## Hyper results

A width-1 structured positive control completed in 1,556 ms.
Interleaved width-1 controls then completed in 1,841 ms and 707 ms.

- Width 8 completed 8 of 8 in 1,282 ms.
- Width 32 completed 32 of 32 in 1,727 ms.
- Width 64 completed 64 of 64 in 2,147 ms.

All 107 structured calls returned HTTP 200.
There were no retries,
invalid answers,
schema mismatches,
refusals,
or transport errors.
The local transport peaks matched each requested width.

An earlier text-only arm also received HTTP 200 at widths 8,
32,
and 64,
but its exact-digits validator rejected mostly verbose answers.
That arm is transport evidence only and is excluded from answer-conformance claims.
The structured positive-control run is the deciding measurement.

## Rate limits are separate

Synthetic's public [rate-limit description][synthetic-rate-limits]
explains per-model concurrency as subscribed packs and says different models do not interfere.
The current account exposed a 2,750-request five-hour maximum,
consistent with the previously recorded 5.5 pack-equivalents.
The observed width-5 result matches the whole concurrent slots implied by that account state.
The five-hour and weekly quota readers were independent admission inputs until 2026-10-19;
since then the weekly budget is the one Synthetic publishes,
and a reply that still carries a five-hour window is honoured.

Hyper's public [overview][hyper-overview]
describes API access but does not publish a concurrency ceiling in the page reviewed on 2026-08-30.
The width-64 result therefore records one model and request shape,
not a provider guarantee.
The 1,000 requests-per-hour account limit must be implemented as a rate budget,
not by lowering simultaneous in-flight work to 8.

## An aggregator's passed-on 429 is one model's, not the account's (2026-09-19)

OpenRouter answers an upstream endpoint's rate limit with the endpoint's own error wrapped
(XingZ612,
Mercury 2.5 on Inception,
06:06 to 06:12 UTC):

```json
// HTTP 429 body, abridged
{"error":{"message":"Provider returned error","code":429,"metadata":{
  "raw":"<model> is temporarily rate-limited upstream. ...",
  "provider_name":"<endpoint>","provider_error_code":"rate_limit_exceeded",
  "limit_source":"upstream_provider_shared_pool"}}}
```

That is a statement about one model's shared pool,
not about the account's minute,
and every other model on the aggregator is still served.
Reading it as the provider's refusal held the whole provider out 60 s per reply
and stopped an entry whose editors and refiners sit on that provider alone
(class sixty-two).
`isUpstreamModelRefusal` in `provider-budget-refusal.ts` reads the body's opening
and the router lets the seat go dark for the round instead of holding the provider.
Letting it go dark for the round alone was not enough
(class sixty-four,
XingZ613):
every later round asked the model again and waited its five-attempt ladder out in grace,
14 to 32 s mean on the stages the model sat on against 0 to 6 s,
and the repair lane took 1h39m against 35m.
`upstream-model-hold.ts` now holds that one model out for 60 s after such a reply,
and the router refuses it at call time as `NoProviderForModelError` until the hold ends,
so a round counts the seat unreachable and neither asks nor waits;
the provider's other models are served throughout.
The OpenRouter error reference is client-rendered and could not be fetched with a plain request;
the shape this section quotes is the live body.

## Hyper's daily limit is not its hourly one (2026-09-07)

Hyper has a second limit the pacer cannot see.
On the fourth Huasheng pass of 2026-09-07,
from 17:00 to 19:54 UTC,
every Hyper call answered HTTP 429 with the body
`You've hit your daily rate limit. Please try again in 2h25m18s.`
(84 bodies logged,
each wait counting down to 19:53 UTC or 19:54 UTC),
while the balance meter read wet at 909 the whole time and the hourly pacer kept the window exactly full.
The retry ladder read digits followed by `s` only,
so a wait written in hours and minutes parsed as no wait;
the router then held the provider out for the 60 s concurrency backoff and walked back in:
2,693 refused attempts,
831 holds,
four consolidation chunks of about 75 min each settling on nobody.

Since `31e67a100` the stated wait parses as hours,
minutes and seconds,
a wait past the ladder's own widest backoff window ends the ladder at once,
and `markRefused` holds the provider out for at least the wait its refusal named,
whatever the meter reads.
The daily quota's size is not published;
today it closed after about 1,000 requests of this pass on top of the day's earlier passes,
and it reopened at the instant the bodies named.
It closed again at 21:57 UTC on the third hakureico pass,
naming 538 s,
and that pass showed the hold's other face:
with Bedrock wet nothing waited,
so the translate lane and the consolidation ran on the two Bedrock seats and settled on nobody.
Since `752bf9a9b` a phase whose benches cannot reach quorum among the seats a wet provider serves
waits out the shortest named hold once before it starts (`JUDGE SEATS ... waited=`),
and the translate lane re-reads its seats when it is about to start.

## Production scheduler contract

For the replacement finite dependency graph:

- dispatch every ready Synthetic node with no per-model semaphore,
  since Synthetic published no concurrency limit on 2026-10-19
  (until then a 5-slot semaphore keyed by model identity,
  aggregate width 20)
- do not configure gpt-oss or Qwen at 10 based on model size
- treat HTTP 429 as rate or admission evidence,
  not successful concurrency
- leave Hyper in-flight concurrency unbounded and keep separate hourly request budget
- retain exact dependency order where a prompt consumes prior node output
- retain one-provider operation by making every required role reachable through one wet provider
- validate corpus-sized payloads before claiming the short-call width is production throughput

The graph should expose ready work to the provider scheduler.
It should not manufacture extra panel calls merely to occupy slots.

## Evidence inventory

Private logs remain outside the repository because they contain provider telemetry.
Their SHA-256 digests are:

- Qwen threshold:
  `7ef309495f4df689baf8214af27f81bc8293743e4ccf71dfadfd239182047c6d`
- Kimi:
  `d801433ec402e01e2a35d9a3951c071b9e24fe206726f4381041165f1c16035f`
- gpt-oss:
  `7ec42862fdf1a2d248a23a1cadd5a944bf358374bfb9952950943e95e8620dc3`
- GLM-5.3-Flash:
  `89e2cb5a548a2e0521dd064322cc5e52f68cebab1886ed22805d0b82734366d8`
- Synthetic aggregate width 20:
  `6fc7e599af177e5b7bb3197937777d449396132870c5b59f1360e423fac638fd`
- Synthetic aggregate width 25:
  `8a5f58bb0262e0877da1f326d0ee45a584f964d379dfba653c3000d22b9ad2da`
- Hyper structured:
  `be9d6f2d9c4922790d8f74a2434570cb9acfba2d8b9d1df09da6ce7db2bdf2a6`

## Upstream disposition

The July stall was not reproduced under the 2026-08-30 short-call conditions.
That different workload does not explain the historical real-review behavior,
so this update neither closes nor escalates the original support question.
Synthetic completed the documented account-aligned width in these arms,
and Hyper completed every tested width on one model.
The historical Synthetic support report remains in
`doc/troubleshooting/synthetic-aggregate-concurrency-stall.md`
and now carries the dated recovery result.

[hyper-overview]: https://hyper.charm.land/docs/
[synthetic-rate-limits]: https://synthetic.new/rate-limits
