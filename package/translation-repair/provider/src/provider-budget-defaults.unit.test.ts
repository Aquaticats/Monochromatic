/**
 Tests the values a budgets instance holds before any meter has answered, the default clock, the error for a hold
 bound that cannot bound a hold, and the cut that keeps a wait a refusal states inside the longest hold a run can use.

 The cut is also read through the budgets in `provider-budget-hold-cap.unit.test.ts`; the cases here read it alone, so
 a change to the cut shows in a case that names it.

 Fixtures are cat-themed invention; no meter is read and no HTTP is made.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';
import {
  warnLinesDuring,
} from '@monochromatic-dev/translation-repair-typography/ts/console-warn-lines.test-fixture.ts';
import {
  createWaitCap,
  HoldBoundError,
  holdEndsNever,
  noLevelBeforeReading,
  notHeldOut,
  spendableBeforeReading,
  systemClock,
  unmarkedPayment,
  unreadBeforeReading,
} from '../dist/final/node/index.mjs';

/**
 Longest hold these cases allow, in milliseconds.
 */
const CAP_MS = 5_000;

/**
 A wait a header can state that no run outlasts.
 */
const STATED_MS = 99_999_999_999_000;

await describe({
  name: 'provider-budget-defaults',
  concurrency: 1,
  children: [
    describe({
      name: 'the starting values of a budgets instance',
      concurrency: 1,
      children: [
        it({
          name: 'READ AS NOTHING KNOWN: spendable, no level, no payment refusal, unreadable, not held out '
            + 'and no hold end',
          fn: async () => {
            expect({
              dry: spendableBeforeReading(),
              level: noLevelBeforeReading(),
              payment: unmarkedPayment(),
              state: unreadBeforeReading(),
              heldOut: notHeldOut(),
              holdEnd: holdEndsNever(),
            },).toEqual({
              dry: false,
              level: '',
              payment: { marked: false, level: '', },
              state: 'unreadable',
              heldOut: false,
              holdEnd: 0,
            },);
          },
        },),

        it({
          name: 'READS THE SYSTEM CLOCK by default, as the instant the clock keeps at the call',
          fn: async (ctx) => {
            ctx.sinon.useFakeTimers({ now: new Date('2026-10-23T12:00:00.000Z',), toFake: ['Date',], },);

            expect(systemClock().toISOString(),).toBe('2026-10-23T12:00:00.000Z',);
          },
        },),

        it({
          name: 'NAMES THE SETTING AND QUOTES THE NUMBER in the error for a hold bound that cannot bound a hold, and '
            + 'declares its message safe to print',
          fn: async () => {
            /**
             Errors for the three kinds of number a bound can be that is no whole number above zero.
             */
            const errors = [Number.POSITIVE_INFINITY, Number.NaN, 0,].map(
              function errorFor(maxHoldMs,): HoldBoundError {
                return new HoldBoundError({ maxHoldMs, },);
              },
            );

            expect(errors.map(function messageOf(error,): string {
              return error.message;
            },),).toEqual([
              'maxHoldMs must be a finite whole number of milliseconds greater than zero, received Infinity',
              'maxHoldMs must be a finite whole number of milliseconds greater than zero, received NaN',
              'maxHoldMs must be a finite whole number of milliseconds greater than zero, received 0',
            ],);
            expect(errors.every(function namesItself(error,): boolean {
              return (error.name === 'HoldBoundError') && error.messageNamesOnly;
            },),).toBe(true,);
          },
        },),
      ],
    },),

    describe({
      name: createWaitCap.name,
      concurrency: 1,
      children: [
        it({
          name: 'PASSES A STATED WAIT AT OR UNDER THE BOUND through whole and writes nothing',
          fn: async () => {
            /**
             Cut over the case's bound.
             */
            const cappedWait = createWaitCap({ maxHoldMs: CAP_MS, },);

            const { result: returned, warned, } = await warnLinesDuring({
              run: async function stateShortWaits(): Promise<readonly number[]> {
                return [3_000, CAP_MS,].map(function cut(statedWaitMs,): number {
                  return cappedWait({ provider: 'hyper', statedWaitMs, },);
                },);
              },
            },);

            expect(returned,).toEqual([3_000, CAP_MS,],);
            expect(warned,).toEqual([],);
          },
        },),

        it({
          name: 'CUTS A WAIT PAST THE BOUND TO IT and writes the stated value once for each provider, '
            + 'not on every repeat',
          fn: async () => {
            /**
             Cut over the case's bound.
             */
            const cappedWait = createWaitCap({ maxHoldMs: CAP_MS, },);

            const { result: returned, warned, } = await warnLinesDuring({
              run: async function stateLongWaits(): Promise<readonly number[]> {
                return (['hyper', 'hyper', 'openrouter',] as const).map(function cut(provider,): number {
                  return cappedWait({ provider, statedWaitMs: STATED_MS, },);
                },);
              },
            },);

            expect(returned,).toEqual([CAP_MS, CAP_MS, CAP_MS,],);
            expect(warned.filter(function namesHyper(line,): boolean {
              return line.includes('hyper:',) && line.includes(String(STATED_MS,),);
            },),).toHaveLength(1,);
            expect(warned.filter(function namesOpenRouter(line,): boolean {
              return line.includes('openrouter:',) && line.includes(String(STATED_MS,),);
            },),).toHaveLength(1,);
          },
        },),
      ],
    },),
  ],
},);
