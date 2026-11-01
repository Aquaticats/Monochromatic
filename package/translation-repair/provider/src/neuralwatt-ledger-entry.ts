import {
  errorName,
  isIsoStampText,
} from '@monochromatic-dev/translation-repair-markup/ts';
import { isJsonRecord, } from '@monochromatic-dev/translation-repair-document-structure/ts';

import { NeuralWattLedgerShapeError, } from './neuralwatt-ledger-config.ts';

//region NeuralWatt ledger entry
// ONE LINE OF THE ENERGY LEDGER, and the reading of it. APPEND-ONLY JSON
// LINES rather than a rewritten total, so two processes writing at once (a
// pass and a budget sample) cannot lose each other's calls, and a reader can
// see what was bought and when.
//
// A LINE THE FIRST KEYED RUN WROTE (2026-10-20, 22:23 UTC) CARRIES `billedUsd`,
// a field the ledger no longer writes. THE READER TAKES THE NAMED FIELDS AND IGNORES ANY
// OTHER, so such a line reads as an ordinary energy line whatever the field
// holds: NeuralWatt reports a nominal `request_cost_usd` on free-energy requests
// too (the owner's dashboard counted a 0.0167 Wh request that reported 0.000107
// USD wholly against the free energy), so a cost is not a billing signal.

/**
 Why a line's watt-hours are an estimate rather than a measurement.

 - `unreported-estimate`: the call finished and its stream carried no
   `: energy` comment, so the estimate stands.
 - `abandoned-estimate`: an attempt delivered something and then failed, so
   the energy it used was never reported; noted at its estimate.
 */
export const NEURALWATT_RECKONINGS = [
  'unreported-estimate',
  'abandoned-estimate',
] as const;

/**
 One of `NEURALWATT_RECKONINGS`.
 */
export type NeuralWattReckoning = typeof NEURALWATT_RECKONINGS[number];

/**
 One call, as the ledger keeps it.

 @example
 ```ts
 const entry: NeuralWattLedgerEntry = { at: '2026-10-20T12:00:00.000Z', model: 'glm-5.3-flash', wh: 0.015, promptTokens: 90, };
 ```
 */
export type NeuralWattLedgerEntry = {
  /**
   When the line was written, as `toISOString` spells it.
   */
  readonly at: string;

  /**
   Model as this provider spells it.
   */
  readonly model: string;

  /**
   Watt-hours the call used: measured unless `estimated` says otherwise.
   */
  readonly wh: number;

  /**
   Prompt tokens the estimate was drawn for.
   */
  readonly promptTokens: number;

  /**
   Present when `wh` is an estimate, saying why.
   */
  readonly estimated?: NeuralWattReckoning;
};

/**
 Whether a value is a reckoning this package writes.

 @param value - parsed field

 @returns Whether it is one of `NEURALWATT_RECKONINGS`

 @example
 ```ts
 isNeuralWattReckoning('abandoned-estimate',); // true
 ```
 */
export function isNeuralWattReckoning(value: unknown,): value is NeuralWattReckoning {
  return NEURALWATT_RECKONINGS.some(function isThis(reckoning,): boolean {
    return reckoning === value;
  },);
}

/**
 Reads one ledger line as an entry, naming the file and the line when it will
 not read.

 @param path - ledger file, for the error

 @param text - one line of the file

 @param line - its one-based number, for the error

 @returns Entry the line holds

 @throws {@link NeuralWattLedgerShapeError} when the line is not an entry

 @example
 ```ts
 const entry = neuralWattEntryOf({ path, text, line: 1, },);
 ```
 */
export function neuralWattEntryOf(
  {
    path,
    text,
    line,
  }: {
    readonly path: string;
    readonly text: string;
    readonly line: number;
  },
): NeuralWattLedgerEntry {
  /**
   Raises the shape error for this line with a given phrase.

   @param detail - authored phrase naming the broken expectation

   */
  function refuse(detail: string,): never {
    throw new NeuralWattLedgerShapeError({
      path,
      line,
      detail,
    },);
  }

  /**
   Parsed line, unknown until its shape is checked.
   */
  const parsed: unknown = (function parse(): unknown {
    try {
      return JSON.parse(text,);
    }
    catch (error) {
      return refuse(`not valid JSON (${errorName({ error, },)})`,);
    }
  })();
  if (!isJsonRecord(parsed,))
    return refuse('not a JSON object',);

  /**
   Fields as the line carries them, each still unknown.
   */
  const {
    at,
    model,
    wh,
    promptTokens,
    estimated,
  } = parsed;
  // Only the spelling `toISOString` writes: `Date.parse` alone also reads a
  // date-time with no zone as local time, which would move the UTC day.
  if (((typeof at) !== 'string') || (!isIsoStampText({ text: at, },)))
    return refuse('at is not an instant spelled as toISOString writes it',);
  if ((typeof model) !== 'string')
    return refuse('model is not a string',);
  if (((typeof wh) !== 'number') || (!Number.isFinite(wh,))
    || (wh < 0))
    return refuse('wh is not a non-negative number',);
  if (((typeof promptTokens) !== 'number') || (!Number.isFinite(promptTokens,)))
    return refuse('promptTokens is not a number',);
  if ((estimated !== undefined) && (!isNeuralWattReckoning(estimated,)))
    return refuse('estimated is not a reckoning this package writes',);
  return {
    at,
    model,
    wh,
    promptTokens,
    // Conditional spreads keep an absent optional field absent.
    ...((estimated === undefined) ? {} : { estimated, }),
  };
}

//endregion NeuralWatt ledger entry
