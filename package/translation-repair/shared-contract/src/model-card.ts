import type {
  AnthropicEffort,
  AnthropicServedId,
  AnthropicWireModel,
  BedrockServedId,
  DeepSeekServedId,
  HyperServedId,
  NeuralWattServedId,
  OpenRouterDecisionId,
  OpenRouterServedId,
  SyntheticServedId,
  XiaomiServedId,
} from './roster-id.ts';

//region Model card
// ONE RECORD PER ROSTER MODEL, from which every provider catalog, the
// completion cap table, the abandoned-spend ratio table and every seat hold
// derive. The owner asked for this on 2026-09-16 ("Adding / removing /
// changing models should never be this difficult"): before it, seating one
// model meant writing it into eight files, and unseating one meant finding
// them all again. Now a model is added by writing one card and removed by
// deleting it and adding a blocklist line (`roster-blocklist.ts`).
//
// THE CARD CARRIES FACTS AND HOLDS, NOT SEAT DECISIONS. Which model holds
// the third editor seat is a measurement recorded in `run-config.ts`; a card
// says where a model is served, what each provider reports for it, what its
// replies measured, and which roles it is held out of. This file holds the
// types alone so `model-cards.ts` reads as data.

/**
 Vendor family of a Synthetic-served model, for cross-family fan-out.

 @example
 ```ts
 const family: SyntheticVendorFamily = 'moonshot';
 ```
 */
export type SyntheticVendorFamily =
  | 'zai'
    | 'qwen'
    | 'moonshot'
    | 'openai';

/**
 Route a Bedrock model answers on, measured per model.

 @example
 ```ts
 const route: BedrockRoute = 'openai-v1';
 ```
 */
export type BedrockRoute = 'openai-v1' | 'v1';

/**
 How a Bedrock stream announces that it is whole, measured per model.

 @example
 ```ts
 const end: BedrockStreamEnd = 'done-sentinel';
 ```
 */
export type BedrockStreamEnd = 'done-sentinel' | 'usage-chunk';

/**
 Role a card is held out of, each on evidence recorded where the hold is read
 (`run-config.ts`, `run-seats.ts`, `openrouter-catalog.ts`).

 @example
 ```ts
 const hold: SeatHold = 'writer-unmeasured';
 ```
 */
export type SeatHold =
  /**
   No judge fidelity probe has seated it, so it holds no seat at all.
   */
  | 'judge-unmeasured'
  /**
   No producer calibration has measured its writing: no translator or
   consolidation seat.
   */
    | 'writer-unmeasured'
  /**
   No transcription has been measured: no picture-reader seat although a
   provider reports image input.
   */
    | 'reader-unmeasured'
  /**
   OpenRouter's listing reports image input for it, and no transcription has
   been measured through that route, so a picture is not sent there. The
   model-wide `reader-unmeasured` would also unseat a reader measured on
   another provider, which this does not. Lifted by `reader-measure`
   (`bench`) passing its two clauses on a keyed run; no card carries it since
   `gemma-4-26b-a4b-it` passed on 2026-10-21.
   */
    | 'openrouter-pictures-unmeasured'
  /**
   A producer calibration measured it out of the translator seat.
   */
    | 'translator-dropped'
  /**
   Out of every nine-wide seat (critic, panel, judge) for wall clock or by
   the owner's decision.
   */
    | 'wide-seat-dropped'
  /**
   Out of the roster-wide rounds after the lanes (contest, slate, gate).
   */
    | 'late-judge-dropped'
  /**
   OpenRouter serves it and the run does not buy it there, on cost.
   */
    | 'openrouter-withheld'
  /**
   OpenRouter stopped serving it for this run on 2026-09-09; named so the
   seat accounting can say so rather than infer it from an absent row.
   */
    | 'openrouter-dropped'
  /**
   Synthetic serves it and the run does not route it there: measured slower
   there than on another provider serving the same model.
   */
    | 'synthetic-withheld'
  /**
   Too slow in the select seats alone while Hyper serves it.
   */
    | 'hyper-slow-select'
  /**
   Out of every seat by the owner's decision, the card kept for the catalogs
   and the unit fixture: the seats go, the identity stays typed.
   */
    | 'owner-culled';

/**
 Completion cap for a model whose own calls were never measured: the pooled
 percentile the cap table falls back to.

 @example
 ```ts
 const cap: CompletionCapPool = 'pooled-p99';
 ```
 */
export type CompletionCapPool = 'pooled-p90' | 'pooled-p99';

/**
 What every provider reports for a model it serves.

 @example
 ```ts
 const served: ServedCard<HyperServedId> = { id: 'minimax-m3', readsImages: true, maxOutputLength: 512_000, };
 ```
 */
export type ServedCard<Id extends string> = {
  /**
   Identifier sent in the request body's `model` field.
   */
  readonly id: Id;

  /**
   Whether the provider reports image input for this model.
   */
  readonly readsImages: boolean;

  /**
   Ceiling the provider reports for completion tokens.
   */
  readonly maxOutputLength: number;
};

/**
 Synthetic's side of a card.

 @example
 ```ts
 const card: SyntheticCard = { id: 'hf:openai/gpt-oss-120b', family: 'openai', readsImages: false, contextLength: 131_072, maxOutputLength: 65_536, promptDollarsPerToken: 1e-7, completionDollarsPerToken: 1e-7, };
 ```
 */
export type SyntheticCard = ServedCard<SyntheticServedId> & {
  /**
   Vendor family for cross-family fan-out and rerouting.
   */
  readonly family: SyntheticVendorFamily;

  /**
   Context window in tokens.
   */
  readonly contextLength: number;

  /**
   Input price in dollars per token; request weighting derives from ratios
   of this field.
   */
  readonly promptDollarsPerToken: number;

  /**
   Output price in dollars per token; feeds weekly-credit spend estimates.
   */
  readonly completionDollarsPerToken: number;
};

/**
 OpenRouter's side of a card.

 @example
 ```ts
 const card: OpenRouterCard = { id: 'minimax/minimax-m3', readsImages: true, maxOutputLength: 512_000, promptUsdPerMillion: 0.3, completionUsdPerMillion: 1.2, ignoredEndpoints: ['parasail',], preferredEndpoints: [], rawCharsPerToken: 130, };
 ```
 */
export type OpenRouterCard = ServedCard<OpenRouterServedId> & {
  /**
   USD per million prompt tokens as the public listing priced it; the
   estimate for an abandoned stream is priced off it.
   */
  readonly promptUsdPerMillion: number;

  /**
   USD per million completion tokens, from the same listing.
   */
  readonly completionUsdPerMillion: number;

  /**
   Endpoint slugs measured as serving this model badly, sent as
   `provider.ignore`; slugs as `GET /api/v1/providers` lists them.
   */
  readonly ignoredEndpoints: readonly string[];

  /**
   Endpoint slugs measured as serving this model well, sent as
   `provider.order` ahead of the price sort with fallbacks allowed, for a
   seat whose cheapest endpoints reason at length by default (class
   ninety-three); empty where the price sort alone decides. Slugs as
   `GET /api/v1/providers` lists them.
   */
  readonly preferredEndpoints: readonly string[];

  /**
   Raw stream characters per completion token, the median over completed
   streams, or `'unmeasured'` for the median of the measured models.
   */
  readonly rawCharsPerToken: number | 'unmeasured';

  /**
   Silence allowed before this model's first generated text, in place of the
   shared first-content window of the stream guard (`STREAM_FIRST_CONTENT_MS`),
   when a measurement of this model alone supports a shorter one; absent where
   the shared window stands. It can only be shorter than the shared window: a
   card that needs a longer one is a decision about the shared window.

   THE PRECEDENT IS THE BEDROCK CARD'S `streamBoundMs`: a bound that comes from
   one model's own measurement, not one number for every call. The shared
   window is set against the providers that send no comment lines, so it says
   nothing of an OpenRouter model whose gateway writes `: OPENROUTER PROCESSING`
   while it waits for the upstream provider (ledger B804, B809).
   */
  readonly firstContentMs?: number;
};

/**
 Bedrock's side of a card.

 @example
 ```ts
 const card: BedrockCard = { id: 'google.gemma-4-e2b', readsImages: false, contextLength: 131_072, maxOutputLength: 131_072, route: 'openai-v1', streamEnd: 'done-sentinel', streamBoundMs: 60_000, promptUsdPerMillion: 0.04, completionUsdPerMillion: 0.08, };
 ```
 */
export type BedrockCard = ServedCard<BedrockServedId> & {
  /**
   Context window in tokens, as the model card states it.
   */
  readonly contextLength: number;

  /**
   Route this model answers on.
   */
  readonly route: BedrockRoute;

  /**
   How this model's stream announces that it is whole.
   */
  readonly streamEnd: BedrockStreamEnd;

  /**
   How long a call to this model may run before the provider is read as no
   longer serving it at its measured pace, or `'unbounded'` where no
   measurement separates a slow healthy call from a queued one (class one
   hundred forty-eight).
   */
  readonly streamBoundMs: number | 'unbounded';

  /**
   USD per million prompt tokens, off the public pricing page.
   */
  readonly promptUsdPerMillion: number;

  /**
   USD per million completion tokens, off the public pricing page.
   */
  readonly completionUsdPerMillion: number;
};

/**
 A reasoning effort NeuralWatt documents for `glm-5.3-flash`: its
 `supported_efforts` are `max`, `high` and `low`
 (`GET https://api.neuralwatt.com/v1/models`, `metadata.reasoning`, fetched
 2026-10-21; the default is `max`).

 @example
 ```ts
 const effort: NeuralWattReasoningEffort = 'low';
 ```
 */
export type NeuralWattReasoningEffort = 'low' | 'high' | 'max';

/**
 NeuralWatt's side of a card.

 @example
 ```ts
 const card: NeuralWattCard = { id: 'deepseek-v4.1-flash', readsImages: true, contextLength: 1_048_560, maxOutputLength: 393_216, };
 ```
 */
export type NeuralWattCard = ServedCard<NeuralWattServedId> & {
  /**
   Context window in tokens, as `GET /v1/models` lists it. The energy
   meter's prompt buckets end at this window.
   */
  readonly contextLength: number;

  /**
   Reasoning effort sent with a call that carries a picture, on this
   provider only. THE ONLY REQUEST PARAMETER ANY CARD CAN NAME, and it lives
   on this side alone, so no other provider's client has a field to read:
   the other providers cleared for thinking parameters have none measured yet:
   DeepSeek and Xiaomi (first rate), and Anthropic, which the owner ruled
   first rate although no direct Anthropic client exists yet (one is being
   built), so no client here sends it one. The rest are third tier. The owner, 2026-10-21: "NeuralWatt is a special case of all the
   providers: It implements what it documents correctly, almost always -
   I'm lifting the ban on passing params, but only for NeuralWatt." On
   2026-10-22: "Anthropic counts as first rate, neuralwatt counts as second
   rate", "NeuralWatt does implement what they document correctly, so
   they're clear on thinking params", and "we should never disable thinking
   on anything", so no value here turns thinking off. Absent where no
   picture reading on this model has been measured to need one.
   */
  readonly pictureReasoningEffort?: NeuralWattReasoningEffort;

  /**
   Reasoning effort sent with a call that carries no picture, on this
   provider only, under the same clearance (2026-10-21, "only for NeuralWatt";
   2026-10-22, second rate). Absent where the repo records no `supported_efforts` list
   for the model, so a call to it sends nothing. Never a field of another
   provider's card.
   */
  readonly textReasoningEffort?: NeuralWattReasoningEffort;
};

/**
 DeepSeek's own API's side of a card: the one model the owner's key there is
 granted for, priced the way DeepSeek's page prices it.

 @example
 ```ts
 const card: DeepSeekCard = { id: 'deepseek-flash', readsImages: true, maxOutputLength: 393_216, promptCacheHitUsdPerMillion: 0.006, promptCacheMissUsdPerMillion: 0.3, completionUsdPerMillion: 1.2, };
 ```
 */
export type DeepSeekCard = ServedCard<DeepSeekServedId> & {
  /**
   USD per million prompt tokens served from DeepSeek's cache, at the peak
   price (the upper bound: peak is 01:00 to 04:00 UTC and 06:00 to 10:00 UTC,
   Monday to Friday, excluding Chinese public holidays, and the run cannot
   know the holidays, so every call is priced as if at peak).
   */
  readonly promptCacheHitUsdPerMillion: number;

  /**
   USD per million prompt tokens not served from the cache, at the peak price.
   */
  readonly promptCacheMissUsdPerMillion: number;

  /**
   USD per million completion tokens, thinking included, at the peak price.
   */
  readonly completionUsdPerMillion: number;
};

/**
 Xiaomi's own API's side of a card: the one model the owner's key there is
 granted for, priced the way Xiaomi's pay-as-you-go page prices it.

 @example
 ```ts
 const card: XiaomiCard = { id: 'mimo-v2.6-flash', readsImages: true, maxOutputLength: 131_072, promptCacheHitUsdPerMillion: 0.0028, promptCacheMissUsdPerMillion: 0.14, completionUsdPerMillion: 0.28, };
 ```
 */
export type XiaomiCard = ServedCard<XiaomiServedId> & {
  /**
   USD per million prompt tokens served from Xiaomi's cache.
   */
  readonly promptCacheHitUsdPerMillion: number;

  /**
   USD per million prompt tokens not served from the cache.
   */
  readonly promptCacheMissUsdPerMillion: number;

  /**
   USD per million completion tokens, thinking included.
   */
  readonly completionUsdPerMillion: number;
};

/**
 One tier of Anthropic's per-token prices, in USD per million tokens.

 @example
 ```ts
 const tier: AnthropicPriceTier = { inputUsdPerMillion: 2, outputUsdPerMillion: 10, cacheWriteUsdPerMillion: 2.5, cacheHitUsdPerMillion: 0.1, };
 ```
 */
export type AnthropicPriceTier = {
  /**
   USD per million prompt tokens past the last cache breakpoint.
   */
  readonly inputUsdPerMillion: number;

  /**
   USD per million completion tokens, thinking included.
   */
  readonly outputUsdPerMillion: number;

  /**
   USD per million prompt tokens written to the five-minute cache.
   */
  readonly cacheWriteUsdPerMillion: number;

  /**
   USD per million prompt tokens read back from the cache.
   */
  readonly cacheHitUsdPerMillion: number;
};

/**
 Anthropic's prices for one model: a base tier and, where the page prices a long prompt apart, the tier above a
 prompt size.

 @example
 ```ts
 const prices: AnthropicPrices = { inputUsdPerMillion: 0.1, outputUsdPerMillion: 0.5, cacheWriteUsdPerMillion: 0.125, cacheHitUsdPerMillion: 0.01, longPrompt: { aboveTokens: 100_000, inputUsdPerMillion: 0.5, outputUsdPerMillion: 2.5, cacheWriteUsdPerMillion: 0.625, cacheHitUsdPerMillion: 0.05, }, };
 ```
 */
export type AnthropicPrices = AnthropicPriceTier & {
  /**
   Tier that prices the whole call once its prompt (input, cache write and cache read together) is over
   `aboveTokens`; absent where the page prices every prompt size alike.
   */
  readonly longPrompt?: AnthropicPriceTier & {
    /**
     Prompt size past which this tier prices the whole call.
     */
    readonly aboveTokens: number;
  };
};

/**
 Anthropic's own API's side of a card: one of the two models the owner's Claude key is granted for, at one effort or
 at the model's own default.

 @example
 ```ts
 const card: AnthropicCard = { id: 'claude-haiku-5-5-effort-low', wireModel: 'claude-haiku-5-5', effort: 'low', readsImages: true, maxOutputLength: 128_000, prices, };
 ```
 */
export type AnthropicCard = ServedCard<AnthropicServedId> & {
  /**
   Model id the request carries in its `model` member: one of the two the key is granted for.
   */
  readonly wireModel: AnthropicWireModel;

  /**
   Effort sent as `output_config.effort`; absent sends none, which is the model's own default (Sonnet `high`, Haiku
   `medium`).
   */
  readonly effort?: AnthropicEffort;

  /**
   Prices the call is reckoned at.
   */
  readonly prices: AnthropicPrices;
};

/**
 The decisions endpoint's side of a card: a typed-decision model reached
 through OpenRouter's `/api/alpha/decisions`, which answers a choice, a
 yes-probability or a score and never a completion. A card carrying this
 side and no chat side is a decision-only seat: no chat catalog lists it,
 no chat bench seats it, and only the stages with a decision adapter ask it.

 @example
 ```ts
 const card: DecisionsCard = { id: 'typesafe/jev-1.13', contextLength: 32_000, promptUsdPerMillion: 0.042, completionUsdPerMillion: 0, };
 ```
 */
export type DecisionsCard = {
  /**
   Identifier sent in the request body's `model` field.
   */
  readonly id: OpenRouterDecisionId;

  /**
   Tokens of state plus the longest question one request may carry, as the
   vendor's models page states it.
   */
  readonly contextLength: number;

  /**
   USD per million input tokens as the listing priced it.
   */
  readonly promptUsdPerMillion: number;

  /**
   USD per million output tokens, zero where the listing charges none.
   */
  readonly completionUsdPerMillion: number;
};

/**
 One roster model: every provider's view of it, its measured completion
 cap and the roles it is held out of. Its identity is the key it sits under
 in `MODEL_CARDS`.

 @example
 ```ts
 const card: ModelCard = MODEL_CARDS['minimax-m3'];
 ```
 */
export type ModelCard = {
  /**
   Synthetic's entry, where it serves the model.
   */
  readonly synthetic?: SyntheticCard;

  /**
   Charm Hyper's entry, where it serves the model.
   */
  readonly hyper?: ServedCard<HyperServedId>;

  /**
   OpenRouter's entry, where it serves the model.
   */
  readonly openrouter?: OpenRouterCard;

  /**
   Amazon Bedrock's entry, where it serves the model.
   */
  readonly bedrock?: BedrockCard;

  /**
   NeuralWatt's entry, where it serves the model on the standard lane.
   */
  readonly neuralwatt?: NeuralWattCard;

  /**
   DeepSeek's own API's entry, on the one card the owner's key there is
   granted for.
   */
  readonly deepseek?: DeepSeekCard;

  /**
   Xiaomi's own API's entry, on the one card the owner's key there is granted
   for.
   */
  readonly xiaomi?: XiaomiCard;

  /**
   Anthropic's own API's entry, on the eight cards the owner's Claude key is granted for.
   */
  readonly anthropic?: AnthropicCard;

  /**
   OpenRouter's decisions endpoint's entry, where it serves the model as a
   typed-decision model rather than a chat one.
   */
  readonly decisions?: DecisionsCard;

  /**
   Completion token ceiling every client sends as `max_tokens`: the highest
   99th percentile any provider measured, or a pooled percentile until the
   model's own calls are read (`completion-cap.ts` has the method).
   */
  readonly completionCap: number | CompletionCapPool;

  /**
   Roles held out of, on the evidence recorded where each hold is read.
   */
  readonly holds: readonly SeatHold[];
};

//endregion Model card
