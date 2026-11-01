import {
  holdSet,
  type RosterModelId,
  servedRecord,
  type SyntheticServedId,
  type SyntheticVendorFamily,
} from '@monochromatic-dev/translation-repair-shared-contract/ts';


//region Synthetic model catalog
// Facts verified live on 2026-08-29 against `GET /openai/v1/models` (prices, context
// lengths, modalities, feature flags) and https://synthetic.new/rate-limits (weighting
// rule: requests are scaled by model input price; the baseline is the provider default
// model, currently Kimi-K3, counting as exactly one request). Weights here are
// planning estimates for routing; live budget truth always comes from `/quotas`.

/**
 OpenAI-compatible base URL for Synthetic chat completions,
 verified from the API overview page.

 @example
 ```ts
 const url = `${SYNTHETIC_CHAT_BASE_URL}/chat/completions`;
 ```
 */
export const SYNTHETIC_CHAT_BASE_URL = 'https://api.synthetic.new/openai/v1';

/**
 Quota endpoint URL;
 quota reads do not count against subscription limits (documented tip).

 @example
 ```ts
 const reply = await transport({ url: SYNTHETIC_QUOTAS_URL, method: 'GET', ... },);
 ```
 */
export const SYNTHETIC_QUOTAS_URL = 'https://api.synthetic.new/v2/quotas';

/**
 Verified per-model facts routing and budgeting read.

 @example
 ```ts
 const info: SyntheticModelInfo = SYNTHETIC_MODELS['hf:zai-org/GLM-5.3-Flash'];
 ```
 */
export type SyntheticModelInfo = {
  /**
   Model identifier sent in request bodies.
   */
  readonly id: SyntheticServedId;

  /**
   Vendor family for cross-family fan-out and rerouting.
   */
  readonly family: SyntheticVendorFamily;

  /**
   Whether this model can be sent an image alongside its text.

   READ FROM THE PROVIDER RATHER THAN ASSUMED. `GET
   https://api.synthetic.new/openai/v1/models` reports `input_modalities` per
   model, and the values here are that response as of 2026-08-29: three of the
   four Synthetic roster models read images. The provider's other vision
   entries, `syn:large:vision` and `syn:small:vision`, are aliases of two of
   those same three. The vision sub-roster is now EXACTLY THREE.

   WHY IT IS A FIELD RATHER THAN A FETCH: the rest of this catalog is static,
   a build that reached the network would fail closed on a provider outage,
   and a wrong value here fails loudly at the first call rather than quietly.
   */
  readonly readsImages: boolean;

  /**
   Context window in tokens.
   */
  readonly contextLength: number;

  /**
   Maximum output tokens per completion.
   */
  readonly maxOutputLength: number;

  /**
   Input price in dollars per token;
   request weighting derives from ratios of this field.
   */
  readonly promptDollarsPerToken: number;

  /**
   Output price in dollars per token;
   feeds weekly-credit spend estimates.
   */
  readonly completionDollarsPerToken: number;
};

/**
 Every always-on Synthetic chat model this pipeline may call, keyed by id
 and read off the cards. All entries support `json_mode` and
 `structured_outputs` per live feature flags, so schema-constrained calls
 need no per-model capability branching; client-side validation stays
 regardless because schema strictness is unverified.

 These are the models the provider offers that this pipeline seats. The models
 endpoint also lists `syn:large:text`, `syn:large:vision`, `syn:small:text`,
 and `syn:small:vision`, and those are DELIBERATELY ABSENT here: each is a
 moving alias, which the endpoint states in its `hugging_face_id` field.
 Live on 2026-08-29 they point to GLM-5.3-Flash, Kimi-K3, GLM-4.7-Flash,
 and Qwen3.8-27B respectively. An alias onto a seated model duplicates its
 vote; an alias onto excluded GLM-4.7-Flash bypasses owner's blocklist.

 TWO REASONS, AND THE SECOND IS THE SERIOUS ONE.

 Admitting an alias would let one model occupy two seats on a voting panel,
 so a single opinion would be counted as two independent confirmations and
 the adjudication tally would silently overstate agreement. Any future roster
 edit must dedupe on `hugging_face_id`, never on id.

 Worse, an alias is a promise the provider can move. This one is a very small
 operation with no service level agreement, no guaranteed support window, and
 partial support for the API surface it advertises. A repointed alias changes
 WHICH MODEL VOTES with nothing in this repository changing, no build failing
 and no log line saying so, which is the one class of roster change no guard
 here can catch. Naming models outright makes a retirement an HTTP 404 we can
 see rather than a substitution we cannot.

 One id was REPLACED 2026-08-29, `zai-org/GLM-5.2` by
 `hf:zai-org/GLM-5.3-Flash`, after the live endpoint confirmed the successor
 and the operational request reported Synthetic's plan to retire the older model. The endpoint reports
 the successor as always-on beta with text and image input, 524288-token
 context, 65536-token output, FP8 quantization, and tools, JSON mode,
 structured outputs, and reasoning. The pipeline pins the successor rather
 than following `syn:large:text`, which still resolved to the retiring model
 during this migration.

 One id was REPLACED 2026-08-20, `Qwen/Qwen3-point-6-27B` by
 `hf:Qwen/Qwen3.8-27B`, on notice that the provider would retire the older one
 shortly and offers no service level agreement, so a retirement lands without
 warning. The two are identical on every field this catalog records and on
 every capability flag the endpoint reports: context, output ceiling,
 modalities, features, quantization, always-on and price. The alias
 `syn:small:vision` still resolved to the OLDER id when this was written, so
 that alias breaks or repoints when the retirement lands; nothing here calls
 it, and deduping on `hugging_face_id` is what keeps that from mattering.

 Two ids were REMOVED 2026-08-05, `moonshotai/Kimi-K2.7-Code` and
 `MiniMaxAI/MiniMax-M3` (written without the `hf:` prefix here so a future
 bulk id rewrite cannot silently edit this sentence, which is exactly what
 happened once). Both now answer HTTP 404 "is no longer supported", and 404 is
 not in the transient retry set, so leaving them listed cost a lost voice per
 stage per call, silently.

 One id was REMOVED 2026-08-24, `zai-org/GLM-4.7-Flash` (again without the
 prefix), blocklisted by the owner. A measurement had found that it should stay,
 and the owner overruled that on a roster that later changed independently.
 It answers normally; nothing here calls it.

 One id was REMOVED 2026-08-29,
 `nvidia/NVIDIA-Nemotron-3-Super-120B-A12B-NVFP4`, at the owner's instruction
 after adjacent required-correction reviews produced contradictory guidance.
 Historical artifacts retain the exact departed identity, but no active
 roster or callable catalog row reaches it.

 @example
 ```ts
 const flash = SYNTHETIC_MODELS['hf:openai/gpt-oss-120b'];
 ```
 */
export const SYNTHETIC_MODELS: Readonly<Record<SyntheticServedId, SyntheticModelInfo>> = servedRecord({
  provider: 'synthetic',
  toRow: function syntheticRow(card,): SyntheticModelInfo {
    return card.synthetic;
  },
},);

/**
 Whether Synthetic serves a roster model at all.

 THE CHECK THE WIRE NEVER MADE. Five roster seats are Charm Hyper endpoint
 labels with no Synthetic spelling, and a client handed one of them used to
 send it anyway and collect an HTTP 400 per call. This is the one
 question the catalog can answer before a request is built.

 @param modelId - roster model a caller wants to address

 @returns Whether `SYNTHETIC_MODELS` has a row for it

 @example
 ```ts
 if (!syntheticServes(modelId,)) throw new SyntheticModelNotServedError({ modelId, },);
 ```
 */
export function syntheticServes(modelId: RosterModelId,): modelId is SyntheticServedId {
  return Object.hasOwn(
    SYNTHETIC_MODELS,
    modelId,
  );
}

/**
 Roster seats this provider serves and the run does not route here, on
 measured latency (class one hundred eighteen, 2026-09-24).

 HONOURED BY THE REACH, the way `OPENROUTER_WITHHELD` is: `reachOf` and
 `visionReachOf` say Synthetic does not serve a withheld model, so the router
 sends it to the next wet provider in `PROVIDER_ORDER` and never re-routes it
 here mid-phase. The catalog row stays, since Synthetic still lists the model
 and `syntheticServes` answers what the provider serves, not what the run buys.

 @example
 ```ts
 const withheld = SYNTHETIC_WITHHELD.has('hf:zai-org/GLM-5.3-Flash',);
 ```
 */
export const SYNTHETIC_WITHHELD: ReadonlySet<RosterModelId> = holdSet({ hold: 'synthetic-withheld', },);

//endregion Synthetic model catalog
