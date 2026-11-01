import {
  type BedrockRoute,
  type BedrockServedId,
  type BedrockStreamEnd,
  type RosterModelId,
  servedRecord,
} from '@monochromatic-dev/translation-repair-shared-contract/ts';


//region Bedrock catalog
// What Amazon Bedrock serves for this pipeline, how it spells it, where it
// routes it and what it charges.
//
// WHY THIS PROVIDER EXISTS IN THE PIPELINE. The owner, 2026-09-07: "I have
// 200USD of credits in my Amazon Bedrock account. With ZDR enabled, only these
// models are usable (and approved): Claude Sonnet 5, Gemma 4 E2B, Gemma 4 31B,
// Gemma 4 26B-A4B, GPT OSS 120B"; then "The credits in my Amazon Bedrock
// account will expire early next year, so you're allowed to use it as much as
// you like. But I will NEVER top it up"; then "I retract Sonnet 5. Looks like
// Amazon didn't properly display it as not supporting ZDR." It is the fourth
// provider in `PROVIDER_ORDER`, ahead of OpenRouter, since its credits are
// prepaid and expiring where OpenRouter's are the owner's to top up.
//
// RAW FETCH, NO SDK. The owner, 2026-09-07: "Please do not introduce Amazon
// Bedrock SDK, Anthropic SDK, OpenAI SDK because these are poorly written. Use
// raw fetch." The client rides the same transport seam as the other three.
//
// TWO ROUTES ON ONE HOST, MEASURED 2026-09-07 with the bearer key in us-east-1
// (probes recorded in
// `doc/planning/translation-repair-openrouter-2026-09-03.md`). The three Gemma
// 4 sizes answer only under `/openai/v1/chat/completions`; the default
// `/v1/chat/completions` says "model isn't supported on this route", and each
// model card says "On bedrock-mantle, this model is served at /openai/v1".
// gpt-oss-120b answers only under `/v1/chat/completions` and its card gives
// that base. Both stream, both report `usage` on the last chunk, and both
// honoured a json_schema `response_format`. The Gemma streams end on
// `data: [DONE]`; the gpt-oss stream ends on its usage chunk with no sentinel,
// so the terminator is a per-model fact here.
//
// CLAUDE SONNET 5 IS NOT SERVED, by measurement before the owner's retraction:
// mantle listed it but its chat, responses and messages routes refused it, and
// bedrock-runtime answered 403 "not available for this account".
//
// THE ACCOUNT IS IN DATA-RETENTION MODE `none`: every listed model reported
// `data_retention.mode: "none", source: "account"` on `/v1/models`, and a
// request carrying `store` answered "This account requires store=false for
// this model". Chat completions store nothing and the client sends no `store`.
//
// PRICES ARE THE PUBLIC PRICING PAGE'S, read 2026-09-07, US regions: Gemma 4
// E2B 0.04 in and 0.08 out per million tokens, Gemma 4 31B 0.14 and 0.40,
// Gemma 4 26B-A4B 0.13 and 0.40; gpt-oss-120b 0.1545 and 0.618 as the page
// listed for Sydney, the one region it showed. The ledger meters the owner's
// 200 USD off these, since the account exposes no balance endpoint to a
// bearer key.
//
// THE GEMMA 4 CARDS REPORT IMAGE INPUT AND THE CATALOG SAID FALSE until a
// transcription through this serving stack was measured, for the reason the
// OpenRouter catalog gives for the same model. Measured 2026-09-08 02:20 UTC
// over nine pictures the seated readers had corroborated (planning log,
// "The three Gemma sizes against the seated readers"): 26B-A4B and 31B
// produced eight readings each and every one was corroborated by every
// seated reader, so both read pictures here; E2B produced two readings no
// seated reader corroborated and stays a text seat. gpt-oss-120b reads no
// pictures anywhere. A 1.27 MB picture came back as an empty stream from all
// three sizes; the reader stage records that reader as `empty-reply`.
//
// NO PUBLISHED OUTPUT CEILING FOR GEMMA 4. The model cards give context
// windows (128K for E2B, 256K for the other two) and no maximum output; the
// context window stands in and the measured answer bound applies as it does
// everywhere else. gpt-oss-120b's card says 16K.

/**
 Host every route hangs off: the mantle endpoint in the one region measured
 to list all four models. us-west-2 listed the Gemma sizes and gpt-oss too;
 the region is a constant rather than a knob until a second one is needed.
 */
export const BEDROCK_MANTLE_BASE_URL = 'https://bedrock-mantle.us-east-1.api.aws';

/**
 Header carrying the key, a bearer token as on the other three providers.
 */
export const BEDROCK_AUTH_HEADER = 'Authorization';


/**
 Path prefix of each route, before `/chat/completions`.
 */
export const BEDROCK_ROUTE_PREFIX: Readonly<Record<BedrockRoute, string>> = {
  'openai-v1': '/openai/v1',
  v1: '/v1',
};





/**
 Verified per-model facts the router, the request builder and the ledger read.

 @example
 ```ts
 const info: BedrockModelInfo = BEDROCK_MODELS['google.gemma-4-e2b'];
 ```
 */
export type BedrockModelInfo = {
  /**
   Identifier sent in the request body's `model` field.
   */
  readonly id: BedrockServedId;

  /**
   The roster seat this spelling reaches: another provider's spelling where
   one serves the same model, and this provider's own spelling where none
   does, since a Bedrock-only model has no other name to be known by.
   */
  readonly sharedWith: RosterModelId;

  /**
   Whether a picture may be sent here: the card's image input, held false
   until a transcription is measured, as the OpenRouter catalog holds it.
   */
  readonly readsImages: boolean;

  /**
   Context window in tokens, as the model card states it.
   */
  readonly contextLength: number;

  /**
   Ceiling the model card states for completion tokens, or the context
   window where the card states none.
   */
  readonly maxOutputLength: number;

  /**
   Route this model answers on, measured per model.
   */
  readonly route: BedrockRoute;

  /**
   How this model's stream announces that it is whole, measured per model.
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
 Every model this provider serves for this pipeline, read off the cards.

 @example
 ```ts
 const info = BEDROCK_MODELS['openai.gpt-oss-120b'];
 ```
 */
export const BEDROCK_MODELS: Readonly<Record<BedrockServedId, BedrockModelInfo>> = servedRecord({
  provider: 'bedrock',
  toRow: function bedrockRow(card,): BedrockModelInfo {
    /**
     Bedrock's side of the card.
     */
    const { bedrock, } = card;
    return {
      id: bedrock.id,
      sharedWith: card.id,
      readsImages: bedrock.readsImages,
      contextLength: bedrock.contextLength,
      maxOutputLength: bedrock.maxOutputLength,
      route: bedrock.route,
      streamEnd: bedrock.streamEnd,
      streamBoundMs: bedrock.streamBoundMs,
      promptUsdPerMillion: bedrock.promptUsdPerMillion,
      completionUsdPerMillion: bedrock.completionUsdPerMillion,
    };
  },
},);

/**
 Chat completions URL one model answers on.

 @param baseUrl - host, overridable for tests

 @param servedId - model whose route decides the path

 @returns Absolute URL to POST the body to

 @example
 ```ts
 bedrockChatUrlFor({ baseUrl: BEDROCK_MANTLE_BASE_URL, servedId: 'google.gemma-4-e2b', },);
 // => 'https://bedrock-mantle.us-east-1.api.aws/openai/v1/chat/completions'
 ```
 */
export function bedrockChatUrlFor(
  {
    baseUrl,
    servedId,
  }: {
    readonly baseUrl: string;
    readonly servedId: BedrockServedId;
  },
): string {
  /**
   Route this model answers on.
   */
  const { route, } = BEDROCK_MODELS[servedId];

  return `${baseUrl}${BEDROCK_ROUTE_PREFIX[route]}/chat/completions`;
}

//endregion Bedrock catalog
