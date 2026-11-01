# OpenRouter and the corpus passes: the proposal, the transport probes and the build plan

Why OpenRouter was proposed as the paid fallback provider on 2026-09-03,
what it answered and would cost,
the decisions and constraints taken then,
the transport probes,
and the build plan.

Part of [Translation repair: OpenRouter as the paid fallback provider](translation-repair-openrouter-2026-09-03.md),
which lists every file of this record.

## Why now

The owner on 2026-09-03:
Charm Hyper ended its bundle subsidization,
so there is no reason to recharge Hyper from the usual subscription;
OpenRouter is preferred for paid per-token work,
and Synthetic and Hyper are expected to run dry often.
`TRANSLATION_REPAIR_OPENROUTER_API_KEY` was added to the main worktree's `.env.local.json`.
The feature worktree's copy predated it;
copying the encrypted file across was refused by the session's permission classifier,
so the day's OpenRouter probes and both live passes ran with the main repo as the mise config root
(the bridge `doc/decision/translation-repair-multi-provider.md` records under "The worktree's secrets file is stale").
The owner copied the file at 17:40 UTC;
`mise exec` from the worktree root now injects all four names,
and later passes launch from the worktree as the runbook's ordinary path says.

State of the other two providers at the time of writing:

- Synthetic weekly read `0%` from 13:15:51 UTC on the stub-fix XIEPT2 run (`~/temp/agent/xiept2-stub-20260903.log`).
- Hyper `GET /v1/credits` read 2497.
    Runs measured off the `METERS hyperBalance=` line,
    first to last,
    cost 724 (XIEPT2 postscript),
    654 (XIEPT2 stub fix),
    654 (Carena0442 rerun of 2026-09-02) and 428 (Toka_ls rerun 2 of 2026-09-02).
    That balance buys three to five more entry runs and will not be topped up.

## What OpenRouter answered

### Account and meter

- `GET https://openrouter.ai/api/v1/key` with the ordinary key:
  `limit: null`,
  `limit_remaining: null`,
    `usage: 0`,
    `is_free_tier: false`,
    `is_management_key: false`.
- `GET https://openrouter.ai/api/v1/credits` with the same ordinary key answered `200` with
    `total_credits: 1913` and `total_usage: 1855.383100082`,
    so the balance is about 57.62 USD,
    matching the owner's figure.
    The endpoint's own page says a management key is required;
    the live call says otherwise.
    Recorded as a discrepancy rather than resolved;
    the meter will be built on the live behaviour with an unreadable meter counting as spendable,
    which is this package's existing rule (`provider-budget.ts`).
- Purchase fee per the FAQ:
  5.5% with a 0.80 USD minimum by card,
  5% by cryptocurrency.
    Credits may expire one year after purchase.
- Rate limits page:
  paid models carry no documented request ceiling beyond DDoS protection;
    `429` arrives as a status or as an in-stream `finish_reason: "error"`,
    `402` when credits are insufficient.
    Free variants (`:free`) are capped at 20 requests per minute and 1,000 per day.

### Transport

- `POST https://openrouter.ai/api/v1/messages` speaks the Anthropic Messages format:
    `Authorization: Bearer`,
    streaming SSE in Anthropic event shapes,
    `tools` and `tool_choice: {type: "tool", name}` forced tool use,
    OpenRouter model slugs in `model`,
    a `provider` object in the body,
    and usage carrying `input_tokens`,
    `output_tokens` and an optional `cost` in USD.
    This is the protocol `hyper-client.ts` already speaks,
    so `buildAnthropicBody`,
    `extractAnthropicCompletion` and `wireFormat: 'anthropic'` are reusable.
- The `provider` object supports `order`,
  `only`,
  `ignore`,
  `allow_fallbacks`,
  `require_parameters`,
    `data_collection: 'deny'`,
    `zdr: true`,
    `sort`,
    `max_price` and quantization filters.
    The account-level allowed-provider list is a ceiling over request-level `only`;
    account-level ignores merge with request-level ones.
- Zero data retention:
  `zdr: true` per request ORs with the account setting.
    `GET https://openrouter.ai/api/v1/endpoints/zdr` listed 834 ZDR endpoints.

### Batch

From the batch quickstart:
`POST https://openrouter.ai/api/beta/batches` with `endpoint`,
`model` and inline `requests[]`,
polled at `GET /api/beta/batches/:id`,
results inline on completion,
text only,
the only completion window `24h`,
priced at "50% of the model's standard per-token pricing".

The `:batch` entries in the public listing,
USD per million prompt and completion tokens,
against the realtime entry of the same model:

- `moonshotai/kimi-k3`:
  batch 3 and 15;
  realtime 3 and 15.
- `minimax/minimax-m3`:
  batch 0.3 and 1.2;
  realtime 0.3 and 1.2.
- `deepseek/deepseek-v4-flash-0731`:
  batch 0.14 and 0.28;
  realtime 0.065 and 0.18.
- `z-ai/glm-5.3-flash`:
  batch 0.15 and 0.5;
  realtime 0.075 and 0.25.
- `openai/gpt-oss-120b`:
  batch 0.15 and 0.6;
  realtime 0.037 and 0.17.
- `qwen/qwen3.8-27b` and `z-ai/glm-5.3`:
  no `:batch` entry.

Which price the "50% of standard" claim discounts from is not established here;
the listing's realtime column is the cheapest endpoint,
and the batch column may discount a single provider's list price.
What is established:
for this roster,
the listed batch price is never below the listed realtime price,
and two roster models have no batch entry at all.

The structural fact matters more than the price.
A corpus pass is a chain of dependent rounds (critics,
panel,
editors,
judges,
contest,
consolidation),
about fifty per entry,
each waiting on the previous one,
and every round is built on streaming quorum with straggler and writer grace.
A batch whose only window is 24 hours cannot carry that chain within rule `FIT` or `FT2`,
and the guards that keep a stage honest (quorum,
grace,
abandonment) have no meaning over a batch.

## Roster models on OpenRouter

Spellings,
realtime USD per million prompt and completion tokens,
total endpoints,
ZDR endpoints,
and whether the owner's allowlist carries the model:

- `moonshotai/kimi-k3`:
  3 and 15;
  18 endpoints,
  16 ZDR;
  allowlisted.
- `minimax/minimax-m3`:
  0.3 and 1.2;
  11 endpoints,
  7 ZDR;
  allowlisted.
- `deepseek/deepseek-v4-flash-0731`:
  0.065 and 0.18;
  30 endpoints,
  22 ZDR;
  allowlisted.
- `deepseek/deepseek-v4-pro-0813`:
  0.66 and 1.98;
  18 endpoints,
  13 ZDR;
  NOT allowlisted.
- `qwen/qwen3.8-27b`:
  0.425 and 2.55;
  12 endpoints,
  9 ZDR;
  allowlisted.
- `z-ai/glm-5.3`:
  1.4 and 4.4;
  25 endpoints,
  22 ZDR;
  allowlisted.
- `z-ai/glm-5.3-flash`:
  0.075 and 0.25;
  23 endpoints,
  19 ZDR;
  allowlisted.
- `google/gemma-4-26b-a4b-it`:
  0.07 and 0.34;
  9 endpoints,
  7 ZDR;
  NOT allowlisted.
- `openai/gpt-oss-120b`:
  0.037 and 0.17;
  20 endpoints,
  20 ZDR;
  NOT allowlisted.

Every roster model has at least seven ZDR endpoints,
so `zdr: true` removes no model.

Spelling map used by this section's pricing and to be carried into the catalog:

- `hf:moonshotai/Kimi-K3` and `kimi-k3` -> `moonshotai/kimi-k3`
- `minimax-m3` -> `minimax/minimax-m3`
- `deepseek-v4-flash-0731` -> `deepseek/deepseek-v4-flash-0731`
- `deepseek-v4-pro-0813` -> `deepseek/deepseek-v4-pro-0813`
- `hf:Qwen/Qwen3.8-27B` and `qwen3.8-27b` -> `qwen/qwen3.8-27b`
- `glm-5.3` -> `z-ai/glm-5.3`
- `hf:zai-org/GLM-5.3-Flash` and `glm-5.3-flash` -> `z-ai/glm-5.3-flash`
- `gemma-4-26b-a4b-it` -> `google/gemma-4-26b-a4b-it`
- `hf:openai/gpt-oss-120b` and `gpt-oss-120b` -> `openai/gpt-oss-120b`

## What an entry would cost bought entirely on OpenRouter

Method:
every `SPEND provider=... model=... prompt=N completion=N` line of a completed run,
mapped through the spelling map and priced at the realtime rates of the 2026-09-03 listing
(`~/temp/agent/openrouter-models-20260903.json`).
Calls whose provider reported zero usage (15 to 42 per run,
mostly `glm-5.3` on Hyper) price at zero,
so every total is a floor.
Script:
`price-runs-on-openrouter.mjs` in the session scratchpad;
it is a measurement aid,
not package code.

- XIEPT2 postscript (219 minutes,
  2197 calls):
  14.41 USD,
  Kimi-K3 8.74 (61%),
  5.67 without Kimi.
- XIEPT2 stub fix (146 minutes,
  2024 calls):
  12.48 USD,
  Kimi-K3 7.36 (59%),
  5.13 without Kimi.
- Carena0442 rerun 2026-09-02 (190 minutes,
  1938 calls):
  13.52 USD,
  Kimi-K3 7.76 (57%),
  5.75 without Kimi.
- Carena0442 four-entry pass 2026-09-01 (94 minutes,
  1749 calls):
  11.40 USD,
  Kimi-K3 5.90 (52%),
  5.50 without Kimi.
- Toka_ls rerun 2 2026-09-02 (100 minutes,
  1259 calls):
  6.78 USD,
  Kimi-K3 4.08 (60%),
  2.70 without Kimi.

After Kimi,
the next largest lines are Qwen3.8-27B (0.88 to 2.74),
minimax-m3 (0.69 to 1.29),
GLM-5.3 (0.48 to 1.15) and DeepSeek V4 Pro (0.47 to 1.08).
The three models absent from the allowlist together cost 0.12 to 1.26 per entry.

Auto top-up arithmetic:
the observed cadence is three to four entries a day,
so a day bought entirely on OpenRouter is about 50 USD with Kimi-K3 seated and about 20 USD without.
The fee is a percentage with a 0.80 floor,
so a top-up above about 15 USD pays the same rate whatever its size;
the amount only decides how often the purchase happens.
A threshold of 20 USD (one XIEPT2-scale entry plus margin) and a top-up of one day's spend follows.

## Decisions taken here, open to veto

- Routing order:
  Synthetic while wet,
  then Hyper while its balance lasts,
  then OpenRouter.
    This is the plain reading of the owner's two messages of 2026-09-03 taken together.
- Transport:
  the Anthropic Messages endpoint with forced tool use,
    reusing the Hyper request builder and stream reader with a different URL and auth header.
    `provider.require_parameters: true` so only endpoints supporting `tools` and `tool_choice` are eligible,
    `provider.ignore` for any endpoint that measures badly.
    Conformance is measured twenty times per model before a model is seated on this provider,
    as the Hyper decision did.
- Meter:
  `GET /api/v1/credits`,
  balance as `total_credits - total_usage`;
    dry at or below zero;
    unreadable counts as spendable;
    `402` and `429` are refusal holds.
    No management key is needed while the ordinary key answers.
- Gemini 3.8 Flash joins the roster blocklist on the owner's words ("a wildly misaligned model").
- `:free` variants are not used:
  20 requests per minute and 1,000 per day cannot carry a corpus pass,
    and their data policy is the provider's,
    not ours.
- Spend lines carry OpenRouter's reported `cost` in USD when present,
    so the price table is a fallback rather than the source.

## Constraints for the build

- Generalize the router to an ordered provider list rather than a third boolean;
    `ProviderName`,
    `BudgetView`,
    `ModelReach`,
    `readBudgetsPastHolds`,
    `secondOpinionFrom` and the all-dry error all encode two providers.
- Seat withholding in `run-seats.ts` is keyed to `syntheticDry` alone;
    with OpenRouter serving Qwen3.8-27B and Kimi-K3,
    re-derive withholding from where each model would be served.
- Check `anthropic-delta-scan.ts` against OpenRouter's stream:
  a `[DONE]` sentinel and comment keep-alives must not count as unreadable frames;
    pin with a case taken off the wire.
- Verify where usage lands in the OpenRouter stream before trusting `SPEND` lines.
- `required-providers.ts` accepts `openrouter`;
  the key is optional and loud like Hyper's.
- Concurrency and any request-rate ceiling on OpenRouter are unmeasured;
  run the width probe first.

## Questions put to the owner

Asked on 2026-09-03 after this record was written:

1. Batch API:
not viable for the pass as designed;
use realtime,
or redesign for batch.
2. Kimi-K3 when only OpenRouter would buy it:
seat at 3 and 15 USD per million,
or withhold it there.
3. Allowlist:
add `openai/gpt-oss-120b`,
`google/gemma-4-26b-a4b-it` and `deepseek/deepseek-v4-pro-0813`,
    or accept an OpenRouter tier without them.
    (As asked,
    the option text said this "leaves the editor stage on Qwen alone";
    that was wrong,
    gemma is a translator and Qwen a checker.
    Corrected in the decision doc.)
4. Zero data retention:
`zdr: true` on every request,
or plain routing.

Answers,
recorded in `doc/decision/translation-repair-openrouter-fallback.md`:
realtime;
withhold Kimi-K3 where only OpenRouter would buy it;
all three added to the allowlist;
ZDR on every request.

## The `[DONE]` sentinel, fixed before the probe could measure

The first probe against `/api/v1/messages` answered 200 on every call and conformed on none,
because the gateway appends an `event: data` frame carrying `data: [DONE]` after `message_stop`
and `extractAnthropicCompletion` refused the body as "anthropic stream event is not JSON",
while `scanAnthropicDeltas` counted the frame as unreadable.
Both readers now skip the sentinel (`90dcb8745`),
each pinned by a case taken off the wire,
and both cases were shown to fail with the skip removed.
The same capture showed the tool arguments arriving in several `input_json_delta` pieces,
usage and `cost` on `message_delta`,
the serving provider on `message_start.message.provider`,
and `stop_reason: "end_turn"` on a completed tool call,
none of which the reader minded.

## Probe v2: three transports per model

The owner,
mid-session:
OpenRouter likely supports OpenAI chat completions better,
and the Responses API is also supported (and may be the only route to GPT-5.6 Luna).
`~/temp/agent/openrouter-probe-v2-20260903.mjs` therefore drives each roster model twenty times through
each of `/api/v1/messages` (Anthropic format,
forced tool,
the pipeline's own builder and reader),
`/api/v1/chat/completions` (OpenAI format,
`response_format` json_schema,
the Synthetic body shape with the schema
restated in the system prompt) and `/api/v1/responses` (`text.format` json_schema),
every request carrying `provider: { zdr: true, require_parameters: true }`,
four calls in flight per model and transport,
and writes one raw stream per model and transport plus a summary to `~/temp/agent/openrouter-probe-v2-20260903/`.
GPT-5.6 Luna rides along as a candidate,
not a roster model.

### Probe v2 results, 15:49 to 16:05 UTC

Conformant attempts of twenty,
median milliseconds,
and the endpoints that served,
per model and transport (chat = chat completions,
msg = Messages,
resp = Responses):

- `moonshotai/kimi-k3`:
  chat 20,
  3819 ms (Fireworks,
  DeepInfra);
  resp 20,
  4554 ms;
  msg 20,
  30660 ms (DeepInfra).
- `minimax/minimax-m3`:
  chat 20,
  1367 ms (ModelRun);
  resp 20,
  1584 ms;
  msg 20,
  3461 ms (Venice).
- `deepseek/deepseek-v4-flash-0731`:
  chat 20,
  7252 ms (Inceptron,
  Parasail,
  Makora);
  resp 20,
  5803 ms;
    msg 9 of 20,
    7425 ms (DigitalOcean,
    Inceptron):
    eleven answers were not JSON.
- `deepseek/deepseek-v4-pro-0813`:
  chat 20,
  6332 ms (Parasail);
  resp 20,
  6755 ms;
  msg 20,
  1566 ms (BaseTen).
- `qwen/qwen3.8-27b`:
  chat 20,
  4856 ms (Parasail);
  resp 20,
  5208 ms;
  msg 20,
  10123 ms (Reka,
  AkashML).
- `z-ai/glm-5.3`:
  chat 20,
  1068 ms (Together);
  resp 20,
  1058 ms;
  msg 20,
  1126 ms (Together).
- `z-ai/glm-5.3-flash`:
  chat 20,
  7117 ms (Together,
  Modal);
  resp 20,
  7639 ms;
  msg 20,
  6890 ms (Together).
- `google/gemma-4-26b-a4b-it`:
  chat 20,
  1325 ms (Google,
  Parasail);
  resp 20,
  1813 ms;
  msg 20,
  1780 ms (Google,
  NextBit).
- `openai/gpt-oss-120b`:
  chat 20,
  664 ms (Cerebras);
  resp 20,
  717 ms;
  msg 20,
  721 ms (Cerebras).
- `openai/gpt-5.6-luna` (candidate):
  chat 9 of 20 (Azure),
  resp 11 of 20,
  msg 0 of 20 with HTTP 404 "No endpoints found matching your data policy (Zero data retention)";
    the failed chat and Responses attempts answered empty text (`response.failed`).
    Not viable under zero data retention as measured.

Every request cost is on the wire:
20 of 20 priced on every conformant row.
Whole probe:
about 0.55 USD.

THE TRANSPORT IS CHAT COMPLETIONS,
as the owner suggested mid-session:
it conformed on every roster attempt and answered fastest or within noise of fastest on every model,
where Messages answered Kimi-K3 eight times slower and conformed on 9 of 20 DeepSeek Flash attempts.
The Responses endpoint conformed too and is not used;
one OpenAI-shaped path is one reader to maintain.

Qwen3.8-27B's chat median (4856 ms on a short prompt) sits in the band of the other models,
unlike its Hyper serving,
so its withholding rule stays "served by Hyper" and it is seated when OpenRouter would serve it;
the first live pass ("The first live pass with OpenRouter in the order")
is where that is checked on corpus-sized prompts.

Width:
32 concurrent chat completions per model on `deepseek/deepseek-v4-flash-0731` (32 of 32 conformant,
median 2852 ms,
max 7776 ms;
Together,
Makora,
OpenInference) and `z-ai/glm-5.3-flash` (32 of 32,
median 7955 ms,
max 49581 ms;
Together,
Modal,
Makora),
no refusal.
The client therefore carries no per-model ceiling by default,
like Hyper's.

## Build plan, transport-independent layers first

In commit order,
each unit tested and committed before the next:

1. `provider-name.ts`:
`ProviderName` gains `openrouter`;
`PROVIDER_ORDER` states the routing preference (Synthetic,
    Hyper,
    OpenRouter);
    helpers over the record shape.
2. `openrouter-catalog.ts` and `roster-reach.ts`:
slugs,
`sharedWith` for all nine,
vision flags and output ceilings from the listing;
    `ModelReach` becomes a record keyed by provider;
    Gemini 3.8 Flash joins the blocklist.
3. `budget-routing.ts`:
`routeProviderFor` walks `PROVIDER_ORDER` over a dryness record and a saturation record;
    the two-provider dry error is renamed `EveryProviderDryError` (misleading name,
    CRN) and the old name goes to the local forbidden-strings appendix;
    `openRouterIsDry` and its meter fields.
4. `provider-budget.ts` and `budget-hold-wait.ts`:
a third meter,
`METERS` gains `openrouter=` and `openrouterUsd=`,
    a refusal hold moves traffic only while some other provider reads wet,
    the all-dry wait generalizes.
5. `provider-router.ts`:
callers keyed by provider,
a bounded re-route (one attempt per provider),
the re-ask asks the next wet provider serving the model;
    the re-ask path splits into its own file at the line budget.
6. `required-providers.ts`,
`run-config.ts`,
`budget-sample.ts`:
the third key,
optional and loud;
the run client exposes the dryness record for the seat reader.
7. `run-seats.ts`:
benches derive from where each model would be served (first provider in order that serves it and reads wet):
    Hyper-slow rules apply when served by Hyper,
    Kimi-K3 is withheld from every seat when served by OpenRouter,
    and `gemma-4-26b-a4b-it` takes the third checker seat there so both checker assertions hold per phase.
    Whether Qwen3.8-27B's rule is "served by Hyper" or "not served by Synthetic" is decided by its OpenRouter median in
    the probe.
8. `spend-line.ts`,
`spend-read.ts`,
`spend-cost.ts`:
a `cost=` field in USD from the wire,
an OpenRouter bucket in USD kept apart from hypercredits;
    `meter-sample-read.ts` and `meter-report.ts` accept the third field and its absence in older logs.
9. `openrouter-client.ts` and its credits parser,
on the transport the probe picks.
10. Live verification,
then the decision doc,
README and runbook.
