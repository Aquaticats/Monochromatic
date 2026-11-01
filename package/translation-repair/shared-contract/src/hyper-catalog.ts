import { servedRecord, } from './model-card-derive.ts';
import type {
  HyperServedId,
  RosterModelId,
  SyntheticServedId,
} from './roster-id.ts';

export type { HyperServedId, } from './roster-id.ts';

//region Hyper catalog
// What Charm Hyper serves, and the two ways it differs from the other provider
// at the request layer.
//
// EVERY FIELD HERE WAS MEASURED against the live API on 2026-08-24 rather than
// read off a docs page, because two of them contradict what the catalog
// endpoint alone would suggest.
//
// WHY THIS PROVIDER EXISTS IN THE PIPELINE AT ALL. A corpus pass exhausted the
// other provider's weekly credit and 866 of 875 lost voices carried
// `HTTP 429: You've exceeded your subscription rate limits`. A second provider
// is the only remedy that works against an exhausted budget: retrying it never
// succeeds, and refusing to settle turns a budget problem into holes in the
// deliverable.
//
// SCHEMA'D OUTPUT ONLY WORKS OVER ANTHROPIC MESSAGES HERE. The OpenAI Chat
// Completions endpoint accepts `response_format` and ignores it: a deliberately
// invalid `{type: 'not_a_real_mode_xyz'}` returns 200, an unknown top-level
// field returns 200, and every mode answers with markdown-fenced JSON carrying
// invented keys. Nothing in this file supports that endpoint.

/**
 Marker for a Hyper-origin identity without a Synthetic counterpart.
 It does not exclude an OpenRouter route for the same roster identity.

 A NAMED READING rather than a nullish union, matching how the rest of this
 package models absence.
 */
export const NO_SYNTHETIC_COUNTERPART = 'no-synthetic-counterpart';

/**
 Endpoint one call is POSTed to, measured live on 2026-08-24.

 THE MESSAGES API RATHER THAN CHAT COMPLETIONS, for the reason the module
 note records: the OpenAI-shaped endpoint accepts `response_format` and
 ignores it, so structured output has nowhere else to go.
 */
export const HYPER_MESSAGES_URL = 'https://hyper.charm.land/v1/messages';

/**
 Endpoint reporting the remaining balance, measured live on 2026-08-24.
 */
export const HYPER_CREDITS_URL = 'https://hyper.charm.land/v1/credits';

/**
 Value of the `anthropic-version` header every call carries.

 REQUIRED BY THE PROTOCOL rather than chosen: this is the dated contract the
 request and response shapes belong to, and the frames this package parses
 were captured under it.
 */
export const HYPER_API_VERSION = '2023-06-01';

/**
 Header carrying the key, which is NOT the one the protocol usually uses.

 MEASURED, AND THE MEASUREMENT CONTRADICTS THE OBVIOUS GUESS. The Messages
 API is normally keyed by `x-api-key`, and this gateway answers that header
 with `401 missing authorization`. It takes a bearer token instead, so a
 client written from the protocol docs alone would fail to authenticate every
 call and read it as a bad key.
 */
export const HYPER_AUTH_HEADER = 'Authorization';



/**
 Verified per-model facts the router and the request builder read.

 @example
 ```ts
 const info: HyperModelInfo = HYPER_MODELS['deepseek-v4.1-flash-1017'];
 ```
 */
export type HyperModelInfo = {
  /**
   Identifier sent in the request body's `model` field.
   */
  readonly id: HyperServedId;

  /**
   The roster seat this spelling reaches: the card it sits on.

   NOT ALWAYS THE SAME STRING AS `id`. The roster names a seat by its first
   serving provider's spelling, and the card of `deepseek-v4.1-flash` kept
   that name when Hyper and OpenRouter moved the seat to the 1017 weights
   (`deepseek-v4.1-flash-1017`, 2026-10-20), so the reach lookup reads this
   field and never compares `id` with a roster id.
   */
  readonly rosterId: RosterModelId;

  /**
   Same model reached through the other provider, where there is one.

   PROVIDER IS NOT PART OF PANELIST IDENTITY. `kimi-k3` here and
   `hf:moonshotai/Kimi-K3` there are one panelist for self-certification
   weighting and for the cache key, so a slice judged by that model counts
   once however it was reached. Which provider actually served a call is
   recorded per call, for diagnosis, and nowhere else.
   */
  readonly sharedWith: SyntheticServedId | typeof NO_SYNTHETIC_COUNTERPART;

  /**
   Whether this model can be sent an image alongside its text.

   READ FROM `capabilities.vision` on this provider's own catalog endpoint.
   Three of the seven report true. The other provider now serves three image
   readers after GLM-5.3-Flash replaced GLM-5.2.
   */
  readonly readsImages: boolean;

  /**
   Ceiling this model will emit, from `max_output_tokens` on the catalog.

   TWO OF THESE SIT BELOW the measured 32000 answer bound, so the bound
   has to be read per model rather than globally: `gpt-oss-120b` stops at
   13107 and `kimi-k3` at 16000. A request that asks for more than a model can
   emit buys a truncation and reports it as a schema mismatch, which sends a
   reader to the prompt instead of to the ceiling.
   */
  readonly maxOutputLength: number;
};

/**
 Every model this provider serves for this pipeline, read off the cards.

 CONFORMANCE MEASURED OVER 20 STREAMING ATTEMPTS EACH on 2026-08-24 for the
 models of that day and on the day each later one joined: every current
 model accepts forced tool choice and answers with schema-conformant input.

 @example
 ```ts
 const info = HYPER_MODELS['deepseek-v4.1-flash-1017'];
 ```
 */
export const HYPER_MODELS: Readonly<Record<HyperServedId, HyperModelInfo>> = servedRecord({
  provider: 'hyper',
  toRow: function hyperRow(card,): HyperModelInfo {
    /**
     Hyper's side of the card, beside Synthetic's where there is one.
     */
    const {
      hyper,
      synthetic,
    } = card;
    return {
      id: hyper.id,
      rosterId: card.id,
      sharedWith: synthetic?.id ?? NO_SYNTHETIC_COUNTERPART,
      readsImages: hyper.readsImages,
      maxOutputLength: hyper.maxOutputLength,
    };
  },
},);

/**
 Measured bound for answer volume, in tokens.

 KEPT HERE BESIDE THE PER-MODEL CEILINGS it has to be reconciled against,
 rather than imported from the guard that enforces it, so a reader comparing
 the two numbers sees both at once.
 */
const MEASURED_ANSWER_BOUND = 32_000;

/**
 How many tokens to ask one model for, honouring both bounds.

 ONE-SIDED ON PURPOSE: this only ever lowers the ask. A model that can emit
 more than was measured is still held to the measured bound, because that
 bound is about what an answer should be rather than what a model can do.

 @param modelId - model the request is for

 @returns Token ceiling to send, never above either bound

 @example
 ```ts
 const maxTokens = answerCeilingFor({ modelId: 'gpt-oss-120b', },);
 ```
 */
export function answerCeilingFor(
  { modelId, }: { readonly modelId: HyperServedId; },
): number {
  /**
   What this model says it can emit, before the measured bound is applied.
   */
  const { maxOutputLength, } = HYPER_MODELS[modelId];

  return Math.min(
    MEASURED_ANSWER_BOUND,
    maxOutputLength,
  );
}

//endregion Hyper catalog
