import { tagged, } from '@monochromatic-dev/logger-core/ts';

import { contextRoot, } from '@monochromatic-dev/translation-repair-typography/ts';
import { sseCommentOf, } from '@monochromatic-dev/translation-repair-provider-stream/ts';
import { errorName, } from '@monochromatic-dev/translation-repair-markup/ts';
import { isJsonRecord, } from '@monochromatic-dev/translation-repair-document-structure/ts';

//region NeuralWatt stream comments
// WHAT NEURALWATT SAYS ABOUT ONE STREAMED REQUEST, in two comment lines sent
// before `data: [DONE]` (`https://docs.neuralwatt.com/energy-methodology.md`,
// fetched 2026-10-20):
//
//   : energy {"energy_kwh": 0.000015, "energy_joules": 54.0, ...}
//   : cost {"request_cost_usd": 0.00001, "cache_savings_usd": 0.0, ...}
//
// The `energy` comment appears only when measurement is available. The `cost`
// comment appears on every request with cost metadata, "including trial and
// free requests, where `request_cost_usd` may be `0`". `energy_kwh` is the
// energy the request CONSUMED; what is charged is that times the lane's
// pricing multiplier, which is 1.0 on the standard lane this pipeline uses.
//
// A COMMENT THAT WILL NOT READ IS READ AS ABSENT, NEVER AS AN ERROR. The call
// has been paid for by the time its comments are read, and failing it would
// throw away an answer to say the bookkeeping beside it was odd. The cost of
// reading a bad comment as absent is that the call settles at its estimate,
// marked estimated, and that is said on a warning with the reason.

/**
 Logger root for the comment reader.
 */
const l = contextRoot({ tag: 'translation-repair', },);

/**
 Watt-hours in one kilowatt-hour.
 */
const WH_PER_KWH = 1_000;

/**
 What the comments of one stream say, each field absent when its comment was
 absent or would not read.

 @example
 ```ts
 const comments: NeuralWattComments = { energyWh: 0.015, requestCostUsd: 0, };
 ```
 */
export type NeuralWattComments = {
  /**
   Energy the request consumed, in watt-hours.
   */
  readonly energyWh?: number;

  /**
   USD cost the stream reports for the request: zero for some, and a small
   nominal figure for others that free-energy requests carry too, which is no
   sign of plan spend.
   */
  readonly requestCostUsd?: number;
};

/**
 Reads one comment's JSON as the record it carries.

 @param text - what the comment carries after its name

 @param name - comment's name, for the warning

 @returns One record, or none when the text is not a JSON object

 @example
 ```ts
 const [record,] = commentRecordsOf({ text: '{"energy_kwh":1}', name: 'energy', },);
 ```
 */
function commentRecordsOf(
  {
    text,
    name,
  }: {
    readonly text: string;
    readonly name: string;
  },
): readonly Readonly<Record<string, unknown>>[] {
  /**
   Logger pre-tagged with this function's name.
   */
  const rl = tagged({
    tag: commentRecordsOf.name,
    l,
  },);
  try {
    /**
     Parsed comment, unknown until checked.
     */
    const parsed: unknown = JSON.parse(text,);
    if (isJsonRecord(parsed,))
      return [parsed,];
    rl.warn(`the ${name} comment is not a JSON object; it is read as absent`,);
    return [];
  }
  catch (error) {
    rl.warn(`the ${name} comment is not valid JSON (${errorName({ error, },)}); it is read as absent`,);
    return [];
  }
}

/**
 Values of one named comment's field across a drained body, each a finite
 non-negative number, in stream order.

 @param bodyText - whole drained body

 @param name - comment's name

 @param field - field of its JSON to read

 @returns Numbers; empty when no comment of the name carried one

 @example
 ```ts
 const kwh = quantitiesOf({ bodyText, name: 'energy', field: 'energy_kwh', },).at(-1,);
 ```
 */
function quantitiesOf(
  {
    bodyText,
    name,
    field,
  }: {
    readonly bodyText: string;
    readonly name: string;
    readonly field: string;
  },
): readonly number[] {
  /**
   Logger pre-tagged with this function's name.
   */
  const rl = tagged({
    tag: quantitiesOf.name,
    l,
  },);
  return bodyText
    .split('\n',)
    .flatMap(function commentsOfLine(line,): readonly Readonly<Record<string, unknown>>[] {
      /**
       Text this line's comment carries, empty for any other line.
       */
      const text = sseCommentOf({
        line,
        name,
      },);
      return (text === '') ? [] : commentRecordsOf({
        text,
        name,
      },);
    },)
    .flatMap(function quantityOfRecord(record,): readonly number[] {
      /**
       The field as the comment carries it.
       */
      const quantity = record[field];
      if (((typeof quantity) === 'number') && Number.isFinite(quantity,)
        && (quantity >= 0))
        return [quantity,];
      rl.warn(`the ${name} comment carries no ${field} that is a non-negative number; it is read as absent`,);
      return [];
    },);
}

/**
 Reads the energy and the cost NeuralWatt reported for one streamed request.

 THE LAST COMMENT OF A NAME WINS, since the stream's own accounting closes it.

 @param bodyText - whole drained body, comment lines included

 @returns What the comments said, each field absent when its comment was

 @example
 ```ts
 const { energyWh, requestCostUsd, } = readNeuralWattComments({ bodyText: reply.bodyText, },);
 ```
 */
export function readNeuralWattComments({ bodyText, }: { readonly bodyText: string; },): NeuralWattComments {
  /**
   Energy in kilowatt-hours, as the last energy comment states it.
   */
  const energyKwh = quantitiesOf({
    bodyText,
    name: 'energy',
    field: 'energy_kwh',
  },)
    .at(-1,);

  /**
   USD cost as the last cost comment states it, nominal on free-energy requests.
   */
  const requestCostUsd = quantitiesOf({
    bodyText,
    name: 'cost',
    field: 'request_cost_usd',
  },)
    .at(-1,);
  return {
    // Conditional spreads keep an absent comment absent instead of undefined.
    ...((energyKwh === undefined) ? {} : { energyWh: energyKwh * WH_PER_KWH, }),
    ...((requestCostUsd === undefined) ? {} : { requestCostUsd, }),
  };
}

//endregion NeuralWatt stream comments
