import { isDeepStrictEqual, } from 'node:util';

import { isJsonRecord, } from '@monochromatic-dev/translation-repair-document-structure/ts';
import type { ListedCardProvider, } from './roster-card-ask.ts';
import { StatedRefusalError, } from '@monochromatic-dev/translation-repair-glossary/ts';
import { isUnsignedNumberText, } from '@monochromatic-dev/translation-repair-markup/ts';

//region Roster card rendering
// TURNS ONE ROW OF A PROVIDER'S LIVE LISTING INTO THE CARD FRAGMENT a person
// pastes into `model-cards.ts`. Each provider lists its models under its own
// field names; this is the one place those names are known, and the output
// is the card's field names, so seating a model no longer means reading four
// listings by hand and remembering which field is which.
//
// FIELDS THE LISTING DOES NOT CARRY ARE PRINTED AS QUESTIONS rather than
// guessed: a Bedrock row says nothing about image input, and the cap and
// the abandoned-stream ratio are measured off completed calls, never listed.

/**
 One row of a listing as decoded JSON.
 */
type ListingRow = Readonly<Record<string, unknown>>;

/**
 A field the listing does not carry, travelling as a value so the rendered
 card can print the question in its place.

 @example
 ```ts
 const cap: Listed<number> = NOT_LISTED;
 ```
 */
export const NOT_LISTED: unique symbol = Symbol('field the provider listing does not carry',);

/**
 A field's value where the listing carries it, else {@link NOT_LISTED}.

 @example
 ```ts
 const reads: Listed<boolean> = true;
 ```
 */
export type Listed<Value> = Value | typeof NOT_LISTED;

/**
 Currency sign Synthetic writes before its per-token prices.
 */
const CURRENCY_SIGN = '$';

/**
 Tokens in one million, the unit OpenRouter prices are converted to.
 */
const TOKENS_PER_MILLION = 1_000_000;

/**
 Whether a value is a plain object, for walking a row.

 @param value - anything decoded from the listing

 @returns Whether it can be indexed by field name
 */
function isRow(value: unknown,): value is ListingRow {
  return isJsonRecord(value,);
}

/**
 Field of a row by path, or nothing where any step is missing.

 @param row - listing row

 @param path - field names from the row down

 @returns Value at the path, or `undefined` as the row itself would hold it

 @example
 ```ts
 const cap = fieldAt({ row, path: ['top_provider', 'max_completion_tokens',], },);
 ```
 */
export function fieldAt(
  {
    row,
    path,
  }: {
    readonly row: ListingRow;
    readonly path: readonly string[];
  },
): unknown {
  /**
   Value reached so far.
   */
  let current: unknown = row;
  for (const name of path) {
    current = isRow(current,) ? current[name] : undefined;
  }
  return current;
}

/**
 Finds the row naming one served id in a listing body of the
 `{ data: [...] }` shape every provider here answers with.

 EVERY ROW UNDER THE ID IS READ, NOT THE FIRST. The listing is a provider's
 reply, and nothing in it makes an id name one row: rows that say the same
 are one answer, and rows that differ are two sets of fields under one name,
 of which the card would describe whichever the provider happened to list
 first.

 @param body - decoded listing

 @param servedId - spelling the provider serves the model under

 @returns The row, or {@link NOT_LISTED} where the listing has none

 @throws {@link StatedRefusalError} When the listing carries the id on rows
 that differ, so no one row is the model's

 @example
 ```ts
 const row = listingRowFor({ body, servedId: 'minimax/minimax-m3', },);
 ```
 */
export function listingRowFor(
  {
    body,
    servedId,
  }: {
    readonly body: unknown;
    readonly servedId: string;
  },
): Listed<ListingRow> {
  /**
   Rows under `data`, or the body itself when it is the array.
   */
  const rows: unknown = isRow(body,) ? body.data : body;
  if (!Array.isArray(rows,))
    return NOT_LISTED;
  /**
   Rows that are objects, the only kind that can name a model.
   */
  const named = rows.filter(isRow,);
  /**
   First row carrying the served id, if any, and every further one.
   */
  const [row, ...others] = named.filter(function is(candidate,): boolean {
    return candidate.id === servedId;
  },);
  if (row === undefined)
    return NOT_LISTED;
  /**
   Whether every further row under the id says what the first says.
   */
  const agree = others.every(function saysTheSame(other,): boolean {
    return isDeepStrictEqual(
      other,
      row,
    );
  },);
  if (!agree) {
    throw new StatedRefusalError({
      says: `the listing carries more than one row under ${servedId} and they differ, so which of them the card `
        + 'would describe cannot be read; read the listing by hand',
    },);
  }
  return row;
}

/**
 Whether a listing's modality field says the model takes images.

 @param modalities - whatever the listing put under its input modalities

 @returns True on an `image` entry, false on an array without one,
 {@link NOT_LISTED} where the field is not an array
 */
function readsImagesFrom(modalities: unknown,): Listed<boolean> {
  if (!Array.isArray(modalities,))
    return NOT_LISTED;
  return modalities.includes('image',);
}

/**
 Number a listing field holds, whether it wrote it as a number, a string
 or a dollar-prefixed string (Synthetic prices read `$0.000003`).

 A LENGTH OR A PRICE, SO A FINITE NUMBER NOT BELOW ZERO (ledger B73).
 `Number` read a blank field as 0, a free model with no context; `$0x1` as
 one dollar a token; a sign through, so a listing's `-1` (a price it does
 not quote) became a card with a negative price; and a digit run past the
 largest double as Infinity. Each is now left unlisted, and the card asks
 for it.

 @param value - field value

 @returns The number, or {@link NOT_LISTED} where the field is no unsigned
 number as JSON writes one, or is not finite, or is below zero
 */
function numberFrom(value: unknown,): Listed<number> {
  if ((typeof value) === 'number')
    return (Number.isFinite(value,) && (value >= 0)) ? value : NOT_LISTED;
  if ((typeof value) !== 'string')
    return NOT_LISTED;
  /**
   The string without the currency sign Synthetic prefixes its prices with.
   */
  const bare = value.startsWith(CURRENCY_SIGN,) ? value.slice(CURRENCY_SIGN.length,) : value;
  /**
   What the string reads as, Infinity for a digit run past the largest double.
   */
  const read = Number(bare,);
  return (isUnsignedNumberText({ text: bare, },) && Number.isFinite(read,)) ? read : NOT_LISTED;
}

/**
 Scales a per-token price to per million tokens, keeping an absent one absent.

 @param perToken - price per token, where listed

 @returns Price per million tokens, where listed
 */
function perMillion(perToken: Listed<number>,): Listed<number> {
  return ((typeof perToken) === 'symbol') ? NOT_LISTED : (perToken * TOKENS_PER_MILLION);
}

/**
 Card fields read off one provider's row, each {@link NOT_LISTED} where the
 listing does not say.

 @example
 ```ts
 const fields: CardFields = cardFieldsFrom({ provider: 'hyper', row, },);
 ```
 */
export type CardFields = {
  /**
   Whether the listing reports image input.
   */
  readonly readsImages: Listed<boolean>;

  /**
   Completion token ceiling the listing reports.
   */
  readonly maxOutputLength: Listed<number>;

  /**
   Context window the listing reports.
   */
  readonly contextLength: Listed<number>;

  /**
   Prompt price in the card's unit for that provider.
   */
  readonly promptPrice: Listed<number>;

  /**
   Completion price in the card's unit for that provider.
   */
  readonly completionPrice: Listed<number>;
};

/**
 Reads a nested numeric field off a row.

 @param row - listing row

 @param path - field names from the row down

 @returns The number, where listed
 */
function numberAt(
  {
    row,
    path,
  }: {
    readonly row: ListingRow;
    readonly path: readonly string[];
  },
): Listed<number> {
  return numberFrom(fieldAt({
    row,
    path,
  },),);
}

/**
 Reads the card fields off one provider's row, in that provider's field
 names.

 @param provider - whose listing the row came from

 @param row - the row

 @returns Fields the listing carries

 @example
 ```ts
 const fields = cardFieldsFrom({ provider: 'openrouter', row, },);
 ```
 */
export function cardFieldsFrom(
  {
    provider,
    row,
  }: {
    readonly provider: ListedCardProvider;
    readonly row: ListingRow;
  },
): CardFields {
  if (provider === 'synthetic') {
    return {
      readsImages: readsImagesFrom(row.input_modalities,),
      maxOutputLength: numberFrom(row.max_output_length,),
      contextLength: numberFrom(row.context_length,),
      promptPrice: numberAt({
        row,
        path: [
          'pricing',
          'prompt',
        ],
      },),
      completionPrice: numberAt({
        row,
        path: [
          'pricing',
          'completion',
        ],
      },),
    };
  }
  if (provider === 'hyper') {
    /**
     Vision flag as Hyper's catalog reports it.
     */
    const vision = fieldAt({
      row,
      path: [
        'capabilities',
        'vision',
      ],
    },);
    return {
      readsImages: ((typeof vision) === 'boolean') ? vision : NOT_LISTED,
      maxOutputLength: numberFrom(row.max_output_tokens,),
      contextLength: numberFrom(row.context_window,),
      promptPrice: NOT_LISTED,
      completionPrice: NOT_LISTED,
    };
  }
  if (provider === 'openrouter') {
    /**
     Ceiling the top endpoint reports, where it reports one.
     */
    const topCeiling = numberAt({
      row,
      path: [
        'top_provider',
        'max_completion_tokens',
      ],
    },);
    return {
      readsImages: readsImagesFrom(fieldAt({
        row,
        path: [
          'architecture',
          'input_modalities',
        ],
      },),),
      maxOutputLength: ((typeof topCeiling) === 'symbol') ? numberFrom(row.context_length,) : topCeiling,
      contextLength: numberFrom(row.context_length,),
      promptPrice: perMillion(numberAt({
        row,
        path: [
          'pricing',
          'prompt',
        ],
      },),),
      completionPrice: perMillion(numberAt({
        row,
        path: [
          'pricing',
          'completion',
        ],
      },),),
    };
  }
  if (provider === 'neuralwatt') {
    /**
     Vision flag as NeuralWatt's catalog reports it, under `metadata`.
     */
    const vision = fieldAt({
      row,
      path: [
        'metadata',
        'capabilities',
        'vision',
      ],
    },);
    /**
     Context window the listing states under `metadata.limits`.
     */
    const context = numberAt({
      row,
      path: [
        'metadata',
        'limits',
        'max_context_length',
      ],
    },);
    /**
     Output ceiling the listing states, which is null for some models.
     */
    const ceiling = numberAt({
      row,
      path: [
        'metadata',
        'limits',
        'max_output_tokens',
      ],
    },);
    return {
      readsImages: ((typeof vision) === 'boolean') ? vision : NOT_LISTED,
      // The context window stands in where the listing states no ceiling, as
      // for OpenRouter's top endpoint.
      maxOutputLength: ((typeof ceiling) === 'symbol') ? context : ceiling,
      contextLength: context,
      // The card side carries no price: the free energy bills none.
      promptPrice: NOT_LISTED,
      completionPrice: NOT_LISTED,
    };
  }
  // Bedrock's listing names the model and its retention mode and nothing a
  // card needs; DeepSeek's and Xiaomi's name the model and its owner and
  // nothing else; the model card page carries the rest.
  return {
    readsImages: NOT_LISTED,
    maxOutputLength: numberFrom(row.max_output_tokens,),
    contextLength: numberFrom(row.context_length,),
    promptPrice: NOT_LISTED,
    completionPrice: NOT_LISTED,
  };
}

/**
 One card line, or a question where the listing did not say.

 @param name - card field

 @param value - what was read, where listed

 @param says - where to look when nothing was read

 @returns Line to print
 */
function cardLine(
  {
    name,
    value,
    says,
  }: {
    readonly name: string;
    readonly value: Listed<boolean | number>;
    readonly says: string;
  },
): string {
  if ((typeof value) === 'symbol')
    return `      ${name}: /* not in the listing: ${says} */,`;
  return `      ${name}: ${String(value,)},`;
}

/**
 Renders the provider side of a card for one served id.

 @param provider - whose side

 @param servedId - spelling the provider serves the model under

 @param fields - what the listing said

 @returns Lines to paste into `model-cards.ts`

 @example
 ```ts
 console.log(renderProviderCard({ provider: 'hyper', servedId: 'minimax-m3', fields, },),);
 ```
 */
export function renderProviderCard(
  {
    provider,
    servedId,
    fields,
  }: {
    readonly provider: ListedCardProvider;
    readonly servedId: string;
    readonly fields: CardFields;
  },
): string {
  /**
   Lines every provider's side carries.
   */
  const shared = [
    `    ${provider}: {`,
    `      id: '${servedId}',`,
    cardLine({
      name: 'readsImages',
      value: fields.readsImages,
      says: 'send it a picture and measure',
    },),
    cardLine({
      name: 'maxOutputLength',
      value: fields.maxOutputLength,
      says: 'the model card page',
    },),
  ];
  /**
   Lines only some providers' sides carry.
   */
  const own: Readonly<Record<ListedCardProvider, readonly string[]>> = {
    synthetic: [
      `      family: /* 'zai' | 'qwen' | 'moonshot' | 'openai' */,`,
      cardLine({
        name: 'contextLength',
        value: fields.contextLength,
        says: 'the model card page',
      },),
      cardLine({
        name: 'promptDollarsPerToken',
        value: fields.promptPrice,
        says: 'the listing\'s pricing',
      },),
      cardLine({
        name: 'completionDollarsPerToken',
        value: fields.completionPrice,
        says: 'the listing\'s pricing',
      },),
    ],
    hyper: [],
    openrouter: [
      cardLine({
        name: 'promptUsdPerMillion',
        value: fields.promptPrice,
        says: 'the listing\'s pricing',
      },),
      cardLine({
        name: 'completionUsdPerMillion',
        value: fields.completionPrice,
        says: 'the listing\'s pricing',
      },),
      '      ignoredEndpoints: [],',
      '      preferredEndpoints: [],',
      `      rawCharsPerToken: 'unmeasured',`,
    ],
    bedrock: [
      cardLine({
        name: 'contextLength',
        value: fields.contextLength,
        says: 'the model card page',
      },),
      `      route: /* 'openai-v1' | 'v1', measured */,`,
      `      streamEnd: /* 'done-sentinel' | 'usage-chunk', measured */,`,
      '      promptUsdPerMillion: /* the public pricing page */,',
      '      completionUsdPerMillion: /* the public pricing page */,',
    ],
    neuralwatt: [
      cardLine({
        name: 'contextLength',
        value: fields.contextLength,
        says: 'the model card page',
      },),
    ],
    // THE DIRECT PROVIDERS' LISTINGS CARRY NO PRICE, so the three rates are
    // read off the provider's public pricing page, as Bedrock's are.
    deepseek: [
      '      promptCacheHitUsdPerMillion: /* the public pricing page */,',
      '      promptCacheMissUsdPerMillion: /* the public pricing page */,',
      '      completionUsdPerMillion: /* the public pricing page */,',
    ],
    xiaomi: [
      '      promptCacheHitUsdPerMillion: /* the public pricing page */,',
      '      promptCacheMissUsdPerMillion: /* the public pricing page */,',
      '      completionUsdPerMillion: /* the public pricing page */,',
    ],
  };
  return [
    ...shared,
    ...own[provider],
    '    },',
  ].join('\n',);
}

//endregion Roster card rendering
