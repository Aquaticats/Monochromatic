# Captures the tests replay

Each capture below is kept on one line, however long, because a wrap would change the text the
tests compare.

Every fixture text this package tests against is recorded here. A captured text carries the run
folder whose `summary.json` holds it under `${HOME}/temp/agent`, the file’s modification date and its
length in characters; the tests cite this file instead of a run folder that cleanup removes. A text
with no capture carries either the installed Pi file and line that produces it, or a statement that
it is made up and the boundary it tests.

## OpenAI’s policy stop

Two runs carry the same 311 characters, field `modelError`:

- `pi-runs/prewarm-of-a-new-worktree-acts-only-on-the-entry-it-checked-1/summary.json`,
  2026-11-18T21:46:56-05:00.
- `pi-codemode-abort-crash-troubleshooting-2026-11-18/run-1/summary.json`,
  2026-11-18T22:15:17-05:00.

The text alone is recorded, not the wire event that carried it: it begins with the sentence and
carries no status prefix.

```text
This content was flagged for possible cybersecurity risk. If this seems wrong, try rephrasing your request. If you’re doing authorized security work that requires more cyber permissive safeguards, apply for Daybreak access via https://platform.openai.com/settings/organization/status-and-access before retrying.
```

## Synthetic’s subscription refusal, quoted form

- `pi-runs/kimi-review-since-alpha8-landing-waits-1/summary.json`, 2026-11-10T17:43:34-05:00,
  138 characters.

The quoted form `429 "..."` is the other way a recorded refusal begins; ten recorded values carry
it and 201 begin `429: {`.

```text
429 "You've exceeded your subscription rate limits. Upgrade, or try again later. You can view your usage at https://synthetic.new/billing"
```

## NeuralWatt’s overage refusal

- `pi-runs/worktree-remove-does-not-wait-on-a-read-only-git-2/summary.json`,
  2026-11-10T21:11:23-05:00, 795 characters.

It names `retry_strategy.suggested_initial_delay_s`, the provider-given delay the plugin takes as
the member’s hold.

```text
429: {"type":"rate_limit_error","code":"concurrent_budget_exceeded","message":"Concurrent limit reached for deepseek-ai/DeepSeek-V4.1-Flash: 6/6 slots in use. Wait for a request to complete and retry. Your subscription is in overage with auto-renew off, so pay-as-you-go limits apply until it renews. Turn auto-renew back on to restore your account level's limits: https://docs.neuralwatt.com/billing/account-levels#overage-with-auto-renew-off","retry_strategy":{"type":"concurrent_drain","suggested_initial_delay_s":1,"max_delay_s":30,"backoff":"exponential","backoff_base":2,"jitter":true},"retryable":true,"context":{"budget":6,"in_flight":6,"model":"deepseek-ai/DeepSeek-V4.1-Flash","limit_type":"concurrent","tpm_window_tokens":0,"tpm_limit":5242880,"floor":5,"burst":6,"burst_shed":false}}
```

## Texts taken from the installed Pi, with file and line

These stand for Pi's own behaviour. The source is the installed build, not a capture; each line was
read at `@earendil-works/pi-coding-agent` 1.5.2 and `@earendil-works/pi-ai` 1.5.2 in this worktree's
`node_modules`.

- The pool's provisional window and output limit, 128000 and 16384
  (`src/stand-in-pi-fixture.ts`, `src/model-pools.ts`): Pi's own fallback for a model that declares
  no window, `dist/core/provider-composer.js` `modelFromJson`.
- The `<provider> API error (<status>): <body>` and `<status>: <body>` refusal shapes, and the
  `unknown: <body>` form a failure with no status takes
  (`src/stand-in-pi-fixture.ts` `rateLimitRefusal`, `src/policy-stop-fixture.ts`,
  `src/adapter-refusal.unit.test.ts`): `dist/utils/error-body.js` `formatProviderError`.
- The overflow body `Input length (N) exceeds model's maximum context length (M).`
  (`src/verify-stand-in-member-extension.ts` `overflowRefusal`, the overflow arm of
  `src/verify-overflow-session.ts`): Pi's overflow detector lists the OpenAI-compatible shape,
  `@earendil-works/pi-ai` `dist/utils/overflow.js`.
- The retry and non-retry vocabulary `src/policy-stop-message.ts` strips
  (`RETRY_PHRASES`, `SEPARATED_PHRASES`): Pi's transient-error classifier,
  `@earendil-works/pi-ai` `dist/utils/retry.js`.
- The classifier examples `src/policy-stop-routing.unit.test.ts` replays under `TRANSIENT_SPANS`
  (`'503'`, `'networkerror'`, `'timed out'`, `'server_busy'`, `'ResourceExhausted'`, the fetch and
  HTTP/2 phrasings): the same classifier's patterns, `dist/utils/retry.js`.
- The default stand-in member windows and output limits, and every `TARGETS` model id and base URL
  (`src/stand-in-pi-fixture.ts`): the installed provider catalogs,
  `@earendil-works/pi-ai` `dist/providers/data/*.json`, and the member extension sources under
  `package/pi-plugin/`.
- The pool order and the reviewer order (`src/member-order-fixture.ts`): `.agents/skills/pi-as-subagent`
  `SKILL.md`, section "Who writes and who reviews", as that file cites its owner decisions.
- Qwen's `Requests rate limit exceeded` wording and its one-minute advice
  (`src/quota-reentry.unit.test.ts`): the Token Plan FAQ, section "Rate limiting and concurrency",
  as the constant's comment quotes it.

## Made-up texts, and the boundary each tests

No capture of these exists on this machine, and none is a provider's documented wording. Each is
here so a reader never mistakes it for evidence.

- `Too many requests` and `Request allowance exhausted for the current window`
  (`src/stand-in-pi-fixture.ts` `rateLimitRefusal`, `src/adapter-refusal.unit.test.ts`): a 429 body.
  Boundary: the installed adapter's refusal formatting, compared byte for byte in
  `src/adapter-refusal.unit.test.ts`.
- `Fixture system instruction` and `Fixture request` (`src/stand-in-pi-fixture.ts` `TRANSCRIPT`):
  boundary: the transcript object reaching a member is the one the caller supplied.
- `503: {"error":{"message":"Service temporarily unavailable","type":"server_error"}}`
  (`src/transient-refusal.unit.test.ts`, `src/verify-stand-in-member-extension.ts`): a provider's
  capacity 503. Boundary: the status a refusal opens with, which is the only part the pool reads.
- `503: {"error":{"message":"Request blocked by safety policy","type":"server_error"}}` and
  `503: {"error":{"message":"Upstream says a possible risk applies","type":"server_error"}}`
  (`src/transient-refusal.unit.test.ts` `POLICY_WORDED_503S`): a capacity 503 whose body carries
  policy wording. Boundary: the transient status the refusal opens with decides the handoff, so
  wording inside the body cannot turn it into a stop that ends the request at that member.
- `523: Error 523: origin is unreachable` and `522: Error 522: connection timed out`
  (`src/transient-refusal.unit.test.ts`, `src/verify-stand-in-member-extension.ts`): the Cloudflare
  origin error headings, which no capture records. Boundary: the status a refusal opens with, so the
  pool moves the request on before Pi's own classifier reads `timed out`. The wording is
  Cloudflare's published error page for each status
  (https://developers.cloudflare.com/support/troubleshooting/http-status-codes/cloudflare-5xx-errors/,
  read 2026-11-19), and the `<status>: ` prefix is the installed adapter's formatting.
- `429: OpenAI API error (523): upstream unavailable` and
  `429:openai api error (523): upstream unavailable` (`src/transient-refusal.unit.test.ts`):
  boundary: the refusal's own leading 429 decides, so a body that quotes a Responses failure label
  for a Cloudflare status stays a rate limit instead of becoming transient. The unspaced spelling
  writes the same quote with no space after the 429's colon, which no adapter writes.
- `standin API error (523): origin is unreachable` (`src/transient-refusal.unit.test.ts`): a
  Responses failure label for a non-OpenAI provider. Boundary: the label opens the text, so the pool
  reads the status from it with no response exposed by the adapter.
- The `Retry-After` hold (`src/transient-refusal.unit.test.ts`, the `transientHold` case) drives a
  hand-made `onResponse` call on a 503. No installed adapter makes that call, because each calls the
  hook after a successful response alone, as `README.md` states under "Changing limits". Boundary:
  the header path the plugin keeps for an adapter that starts exposing failed responses.
- `401 Unauthorized: contact support with request 429`, `Request timed out`,
  `400 Context window exceeded`, `4290 tokens exceeded the window`
  (`src/routing.unit.test.ts` `TERMINAL_ERRORS`): boundary: a failure is not a refusal unless it
  opens with a status, so a mention of 429, a timeout phrase, an overflow body and a number that
  begins with 429 all end the request at that member.
- `Provider returned an error` (`src/routing.unit.test.ts`,
  `src/transient-refusal.unit.test.ts`): a failure whose text names no status. Boundary: the status
  the response hook exposes decides, and without one the text does.
- `429: {not json` (`src/routing.unit.test.ts`): a refusal whose body cannot be parsed. Boundary: an
  unreadable body names no delay, and the refusal still moves the request on.
- `'six minutes'`, `'6w'`, `'1.'`, `'1m30'` (`src/quota-routing.unit.test.ts`): malformed reset
  durations. Boundary: a partially parsed duration names no deadline.
- The single-slot overage bodies modelled on the recorded refusal
  (`src/quota-routing.unit.test.ts`) and the one-hour `retry_strategy` body: boundaries: an observed
  cap of one slot, and the cap on a delay a body may set.
- The Synthetic weekly, legacy-weekly and subscription bodies (`src/synthetic-quota-fixture.ts`):
  boundaries: the current and previous weekly key names, and a request count at its limit. Their key
  names come from `doc/troubleshooting/synthetic-quota-weekly-block-renamed.md`; that a depleted
  window holds the member only when it also names a reset time is an inference recorded in the
  fixture's own comment.
- `Bearer stand-in-header-only`, `Basic stand-in`, `Stand-in registry resolution failure`,
  `Stand-in registry refusal`, `stand-in transport failure`, `{ name: 'stand-in resolver failure' }`
  (`src/quota-auth-and-cancel.unit.test.ts`): boundaries: a header-only bearer is used and unrelated
  headers are not forwarded; a non-Bearer value, a resolver exception, a registry refusal and a
  non-Error transport failure all leave the member admitted and send no quota request.
- `stand-in-outer-key`, `stand-in-outer-header`, `CREDENTIAL_MARKER` (`src/routing.unit.test.ts`):
  boundary: nothing the caller supplied as a credential reaches a member.
- `tool-fixture` and `fixtureTool` (`src/routing.unit.test.ts`): boundary: once a tool event was
  exposed, a failure ends the request at that member instead of being retried elsewhere.
- `Request flagged as a possible risk`, `Rejected by the content-filter. Contact support.`,
  `Blocked by safety_policy. Contact support.`, `Blocked for a safety violation. Contact support.`
  and `Blocked for a policy violation. Contact support.` (`src/policy-stop-routing.unit.test.ts`
  `OTHER_POLICY_MESSAGES`): boundary: a content stop another vendor produces ends the request, even
  when it uses the same kinds of wording.
- `Server_busy, RAtE-Limit, and please retry your request. Reference abc429xyz.` and the retained
  detail `Reference İ😀abc429xyz, 1503, and rate\nlimit.` (`src/policy-stop-routing.unit.test.ts`):
  boundaries: every span Pi's classifier would retry is removed from a stop's message, and near
  matches, Unicode text and split words are kept.
- `Stand-in <member> answered a <N>-character request.` (`src/verify-stand-in-member-extension.ts`):
  boundary: a stand-in answer names the member that produced it and the request size it saw.
