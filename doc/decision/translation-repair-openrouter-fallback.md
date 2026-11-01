# Translation repair: OpenRouter is the paid fallback provider

Decided by the owner on 2026-09-03,
answering four separable questions.
Evidence and method:
`doc/planning/translation-repair-openrouter-2026-09-03.md`.
Extends `doc/decision/translation-repair-multi-provider.md`,
which stays the record for Synthetic and Charm Hyper.

## Why

The owner,
2026-09-03:
Charm Hyper ended its bundle subsidization,
so Hyper will not be recharged from the usual subscription;
OpenRouter is preferred for paying per token,
and Synthetic and Hyper are expected to run dry often.
`TRANSLATION_REPAIR_OPENROUTER_API_KEY` names the key.

## The owner's answers

- **Realtime,
  not batch.**
    A corpus pass is a chain of dependent rounds built on streaming quorum and grace,
    and OpenRouter batch's only completion window is 24 hours.
    The listed batch prices for this roster are never below the realtime prices,
    and two roster models have no batch entry.
- **Kimi-K3 is withheld where only OpenRouter would buy it.**
    At 3 and 15 USD per million tokens it was 52 to 61 percent of an entry's all-OpenRouter cost.
    It keeps every seat while Synthetic or Hyper serves it.
    This is the shape of the existing slow-judge withholding (`run-seats.ts`),
  keyed to reach rather than to dryness alone.

    CORRECTION,
    same day:
    the question told the owner Kimi-K3 "holds editor,
    refiner and checker seats".
    That was wrong.
    `run-config.ts` seats the editors as GLM-5.3-Flash,
    `glm-5.3` and `deepseek-v4-pro-0813`
    and the refiners as GLM-5.3-Flash,
    `deepseek-v4-pro-0813` and `minimax-m3`;
    Kimi-K3 holds the critic,
    panel,
    select-judge,
    late-judge and checker seats.
    The cost figure the answer rests on is unchanged.
    What thins on an all-OpenRouter day is each wide bench by one voice and the checker roster from three to two,
    which `assertCheckerQuorumReachable` refuses,
    so a disinterested substitute checker is seated there:
    `gemma-4-26b-a4b-it`,
    chosen because it holds no editor or refiner seat,
    answered 40 of 40 writer rounds with zero cuts and threw 5 of 289 asks on the stub-fix XIEPT2 run,
    and sat above the pooled null as a writer where `deepseek-v4-flash-0731`,
    the other disinterested candidate,
    sat below it in both writing measurements.
    That seat is provisional and open to the owner's veto;
    no checker-side measurement exists for any model.
- **The allowlist carries the whole roster.**
    The owner added `openai/gpt-oss-120b`,
  `google/gemma-4-26b-a4b-it` and `deepseek/deepseek-v4-pro-0813`
    to the account's model allowlist,
  so every roster model is buyable on OpenRouter.
    The question described gemma as a writer seat and the "add none" option as leaving "the editor stage on Qwen alone";
    gemma is a translator and Qwen3.8-27B a checker,
  and the editor stage was never at stake.
    The answer stands on its own terms:
  every roster model is now buyable there.
- **Zero data retention on every request.**
    `provider.zdr: true` goes on every call;
    every roster model kept at least seven ZDR endpoints on the day of measurement.

## Standing rules that follow

- Routing order:
  Synthetic while wet,
  then Charm Hyper while its balance lasts,
  then OpenRouter.
- Transport:
  OpenAI chat completions with `response_format` json_schema,
    the same body the Synthetic client sends,
  read by the same stream reader
    (chosen by measurement;
  the "What landed" section has the numbers).
    `provider.require_parameters: true` restricts routing to endpoints that accept `response_format`;
    each catalog row's `ignoredEndpoints` goes out as `provider.ignore` and names any endpoint that measures badly.
    A model is seated on this provider only after a measured conformance probe,
  as on Hyper.
- Meter:
  `GET /api/v1/credits`,
  balance as purchased minus used;
    dry at or below zero,
  unreadable counts as spendable,
  `402` and `429` are refusal holds.
    The endpoint's page says a management key is required;
  the ordinary key answered `200` on 2026-09-03,
    and the meter is built on that live behaviour.
- Gemini 3.8 Flash is blocklisted on the owner's words:
  "a wildly misaligned model".
- `:free` variants are never used.
- Auto top-up recommended to the owner:
  threshold 20 USD,
  amount 20 USD a day's spend without Kimi on OpenRouter.

## What landed, 2026-09-03

Measurements and the build record are in `doc/planning/translation-repair-openrouter-2026-09-03.md`.

- **Transport:
  chat completions**,
  by the owner's mid-session suggestion and by measurement.
    The three-transport probe (twenty attempts per roster model on chat completions,
  the Anthropic
    Messages endpoint and the Responses endpoint,
  every request under zero data retention and
    `require_parameters`) conformed on every chat completions attempt and answered fastest there;
    Messages answered Kimi-K3 eight times slower and conformed on 9 of 20 DeepSeek Flash attempts.
    The client (`openrouter-client.ts`) is the Synthetic body plus the `provider` field,
  read by the same
    stream reader;
  the Anthropic readers also learned to skip the `[DONE]` sentinel OpenRouter appends,
    since the Messages endpoint stays available for measurement.
- **Roster on this provider**:
  all nine seats,
  as third spellings of existing roster identities
    (`openrouter-catalog.ts`);
  no new roster name.
    gemma's row reports no pictures until a transcription is measured,
  so the reader roster stays four.
- **Routing**:
  `PROVIDER_ORDER` (Synthetic,
  Hyper,
  OpenRouter) walked by `routeProviderFor`;
    a refusal re-routes at most once per provider;
  the non-conformant re-ask goes to the next wet provider
    serving the model;
  the all-dry error is `EveryProviderDryError`.
- **Seats**:
  derived from where each model would be served (`providerServing`);
    Kimi-K3 withheld everywhere OpenRouter would serve it,
  gemma seated as the substitute checker there.
- **Accounting**:
  `METERS` carries `openrouter=` and `openrouterUsd=`;
  `SPEND` lines carry `cost=` in
    USD from the wire;
  `spend-cost` keeps an OpenRouter USD bucket apart from hypercredits.
- **Width**:
  32 concurrent chat completions per model completed 32 of 32 on two models,
  no refusal;
    the client carries no per-model ceiling by default.
- **GPT-5.6 Luna**,
  named by the owner as Responses-only,
  is not viable under zero data retention as
    measured (Messages 404 on data policy;
  chat and Responses 9 and 11 of 20 with empty answers on the rest)
    and is not seated.
- **Parasail is ignored for MiniMax M3** (`7d680d7fa`):
  on the first live pass with OpenRouter in the order
    (keyword233,
  16:38 to 16:54 UTC) 16 of 31 MiniMax calls came back with an empty content channel,
  and the
    per-endpoint probe showed Parasail writing the whole JSON answer into the reasoning channel while ModelRun
    answered every attempt.
  This is the owner's "some providers might serve some models in a horribly broken way",
    met on the first day,
  and the ignore list is the standing remedy:
  an endpoint measured broken for a model is
    named in that model's catalog row,
  with the measurement beside it.
- **The first live pass settled** in 958 seconds for three slices,
  inside the band of the day's earlier
    keyword233 runs,
  with 146 OpenRouter calls at 0.38 USD,
  111 Synthetic calls,
  no refusal,
  and
    `verify-published` at 1 of 1.
  The all-dry benches and Qwen3.8-27B served by OpenRouter were not exercised,
    since Synthetic stayed wet.
- **OpenInference is ignored for DeepSeek V4 Flash** (`08dffd481`):
  on the second live pass (keyword233,
  17:06 to
    17:28 UTC),
  with the serving endpoint now on every stream line,
  OpenInference finished 2 of its 6 streams
    against Parasail's 12 of 13 and Inceptron's 4 of 5,
  every cut being the straggler grace ending a stream still
    in its reasoning channel.
  The same reading left DeepInfra in place for gemma 4 26B (9 of 10 finished,
  and
    SiliconFlow would otherwise serve alone at the end of the provider order) and left Modal alone for glm-5.3
    on one slow sample;
  the planning record carries the figures.
- **The second live pass settled** in 1,293 seconds,
  above the day's band,
  with the lanes phase carrying the
    difference during a Synthetic 502/500 burst;
  the Parasail ignore held (36 of 36 MiniMax calls to ModelRun,
    all completed),
  135 OpenRouter calls cost 0.45 USD,
  and `verify-published` answered 1 of 1.
- **Rejected:
  re-routing on an exhausted transient-retry ladder.**
  Measured before building:
  no exhausted ladder
    on the four-entry Carena run,
  one voice lost that way in the whole archive,
  none on either OpenRouter pass.
- **The OpenRouter-only bench ran live** (keyword233,
  18:15 to 18:36 UTC,
  Synthetic and Hyper keys unset):
    settled in 1,248 seconds with `withheld=hf:moonshotai/Kimi-K3` and gemma as the substitute checker at every
    phase,
  0.76 USD over 235 costed calls,
  `verify-published` 1 of 1,
  Qwen3.8-27B served by OpenRouter usable on
    31 of 31 completed answers.
- **The withhold reached only the judge benches** until `8848f070e`:
  that pass bought six Kimi-K3 translations
    on OpenRouter,
  a quarter of its bill,
  because the translator roster and the picture readers were the static
    lists.
  `JudgeSeats` now carries `translators` and `readers` filtered by the same rule,
  and the picture stage
    reads its own seats.
  The next pass then bought the model's first call from the block-pairing round,
  which
    took the static roster as insertion admission and the consolidation writers did,
  so `68ad11530` adds
    `roster` to `JudgeSeats` and every roster-wide stage takes it from its own reading.
  The Kimi-K3 rule in
    "Standing rules that follow" means every call,
  not every judge call.
  Verified live on the fifth pass
    (keyword233,
  19:58 to 20:21 UTC,
  OpenRouter alone):
  no Kimi-K3 call from preparation to consolidation,
    page verified,
  0.50 USD.
- **The straggler grace is 120 seconds**,
  the owner's decision of 2026-09-03 on a measured pair:
  every cut on
    the 60 s pass was a reasoning-only stream still working (Qwen 7 of 38,
  DeepSeek Flash 6 of 27),
  and the
    120 s pass halved the cuts (14 to 7) and the voices never heard (19 to 8) for a wall-clock difference
    inside the day's band.
  No ignore fits,
  since Parasail's completed latencies match Synthetic's for the same
    model.
  Record:
  `doc/decision/translation-repair-straggler-grace.md`,
  "Decision 2026-09-03".

## Rollback

Seating Kimi-K3 on OpenRouter,
or turning ZDR off,
needs an explicit owner decision and fresh cost evidence.

## Addendum 2026-09-07: Amazon Bedrock sits ahead of Charm Hyper

Decided by the owner on 2026-09-07 ("Yes Bedrock sit ahead"),
answering the order question the fourth-provider record left open to veto
(`doc/planning/translation-repair-openrouter-2026-09-03.md`,
"The owner adds Amazon Bedrock" and "The first hakureico launch was on the wrong roster").
`PROVIDER_ORDER` reads Synthetic,
Bedrock,
Hyper,
OpenRouter.

Why:
Bedrock is 200 USD of prepaid credits that expire early next year and will never be topped up,
Hyper is a prepaid balance the owner will not recharge either,
and the two seats both serve
(`gemma-4-26b-a4b-it`,
`gpt-oss-120b`)
had gone to Hyper the moment Synthetic dried on the second hakureico launch,
108 credits in one pass,
while the expiring money answered nothing.
Ahead of Hyper,
those seats spend Bedrock first,
and Hyper's balance is kept for the seats only Hyper serves.
Landed with the probe candidate flags;
the running pass was killed and relaunched under the kill-and-relaunch rule.

Rollback:
swap the two names in `provider-name.ts`;
the routing arithmetic walks the list and names no provider by position.

## Addendum 2026-09-24: LLM Gateway DevPass stays out of the pipeline

The owner subscribed to LLM Gateway DevPass on 2026-09-24,
added `LLMGATEWAY_DEVPASS_API_KEY` to the sops file,
and asked for it to sit ahead of OpenRouter.
Read before wiring,
the DevPass Supplemental Terms of Use
(`https://devpass.llmgateway.io/legal/terms`,
last updated 2026-09-18) say:

- Section 1 and section 4,
  in force today:
  the key may not be used "directly from your own applications,
  backends,
  products,
  services,
  scripts,
  batch pipelines,
  or any other integration outside of an approved coding or agent tool".
- Section 1 and section 4,
  from billing periods starting on or after 2026-10-15:
  translation is named among the excluded non-coding uses,
  and API,
  automation or non-coding use can ban the account without a refund.
- Provider-prefixed model ids are refused with HTTP 403 on DevPass
  (`https://docs.llmgateway.io/learn/model-categories`),
  so the `preferredEndpoints` routing of class ninety-three cannot be sent.
- Nothing promises zero data retention upstream;
  the only upstream control is the "No AI training" setting.
  This decision's standing rule is ZDR on every request,
  and dropping it needs an explicit owner decision.

Asked on 2026-09-24,
the owner chose to leave DevPass out.
The key stays in the sops file unused;
nothing in `package/translation-repair` reads it.
The terms allow a self-service refund within 14 days of purchase while usage stays under 20 percent of the allowance.
Rejected alternatives:
wiring DevPass in anyway (terms,
no ZDR,
no endpoint pinning);
LLM Gateway pay-as-you-go,
the general product that does allow apps,
unmeasured on price and ZDR,
so it is not a candidate until both are measured.
