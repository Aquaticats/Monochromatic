# @monochromatic-dev/translation-repair-provider

Everything that talks to a model provider:
routing a call to the provider that serves its model and still has budget,
the NeuralWatt,
Synthetic,
Hyper,
OpenRouter,
Bedrock,
DeepSeek and Xiaomi clients,
the NeuralWatt,
Synthetic,
OpenRouter,
Bedrock,
DeepSeek and Xiaomi catalogues,
the Anthropic Messages wire format that Hyper speaks,
retries,
deadlines and spend reporting.
It is its own package so the stage and lane layers that build on it call one routing client
and the clients load from one built file.

## What it provides

`src/index.ts` re-exports each public name from the module that declares it.
The groups that follow name the main ones.

- Routing and budgets:
  choose the provider for a model,
  pass calls down the provider order when one is dry,
  hold a provider out for the run when it rejects its credential,
  read the provider meters,
  and pace requests to a provider's rate limit.
  Names:
  `createRoutingClient`,
  `createCredentialHolds`,
  `foldCredentialHolds`,
  `isCredentialRefusal`,
  `routeProviderFor`,
  `createProviderBudgets`,
  `readEveryMeter`,
  `EveryProviderDryError`,
  `createRequestPace`.
  Declared in `provider-router.ts`,
  `credential-hold.ts`,
  `budget-routing.ts`,
  `provider-budget.ts`,
  `provider-meters.ts`,
  and `request-pace.ts`.
- Provider clients:
  one client for each provider,
  each with its per-model concurrency,
  which defaults to no ceiling (Synthetic's since 2026-10-19).
  Names:
  `createNeuralWattClient`,
  `createSyntheticClient`,
  `createHyperClient`,
  `createOpenRouterClient`,
  `createBedrockClient`,
  `SYNTHETIC_PER_MODEL_CONCURRENCY`,
  `OPENROUTER_PER_MODEL_CONCURRENCY`.
  Declared in `synthetic-client.ts`,
  `hyper-client.ts`,
  `openrouter-client.ts`,
  `bedrock-client.ts`,
  and `neuralwatt-client.ts`.
- Catalogues and reach:
  the models NeuralWatt,
  Synthetic,
  OpenRouter and Bedrock serve for the pipeline,
  and which providers can take a call for a roster model,
  with or without a picture.
  Names:
  `NEURALWATT_MODELS`,
  `SYNTHETIC_MODELS`,
  `OPENROUTER_MODELS`,
  `BEDROCK_MODELS`,
  `syntheticServes`,
  `reachOf`,
  `visionReachOf`,
  `OPENROUTER_PICTURES_UNMEASURED`
  (the roster seats OpenRouter lists as reading pictures but whose transcription is unmeasured,
  kept off the picture route by the card hold `openrouter-pictures-unmeasured`;
  no card carries the hold since `gemma-4-26b-a4b-it` was measured on 2026-10-21 (UTC),
  so the set is empty until the next candidate is listed behind it).
  Declared in `synthetic-catalog.ts`,
  `openrouter-catalog.ts`,
  `bedrock-catalog.ts`,
  `neuralwatt-catalog.ts`,
  `direct-catalog.ts` (the DeepSeek and Xiaomi catalogues,
  one model each),
  `direct-reach.ts`,
  and `roster-reach.ts`.
- NeuralWatt free daily energy:
  meter the 550 Wh a day the owner's NeuralWatt account covers on DeepSeek V4.1 Flash and GLM-5.3-Flash,
  because nothing reports what is left and overage is the owner's plan
  (`doc/decision/translation-repair-neuralwatt-provider.md`).
  A durable ledger by NeuralWatt day (the 24 hours from the UTC time of day of the quota's
  `current_period_start`,
  or of `TRANSLATION_REPAIR_NEURALWATT_DAY_START`),
  floored by the account's covered energy from `GET /v1/usage/by-model`
  (tracking aliases such as `nw-flash` included,
  read off `GET /v1/models`),
  and held dry while the key's own allowance is blocked,
  a hold taken from the matrix mean times a margin before each call and replaced by the measured figure at settlement,
  and no stop on a reported cost,
  since NeuralWatt reports a nominal cost on free-energy requests too
  (the `SPEND` line still carries it).
  The run's plan energy is the difference of two quota readings;
  a first reading that fails is tried again no sooner than a minute later,
  and a comparison that starts at that later reading says so in the end report.
  Names:
  `createNeuralWattLedger`,
  `neuralWattLedgerFromEnv`,
  `estimateNeuralWattWh`,
  `NEURALWATT_ENERGY_MARGIN`,
  `neuralWattIsDry`,
  `NeuralWattAllowanceError`,
  `neuralWattDayStartFrom`,
  `neuralWattDayStartAt`,
  `coveredWhOf`,
  `aliasNamesOf`,
  `keyAllowanceOf`,
  `readNeuralWattComments`,
  `createNeuralWattQuotaSession`,
  `describePlanSpend`,
  `promptTokensOfRequest`,
  `imagePixelSizeOf`,
  `pictureTokensOfSize`.
  A picture in a request is held by its patches,
  round(width / 28) times round(height / 28) tokens read off the file header,
  not by its base64 length (ledger B720),
  and a NeuralWatt card side may name a `pictureReasoningEffort` and a `textReasoningEffort`
  that the client sends as `reasoning_effort` on a call that carries a picture and on one that does not
  (ledger B719 and B807).
  Declared in `neuralwatt-ledger.ts`,
  `neuralwatt-ledger-config.ts`,
  `neuralwatt-estimate.ts`,
  `neuralwatt-picture-tokens.ts`,
  `neuralwatt-image-size.ts`,
  `neuralwatt-meter.ts`,
  `neuralwatt-energy.ts`,
  `neuralwatt-day.ts`,
  `neuralwatt-day-anchor.ts`,
  `neuralwatt-day-session.ts`,
  `neuralwatt-day-energy.ts`,
  `neuralwatt-account-read.ts`,
  `neuralwatt-usage.ts`,
  `neuralwatt-aliases.ts`,
  `neuralwatt-key-allowance.ts`,
  `neuralwatt-comments.ts`,
  `neuralwatt-settlement.ts`,
  `neuralwatt-quota-session.ts`,
  and `neuralwatt-quota.ts`.
- Anthropic wire:
  build the request body and read the completion of the Anthropic messages format.
  Names:
  `buildAnthropicBody`,
  `extractAnthropicCompletion`,
  `contentBlocksFor`,
  `answerToolDefinition`,
  `requireWholeAnthropicMessage`.
  Declared in `anthropic-request.ts`,
  `anthropic-completion.ts`,
  `anthropic-content.ts`,
  `anthropic-tool.ts`,
  and `anthropic-whole-message.ts`.
- Chat contract and retry:
  the typed model caller,
  structured-answer calls,
  transient retries,
  call deadlines and stream bounds.
  Names:
  `ModelCaller`,
  `chatJsonThrough`,
  `readJsonOutcome`,
  `exchangeWithRetry`,
  `DEFAULT_RETRY_POLICY`,
  `armCallDeadline`,
  `armStreamBound`.
  Declared in `chat-contract.ts`,
  `chat-json-through.ts`,
  `chat-json-outcome.ts`,
  `transient-retry.ts`,
  `call-deadline.ts`,
  and `stream-bound.ts`.
- Spend and cost:
  report what a call cost and keep the credit ledgers.
  Names:
  `reportSpend`,
  `SPEND_MARKER`,
  `openRouterCostOf`,
  `bedrockCostOf`,
  `createBedrockLedger`,
  `estimateAbandonedSpend`.
  Declared in `spend-line.ts`,
  `openrouter-cost.ts`,
  `bedrock-cost.ts`,
  `bedrock-ledger.ts`,
  and `openrouter-abandoned-spend.ts`.

## What a refused credential holds out

A provider's HTTP 401 holds that provider out of every model for the rest of the run
(`credential-hold.ts`),
and the seat goes to its next serving provider in the same call.
One set of holds belongs to one router.
`createRoutingClient` takes the set as `credentialHolds`,
a fresh one by default,
and the run client (`run-config.ts`) hands the same set to its router and to its dryness view,
so the benches read a held provider as dry.
A corpus pass builds one run client for the whole run,
at the `input.newClient` call in `corpus-pass-run.ts`,
so it learns a 401 once per run.
The calibration and probe commands that build a client per arm,
slice or region learn it again for each,
and their clients do not share holds:
`roster-bench-run.ts`,
`producer-calibrate-run.ts`,
`checker-sensitivity-run.ts`,
`audit-sensitivity-run.ts`,
`probe-verify-collect.ts`,
`probe-relabel-pair.ts`
and `rendering-audit-settled-drive.ts`.
That limit is left as it stands;
sharing one set across those clients would need each command to build it.

## Family dependencies

Short names are directory names under `package/translation-repair`;
a package is named `@monochromatic-dev/translation-repair-<short name>`.

### Depends on

Production dependencies (`dependencies` in `package.json`):

- `document-structure`
- `glossary`
- `house-rule`
- `markup`
- `provider-stream`
- `shared-contract`
- `typography`

### Used by

Packages that list it in `dependencies`:

- `adjudication`
- `archive`
- `artifact`
- `bench`
- `consistency`
- `consolidate`
- `corpus-pass`
- `coverage-census`
- `lane-contest`
- `page-check`
- `probe`
- `reference`
- `rendering-audit`
- `repair`
- `run`
- `slice-reading`
- `stage`
- `translate-floor`
- `translate-lane`
- `translate-stage`

Packages that list it only in `devDependencies`:

- `slice-cache`

## Tests

Tests sit beside the sources as `*.unit.test.ts` and import built output:
this package through its own build,
`../dist/final/node/index.mjs`,
and the siblings they need through their package names,
which resolve to each sibling's built entry.

Build before testing:
`mise run //package/translation-repair/provider:build` builds this package,
and the library build keeps family siblings external,
so this package's built entry imports the siblings' built entries.
Its `build:js:node` task therefore declares `depends` on the `build:js:node` of every sibling listed in
`package.json`,
and the siblings build first.

Run the tests with `mise run //package/translation-repair/provider:test:unit`,
or build and test together with `mise run //package/translation-repair/provider:buildAndTest`.
Pass test file paths to run only those files.

Its `./ts/*` export also serves test fixtures that sibling tests import,
for example `provider-status-failure.test-fixture.ts`,
`stage-trivial-reply.test-fixture.ts`,
and `chat-message-reading.test-fixture.ts`.

Test helpers used across the family,
such as scratch directories and capturing loggers,
come from `test-fixture`,
imported as `@monochromatic-dev/translation-repair-test-fixture/ts/<file>`.

## Family docs

- [Family README](../README.md):
  the design,
  the package list and where the shared docs live.
- [Package split plan](../../../doc/planning/translation-repair-package-split.md):
  the split plan,
  with the package map and the rules the move keeps.

## DeepSeek and Xiaomi, called directly

`direct-client.ts` is one OpenAI-compatible client over a route (endpoint,
model lock,
response format,
price),
and `deepseek-client.ts` and `xiaomi-client.ts` are the two thin wrappers.
`createDeepSeekClient` also reads the account balance (`credits`,
parsed by `deepseek-balance.ts`),
and `createXiaomiClient` paces its calls to 100 a minute and reports the run's priced spend (`spend`).
Each client serves one model and throws `DeepSeekModelNotServedError` or `XiaomiModelNotServedError`
for any other before a request is built.
No request carries a reasoning member.
Prices are in `direct-cost.ts` (DeepSeek always at peak),
the cache split and `system_fingerprint` readers are in `direct-usage.ts`,
and the record is `doc/decision/translation-repair-direct-providers.md`.

## The owner's Claude key, called directly

`claude-client.ts` speaks the Messages protocol to `https://api.anthropic.com/v1/messages` with an `x-api-key` header
and `anthropic-version: 2023-06-01`,
for Sonnet 5.5 and Haiku 5.5 only
(`TRANSLATION_REPAIR_CLAUDE_API_KEY`,
about $200 a month,
no hard cap in code).
`claude-catalog.ts` and `claude-reach.ts` read the eight Claude cards of `shared-contract/src/claude-card-sides.ts`
(each model with no effort,
and at `low`,
`medium` and `high`),
and `cardFor` throws `ClaudeModelNotServedError` for any other model before a request is built.
`claude-body.ts` builds the request:
`thinking: {type: "adaptive", display: "summarized"}` always,
so the stream carries `thinking_delta` events while the model thinks
(the default `omitted` streams empty thinking blocks,
and the stream idle guard would read a long think as a stalled stream),
the card's effort in `output_config.effort`,
structured output by `output_config.format` and never a forced tool,
no `temperature`,
`top_p`,
`top_k` or assistant prefill,
and one ephemeral cache breakpoint on the system block.
`claude-schema.ts` narrows each stage schema to what structured outputs accept before it is sent:
`additionalProperties: false` on every object,
and the numeric,
string and array bounds dropped
and told to the model in the field's description (`{minimum: 1}`),
which the reply guards of the stages still enforce.
A keyword outside the supported list that no SDK strips (`oneOf`,
`not`,
`if`,
`pattern` and the like) throws `ClaudeSchemaKeywordError`,
naming the keyword and its path,
instead of answering HTTP 400 on every call of the stage.
A Claude call's deadline is the larger of the caller's and `claudeCallDeadlineMs` of its ceiling
(460,000 ms at the 32,000-token cap).
`claude-stream-facts.ts` reads the stop reason,
a refusal's category and the four usage counts,
`claude-cost.ts` prices them in USD (`SPEND` line),
and a `stop_reason` of `refusal` is its own outcome,
never an outage.
The client logs the `anthropic-ratelimit-*` headers once on the first reply that carries them,
paces itself at 1000 requests a minute,
and records a billing error (402,
or the older 400 naming a low credit balance) on itself,
which `claudeIsDry` reads as dry for the rest of the run (`claude-spend.ts`).
`retry-after-header.ts` and status 529 extend the retry ladder for Anthropic's 429 and 529 replies.
A `retry-after` past the ladder's reach rides on the failure as `retryAfterMs`,
and `statedWaitMsOf` takes the larger of that and the body's stated wait,
so the budget layer holds Anthropic out for the wait the header names.
The 401 guard shared with the DeepSeek and Xiaomi routes is `refused-key-body.ts`.
The record is the last section of `doc/decision/translation-repair-direct-providers.md`.

## Thinking-parameter evidence

The owner's rulings of 2026-10-22 decide which providers may be sent a thinking parameter:
none on third tier,
the documented ones on NeuralWatt (second rate),
and a value that disables thinking on none.
This section holds the measurements the card comments cite
(`shared-contract/src/neuralwatt-card-sides.ts`,
`synthetic-client.ts`),
so their figures can be checked.

### NeuralWatt `glm-5.3-flash`, 2026-10-22

A keyed probe of one digit-sum question,
text only,
three repeats at each setting.
Reasoning tokens in each repeat:

- No effort sent:
  3947,
  558,
  1144.
- `low`:
  143,
  116,
  118.
- `high`:
  169,
  222,
  182.
- `max`:
  555,
  579,
  355.

Every `low` repeat reasoned,
so `low` keeps thinking on.
`low` answered 2 of 3 right,
where `high`,
`max` and the unset default answered 3 of 3.
The limits are the probe's:
one question,
three repeats,
no picture call,
and the unset default's spread (558 to 3947) is wider than any gap between settings.

### Synthetic `glm-5.3-flash`, 2026-10-22

Synthetic's probes of the same day on the same question:
`low` turned thinking off,
and `high` and `max` did not separate,
at about 430 to 740 reasoning tokens each.
NeuralWatt's copy of the same model separated them
(`low` 116 to 143,
`high` 169 to 222,
`max` 355 to 579).
A setting that turns thinking off is not one to send,
and two settings that do not separate cannot size a call.
Only the ranges reached this record,
not each repeat's figure.
