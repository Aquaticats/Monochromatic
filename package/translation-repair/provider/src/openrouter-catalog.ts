import {
  holdSet,
  type OpenRouterServedId,
  type RosterModelId,
  servedRecord,
} from '@monochromatic-dev/translation-repair-shared-contract/ts';


//region OpenRouter catalog
// What OpenRouter serves for this pipeline, and how it spells it.
//
// WHY THIS PROVIDER EXISTS IN THE PIPELINE. The owner, 2026-09-03: Charm Hyper
// ended its bundle subsidization and will not be recharged, Synthetic and
// Hyper are expected to run dry often, and OpenRouter is where the owner would
// rather pay per token. It is the third and last provider in `PROVIDER_ORDER`
// (`doc/decision/translation-repair-openrouter-fallback.md`).
//
// IT SERVES NO MODEL THE ROSTER DOES NOT ALREADY NAME. Every row here is a
// third spelling of a seat Synthetic or Hyper already reaches, so `sharedWith`
// is never absent and the roster identity in `roster-id.ts` is unchanged.
// Provider is not part of panelist identity; a slice judged by
// `moonshotai/kimi-k3` counts once, as `hf:moonshotai/Kimi-K3`.
//
// THE OWNER'S ALLOWLIST ON THE ACCOUNT carries all nine as of 2026-09-03; a
// row here that the account no longer allows answers a refusal at the wire,
// which is the loud failure this catalog prefers over a silent substitution.
//
// FIELDS WERE READ OFF THE PUBLIC MODELS LISTING on 2026-09-03
// (`GET https://openrouter.ai/api/v1/models`, whose reading
// `doc/planning/translation-repair-openrouter-2026-09-03.md` records): `architecture.input_modalities`
// for pictures and `top_provider.max_completion_tokens` for the ceiling. Under
// zero data retention and `require_parameters` the endpoint actually chosen may
// cap lower; the ceiling is recorded as the listing's, and the client sends no
// `max_tokens` unless a caller sets one, exactly as the Synthetic client does.

/**
 OpenAI-compatible chat completions endpoint, measured live on 2026-09-03.

 CHAT COMPLETIONS RATHER THAN THE MESSAGES OR RESPONSES ENDPOINTS, by the
 probe recorded in `doc/planning/translation-repair-openrouter-2026-09-03.md`:
 it conformed on every attempt and answered fastest on every roster model,
 where the Messages endpoint answered Kimi-K3 eight times slower and
 conformed on 9 of 20 DeepSeek Flash attempts.
 */
export const OPENROUTER_CHAT_URL = 'https://openrouter.ai/api/v1/chat/completions';

/**
 Endpoint reporting credits purchased and used, measured live on 2026-09-03.

 ITS OWN PAGE SAYS A MANAGEMENT KEY IS REQUIRED; the ordinary inference key
 answered `200` with `{"data":{"total_credits":1913,"total_usage":1855.38}}`.
 The meter is built on the live behaviour; should the page become true, the
 meter reads as unreadable and the budget layer counts the provider as
 spendable, which is that layer's standing rule.
 */
export const OPENROUTER_CREDITS_URL = 'https://openrouter.ai/api/v1/credits';

/**
 Header carrying the key, a bearer token as on both other providers.
 */
export const OPENROUTER_AUTH_HEADER = 'Authorization';

/**
 Routing preferences every request carries in its `provider` field, before
 the per-model `ignore` list is added by `openRouterProviderPreferencesFor`.

 `zdr: true` IS THE OWNER'S DECISION of 2026-09-03: only endpoints with a
 zero-data-retention policy may serve a corpus passage, matching the stance
 taken on Charm Hyper. Every roster model kept at least seven such endpoints
 on the day of measurement.

 `require_parameters: true` keeps a request off any endpoint that does not
 support every parameter it carries, which for a schema'd call means
 `response_format`; an endpoint that ignored it would answer prose to a
 schema and cost a lost voice.

 `sort: 'price'` SINCE 2026-09-09, on the owner's instruction to stop
 bleeding. Between the top-up of 2026-09-08 and the refusal of 2026-09-09
 the default load balancing sent 3,894 of `deepseek-v4-pro-0813`'s calls
 to Parasail and CoreWeave at 1.85 and 2.11 times the listing's price
 (55.14 USD paid against 29.56 at listing), and the endpoints listing of
 2026-09-09 prices Baidu, Alibaba and DeepSeek at 0.58 to 0.66 USD per
 million in and 1.74 to 1.98 out against their 1.32 and 3.96. The routing
 page says sorting disables load balancing and picks the cheapest; the
 ignore list, zero data retention and `require_parameters` still apply,
 so the cheapest endpoint that keeps nothing and takes every parameter
 serves.

 @example
 ```ts
 const body = { model, messages, provider: openRouterProviderPreferencesFor({ servedId, },), };
 ```
 */
export const OPENROUTER_PROVIDER_PREFERENCES = {
  zdr: true,
  require_parameters: true,
  sort: 'price',
} as const;

/**
 The `provider` field as it goes on the wire for one model: the shared
 preferences plus that model's ignored endpoints.

 @example
 ```ts
 const preferences: OpenRouterProviderPreferences = openRouterProviderPreferencesFor({ servedId, },);
 ```
 */
export type OpenRouterProviderPreferences = typeof OPENROUTER_PROVIDER_PREFERENCES & {
  /**
   Provider slugs OpenRouter must not route this model to.
   */
  readonly ignore: readonly string[];

  /**
   Provider slugs OpenRouter tries first, in order, with fallbacks allowed;
   absent where the row names none (class ninety-three).
   */
  readonly order?: readonly string[];
};



/**
 Verified per-model facts the router and the request builder read.

 @example
 ```ts
 const info: OpenRouterModelInfo = OPENROUTER_MODELS['moonshotai/kimi-k3'];
 ```
 */
export type OpenRouterModelInfo = {
  /**
   Identifier sent in the request body's `model` field.
   */
  readonly id: OpenRouterServedId;

  /**
   The roster seat this spelling reaches, never absent here.
   */
  readonly sharedWith: RosterModelId;

  /**
   Whether the listing reports image input for this model.
   */
  readonly readsImages: boolean;

  /**
   Ceiling the listing's top provider reports for completion tokens.
   */
  readonly maxOutputLength: number;

  /**
   USD per million prompt tokens, as the public listing priced the model on
   2026-09-09 (`GET https://openrouter.ai/api/v1/models`); the
   estimate for an abandoned stream is priced off it.
   */
  readonly promptUsdPerMillion: number;

  /**
   USD per million completion tokens, from the same listing.
   */
  readonly completionUsdPerMillion: number;

  /**
   Provider slugs measured as serving this model badly, sent as
   `provider.ignore`; the owner warned on 2026-09-03 that "some providers
   might serve some models in a horribly broken way", and this is where a
   measured one is kept off the wire. Slugs are what `GET /api/v1/providers`
   lists, NOT the display name lower-cased: `OpenInference` is
   `open-inference`, and the day it was spelled without the hyphen the
   ignore silently did nothing (2026-09-04, 43 streams served). The catalog
   test checks every slug here against a snapshot of that listing.
   */
  readonly ignoredEndpoints: readonly string[];

  /**
   Provider slugs measured as serving this model well, sent as
   `provider.order` ahead of the price sort with fallbacks allowed; empty
   where the price sort alone decides. The routing page (read 2026-09-23)
   says the router tries the listed providers one at a time and proceeds to
   the others if none is operational, and that `sort` or `order` disables
   load balancing. Checked against the same listing snapshot.
   */
  readonly preferredEndpoints: readonly string[];

  /**
   Silence allowed before this model's first generated text, from its card,
   in place of the shared first-content window of the stream guard; absent
   where the card names none.
   */
  readonly firstContentMs?: number;
};

/**
 Every model this provider serves for this pipeline, read off the cards.

 CONFORMANCE MEASURED OVER 20 CHAT COMPLETIONS ATTEMPTS EACH on 2026-09-03
 with `response_format` json_schema, the schema restated in the system
 prompt, zero data retention and `require_parameters`; the per-model rates
 and the endpoints that served them are in the planning record. Later
 arrivals read their conformance off the fidelity probe's usable-ask count.

 @example
 ```ts
 const info = OPENROUTER_MODELS['deepseek/deepseek-v4.1-flash-1017'];
 ```
 */
export const OPENROUTER_MODELS: Readonly<Record<OpenRouterServedId, OpenRouterModelInfo>> = servedRecord({
  provider: 'openrouter',
  toRow: function openRouterRow(card,): OpenRouterModelInfo {
    /**
     OpenRouter's side of the card.
     */
    const { openrouter, } = card;
    return {
      id: openrouter.id,
      sharedWith: card.id,
      readsImages: openrouter.readsImages,
      maxOutputLength: openrouter.maxOutputLength,
      promptUsdPerMillion: openrouter.promptUsdPerMillion,
      completionUsdPerMillion: openrouter.completionUsdPerMillion,
      ignoredEndpoints: openrouter.ignoredEndpoints,
      preferredEndpoints: openrouter.preferredEndpoints,
      // Conditional spread keeps the window absent instead of undefined.
      ...((openrouter.firstContentMs === undefined) ? {} : { firstContentMs: openrouter.firstContentMs, }),
    };
  },
},);

/**
 The first-content window one served model's card names, as the field the
 exchange carries it in, or nothing where the card names none and the shared
 window of the stream guard stands (ledger B809).

 @param servedId - OpenRouter slug the request will name

 @returns `{ firstContentMs }` for a card that names a window, an empty record otherwise

 @example
 ```ts
 const exchange = { ...base, ...openRouterFirstContentWindowFor({ servedId: 'inception/mercury-2.5', },), };
 ```
 */
export function openRouterFirstContentWindowFor(
  { servedId, }: { readonly servedId: OpenRouterServedId; },
): { readonly firstContentMs?: number; } {
  /**
   Window the card names, absent where it names none.
   */
  const { firstContentMs, } = OPENROUTER_MODELS[servedId];
  // Conditional spread keeps the window absent instead of undefined.
  return (firstContentMs === undefined) ? {} : { firstContentMs, };
}

/**
 The `provider` field for one served model.

 COPIES THE IGNORE AND ORDER LISTS rather than aliasing the catalog's
 arrays, so the body builder can never hand the catalog's own row to
 `JSON.stringify` callers that might be tempted to push onto it. `order` is
 left off the wire where the row names no endpoint, so the gateway's own
 reading of an empty list never enters into it.

 @param servedId - OpenRouter slug the request will name

 @returns Shared preferences plus that model's ignored and named endpoints

 @example
 ```ts
 const provider = openRouterProviderPreferencesFor({ servedId: 'minimax/minimax-m3', },);
 ```
 */
export function openRouterProviderPreferencesFor(
  { servedId, }: { readonly servedId: OpenRouterServedId; },
): OpenRouterProviderPreferences {
  /**
   Catalog row for this slug.
   */
  const row = OPENROUTER_MODELS[servedId];
  /**
   Endpoints the row names ahead of the price sort.
   */
  const { preferredEndpoints, } = row;
  if (preferredEndpoints.length === 0) {
    return {
      ...OPENROUTER_PROVIDER_PREFERENCES,
      ignore: [...row.ignoredEndpoints,],
    };
  }
  return {
    ...OPENROUTER_PROVIDER_PREFERENCES,
    ignore: [...row.ignoredEndpoints,],
    order: [...preferredEndpoints,],
  };
}

/**
 Roster seats this provider stopped serving on 2026-09-09, named so the
 seat accounting and the tests can say which seats a dry Synthetic leaves
 unreachable rather than rediscovering it from an absent row.

 @example
 ```ts
 const dropped = OPENROUTER_DROPPED_SEATS.has('glm-5.3',);
 ```
 */
export const OPENROUTER_DROPPED_SEATS: ReadonlySet<RosterModelId> = holdSet({ hold: 'openrouter-dropped', },);

/**
 Roster seats this provider serves and the run does not buy from it, by the
 owner's decision of 2026-09-03 on cost (`moonshotai/kimi-k3` lists at 3 and
 15 USD per million against the anchor judge's 0.58 and 1.74).

 HONOURED BY THE REACH SINCE 2026-09-09, not only by the seat reader. Until
 then the seat reader withheld such a model at phase start when OpenRouter
 would serve it, and the router still re-routed a seated model there when its
 provider dried mid-phase: the third `noname` pass of that day bought 22
 Kimi-K3 calls on OpenRouter for 1.14 USD, 61 percent of what it spent
 there, two of them abandoned streams at 0.28 and 0.08 USD. With
 `reachOf` saying OpenRouter does not serve a withheld model, the router
 refuses the call as `NoProviderForModelError` and the seat is an
 unreachable one, which the select minimum of the same day sizes for.

 @example
 ```ts
 const withheld = OPENROUTER_WITHHELD.has('hf:moonshotai/Kimi-K3',);
 ```
 */
export const OPENROUTER_WITHHELD: ReadonlySet<RosterModelId> = holdSet({ hold: 'openrouter-withheld', },);

/**
 Roster seats OpenRouter's listing reports image input for and whose
 transcription has not been measured through it, so the reach sends them no
 picture there (`openrouter-pictures-unmeasured`). Unlike `reader-unmeasured`,
 this leaves a reader measured on another provider seated. No card carries
 the hold since `gemma-4-26b-a4b-it` was measured through OpenRouter on
 2026-10-21, so the set reads empty until `reader-measure` has another
 candidate to measure.

 @example
 ```ts
 const heldSeats = OPENROUTER_PICTURES_UNMEASURED.size;
 ```
 */
export const OPENROUTER_PICTURES_UNMEASURED: ReadonlySet<RosterModelId> = holdSet({
  hold: 'openrouter-pictures-unmeasured',
},);

//endregion OpenRouter catalog
