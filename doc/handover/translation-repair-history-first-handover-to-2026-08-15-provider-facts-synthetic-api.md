# Translation repair history, first handover document: provider facts for the Synthetic API

What was verified about the model provider:
quotas and request weights,
concurrency and streaming behaviour,
the user directives on reasoning effort and temperature,
and the measured effect of aggregate stream count.

Part of [Translation repair history: the first handover document, to 2026-08-15](translation-repair-history-first-handover-to-2026-08-15.md),
which lists every file of this record.

Historical evidence only:
no next-action list here is a current instruction,
and the stop condition in the index applies.

## Provider facts (verified from Synthetic docs and pricing)

- Per pack ($30/mo):
  500 price-weighted requests per 5 hours,
  regenerating 5% per 15 minutes;
  $24/week credits regenerating 2% per ~3.4 hours;
  1 concurrent request per model per pack,
  different models fully parallel
  (same-model excess queues server-side,
  it does not error).
  The user bought 4 more packs on 2026-07-16,
  joining a founder's pack
  worth 1.5 normal packs;
  the live account now shows a 2750-request five-hour ceiling
  (5.5 pack-equivalents at 500 each).
- Never set reasoning effort on Synthetic calls (user directive 2026-07-16):
  non-default values sometimes error,
  sometimes produce low-quality or worse
  output.
  Default only;
  there is no safe latency knob there.
- Run five concurrent streams per model and absorb burst weather with
  retries,
  not gentle dispatch (user directive 2026-07-16).
  Implemented in
  `transient-retry.ts` (commit `eb32173bf`):
  four transport retries on an
  equal-jitter ladder (half fixed,
  half random,
  doubling from 1 s),
  retryable statuses 408/429/500/502/503/504,
  thrown transport failures
  (mid-stream connection resets) retried too,
  caller aborts always
  propagate untouched,
  and the policy is injectable
  (`retryPolicy: { limit, baseMs }`) so tests run on tiny backoffs.
- Refined directive (2026-07-16,
  after the first concurrency-5 run failed):
  probe/bench the fastest dispatch strategy for this plan and use that.
  `bench-dispatch.ts` in the session scratchpad sweeps per-model
  concurrency 1/2/3/5 over identical small-entry critic calls and reports
  ok-per-minute plus forfeits per level;
  the milestone run uses the winner.
- Bench verdict (2026-07-17):
  one stream per model wins by a factor of
  six.
  Level 1:
  7/7 ok,
  71 s wall,
  5.9 ok/min (median call 39 s).
  Levels 2/3/5:
  0.8/1.0/0.6 ok/min,
  with every vendor except zai-org
  stalling to a 5-minute cap while both GLMs completed (slowed 2-4x).
  Aggregate concurrency beyond one-per-model collapses throughput on
  this plan,
  at least during this window;
  pack count does not translate
  into usable same-model parallelism.
  Milestone runs use
  `perModelConcurrency: 1`.
  Full fact base for the provider report:
  `doc/troubleshooting/synthetic-aggregate-concurrency-stall.md`.
- Benchmarks are time-boxed (user directive 2026-07-17:
  about 30 minutes
  each,
  or they run too rarely to be useful;
  commit `275f7b6ab`).
  `runCriticBenchmark` takes `runBudgetMs`:
  models work entry queues
  sequentially,
  every attempt and retry is budget-gated,
  exchange
  deadlines cap to the remaining budget,
  and cut attempts record as
  `skipped`.
  The scorecard excludes skipped records from all rates and
  recall denominators and reports `coverage`;
  drivers use
  `runBudgetMs: 25 min` + `perCallTimeoutMs: 5 min` + a 45-minute outer
  signal as safety net only (an outer abort throws away every in-memory
  record),
  and shuffle their entry sample per run so repeated
  budget-bound runs accumulate coverage.
- Deadline placement is load-bearing (commit `7c0e41532`):
  the first
  concurrency-5 run armed every fan-out call's deadline at dispatch while
  the limiter ran five per model,
  so queued calls burned their whole budget
  waiting and 124 of 126 expired in one synchronized wall;
  the mass abort
  then crashed Node via an orphaned HTTP/2 stream error
  (`ERR_HTTP2_STREAM_ERROR`;
  Node 26 fetch speaks h2 to this origin).
  Deadlines now arm inside the client's per-model slot
  (`exchangeTimeoutMs` on `ChatTextRequest`,
  `call-deadline.ts`),
  so only
  the exchange counts.
  Drivers keep a scoped `uncaughtException` guard
  that swallows only `ERR_HTTP2_STREAM_ERROR`.
- 35-stream probe (5 per model,
  tiny prompts):
  the dispatch burst drew 27
  instant 502s that the transport retries fully absorbed,
  but service
  under 35 streams is heavily stalled:
  one-character answers took
  78 to 119 s and 14 of 35 calls missed a 120 s cap.
  Aggregate stream
  count,
  not just per-model entitlement,
  governs real throughput.
- Never set temperature either (user directive 2026-07-16):
  per the user,
  this is less a Synthetic API issue and more their upstream GPU providers
  plus inference pipelines plus the models' inherent issues;
  either way the
  knob is not honored reliably.
  It was removed from `ChatTextRequest`
  entirely so nothing can set it;
  a unit test asserts the wire body carries
  no `temperature` key.
  All calls run on defaults,
  which also means past
  temperature-0 runs never had the determinism the setting promised
  (consistent with the observed completion-vs-ceiling flips on identical
  input).
- Live probe results (2026-07-16,
  after run 4):
  three concurrent tiny GLM-4.7-Flash calls all completed in 2.0 to 2.4 s,
  fully overlapped (no server-side serialization of dispatched requests);
  a real critic call on the smallest entry (DarlinChit,
  1.4 KB translation)
  completed in 28.6 s on GLM-5.2 (6_621 completion tokens,
  9 issues) and in
  35.4 s on gpt-oss-120b (2_246 tokens,
  6 issues).
  Contrast run 4:
  an 8 KB translation drove GLM-5.2 to the 65_536-token
  output ceiling without finishing its JSON,
  and every other call starved
  behind long-running ones.
  Work-unit conclusion:
  critic calls must stay near DarlinChit scale
  (roughly 1 to 4 KB of translation);
  document chunking is mandatory for
  the full pipeline.
- Request weight = model input price / baseline input price (baseline is the provider
  default model,
  currently GLM-5.2 at exactly 1).
  Verified empirically:
  one GLM-4.7-Flash call deducted exactly 0.0714 (1/14) from
  `/quotas` remaining,
  matching `estimateRequestWeight` in `synthetic-catalog.ts`.
- `GET /openai/v1/models` carries per-model pricing,
  context length,
  max output
  (65536 for all),
  and feature flags;
  every catalog model advertises `json_mode` and
  `structured_outputs`.
  Per-model strictness still unverified,
  so client-side validation stays.
- `GET https://api.synthetic.new/v2/quotas` (free,
  does not count against limits);
  live shape verified 2026-07-16:
  `rollingFiveHourLimit {remaining,max,limited,nextTickAt,tickPercent}`,
  `weeklyTokenLimit {percentRemaining,maxCredits,remainingCredits,nextRegenAt,...}`,
  plus `subscription`,
  `search`,
  `freeToolCalls` (unmodeled).
- Models:
  GLM-5.2 (512k),
  GLM-4.7-Flash,
  Qwen3.6-27B,
  Kimi-K2.7-Code,
  MiniMax-M3,
  Nemotron-3-Super-120B,
  gpt-oss-120b;
  six vendor families.
- Chat base URL `https://api.synthetic.new/openai/v1`.
- The client (task 4) provides a per-model `p-limit` semaphore sized by
  `perModelConcurrency` (default 1;
  pass the pack count);
  price-aware role routing belongs to the orchestrator.
- End-to-end boundary check passed:
  real GLM-4.7-Flash `chatJson` round trip returned
  guard-validated JSON and the quota delta matched the estimate.
- Chat calls must stream (user directive:
  the provider is finicky without streaming;
  and the first real benchmark died on fetch's five-minute headers timeout while a
  model was thinking).
  The client always sends `stream: true` with `stream_options.include_usage`;
  the transport drains the whole SSE body to text and `stream-completion.ts`
  reassembles it,
  requiring the `[DONE]` terminator (cut-off streams throw instead
  of returning truncated content) and folding `delta.refusal` into the first-class
  refusal field.
  Do not add an undici dependency for timeout control (user directive);
  streaming makes plain platform fetch sufficient.
- These models think heavily:
  expect 90%+ of output tokens to be thinking tokens
  (user guidance;
  live probe confirmed 35 completion tokens for a 1-token answer).
  Consequences already built into the client:
  reasoning arrives in separate `reasoning`/`reasoning_content` message fields with
  clean `content` (verified live on GLM-4.7-Flash);
  the message carries a first-class `refusal` field which outranks heuristics
  (marker `api-refusal-field`);
  embedded `<think>` blocks are split off before parsing and refusal scanning;
  truncation inside thinking is a distinct schema-mismatch detail;
  `maxTokens` must be generous or omitted;
  budget spend estimates must use thinking-inflated completion counts
  (usage is carried on every `chatJson` outcome for the scorecard).
