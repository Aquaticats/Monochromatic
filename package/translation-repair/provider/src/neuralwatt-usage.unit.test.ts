/**
 Tests the reading of the covered energy out of `GET /v1/usage/by-model` (`https://docs.neuralwatt.com/api/usage`, read
 2026-10-20): the rows of the models the free energy covers add up, in watt-hours, and a reply that answers for another
 window than the one asked is refused.

 Fixtures are cat-themed invention.

 @module
 */

import {
  caught,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  coveredWhOf,
  NeuralWattUsageShapeError,
} from '../dist/final/node/index.mjs';

/**
 The window the cases ask for.
 */
const ASKED = {
  start: new Date('2026-10-20T19:15:08.000Z',),
  end: new Date('2026-10-20T23:15:00.741Z',),
};

/**
 The name map of a case that meets no alias.
 */
const NO_ALIASES = { known: true, names: new Set<string>(), } as const;

/**
 The period a reply echoes for that window, spelled as the service spells it.
 */
const ECHO = {
  start: '2026-10-20T19:15:08+00:00',
  end: '2026-10-20T23:15:00.741000+00:00',
};

/**
 A reply body for a period and a list of rows.

 @param period - what the reply echoes

 @param products - its rows

 @returns Body text
 */
function bodyOf({ period, products, }: { readonly period: unknown; readonly products: unknown },): string {
  return JSON.stringify({ period, products, },);
}

await describe({
  name: 'NeuralWatt account usage',
  concurrency: 1,
  children: [
    describe({
      name: coveredWhOf.name,
      children: [
        it({
          name: 'ADDS THE ROWS OF COVERED MODELS AND THEIR FLEX LANES in watt-hours, and leaves speed and unknown models out',
          fn: async () => {
            expect(coveredWhOf({
              bodyText: bodyOf({
                period: ECHO,
                products: [
                  { requested_model: 'glm-5.3-flash', energy_kwh: 0.0025, },
                  { requested_model: 'glm-5.3-flash-flex', energy_kwh: 0.001, },
                  { requested_model: 'glm-5.3-flash-speed', energy_kwh: 9, },
                  { requested_model: 'cat-1', energy_kwh: 9, },
                ],
              },),
              asked: ASKED,
              aliases: NO_ALIASES,
            },),).toBeCloseTo(3.5, 9,);
            expect(coveredWhOf({
              bodyText: bodyOf({ period: ECHO, products: [], },),
              asked: ASKED,
              aliases: NO_ALIASES,
            },),).toBe(0,);
          },
        },),
        it({
          name: 'ACCEPTS AN ECHO THAT NAMES THE SAME INSTANTS in another spelling, and REFUSES A WINDOW nobody asked for',
          fn: async () => {
            expect(coveredWhOf({
              bodyText: bodyOf({
                period: {
                  start: '2026-10-20T19:15:08.000Z',
                  end: '2026-10-20T19:15:00.741-04:00',
                },
                products: [],
              },),
              asked: ASKED,
              aliases: NO_ALIASES,
            },),).toBe(0,);
            for (
              const period of [
                { start: '2026-10-20T00:00:00+00:00', end: ECHO.end, },
                { start: ECHO.start, end: '2026-10-20T23:15:00.742000+00:00', },
              ]
            ) {
              /**
               What the reader raised for this echo.
               */
              const refusal = caught(function readOtherWindow(): unknown {
                return coveredWhOf({
                  bodyText: bodyOf({ period, products: [], },),
                  asked: ASKED,
                  aliases: NO_ALIASES,
                },);
              },);
              expect(refusal,).toBeInstanceOf(NeuralWattUsageShapeError,);
              expect((refusal as Error).message,).toContain('period does not match the window asked',);
            }
          },
        },),
        it({
          name: 'REFUSES A REPLY THAT IS NOT THE DOCUMENTED SHAPE: no object, a period or rows that are not what they '
            + 'should be, an end that is no instant, and a covered row with no usable energy',
          fn: async () => {
            /**
             Each broken body and the phrase the refusal carries.
             */
            const broken: readonly (readonly [string, string])[] = [
              ['not json', 'not valid JSON',],
              ['[]', 'not a JSON object',],
              [bodyOf({ period: 'cat', products: [], },), 'period is not an object',],
              [bodyOf({ period: { start: 7, end: ECHO.end, }, products: [], },), 'period.start is not a string',],
              [bodyOf({ period: { start: ECHO.start, end: 'later', }, products: [], },), 'period.end is not an instant',],
              [bodyOf({ period: ECHO, products: 'cat', },), 'products is not a list',],
              [bodyOf({ period: ECHO, products: ['cat',], },), 'a products row is not an object',],
              [
                bodyOf({ period: ECHO, products: [{ requested_model: 'glm-5.3-flash', energy_kwh: -1, },], },),
                'energy_kwh of the glm-5.3-flash row is not a non-negative number',
              ],
              [
                bodyOf({ period: ECHO, products: [{ requested_model: 'glm-5.3-flash', energy_kwh: '1', },], },),
                'energy_kwh of the glm-5.3-flash row is not a non-negative number',
              ],
            ];
            for (const [bodyText, phrase,] of broken) {
              /**
               What the reader raised for this body.
               */
              const refusal = caught(function readBroken(): unknown {
                return coveredWhOf({ bodyText, asked: ASKED, aliases: NO_ALIASES, },);
              },);
              expect(refusal,).toBeInstanceOf(NeuralWattUsageShapeError,);
              expect((refusal as Error).message,).toContain(phrase,);
            }
          },
        },),
      ],
    },),
  ],
},);
