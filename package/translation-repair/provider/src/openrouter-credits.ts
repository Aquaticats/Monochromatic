import { isJsonRecord, } from '@monochromatic-dev/translation-repair-document-structure/ts';

//region OpenRouter credits
// Typed view over `GET /api/v1/credits`, shape verified with a live call on
// 2026-09-03, which answered
// `{"data":{"total_credits":1913,"total_usage":1855.383100082}}`.
//
// TWO NUMBERS AND ONE DIFFERENCE. The provider reports what was ever bought
// and what was ever spent, in USD, and the budget question this pipeline asks
// is their difference. Both are carried so a reader of the `METERS` line can
// see the spend move, and the difference is computed once here rather than at
// every site that wants it.
//
// A malformed body is a provider protocol failure and throws, matching
// `hyper-credits.ts` and `synthetic-quota.ts`: model-content defects flow as
// data, protocol defects do not.

/**
 Credits at one instant, as this provider reports them.

 @example
 ```ts
 const credits: OpenRouterCredits = { purchasedUsd: 1913, usedUsd: 1855.38, remainingUsd: 57.62, };
 ```
 */
export type OpenRouterCredits = {
  /**
   USD ever purchased on the account.
   */
  readonly purchasedUsd: number;

  /**
   USD ever spent on the account.
   */
  readonly usedUsd: number;

  /**
   USD still available to spend, purchased less used.
   */
  readonly remainingUsd: number;
};

/**
 Signals a `/credits` body that refused to parse or lacked its fields;
 always a provider protocol failure, never a model defect.

 @example
 ```ts
 throw new OpenRouterCreditsShapeError({ detail: 'total_credits is not a number', },);
 ```
 */
export class OpenRouterCreditsShapeError extends Error {
  /**
   Declares this message safe to forward: it names the field that failed its shape, never the body it came from.
   */
  readonly messageNamesOnly: true = true;

  /**
   Builds failure naming the field or parse step at fault.

   @param detail - which expectation the body violated

   @param cause - underlying parse error when JSON itself failed

   @example
   ```ts
   new OpenRouterCreditsShapeError({ detail: 'body is not valid JSON', cause: error, },);
   ```
   */
  public constructor(
    {
      detail,
      cause,
    }: {
      readonly detail: string;
      readonly cause?: unknown;
    },
  ) {
    super(
      `OpenRouter /credits body violated expectations: ${detail}`,
      // Conditional spread keeps cause absent when none was supplied.
      ...(cause === undefined
        ? []
        : [{ cause, },]),
    );
    this.name = 'OpenRouterCreditsShapeError';
  }
}

/**
 Reads one finite number off a parsed object, refusing anything else.

 @param fields - parsed object to read

 @param name - field wanted

 @returns Its finite value

 @throws {@link OpenRouterCreditsShapeError} when the field is missing,
 mistyped, or not finite, since a non-finite figure would compare against
 every threshold as though the budget were unlimited

 @example
 ```ts
 const purchased = finiteField({ fields: data, name: 'total_credits', },);
 ```
 */
function finiteField(
  {
    fields,
    name,
  }: {
    readonly fields: Readonly<Record<string, unknown>>;
    readonly name: string;
  },
): number {
  /**
   Raw value under that name, of unknown type.
   */
  const value = fields[name];

  if ((typeof value) !== 'number')
    throw new OpenRouterCreditsShapeError({ detail: `${name} is not a number`, },);

  if (!Number.isFinite(value,))
    throw new OpenRouterCreditsShapeError({ detail: `${name} is not a finite number`, },);

  return value;
}

/**
 Parses one `/credits` body into the typed credits.

 @param bodyText - raw 200-response body

 @returns Purchased, used, and what is left

 @throws {@link OpenRouterCreditsShapeError} when body is not JSON, lacks
 its `data` envelope, or either figure is missing, mistyped, or not finite

 @example
 ```ts
 const credits = parseOpenRouterCredits({ bodyText: reply.bodyText, },);
 ```
 */
export function parseOpenRouterCredits(
  { bodyText, }: { readonly bodyText: string; },
): OpenRouterCredits {
  /**
   Whole parsed body, probed field by field.
   */
  const parsed: unknown = (function parseBody(): unknown {
    try {
      return JSON.parse(bodyText,);
    } catch (error) {
      throw new OpenRouterCreditsShapeError({
        detail: 'body is not valid JSON',
        cause: error,
      },);
    }
  })();

  if (!isJsonRecord(parsed,))
    throw new OpenRouterCreditsShapeError({ detail: 'body is not a JSON object', },);

  /**
   Envelope the two figures sit inside.
   */
  const { data, } = parsed;

  if (!isJsonRecord(data,))
    throw new OpenRouterCreditsShapeError({ detail: 'data is not a JSON object', },);

  /**
   USD ever purchased.
   */
  const purchasedUsd = finiteField({
    fields: data,
    name: 'total_credits',
  },);

  /**
   USD ever spent.
   */
  const usedUsd = finiteField({
    fields: data,
    name: 'total_usage',
  },);

  return {
    purchasedUsd,
    usedUsd,
    remainingUsd: purchasedUsd - usedUsd,
  };
}

//endregion OpenRouter credits
