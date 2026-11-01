import {
  type NeuralWattReasoningEffort,
  type NeuralWattServedId,
  type RosterModelId,
  servedRecord,
} from '@monochromatic-dev/translation-repair-shared-contract/ts';

//region NeuralWatt catalog
// What NeuralWatt serves for this pipeline, how it spells it and where it is
// called.
//
// WHY THIS PROVIDER EXISTS IN THE PIPELINE. The owner, 2026-10-20: "I just
// added TRANSLATION_REPAIR_NEURALWATT_API_KEY . It includes 550Wh of energy on
// DeepSeek V4.1 Flash and GLM 5.3 Flash per day. And we need to meter (using
// estimations) ourselves because overage eats into my normal plan which I use
// for other stuff." Then: "Use the neuralwatt free energy front and center."
// It is the first provider in `PROVIDER_ORDER`: free energy that does not
// carry over is the most expiring money there is. Record:
// `doc/decision/translation-repair-neuralwatt-provider.md`.
//
// THE API IS OPENAI-COMPATIBLE: base `https://api.neuralwatt.com/v1`, a bearer
// key, `POST /v1/chat/completions` with streaming
// (`https://docs.neuralwatt.com`, fetched 2026-10-20). Raw fetch over the same
// transport seam as every other client, no SDK, as the owner's standing
// instruction for the Bedrock client already was.
//
// ONLY THE STANDARD IDS (`GET /v1/models`, public, fetched 2026-10-20):
// `deepseek-v4.1-flash` (context 1,048,560, maximum output 393,216, 0.15 and
// 0.60 USD per million tokens) and `glm-5.3-flash` (context 1,048,560, no
// listed output ceiling, 0.15 and 0.50). Not the `-flex` ids, which are billed
// at 0.65 of the price and "may be held under load"; not `-speed`, which the
// free energy does not cover; not the `nw-flash` alias. The free energy "covers
// each covered model's standard and Flex tiers" and "Speed lanes are billed
// normally" (`https://docs.neuralwatt.com/billing/daily-free-energy`).
//
// REASONING IS ON BY DEFAULT on both models (GLM-5.3-Flash always, at the
// listing's default effort of `max`; DeepSeek V4.1 Flash at `high`). A call
// that sends no effort spends whatever thinking the default spends, and the
// matrix the meter estimates from was drawn over traffic that ran the same
// defaults, so a call at a shallower effort uses less than its hold.
//
// NEURALWATT IS SECOND RATE AND CLEARED FOR ITS DOCUMENTED THINKING PARAMETERS.
// The owner, 2026-10-21: "NeuralWatt is a special case of all the providers: It
// implements what it documents correctly, almost always - I'm lifting the ban
// on passing params, but only for NeuralWatt." And on 2026-10-22: "No, I'm only
// against thinking params on 3rd tier providers." "Anthropic counts as first
// rate, neuralwatt counts as second rate." "NeuralWatt does implement what they
// document correctly, so they're clear on thinking params." "But we should
// never disable thinking on anything." The 2026-08-25 instruction ("don't set
// any thinking parameter or budget tokens", recorded at the Synthetic body)
// now holds for third tier alone; the other cleared providers are DeepSeek and
// Xiaomi, first rate, and Anthropic, which the owner ruled first rate before
// any direct Anthropic client exists (one is being built).
//
// WHAT THIS PROVIDER IS SENT, only where the card's NeuralWatt side names one:
// `reasoning_effort`, `low` on GLM-5.3-Flash for a call that carries a picture
// (`pictureReasoningEffort`) and for one that does not (`textReasoningEffort`),
// and none on DeepSeek V4.1 Flash, whose supported efforts the repo does not
// record. The clearance does not reach a value the listing does not name, so
// that is a gap in the record and not a ban. `low` keeps thinking on, so it
// complies with "never disable thinking"; no value that turns thinking off is
// sent.

/**
 Base address every NeuralWatt route hangs off.
 */
export const NEURALWATT_BASE_URL = 'https://api.neuralwatt.com/v1';

/**
 Header carrying the key, a bearer token as on the other providers.
 */
export const NEURALWATT_AUTH_HEADER = 'Authorization';

/**
 Verified per-model facts the router and the request builder read.

 @example
 ```ts
 const info: NeuralWattModelInfo = NEURALWATT_MODELS['deepseek-v4.1-flash'];
 ```
 */
export type NeuralWattModelInfo = {
  /**
   Identifier sent in the request body's `model` field.
   */
  readonly id: NeuralWattServedId;

  /**
   The roster seat this spelling reaches.
   */
  readonly sharedWith: RosterModelId;

  /**
   Whether a picture may be sent here: the listing's image input.
   */
  readonly readsImages: boolean;

  /**
   Context window in tokens, as the listing states it.
   */
  readonly contextLength: number;

  /**
   Ceiling the listing states for completion tokens, or the context window
   where it states none.
   */
  readonly maxOutputLength: number;

  /**
   Reasoning effort a call carrying a picture sends, when the card names
   one. Read by this provider's client alone; no other provider's catalog
   row has the field.
   */
  readonly pictureReasoningEffort?: NeuralWattReasoningEffort;

  /**
   Reasoning effort a call carrying no picture sends, when the card names
   one. Read by this provider's client alone, like the picture effort.
   */
  readonly textReasoningEffort?: NeuralWattReasoningEffort;
};

/**
 Every model this provider serves for this pipeline, read off the cards.

 @example
 ```ts
 const info = NEURALWATT_MODELS['glm-5.3-flash'];
 ```
 */
export const NEURALWATT_MODELS: Readonly<Record<NeuralWattServedId, NeuralWattModelInfo>> = servedRecord({
  provider: 'neuralwatt',
  toRow: function neuralWattRow(card,): NeuralWattModelInfo {
    /**
     NeuralWatt's side of the card.
     */
    const { neuralwatt, } = card;
    return {
      id: neuralwatt.id,
      sharedWith: card.id,
      readsImages: neuralwatt.readsImages,
      contextLength: neuralwatt.contextLength,
      maxOutputLength: neuralwatt.maxOutputLength,
      // Conditional spread keeps the knob absent instead of undefined.
      ...((neuralwatt.pictureReasoningEffort === undefined)
        ? {}
        : { pictureReasoningEffort: neuralwatt.pictureReasoningEffort, }),
      // Conditional spread keeps the knob absent instead of undefined.
      ...((neuralwatt.textReasoningEffort === undefined)
        ? {}
        : { textReasoningEffort: neuralwatt.textReasoningEffort, }),
    };
  },
},);

/**
 Chat completions URL.

 @param baseUrl - base address, overridable for tests

 @returns Absolute URL to POST the body to

 @example
 ```ts
 neuralWattChatUrlFor({ baseUrl: NEURALWATT_BASE_URL, },);
 // => 'https://api.neuralwatt.com/v1/chat/completions'
 ```
 */
export function neuralWattChatUrlFor({ baseUrl, }: { readonly baseUrl: string; },): string {
  return `${baseUrl}/chat/completions`;
}

/**
 Quota URL, which reports the plan's charged energy and says nothing about
 the daily free energy (`https://docs.neuralwatt.com/api/quota`, fetched
 2026-10-20).

 @param baseUrl - base address, overridable for tests

 @returns Absolute URL to GET

 @example
 ```ts
 neuralWattQuotaUrlFor({ baseUrl: NEURALWATT_BASE_URL, },);
 // => 'https://api.neuralwatt.com/v1/quota'
 ```
 */
export function neuralWattQuotaUrlFor({ baseUrl, }: { readonly baseUrl: string; },): string {
  return `${baseUrl}/quota`;
}

/**
 Models listing URL, which names what each tracking alias points at.

 @param baseUrl - base address, overridable for tests

 @returns Absolute URL to GET

 @example
 ```ts
 neuralWattModelsUrlFor({ baseUrl: NEURALWATT_BASE_URL, },);
 // => 'https://api.neuralwatt.com/v1/models'
 ```
 */
export function neuralWattModelsUrlFor({ baseUrl, }: { readonly baseUrl: string; },): string {
  return `${baseUrl}/models`;
}

/**
 By-model usage URL for a window, which counts every key on the account.

 FULL TIMESTAMPS: a date-only window with the same day at both ends came back
 empty and zero-length (probe of 2026-10-20), and a malformed date silently
 falls back to the route's default window, so the reader checks the `period`
 the reply echoes (`neuralwatt-usage.ts`). The colons are left as they are,
 the form the probe measured (`https://docs.neuralwatt.com/api-reference/usage/get-usage-broken-down-by-model`).

 @param baseUrl - base address, overridable for tests

 @param start - first instant of the window

 @param end - last instant of the window

 @returns Absolute URL to GET

 @example
 ```ts
 neuralWattUsageByModelUrlFor({ baseUrl: NEURALWATT_BASE_URL, start, end, },);
 // => 'https://api.neuralwatt.com/v1/usage/by-model?start_date=2026-10-20T19:15:08.000Z&end_date=2026-10-20T23:15:00.741Z'
 ```
 */
export function neuralWattUsageByModelUrlFor(
  {
    baseUrl,
    start,
    end,
  }: {
    readonly baseUrl: string;
    readonly start: Date;
    readonly end: Date;
  },
): string {
  return `${baseUrl}/usage/by-model?start_date=${start.toISOString()}&end_date=${end.toISOString()}`;
}

//endregion NeuralWatt catalog
