import { errorName, } from '@monochromatic-dev/translation-repair-markup/ts';
import { isJsonRecord, } from '@monochromatic-dev/translation-repair-document-structure/ts';
import {
  type NeuralWattAliasMap,
  servedByCoveredAlias,
} from './neuralwatt-aliases.ts';
import { NEURALWATT_MODELS, } from './neuralwatt-catalog.ts';

//region NeuralWatt account usage
// THE ENERGY THE WHOLE ACCOUNT SPENT ON THE COVERED MODELS in a window, read from
// `GET /v1/usage/by-model` (`https://docs.neuralwatt.com/api-reference/usage/get-usage-broken-down-by-model`,
// fetched 2026-10-20). The route counts every key on the account, which is what the daily free energy is spent by:
// the owner also uses the covered models on the same account, where this package's ledger cannot see it. Measured
// at 23:15 UTC that day it was current to the request (4.095 Wh at the route against 4.094 Wh in the ledger).
//
// `products[]` ROWS, not `models[]`: a row there is a (requested model, served model) pair, so `glm-5.3-flash`
// and `glm-5.3-flash-flex` are separate rows and the speed lanes are told from the standard ones by the name that
// was called. A row counts when its `requested_model` is a NeuralWatt served id of a card
// (`NEURALWATT_MODELS`) or that id with `-flex`: the free energy "covers each covered model's standard and Flex
// tiers" and "Speed lanes are billed normally" (`https://docs.neuralwatt.com/billing/daily-free-energy`). A model
// the free energy does not cover (the account's `mimo-v2.6-pro` had used 3.83 kWh that day) must not count
// against it.
//
// A CALL MADE UNDER A TRACKING ALIAS counts too (`nw-flash`, whose row carries the alias as its `requested_model`):
// the names that point at a covered model come from the models listing, or from the served model of the row where
// the listing cannot be read (`neuralwatt-aliases.ts`). The owner uses DeepSeek V4.1 Flash on the same account, and a
// count blind to the alias undercounts.
//
// `energy_kwh` IS CONSUMED ENERGY. A flex request is charged at 0.65 of what it consumed
// (`https://docs.neuralwatt.com/api/usage`), so counting consumed energy for a flex row over-counts what the free
// energy was charged, which is the safe side for a stop whose job is to keep calls off the owner's plan.
//
// THE REPLY'S `period` IS CHECKED AGAINST THE WINDOW ASKED. The documentation says an unparseable date falls back to
// a default window and still answers 200, and a date-only window with one day at both ends measured as an empty,
// zero-length one. A figure for a window nobody asked for is worse than none, so a period that is not the asked one
// is refused.

/**
 Watt-hours in one kilowatt-hour.
 */
const WH_PER_KWH = 1_000;

/**
 Suffix of the flex lane's name, which the free energy covers.
 */
const FLEX_SUFFIX = '-flex';

/**
 Raised when a by-model reply will not read as the documented shape, or answers for another window than the one asked.

 NEVER QUOTES THE BODY: the reply is an account's usage record.

 @example
 ```ts
 throw new NeuralWattUsageShapeError({ detail: 'products is not a list', },);
 ```
 */
export class NeuralWattUsageShapeError extends Error {
  /**
   Declares this message safe to forward: an authored phrase.
   */
  readonly messageNamesOnly: true = true;

  /**
   Names what was wrong with the reply.

   @param detail - authored phrase naming the broken expectation

   @example
   ```ts
   new NeuralWattUsageShapeError({ detail: 'not a JSON object', },);
   ```
   */
  public constructor({ detail, }: { readonly detail: string; },) {
    super(`NeuralWatt by-model usage reply violated expectations: ${detail}`,);
    this.name = 'NeuralWattUsageShapeError';
  }
}

/**
 Names the requested models whose energy counts against the free energy: each served id of a card and its flex lane.

 @returns The names as a set

 @example
 ```ts
 coveredNames().has('glm-5.3-flash-flex',); // true
 ```
 */
function coveredNames(): ReadonlySet<string> {
  return new Set(Object.keys(NEURALWATT_MODELS,)
    .flatMap(function withFlex(id,): readonly string[] {
      return [
        id,
        `${id}${FLEX_SUFFIX}`,
      ];
    },),);
}

/**
 One end of the echoed period.

 @param period - the reply's `period` object

 @param field - `start` or `end`

 @returns The instant

 @throws {@link NeuralWattUsageShapeError} when the field is not an instant

 @example
 ```ts
 const start = echoedAt({ period, field: 'start', },);
 ```
 */
function echoedAt({
  period,
  field,
}: {
  readonly period: Readonly<Record<string, unknown>>;
  readonly field: string
},): Date {
  /**
   The field as the reply spelled it.
   */
  const spelled = period[field];
  if ((typeof spelled) !== 'string')
    throw new NeuralWattUsageShapeError({ detail: `period.${field} is not a string`, },);

  /**
   Epoch milliseconds, NaN where the text is not an instant.
   */
  const ms = Date.parse(spelled as string,);
  if (!Number.isFinite(ms,))
    throw new NeuralWattUsageShapeError({ detail: `period.${field} is not an instant`, },);
  return new Date(ms,);
}

/**
 Whether a usage row is for a call that counts against the free energy: a covered model's name or its flex twin, or
 an alias of one.

 @param row - a `products[]` row

 @param covered - the covered models' names with their flex twins

 @param aliases - the tracking aliases, or that they could not be told

 @returns Whether the row counts

 @example
 ```ts
 rowIsCovered({ row, covered, aliases, },);
 ```
 */
function rowIsCovered(
  {
    row,
    covered,
    aliases,
  }: {
    readonly row: Readonly<Record<string, unknown>>;
    readonly covered: ReadonlySet<string>;
    readonly aliases: NeuralWattAliasMap;
  },
): boolean {
  /**
   The name the call used.
   */
  const requestedModel = String(row.requested_model,);
  if (covered.has(requestedModel,))
    return true;
  if (aliases.known)
    return aliases.names
      .has(requestedModel,);
  return servedByCoveredAlias({
    requestedModel,
    servedModelId: String(row.served_model_id,),
  },);
}

/**
 Reads the covered energy out of a by-model reply, after checking that it answers for the window asked.

 @param bodyText - body of a 200 reply

 @param asked - the window the request named

 @param aliases - the tracking aliases that point at a covered model, or that the listing could not say

 @returns Watt-hours the covered models' rows add up to

 @throws {@link NeuralWattUsageShapeError} when the body is not the documented shape, the period is not the asked
 window, or a covered row carries no usable `energy_kwh`

 @example
 ```ts
 const accountWh = coveredWhOf({ bodyText: reply.bodyText, asked: { start, end, }, aliases, },);
 ```
 */
export function coveredWhOf(
  {
    bodyText,
    asked,
    aliases,
  }: {
    readonly bodyText: string;
    readonly asked: {
      readonly start: Date;
      readonly end: Date
    };
    readonly aliases: NeuralWattAliasMap;
  },
): number {
  /**
   Parsed body, unknown until checked.
   */
  const parsed: unknown = (function parse(): unknown {
    try {
      return JSON.parse(bodyText,);
    }
    catch (error) {
      throw new NeuralWattUsageShapeError({ detail: `not valid JSON (${errorName({ error, },)})`, },);
    }
  })();
  if (!isJsonRecord(parsed,))
    throw new NeuralWattUsageShapeError({ detail: 'not a JSON object', },);

  /**
   The echoed window and the rows.
   */
  const {
    period,
    products,
  } = parsed;
  if (!isJsonRecord(period,))
    throw new NeuralWattUsageShapeError({ detail: 'period is not an object', },);

  /**
   The window the reply answers for.
   */
  const echoed = {
    start: echoedAt({
      period,
      field: 'start',
    },),
    end: echoedAt({
      period,
      field: 'end',
    },),
  };
  // THE WINDOWS ARE COMPARED IN THE SPELLING `toISOString` WRITES, which names an instant once whatever offset or
  // fraction the reply used.
  if ((echoed.start
    .toISOString()
    !== asked.start
      .toISOString()) || (echoed.end
        .toISOString()
        !== asked.end
          .toISOString()))
    throw new NeuralWattUsageShapeError({
      detail: `period does not match the window asked (asked ${asked.start
        .toISOString()} to ${asked.end
          .toISOString()}, `
        + `the reply answers for ${echoed.start
          .toISOString()} to ${echoed.end
            .toISOString()})`,
    },);
  if (!Array.isArray(products,))
    throw new NeuralWattUsageShapeError({ detail: 'products is not a list', },);

  /**
   Names that count against the free energy.
   */
  const covered = coveredNames();
  return (products as readonly unknown[]).reduce<number>(
    function addRow(
      running,
      row,
    ): number {
      if (!isJsonRecord(row,))
        throw new NeuralWattUsageShapeError({ detail: 'a products row is not an object', },);
      if (!rowIsCovered({
        row,
        covered,
        aliases,
      },))
        return running;

    /**
     Energy the row consumed, in kWh.
     */
      const kwh = row.energy_kwh;
      if (((typeof kwh) !== 'number') || (!Number.isFinite(kwh,))
        || ((kwh as number) < 0))
        throw new NeuralWattUsageShapeError({
          detail: `energy_kwh of the ${String(row.requested_model,)} row is not a non-negative number`,
        },);
      return running + ((kwh as number) * WH_PER_KWH);
    },
    0,
  );
}

//endregion NeuralWatt account usage
