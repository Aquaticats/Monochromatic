import { isJsonRecord, } from '@monochromatic-dev/translation-repair-document-structure/ts';
import { isUnsignedNumberText, } from '@monochromatic-dev/translation-repair-markup/ts';

//region DeepSeek balance
// Typed view over `GET /user/balance`, shape read from the parent session's
// live call of 2026-10-22, which answered
// `{"is_available":true,"balance_infos":[{"currency":"USD","total_balance":"9.96",`
// `"granted_balance":"0.00","topped_up_balance":"9.96"}]}`
// (DeepSeek's balance reference, `https://api-docs.deepseek.com/api/get-user-balance`).
//
// THE AMOUNTS ARE TEXT, not numbers: `"total_balance":"9.96"`. Each is read
// once, by the rule for a number a service writes as text
// (`isUnsignedNumberText`, then finite), so an empty string, a sign, a hex
// spelling or a word is a shape error and not a balance of zero, which would
// read as an emptied provider.
//
// ONE FIGURE DECIDES, `total_balance` in USD, and `is_available` beside it: the
// account is dry when DeepSeek says it cannot serve or the balance is gone
// (`budget-routing.ts`). A body with no USD entry is a shape error, since a
// balance in another currency cannot be compared with a USD floor.
//
// A malformed body is a provider protocol failure and throws, matching
// `openrouter-credits.ts`: model-content defects flow as data, protocol
// defects do not.

/**
 Balance at one instant, as DeepSeek reports it.

 @example
 ```ts
 const balance: DeepSeekBalance = { isAvailable: true, totalUsd: 9.96, };
 ```
 */
export type DeepSeekBalance = {
  /**
   Whether DeepSeek says the account can make API calls (`is_available`).
   */
  readonly isAvailable: boolean;

  /**
   USD the account holds, granted and topped up together (`total_balance`).
   */
  readonly totalUsd: number;
};

/**
 Signals a `/user/balance` body that refused to parse or lacked its fields;
 always a provider protocol failure, never a model defect.

 @example
 ```ts
 throw new DeepSeekBalanceShapeError({ detail: 'is_available is not a boolean', },);
 ```
 */
export class DeepSeekBalanceShapeError extends Error {
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
   new DeepSeekBalanceShapeError({ detail: 'body is not valid JSON', cause: error, },);
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
      `DeepSeek /user/balance body violated expectations: ${detail}`,
      // Conditional spread keeps cause absent when none was supplied.
      ...(cause === undefined
        ? []
        : [{ cause, },]),
    );
    this.name = 'DeepSeekBalanceShapeError';
  }
}

/**
 Currency the floor and the meter are stated in.
 */
const BALANCE_CURRENCY = 'USD';

/**
 Reads one amount written as text, refusing anything but a plain unsigned number.

 @param text - amount as DeepSeek wrote it

 @returns Its finite value

 @throws {@link DeepSeekBalanceShapeError} when the text is not an unsigned
 number as JSON spells one, or is not finite

 @example
 ```ts
 const usd = readBalanceText({ text: '9.96', },);
 ```
 */
function readBalanceText({ text, }: { readonly text: string; },): number {
  /**
   The text as a number, NaN or infinite where it is not a finite one.
   */
  const read = Number(text,);
  if ((!isUnsignedNumberText({ text, },)) || (!Number.isFinite(read,)))
    throw new DeepSeekBalanceShapeError({ detail: 'total_balance is not an unsigned decimal amount', },);
  return read;
}

/**
 Parses one `/user/balance` body into the typed balance.

 @param bodyText - raw 200-response body

 @returns Whether the account is available, and its USD balance

 @throws {@link DeepSeekBalanceShapeError} when the body is not JSON, is not
 an object, lacks a boolean `is_available`, lacks a `balance_infos` array,
 holds no USD entry (or more than one), or that entry's `total_balance` is
 not an unsigned decimal amount written as text

 @example
 ```ts
 const balance = parseDeepSeekBalance({ bodyText: reply.bodyText, },);
 ```
 */
export function parseDeepSeekBalance({ bodyText, }: { readonly bodyText: string; },): DeepSeekBalance {
  /**
   Whole parsed body, probed field by field.
   */
  const parsed: unknown = (function parseBody(): unknown {
    try {
      return JSON.parse(bodyText,);
    } catch (error) {
      throw new DeepSeekBalanceShapeError({
        detail: 'body is not valid JSON',
        cause: error,
      },);
    }
  })();

  if (!isJsonRecord(parsed,))
    throw new DeepSeekBalanceShapeError({ detail: 'body is not a JSON object', },);

  /**
   Availability and the per-currency entries.
   */
  const {
    is_available: isAvailable,
    balance_infos: infos,
  } = parsed;

  if ((typeof isAvailable) !== 'boolean')
    throw new DeepSeekBalanceShapeError({ detail: 'is_available is not a boolean', },);

  if (!Array.isArray(infos,))
    throw new DeepSeekBalanceShapeError({ detail: 'balance_infos is not an array', },);

  /**
   Entries stated in USD.
   */
  const usd = infos.filter(function isUsd(info: unknown,): boolean {
    return isJsonRecord(info,) && (info.currency === BALANCE_CURRENCY);
  },);

  if (usd.length !== 1)
    throw new DeepSeekBalanceShapeError({ detail: 'balance_infos does not hold exactly one USD entry', },);

  /**
   The one USD entry.
   */
  const entry: unknown = usd[0];
  if (!isJsonRecord(entry,))
    throw new DeepSeekBalanceShapeError({ detail: 'the USD entry is not a JSON object', },);

  /**
   Total as written.
   */
  const { total_balance: total, } = entry;
  if ((typeof total) !== 'string')
    throw new DeepSeekBalanceShapeError({ detail: 'total_balance is not a string', },);

  return {
    isAvailable,
    totalUsd: readBalanceText({ text: total, },),
  };
}

//endregion DeepSeek balance
