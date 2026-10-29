/**
 Tests the seat reader against a provider whose return served nothing: the owner's answer of 2026-10-29, "Remember
 futile returns". A provider whose last return served nothing (its hold ended, nothing it answered came after, and it
 refused again) is not waited for as the lift of a short bench, until it serves a call, and the reader says so with the
 provider, the phase and the return.

 The budgets are the real ones over a stub meter that cannot be read, the seat reader is the real one, and one fake
 clock moves both the budgets' hold arithmetic and the reader's sleeps, so no case waits. The refusals here name their
 return (`statedWaitMs`) unless a case says otherwise, since the cooldown give-up already stops the waits for a refusal
 cooldown (`run-seats-cooldown-repeat.unit.test.ts`), and a case that keeps both on one provider says so.

 Fixtures are cat-themed invention; every meter is a stub and no HTTP is made.

 @module
 */

import {
  describe,
  type DisposableSandbox,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  createProviderBudgets,
  type NeuralWattEnergy,
} from '@monochromatic-dev/translation-repair-provider';
import { consoleLinesDuring, } from '@monochromatic-dev/translation-repair-provider/ts/neuralwatt-ledger.test-fixture.ts';
import { TEST_MAX_HOLD_MS, } from '@monochromatic-dev/translation-repair-provider/ts/budget-hold.test-fixture.ts';
import { unreadableQuotas, } from '@monochromatic-dev/translation-repair-provider/ts/unreadable-quotas.test-fixture.ts';
import { levelCapturingLogger, } from '@monochromatic-dev/translation-repair-test-fixture/ts/capturing-logger.test-fixture.ts';
import { readPhaseSeats, } from '../../dist/final/node/index.mjs';
import { waitsIn, } from './run-seats-waits.test-fixture.ts';

/**
 Abort signal every read here carries.
 */
const SIGNAL: AbortSignal = new AbortController().signal;

/**
 The refusal cooldown the budgets set by default, which a refusal on a meter that cannot be read holds a provider out for
 even when it names a shorter return.
 */
const COOLDOWN_MS = 300_000;

/**
 How often the seat reader's wait checks for abort, the real one-second slice.
 */
const POLL_MS = 1_000;

/**
 A wait a refusal can state for the provider's return: two minutes, inside the cooldown, so the hold is the cooldown.
 */
const STATED_MS = 120_000;

/**
 A NeuralWatt day with all of its energy left, which reads wet.
 */
const NEURALWATT_WET_DAY: NeuralWattEnergy = {
  dayStart: '2026-10-25T00:00:00.000Z',
  dailyWh: 550,
  spentWh: 0,
  reckonedWh: 0,
  reservedWh: 0,
  remainingWh: 550,
  calls: 0,
};

/**
 Which provider reads wet beside the refusing one: none, or NeuralWatt.
 */
type Alternative = 'none' | 'neuralwatt';

/**
 What one case needs of the budgets, the clock and the reader: a rig over the real code.
 */
type Rig = {
  /**
   Refuses a call of Synthetic and reads the seats as the next slice would, letting the clock pass the hold meanwhile.
   */
  readonly refuseAndRead: (args: { readonly statedWaitMs?: number; },) => Promise<void>;

  /**
   The budgets the rig reads.
   */
  readonly budgets: ReturnType<typeof createProviderBudgets>;

  /**
   Lines the seat reader wrote.
   */
  readonly lines: string[];

  /**
   The names of the errors the readings ended with, none when every reading seated its benches.
   */
  readonly endings: string[];
};

/**
 Builds the rig: a fake clock, the real budgets over a Synthetic meter that cannot be read and, where asked, a NeuralWatt
 meter that reads wet, and the real seat reader over them.

 @param sinon - case's own sandbox, which fakes the timers

 @param alternative - which provider reads wet beside Synthetic

 @returns The rig, and the disposable that puts the real timers back

 @example
 ```ts
 using installed = rigOn({ sinon: ctx.sinon, alternative: 'neuralwatt', });
 ```
 */
function rigOn(
  {
    sinon,
    alternative,
  }: {
    readonly sinon: DisposableSandbox;
    readonly alternative: Alternative;
  },
): Rig & Disposable {
  /**
   Fake clock moving the budgets' holds and the reader's sleeps together.
   */
  const clock = sinon.useFakeTimers({
    toFake: [
      'setTimeout',
      'clearTimeout',
    ],
    shouldClearNativeTimers: true,
  },);
  /**
   Lines the seat reader wrote.
   */
  const lines: string[] = [];
  /**
   The names of the errors the readings ended with.
   */
  const endings: string[] = [];
  /**
   The real budgets over the stub meters, on the fake clock.
   */
  const budgets = createProviderBudgets({
    maxHoldMs: TEST_MAX_HOLD_MS,
    synthetic: { quotas: unreadableQuotas, },
    ...((alternative === 'neuralwatt')
      ? {
        neuralwatt: {
          energy: function energy(): Promise<NeuralWattEnergy> {
            return Promise.resolve(NEURALWATT_WET_DAY,);
          },
        },
      }
      : {}),
    now: function now(): number {
      return clock.now;
    },
  },);
  /**
   What the seat reader takes of its client.
   */
  const client = {
    providerDryness: function providerDryness() {
      return budgets.read({ signal: SIGNAL, },);
    },
    providerMeterStates: function providerMeterStates() {
      return budgets.readStates({ signal: SIGNAL, },);
    },
    providerHolds: budgets.holds,
    futileReturns: budgets.futileReturns,
  };
  /**
   Refuses Synthetic and reads the seats as the next slice would, letting the clock pass the hold meanwhile.

   @param statedWaitMs - wait the refusal names for the provider's return, none when absent

   @example
   ```ts
   await refuseAndRead({},);
   ```
   */
  async function refuseAndRead({ statedWaitMs, }: { readonly statedWaitMs?: number; },): Promise<void> {
    await budgets.markRefused({
      provider: 'synthetic',
      signal: SIGNAL,
      ...((statedWaitMs === undefined) ? {} : { statedWaitMs, }),
    },);
    /**
     The reading the next slice takes, started before the clock passes the hold.
     */
    const reading = readPhaseSeats({
      client,
      phase: 'translate lane',
      signal: SIGNAL,
      l: levelCapturingLogger({ lines, },),
      pollMs: POLL_MS,
    },);
    await clock.tickAsync(COOLDOWN_MS + POLL_MS,);
    // The reading's answer is not read, only what it said and whether it ended the entry.
    try {
      await reading;
    } catch (error) {
      endings.push(Error.isError(error,) ? error.name : String(error,),);
    }
  }

  return {
    budgets,
    lines,
    endings,
    refuseAndRead,
    [Symbol.dispose]: function removeClock(): void {
      clock.restore();
    },
  };
}

/**
 The lines that say a provider is not waited for because its return served nothing.

 @param lines - lines the seat reader wrote

 @returns The skip lines

 @example
 ```ts
 expect(skipsIn({ lines, },).length,).toBe(2,);
 ```
 */
function skipsIn({ lines, }: { readonly lines: readonly string[]; },): readonly string[] {
  return lines.filter(function isSkip(line,): boolean {
    return line.includes('not waiting for synthetic',);
  },);
}

await describe({
  name: 'the seat reader and a provider whose return served nothing',
  // One case at a time: each installs a fake clock and stubs the console, which are one for the whole process.
  concurrency: 1,
  children: [
    it({
      name: 'WAITS FOR A RETURNER THAT SERVED: each return is followed by a served call, so the provider counts as '
        + 'lifting the bench every time it is held',
      fn: async ctx => {
        using rig = rigOn({
          sinon: ctx.sinon,
          alternative: 'neuralwatt',
        },);
        await consoleLinesDuring({
          run: async function refuseAndServe(): Promise<void> {
            for (const _refusal of Array.from({ length: 3, },)
              .keys()) {
              // oxlint-disable-next-line no-await-in-loop -- each refusal follows the reading the hold ended
              await rig.refuseAndRead({ statedWaitMs: STATED_MS, },);
              rig.budgets.markServed({ provider: 'synthetic', },);
            }
          },
          sinon: ctx.sinon,
        },);

        expect(rig.endings,).toEqual([],);
        expect(waitsIn({ lines: rig.lines, },).length,).toBe(3,);
        expect(skipsIn({ lines: rig.lines, },),).toEqual([],);
      },
    },),

    it({
      name: 'DOES NOT WAIT FOR A RETURNER WHOSE RETURN SERVED NOTHING: the first hold is waited for, the provider '
        + 'refuses again with nothing served, and the next two holds are skipped, each with a line naming the provider, '
        + 'the phase and the return it refers to',
      fn: async ctx => {
        using rig = rigOn({
          sinon: ctx.sinon,
          alternative: 'neuralwatt',
        },);
        await consoleLinesDuring({
          run: async function refuseWithoutServing(): Promise<void> {
            for (const _refusal of Array.from({ length: 3, },)
              .keys()) {
              // oxlint-disable-next-line no-await-in-loop -- each refusal follows the reading the hold ended
              await rig.refuseAndRead({ statedWaitMs: STATED_MS, },);
            }
          },
          sinon: ctx.sinon,
        },);

        expect(rig.endings,).toEqual([],);
        expect(waitsIn({ lines: rig.lines, },).length,).toBe(1,);
        /**
         The lines that skip the provider.
         */
        const skips = skipsIn({ lines: rig.lines, },);
        expect(skips.length,).toBe(2,);
        expect(skips[0],).toContain('JUDGE SEATS phase=translate lane not waiting for synthetic',);
        expect(skips[0],).toContain('its last return served nothing',);
        expect(skips[0],).toContain('its 300000ms hold ended 1000ms before it refused again',);
      },
    },),

    it({
      name: 'COUNTS A FUTILE RETURNER AGAIN ONCE IT SERVES A CALL: waited for, skipped after a return that served '
        + 'nothing, and waited for again after the call it served',
      fn: async ctx => {
        using rig = rigOn({
          sinon: ctx.sinon,
          alternative: 'neuralwatt',
        },);
        await consoleLinesDuring({
          run: async function refuseServeAndRefuse(): Promise<void> {
            await rig.refuseAndRead({ statedWaitMs: STATED_MS, },);
            await rig.refuseAndRead({ statedWaitMs: STATED_MS, },);
            expect(waitsIn({ lines: rig.lines, },).length,).toBe(1,);

            rig.budgets.markServed({ provider: 'synthetic', },);
            await rig.refuseAndRead({ statedWaitMs: STATED_MS, },);
          },
          sinon: ctx.sinon,
        },);

        expect(rig.endings,).toEqual([],);
        expect(waitsIn({ lines: rig.lines, },).length,).toBe(2,);
        expect(skipsIn({ lines: rig.lines, },).length,).toBe(1,);
      },
    },),

    it({
      name: 'KEEPS THE GIVE-UP AND THE RULE TOGETHER ON ONE PROVIDER: a refusal cooldown refused at once is waited for '
        + 'once and then given up, and a stated return that follows, which the give-up still counts as a hold, is '
        + 'skipped by the rule because the return before it served nothing',
      fn: async ctx => {
        using rig = rigOn({
          sinon: ctx.sinon,
          alternative: 'neuralwatt',
        },);
        await consoleLinesDuring({
          run: async function refuseTwiceThenState(): Promise<void> {
            for (const _refusal of Array.from({ length: 3, },)
              .keys()) {
              // oxlint-disable-next-line no-await-in-loop -- each refusal follows the reading the hold ended
              await rig.refuseAndRead({},);
            }
            expect(waitsIn({ lines: rig.lines, },).length,).toBe(1,);
            expect(rig.budgets.holds().synthetic,).toBe(0,);

            await rig.refuseAndRead({ statedWaitMs: STATED_MS, },);
          },
          sinon: ctx.sinon,
        },);

        expect(rig.endings,).toEqual([],);
        expect(waitsIn({ lines: rig.lines, },).length,).toBe(1,);
        expect(skipsIn({ lines: rig.lines, },).length,).toBe(1,);
      },
    },),

    it({
      name: 'YIELDS WHERE THE GIVE-UP YIELDS: Synthetic the only provider not dry is waited for every time, whether '
        + 'its refusal is a cooldown or names its return, and the run is not ended',
      fn: async ctx => {
        using rig = rigOn({
          sinon: ctx.sinon,
          alternative: 'none',
        },);
        await consoleLinesDuring({
          run: async function refuseWithNobodyElse(): Promise<void> {
            await rig.refuseAndRead({},);
            await rig.refuseAndRead({},);
            await rig.refuseAndRead({ statedWaitMs: STATED_MS, },);
            await rig.refuseAndRead({ statedWaitMs: STATED_MS, },);
          },
          sinon: ctx.sinon,
        },);

        expect(rig.endings,).toEqual([],);
        expect(waitsIn({ lines: rig.lines, },).length,).toBe(4,);
        expect(skipsIn({ lines: rig.lines, },),).toEqual([],);
      },
    },),

    it({
      name: 'GOES ON SHORT AND SAYS WHY when the only provider that could lift a short bench is a futile returner: '
        + 'the reading takes no wait, seats what is reachable, and states the bench short beside the skipped return',
      fn: async ctx => {
        using rig = rigOn({
          sinon: ctx.sinon,
          alternative: 'neuralwatt',
        },);
        await consoleLinesDuring({
          run: async function refuseTwice(): Promise<void> {
            await rig.refuseAndRead({ statedWaitMs: STATED_MS, },);
            await rig.refuseAndRead({ statedWaitMs: STATED_MS, },);
          },
          sinon: ctx.sinon,
        },);

        expect(rig.endings,).toEqual([],);
        /**
         The lines of the reading that took no wait: the bench short of its quorum, and the phase running on what is
         reachable.
         */
        const goesOn = rig.lines.filter(function isNoWaitLine(line,): boolean {
          return line.startsWith('warn JUDGE SEATS phase=translate lane short of quorum',)
            && line.endsWith('so the phase runs on what is reachable',);
        },);
        expect(goesOn.length,).toBe(1,);
        expect(skipsIn({ lines: rig.lines, },).length,).toBe(1,);
      },
    },),
  ],
},);
