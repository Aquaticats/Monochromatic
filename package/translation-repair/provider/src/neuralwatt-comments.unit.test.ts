/**
 Tests the reading of what NeuralWatt says about one streamed request in its
 comment lines: the energy the request consumed, in watt-hours, and what it
 was billed, in USD (`https://docs.neuralwatt.com/energy-methodology.md`,
 fetched 2026-10-20).

 Fixtures are the documented sample lines and cat-themed invention.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { readNeuralWattComments, } from '../dist/final/node/index.mjs';

/**
 The energy comment as the methodology page prints it: 0.000015 kWh.
 */
const ENERGY_LINE =
  ': energy {"energy_kwh": 0.000015, "energy_joules": 54.0, "attribution_method": "counter_exact_token_pool_weighted", "attribution_ratio": 0.25}';

/**
 The cost comment as the methodology page prints it.
 */
const COST_LINE = ': cost {"request_cost_usd": 0.00001, "cache_savings_usd": 0.0, "allowance_remaining_usd": 32.68}';

/**
 A free request's cost comment: the covered request costs nothing.
 */
const FREE_COST_LINE = ': cost {"request_cost_usd": 0, "cache_savings_usd": 0.0}';

/**
 The closing data chunk and sentinel around the comments.
 */
const CLOSING = 'data: {"id":"chatcmpl-whiskers","choices":[{"index":0,"delta":{"content":""},"finish_reason":"stop"}]}\n\n';

/**
 Builds a drained body from lines, each followed by a blank line.

 @param lines - body lines in order

 @returns Body text

 @example
 ```ts
 const bodyText = bodyOf([CLOSING, ENERGY_LINE,],);
 ```
 */
function bodyOf(lines: readonly string[],): string {
  return `${lines.join('\n\n',)}\n\ndata: [DONE]\n\n`;
}

await describe({
  name: readNeuralWattComments.name,
  children: [
    it({
      name: 'READS THE ENERGY IN WATT-HOURS and the cost in USD from the two documented comment lines, 0.000015 kWh '
        + 'being 0.015 Wh',
      fn: async () => {
        /**
         What the two documented lines say.
         */
        const read = readNeuralWattComments({ bodyText: bodyOf([CLOSING, ENERGY_LINE, COST_LINE,],), },);
        expect(read.energyWh,).toBeCloseTo(0.015, 12,);
        expect(read.requestCostUsd,).toBe(0.00001,);
      },
    },),

    it({
      name: 'READS A FREE REQUEST\'S COST AS ZERO, which is a statement that nothing was billed and not an absence',
      fn: async () => {
        expect(readNeuralWattComments({ bodyText: bodyOf([CLOSING, ENERGY_LINE, FREE_COST_LINE,],), },).requestCostUsd,)
          .toBe(0,);
      },
    },),

    it({
      name: 'LEAVES THE ENERGY ABSENT WHEN THE STREAM CARRIED NO ENERGY COMMENT, as the page says it is omitted '
        + 'when measurement is unavailable, and still reads the cost',
      fn: async () => {
        /**
         Reading of a stream with only a cost comment.
         */
        const read = readNeuralWattComments({ bodyText: bodyOf([CLOSING, FREE_COST_LINE,],), },);
        expect(read.energyWh,).toBeUndefined();
        expect(read.requestCostUsd,).toBe(0,);
      },
    },),

    it({
      name: 'LEAVES BOTH ABSENT FROM A STREAM WITH NEITHER COMMENT',
      fn: async () => {
        /**
         Reading of a stream with no comments at all.
         */
        const read = readNeuralWattComments({ bodyText: bodyOf([CLOSING,],), },);
        expect(read.energyWh,).toBeUndefined();
        expect(read.requestCostUsd,).toBeUndefined();
      },
    },),

    it({
      name: 'READS A STREAM WHOSE LINES END IN CARRIAGE RETURNS',
      fn: async () => {
        expect(readNeuralWattComments({
          bodyText: bodyOf([CLOSING, ENERGY_LINE, COST_LINE,],)
            .replaceAll('\n', '\r\n',),
        },).energyWh,).toBeCloseTo(0.015, 12,);
      },
    },),

    it({
      name: 'TAKES THE LAST OF A REPEATED COMMENT, the way the last usage block of a stream wins',
      fn: async () => {
        expect(readNeuralWattComments({
          bodyText: bodyOf([
            ': energy {"energy_kwh": 0.000001}',
            ': energy {"energy_kwh": 0.000002}',],),
        },).energyWh,).toBeCloseTo(0.002, 12,);
      },
    },),

    it({
      name: 'READS NOTHING FROM A COMMENT THAT IS NOT WHOLE OR NOT A QUANTITY: broken JSON, a negative or '
        + 'non-finite figure, a string, a missing field, so a bad comment never fails a call that was paid for',
      fn: async () => {
        for (
          const line of [
            ': energy {"energy_kwh": 0.0000',
            ': energy not json',
            ': energy {"energy_kwh": -0.000001}',
            ': energy {"energy_kwh": "0.000015"}',
            ': energy {"energy_kwh": null}',
            ': energy {"energy_joules": 54.0}',
            ': energy {"measurement_available": false}',
            ': energy [1, 2]',
            ': energy {"energy_kwh": 1e999}',
          ]
        )
          expect(readNeuralWattComments({ bodyText: bodyOf([CLOSING, line,],), },).energyWh,).toBeUndefined();
        for (
          const line of [
            ': cost {"request_cost_usd": -1}',
            ': cost {"request_cost_usd": "0"}',
            ': cost {}',
            ': cost nope',
          ]
        )
          expect(readNeuralWattComments({ bodyText: bodyOf([CLOSING, line,],), },).requestCostUsd,).toBeUndefined();
      },
    },),

    it({
      name: 'READS NO COMMENT OUT OF A DATA LINE THAT QUOTES ONE, since a model\'s own words ride inside data lines',
      fn: async () => {
        /**
         A content chunk whose text imitates the energy comment.
         */
        const quoting = String.raw`data: {"choices":[{"index":0,"delta":{"content":": energy {\"energy_kwh\": 9}"}}]}`;
        expect(readNeuralWattComments({ bodyText: bodyOf([quoting,],), },).energyWh,).toBeUndefined();
      },
    },),
  ],
},);
