/**
 Tests what the NeuralWatt energy ledger's reading means for routing: whether
 the day is dry, and the `key=value` tokens the meter line carries
 (`budget-routing.ts` holds the other providers' meters in the same shape).

 Readings are hand-written, so no case needs a ledger file or a clock.
 Fixtures are cat-themed invention.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  NEURALWATT_SMALLEST_ESTIMATE_WH,
  neuralWattIsDry,
  neuralWattMeterLevel,
  type NeuralWattEnergy,
} from '../dist/final/node/index.mjs';

/**
 A day with room for many calls.
 */
const ROOMY: NeuralWattEnergy = {
  dayStart: '2026-10-20T19:15:08.000Z',
  dailyWh: 550,
  spentWh: 0.014,
  reckonedWh: 0.5,
  reservedWh: 0,
  remainingWh: 549.986,
  calls: 3,
};

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: neuralWattIsDry.name,
      children: [
        it({
          name: 'READS A DAY WITH ROOM FOR THE SMALLEST CALL AS WET, and a day with room for exactly the smallest call '
            + 'as wet too, since that call could still be granted',
          fn: async () => {
            expect(neuralWattIsDry({ energy: ROOMY, },),).toBe(false,);
            expect(neuralWattIsDry({ energy: { ...ROOMY, remainingWh: NEURALWATT_SMALLEST_ESTIMATE_WH, }, },),)
              .toBe(false,);
          },
        },),

        it({
          name: 'READS A DAY WITH LESS LEFT THAN THE SMALLEST CALL WOULD HOLD AS DRY, down to nothing and below it, '
            + 'since no call could then be granted',
          fn: async () => {
            for (const remainingWh of [NEURALWATT_SMALLEST_ESTIMATE_WH / 2, 0, -0.3,]) {
              expect(neuralWattIsDry({ energy: { ...ROOMY, remainingWh, }, },),).toBe(true,);
            }
          },
        },),

        it({
          name: 'READS A DAY WITH NO ANCHOR AS DRY WHATEVER ELSE IT HOLDS, since a day that cannot be placed cannot be '
            + 'counted, and a day with the key\'s allowance blocked as dry as well',
          fn: async () => {
            /**
             The roomy reading with the day start taken out.
             */
            const { dayStart: _dayStart, ...unanchored } = ROOMY;
            expect(neuralWattIsDry({ energy: unanchored, },),).toBe(true,);
            expect(neuralWattIsDry({ energy: { ...ROOMY, keyBlockedUntil: '2026-10-21T19:15:08.000Z', }, },),).toBe(true,);
            expect(neuralWattIsDry({ energy: { ...ROOMY, keyAllowance: { remainingUsd: 3, blocked: false, }, }, },),).toBe(false,);
          },
        },),

        it({
          name: 'READS A DAY BY WHAT IS LEFT ALONE: the spend, the holds and the calls of the day change nothing '
            + 'while the remainder fits the smallest call, and a reading still carrying the old billedToday flag is '
            + 'wet when its remainder is',
          fn: async () => {
            expect(neuralWattIsDry({ energy: { ...ROOMY, spentWh: 449.9, reservedWh: 100, calls: 9_000, }, },),).toBe(false,);

            /**
             Reading as an older build shaped it, with a flag nothing reads any more.
             */
            const older = { ...ROOMY, billedToday: true, };
            expect(neuralWattIsDry({ energy: older, },),).toBe(false,);
          },
        },),
      ],
    },),

    describe({
      name: neuralWattMeterLevel.name,
      children: [
        it({
          name: 'RENDERS WHAT IS LEFT, THE RECKONED SHARE AND THE ACCOUNT FIGURE TO THREE PLACES, the day start and the '
            + 'key allowance as tokens with no value carrying a space, \'none\' for what was not read',
          fn: async () => {
            /**
             Tokens for a day with room.
             */
            const level = neuralWattMeterLevel({ energy: ROOMY, },);
            expect(level,).toEqual([
              'neuralwattWh=549.986',
              'neuralwattReckonedWh=0.500',
              'neuralwattAccountWh=none',
              'neuralwattDayStart=2026-10-20T19:15:08.000Z',
              'neuralwattKeyAllowance=none',
            ],);
            expect(neuralWattMeterLevel({
              energy: { ...ROOMY, accountWh: 4.0951, keyAllowance: { remainingUsd: 3.25, blocked: false, }, },
            },),).toEqual([
              'neuralwattWh=549.986',
              'neuralwattReckonedWh=0.500',
              'neuralwattAccountWh=4.095',
              'neuralwattDayStart=2026-10-20T19:15:08.000Z',
              'neuralwattKeyRemainingUsd=3.25',
              'neuralwattKeyBlocked=no',
            ],);
            expect(level.some(function hasSpace(token,): boolean {
              return token.includes(' ',);
            },),).toBe(false,);
          },
        },),

        it({
          name: 'ROUNDS A FIGURE THAT HAS MORE PLACES AND PRINTS A NEGATIVE REMAINDER, which a measurement over its '
            + 'estimate leaves',
          fn: async () => {
            expect(neuralWattMeterLevel({
              energy: { ...ROOMY, remainingWh: -0.00449, reckonedWh: 1.23456, },
            },).slice(0, 2,),).toEqual(['neuralwattWh=-0.004', 'neuralwattReckonedWh=1.235',],);
            /**
             The reading of a run that could not place its day.
             */
            const { dayStart: _dayStart, ...unanchored } = ROOMY;
            expect(neuralWattMeterLevel({ energy: unanchored, },),).toContain('neuralwattDayStart=none',);
          },
        },),
      ],
    },),
  ],
},);
