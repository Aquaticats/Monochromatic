/**
 Tests the NeuralWatt day-start setting and the arithmetic that places a day: `TRANSLATION_REPAIR_NEURALWATT_DAY_START`
 (`HH:MM`, UTC) is the fallback when the quota does not report `current_period_start`, and the day starts at the latest
 instant at or before now whose UTC time of day is the one given.

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
  NEURALWATT_DAY_START_VAR,
  NeuralWattDayStartOverrideError,
  neuralWattDayStartAt,
  neuralWattDayStartFrom,
} from '../dist/final/node/index.mjs';

/**
 Milliseconds in the 19:15 UTC the owner's 15:15 EDT in America/New_York is.
 */
const NAP_TIME_MS = ((19 * 60) + 15) * 60_000;

await describe({
  name: 'NeuralWatt day start',
  concurrency: 1,
  children: [
    describe({
      name: neuralWattDayStartFrom.name,
      children: [
        it({
          name: 'IS NAMED TRANSLATION_REPAIR_NEURALWATT_DAY_START, READS HH:MM AS UTC, and reads an absent or blank '
            + 'variable as no setting rather than as midnight',
          fn: async () => {
            expect(NEURALWATT_DAY_START_VAR,).toBe('TRANSLATION_REPAIR_NEURALWATT_DAY_START',);
            expect(neuralWattDayStartFrom({ env: { [NEURALWATT_DAY_START_VAR]: '19:15', }, },),).toEqual({
              msOfDay: NAP_TIME_MS,
            },);
            expect(neuralWattDayStartFrom({ env: { [NEURALWATT_DAY_START_VAR]: '00:00', }, },),).toEqual({ msOfDay: 0, },);
            expect(neuralWattDayStartFrom({ env: { [NEURALWATT_DAY_START_VAR]: '23:59', }, },),).toEqual({
              msOfDay: ((23 * 60) + 59) * 60_000,
            },);
            expect(neuralWattDayStartFrom({ env: {}, },),).toEqual({},);
            expect(neuralWattDayStartFrom({ env: { [NEURALWATT_DAY_START_VAR]: '   ', }, },),).toEqual({},);
          },
        },),

        it({
          name: 'REFUSES A VALUE THAT IS NOT HH:MM, naming the variable and quoting the value',
          fn: async () => {
            for (const value of ['24:00', '19:60', '7:05', '19:15:08', '19.15', '1915', 'teatime', '19:15 UTC', '-1:00',]) {
              /**
               What reading that value throws.
               */
              const refusal = caught(function read(): void {
                neuralWattDayStartFrom({ env: { [NEURALWATT_DAY_START_VAR]: value, }, },);
              },);
              expect(refusal,).toBeInstanceOf(NeuralWattDayStartOverrideError,);
              expect(String(refusal,),).toContain(NEURALWATT_DAY_START_VAR,);
              expect(String(refusal,),).toContain(JSON.stringify(value,),);
            }
          },
        },),
      ],
    },),

    describe({
      name: neuralWattDayStartAt.name,
      children: [
        it({
          name: 'PLACES THE DAY AT THE LATEST INSTANT AT OR BEFORE NOW WITH THE GIVEN UTC TIME OF DAY, either side of '
            + 'the anchor time, on it, and across a UTC date line',
          fn: async () => {
            for (
              const [now, expected,] of [
                ['2026-10-20T23:15:00.741Z', '2026-10-20T19:15:00.000Z',],
                ['2026-10-20T19:14:59.999Z', '2026-10-19T19:15:00.000Z',],
                ['2026-10-20T19:15:00.000Z', '2026-10-20T19:15:00.000Z',],
                ['2026-10-21T00:00:00.000Z', '2026-10-20T19:15:00.000Z',],
                ['2026-10-20T00:00:00.000Z', '2026-10-19T19:15:00.000Z',],
                ['2026-03-01T00:00:00.000Z', '2026-02-28T19:15:00.000Z',],
              ] as const
            ) {
              expect(neuralWattDayStartAt({ now: new Date(now,), msOfDay: NAP_TIME_MS, },).toISOString(),).toBe(
                expected,
              );
            }
          },
        },),

        it({
          name: 'KEEPS A MIDNIGHT ANCHOR AT 00:00 UTC and does not move with the zone the process runs in',
          fn: async () => {
            expect(neuralWattDayStartAt({ now: new Date('2026-10-20T23:59:59.999Z',), msOfDay: 0, },).toISOString(),).toBe(
              '2026-10-20T00:00:00.000Z',
            );
          },
        },),
      ],
    },),
  ],
},);
