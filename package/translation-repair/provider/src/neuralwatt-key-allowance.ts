import { isJsonRecord, } from '@monochromatic-dev/translation-repair-document-structure/ts';
import { neuralWattDayAfter, } from './neuralwatt-day.ts';

//region NeuralWatt key allowance
// THE PER-KEY CAP THE OWNER SET. The owner, 2026-10-20: "I've also just setup a 1kWh daily limit on this specific api
// key." `GET /v1/quota` documents it as `key.allowance` (the `KeyAllowance` schema of
// `https://docs.neuralwatt.com/api/quota`, read 2026-10-20): null for an uncapped key, else `limit_usd`, `period`,
// `spent_usd`, `remaining_usd` (`max(0, limit_usd - spent_usd)`), `blocked` ("True once spend has reached the limit,
// further requests are refused") and `reset_at`. The same reply read at 23:15 and 23:20 UTC that day still said
// `allowance: null`, so what the cap reports once it is live is unconfirmed; this reads only what the schema states.
//
// IT IS THE BACKSTOP, NOT THE STOP: the free energy (550 Wh) is under the cap (1 kWh), so the ledger and the account
// figure stop the day first. A cap in USD does not convert to energy here and no kWh field is invented for it.
//
// A MALFORMED ALLOWANCE IS NAMED AND NOT BELIEVED: the quota baseline of the run is read from the same reply, and a
// cap that cannot be read must neither break that nor guess.

/**
 What the key's allowance says, the fields the meter acts on and prints.

 @example
 ```ts
 const allowance: NeuralWattKeyAllowance = { remainingUsd: 3.25, blocked: false, };
 ```
 */
export type NeuralWattKeyAllowance = {
  /**
   What is left of the cap, in USD.
   */
  readonly remainingUsd: number;

  /**
   Whether further requests are refused.
   */
  readonly blocked: boolean;

  /**
   When the cap's window ends, as `toISOString` spells it; absent where the reply gave null.
   */
  readonly resetAt?: string;
};

/**
 What a quota reply's `key` object turned out to hold.

 @example
 ```ts
 const reading: NeuralWattKeyAllowanceReading = { problem: 'remaining_usd is not a non-negative number', };
 ```
 */
export type NeuralWattKeyAllowanceReading = {
  /**
   The allowance, absent for an uncapped key or one that will not read.
   */
  readonly allowance?: NeuralWattKeyAllowance;

  /**
   Why the allowance will not read, an authored phrase; absent when it read or the key has none.
   */
  readonly problem?: string;
};

/**
 Whether a value is a usable amount of money.

 @param value - the field

 @returns Whether it is a finite non-negative number

 @example
 ```ts
 isAmount(3.25,); // true
 ```
 */
function isAmount(value: unknown,): value is number {
  return ((typeof value) === 'number') && Number.isFinite(value,)
    && ((value as number) >= 0);
}

/**
 Reads the allowance object of a quota reply's `key`.

 @param allowance - the object under `key.allowance`

 @returns The allowance, or why it will not read

 @example
 ```ts
 const reading = allowanceFrom({ allowance: { remaining_usd: 0, blocked: true, reset_at: null, }, },);
 ```
 */
function allowanceFrom({ allowance, }: { readonly allowance: Readonly<Record<string, unknown>>; },): NeuralWattKeyAllowanceReading {
  /**
   The fields the schema states, each of unknown type until checked.
   */
  const {
    remaining_usd: remainingUsd,
    blocked,
    reset_at: resetAt,
    limit_usd: limitUsd,
    spent_usd: spentUsd,
    period,
  } = allowance;
  if (!isAmount(remainingUsd,))
    return { problem: 'key.allowance.remaining_usd is not a non-negative number', };
  if ((typeof blocked) !== 'boolean')
    return { problem: 'key.allowance.blocked is not a boolean', };
  if ((limitUsd !== undefined) && (!isAmount(limitUsd,)))
    return { problem: 'key.allowance.limit_usd is not a non-negative number', };
  if ((spentUsd !== undefined) && (!isAmount(spentUsd,)))
    return { problem: 'key.allowance.spent_usd is not a non-negative number', };
  if ((period !== undefined) && ((typeof period) !== 'string'))
    return { problem: 'key.allowance.period is not a string', };
  if ((resetAt === undefined) || (resetAt === null))
    return { allowance: {
      remainingUsd,
      blocked,
    }, };

  /**
   Epoch milliseconds of the reset, NaN where the field is not an instant.
   */
  const resetMs = ((typeof resetAt) === 'string') ? Date.parse(resetAt as string,) : Number.NaN;
  if (!Number.isFinite(resetMs,))
    return { problem: 'key.allowance.reset_at is neither an ISO 8601 instant nor null', };
  return {
    allowance: {
      remainingUsd,
      blocked,
      resetAt: new Date(resetMs,).toISOString(),
    },
  };
}

/**
 Reads the allowance out of a parsed quota reply.

 @param key - the reply's `key` field, whatever it holds

 @returns The allowance, or why it will not read; neither for a key with no cap

 @example
 ```ts
 const reading = keyAllowanceOf({ key: parsed['key'], },);
 ```
 */
export function keyAllowanceOf({ key, }: { readonly key: unknown; },): NeuralWattKeyAllowanceReading {
  if ((key === undefined) || (key === null))
    return {};
  if (!isJsonRecord(key,))
    return { problem: 'key is neither an object nor null', };

  /**
   The cap, null for an uncapped key.
   */
  const { allowance, } = key;
  if ((allowance === undefined) || (allowance === null))
    return {};
  if (!isJsonRecord(allowance,))
    return { problem: 'key.allowance is neither an object nor null', };
  return allowanceFrom({ allowance, },);
}

/**
 When a blocked cap lets requests through again: the cap's own `reset_at`, else the next day's anchor.

 @param allowance - the cap, read as blocked

 @param start - first instant of the current day, the next day's anchor being a day after it

 @returns The instant

 @example
 ```ts
 const until = blockedUntilOf({ allowance, start: new Date('2026-10-20T19:15:08Z',), },);
 ```
 */
export function blockedUntilOf(
  {
    allowance,
    start,
  }: {
    readonly allowance: NeuralWattKeyAllowance;
    readonly start: Date;
  },
): Date {
  if (allowance.resetAt === undefined)
    return neuralWattDayAfter({ start, },);
  return new Date(allowance.resetAt,);
}

//endregion NeuralWatt key allowance
