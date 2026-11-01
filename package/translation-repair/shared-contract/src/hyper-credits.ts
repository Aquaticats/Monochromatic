import { isJsonRecord, } from '@monochromatic-dev/translation-repair-document-structure/ts';

//region Hyper credits
// Typed view over `GET /v1/credits`, shape verified with a live call on
// 2026-08-24, which answered `{"balance": 249}`.
//
// ONE NUMBER, unlike the other provider's two-limit reading. There is no
// rolling window and no percentage here: the balance is spent down and refills
// whole, measured at every 24 hours, so the only budget question this provider
// can answer is how much is left right now.
//
// THE REFILL INSTANT IS NOT MODELLED. The refresh was observed at 02:53 and the
// zone it was observed in was never established, so computing a wall-clock
// "back at" from it would state a fact nobody measured. A caller that needs to
// report when the budget returns should read the balance again rather than
// predict it.
//
// A malformed body is a provider protocol failure and throws, matching
// `synthetic-quota.ts`: model-content defects flow as data, protocol defects do
// not. Quota reads cost nothing, so a driver may poll.

/**
 Balance at one instant, as this provider reports it.

 @example
 ```ts
 const credits: HyperCredits = { balance: 249, };
 ```
 */
export type HyperCredits = {
  /**
   Hypercredits still available to spend.
   */
  readonly balance: number;
};

/**
 Signals a `/credits` body that refused to parse or lacked its one field;
 always a provider protocol failure, never a model defect.

 @example
 ```ts
 throw new CreditsShapeError({ detail: 'balance is not a number', },);
 ```
 */
export class CreditsShapeError extends Error {
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
   new CreditsShapeError({ detail: 'body is not valid JSON', cause: error, },);
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
      `Charm Hyper /credits body violated expectations: ${detail}`,
      // Conditional spread keeps cause absent when none was supplied.
      ...(cause === undefined
        ? []
        : [{ cause, },]),
    );
    this.name = 'CreditsShapeError';
  }
}

/**
 Parses body text as JSON, converting parse failures into shape errors.

 @param bodyText - raw response body

 @returns Parsed JSON value

 @throws {@link CreditsShapeError} when body is not valid JSON

 @example
 ```ts
 const parsed = parseCreditsJson({ bodyText, },);
 ```
 */
function parseCreditsJson(
  { bodyText, }: { readonly bodyText: string; },
): unknown {
  try {
    return JSON.parse(bodyText,);
  }
  catch (error) {
    throw new CreditsShapeError({
      detail: 'body is not valid JSON',
      cause: error,
    },);
  }
}

/**
 Parses one `/credits` body into the typed balance.

 REFUSES A BALANCE THAT IS NOT FINITE, which a JSON body cannot carry but a
 gateway rewriting one could produce. A non-finite balance would compare
 against every threshold as though the budget were unlimited, which is the one
 wrong answer this reader exists to prevent.

 @param bodyText - raw 200-response body

 @returns Typed balance

 @throws {@link CreditsShapeError} when body is not JSON, or balance is
 missing, mistyped, or not finite

 @example
 ```ts
 const credits = parseHyperCredits({ bodyText: reply.bodyText, },);
 ```
 */
export function parseHyperCredits(
  { bodyText, }: { readonly bodyText: string; },
): HyperCredits {
  /**
   Whole parsed body, probed field by field.
   */
  const parsed = parseCreditsJson({ bodyText, },);

  if (!isJsonRecord(parsed,))
    throw new CreditsShapeError({ detail: 'body is not a JSON object', },);

  /**
   Balance as delivered, before its type is checked.
   */
  const { balance, } = parsed;

  if ((typeof balance) !== 'number')
    throw new CreditsShapeError({ detail: 'balance is not a number', },);

  if (!Number.isFinite(balance,))
    throw new CreditsShapeError({ detail: 'balance is not a finite number', },);

  return { balance, };
}

//endregion Hyper credits
