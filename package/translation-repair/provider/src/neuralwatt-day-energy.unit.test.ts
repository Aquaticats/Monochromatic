/**
 Tests what a NeuralWatt day has spent and what is left of it: the ledger's lines inside the day, the larger of their
 sum and the account's covered energy, and the holds of calls in flight. The day is the 24 hours from its start, the
 start included and the end not.

 Fixtures are cat-themed invention.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  dayIsClosed,
  energyOfDay,
  entryIsInDay,
  type NeuralWattDay,
} from '../dist/final/node/index.mjs';

/**
 An anchored day starting at 19:15 UTC (15:15 EDT in America/New_York), with whatever else a case adds.
 */
const DAY: Extract<NeuralWattDay, { readonly kind: 'anchored'; }> = {
  kind: 'anchored',
  start: new Date('2026-10-20T19:15:08.000Z',),
};

/**
 One ledger line.

 @param at - when it was written

 @param wh - watt-hours it used

 @param estimated - whether the figure is an estimate

 @returns Line
 */
function lineOf(
  {
    at,
    wh,
    estimated = false,
  }: {
    readonly at: string;
    readonly wh: number;
    readonly estimated?: boolean;
  },
): Parameters<typeof entryIsInDay>[0]['entry'] {
  return {
    at,
    model: 'glm-5.3-flash',
    wh,
    promptTokens: 90,
    ...(estimated ? { estimated: 'unreported-estimate', } as const : {}),
  };
}

await describe({
  name: 'NeuralWatt day energy',
  concurrency: 1,
  children: [
    describe({
      name: dayIsClosed.name,
      children: [
        it({
          name: 'CLOSES A DAY WHOSE START IS UNKNOWN OR WHOSE KEY IS BLOCKED, and leaves an ordinary day open',
          fn: async () => {
            expect(dayIsClosed({ day: { kind: 'unanchored', }, },),).toBe(true,);
            expect(dayIsClosed({ day: { ...DAY, keyBlockedUntil: new Date('2026-10-21T00:00:00.000Z',), }, },),).toBe(true,);
            expect(dayIsClosed({ day: DAY, },),).toBe(false,);
          },
        },),
      ],
    },),
    describe({
      name: entryIsInDay.name,
      children: [
        it({
          name: 'COUNTS A LINE FROM THE DAY\'S FIRST INSTANT UP TO BUT NOT INCLUDING THE NEXT DAY\'S, whatever the date',
          fn: async () => {
            /**
             Whether a line stamped at an instant counts.

             @param at - stamp

             @returns Whether it is inside
             */
            function inside(at: string,): boolean {
              return entryIsInDay({ entry: lineOf({ at, wh: 1, },), day: DAY, },);
            }
            expect(inside('2026-10-20T19:15:07.999Z',),).toBe(false,);
            expect(inside('2026-10-20T19:15:08.000Z',),).toBe(true,);
            expect(inside('2026-10-20T23:59:59.999Z',),).toBe(true,);
            expect(inside('2026-10-21T00:00:00.000Z',),).toBe(true,);
            expect(inside('2026-10-21T19:15:07.999Z',),).toBe(true,);
            expect(inside('2026-10-21T19:15:08.000Z',),).toBe(false,);
          },
        },),
      ],
    },),
    describe({
      name: energyOfDay.name,
      children: [
        it({
          name: 'SUMS THE LINES, names the part that is estimated, and subtracts the holds, with no account figure',
          fn: async () => {
            expect(energyOfDay({
              entries: [lineOf({ at: '2026-10-20T20:00:00.000Z', wh: 2, },), lineOf({ at: '2026-10-20T21:00:00.000Z', wh: 1, estimated: true, },),],
              reservedWh: 0.5,
              dailyWh: 10,
              day: DAY,
            },),).toEqual({
              dayStart: '2026-10-20T19:15:08.000Z',
              dailyWh: 10,
              spentWh: 3,
              reckonedWh: 1,
              reservedWh: 0.5,
              remainingWh: 6.5,
              calls: 2,
            },);
          },
        },),
        it({
          name: 'COUNTS THE LARGER OF THE LINES AND THE ACCOUNT, never their sum, and carries the account\'s figure',
          fn: async () => {
            /**
             Two watt-hours on the ledger.
             */
            const entries = [lineOf({ at: '2026-10-20T20:00:00.000Z', wh: 2, },),];
            expect(energyOfDay({ entries, reservedWh: 0, dailyWh: 10, day: { ...DAY, accountWh: 7, }, },),).toMatchObject({
              remainingWh: 3,
              accountWh: 7,
              spentWh: 2,
            },);
            expect(energyOfDay({ entries, reservedWh: 0, dailyWh: 10, day: { ...DAY, accountWh: 1, }, },),).toMatchObject({
              remainingWh: 8,
              accountWh: 1,
              spentWh: 2,
            },);
          },
        },),
        it({
          name: 'CARRIES THE KEY\'S CAP AND WHEN IT LIFTS, and leaves them out of a day that has neither',
          fn: async () => {
            /**
             The reading for a blocked key.
             */
            const blocked = energyOfDay({
              entries: [],
              reservedWh: 0,
              dailyWh: 10,
              day: {
                ...DAY,
                keyAllowance: { remainingUsd: 0, blocked: true, },
                keyBlockedUntil: new Date('2026-10-21T19:15:08.000Z',),
              },
            },);
            expect(blocked.keyAllowance,).toEqual({ remainingUsd: 0, blocked: true, },);
            expect(blocked.keyBlockedUntil,).toBe('2026-10-21T19:15:08.000Z',);
            expect(Object.hasOwn(energyOfDay({ entries: [], reservedWh: 0, dailyWh: 10, day: DAY, },), 'keyAllowance',),).toBe(false,);
          },
        },),
        it({
          name: 'LEAVES NOTHING OF A DAY WHOSE START IS UNKNOWN, and names no start',
          fn: async () => {
            expect(energyOfDay({ entries: [], reservedWh: 0.25, dailyWh: 10, day: { kind: 'unanchored', }, },),).toEqual({
              dailyWh: 10,
              spentWh: 0,
              reckonedWh: 0,
              reservedWh: 0.25,
              remainingWh: 0,
              calls: 0,
            },);
          },
        },),
      ],
    },),
  ],
},);
