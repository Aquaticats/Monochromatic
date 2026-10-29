/**
 Tests the seat reader against the XingZ6020 shape: a provider whose meter reads unreadable (so the budgets count it
 spendable) refuses every call, each refusal sets the 300 s refusal cooldown, and the reader waits the cooldown out each
 time only to be refused at once.

 The budgets are the real ones over a stub meter that cannot be read, the seat reader is the real one, and one fake
 clock moves both the budgets' hold arithmetic and the reader's sleeps, so no case waits. The cooldown is the real
 300 s, which the clock passes in an instant.

 Two shapes are kept apart. With a live provider (NeuralWatt, which serves two of the translators and so keeps the
 translate lane at its floor) reading wet beside it, the entry gives the refusing provider up and moves on to the live
 one. With the refusing provider the only one not dry, which is the XingZ6020 shape itself, giving it up would leave
 every provider dry and end the run for a provider that would serve again after its cooldown, so the cooldown is waited
 out as before and the run is not ended.

 The budgets here are given no floor source, so any wet writing provider is an alternative: the cases pin the marks (the
 give-up, a stated return, a served call, a second entry) and not whether waiting for the provider would make the entry
 finishable. That is the run client's, with the bench floors it hands the budgets, and its cases are in
 `run-client-create.unit.test.ts`.

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
 The refusal cooldown the budgets set by default, which a refusal on a meter that cannot be read waits out.
 */
const COOLDOWN_MS = 300_000;

/**
 How often the seat reader's wait checks for abort, the real one-second slice.
 */
const POLL_MS = 1_000;

/**
 How many times the provider refuses at once after its cooldown.
 */
const REFUSALS = 3;

/**
 A wait a refusal can state for the provider's return: two minutes.
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
   Refuses Synthetic at once after each cooldown the reading waited out, as many times as the provider does in the
   XingZ6020 shape.
   */
  readonly refuseAtOnceAfterEachCooldown: () => Promise<void>;

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

 @returns Rig, and the disposable that puts the real timers back

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
    refuseAtOnceAfterEachCooldown: async function refuseAtOnceAfterEachCooldown(): Promise<void> {
      for (const _refusal of Array.from({ length: REFUSALS, },)
        .keys()) {
        // oxlint-disable-next-line no-await-in-loop -- each refusal follows the reading the cooldown ended
        await refuseAndRead({},);
      }
    },
    [Symbol.dispose]: function removeClock(): void {
      clock.restore();
    },
  };
}

await describe({
  name: 'the seat reader and a provider that refuses at once after its cooldown',
  // One case at a time: each installs a fake clock and stubs the console, which are one for the whole process.
  concurrency: 1,
  children: [
    it({
      name: 'WAITS OUT THE COOLDOWN ONCE AND NO MORE when the provider refuses again right after it and a live '
        + 'provider reads wet beside it, so the entry spends one cooldown and not one for every slice that is refused',
      fn: async ctx => {
        using rig = rigOn({
          sinon: ctx.sinon,
          alternative: 'neuralwatt',
        },);
        await consoleLinesDuring({
          run: rig.refuseAtOnceAfterEachCooldown,
          sinon: ctx.sinon,
        },);

        expect(rig.endings,).toEqual([],);
        expect(waitsIn({ lines: rig.lines, },).length,).toBe(1,);
      },
    },),

    it({
      name: 'KEEPS WAITING OUT THE COOLDOWN, AND NEVER ENDS THE RUN, when the refusing provider is the only one not dry: '
        + 'giving it up would leave every provider dry for a provider that serves again after its cooldown (the '
        + 'XingZ6020 shape, which ended the run on the first give-up)',
      fn: async ctx => {
        using rig = rigOn({
          sinon: ctx.sinon,
          alternative: 'none',
        },);
        await consoleLinesDuring({
          run: rig.refuseAtOnceAfterEachCooldown,
          sinon: ctx.sinon,
        },);

        expect(rig.endings,).toEqual([],);
        expect(waitsIn({ lines: rig.lines, },).length,).toBe(REFUSALS,);
      },
    },),

    it({
      name: 'LOGS THE DECISION with the provider, the entry and the cooldown that was waited out, once however often the '
        + 'provider refuses again',
      fn: async ctx => {
        using rig = rigOn({
          sinon: ctx.sinon,
          alternative: 'neuralwatt',
        },);
        rig.budgets.beginEntry({ entryId: 'XingZ6020', },);
        const lines = await consoleLinesDuring({
          run: rig.refuseAtOnceAfterEachCooldown,
          sinon: ctx.sinon,
        },);

        /**
         The lines that give the provider up.
         */
        const decisions = lines.filter(function isDecision(line,): boolean {
          return line.includes('stops waiting for synthetic\'s cooldowns',);
        },);
        expect(decisions.length,).toBe(1,);
        expect(decisions[0],).toContain('synthetic: refused again right after its 300000ms refusal cooldown ran out',);
        expect(decisions[0],).toContain('entry XingZ6020',);
      },
    },),

    it({
      name: 'KEEPS WAITING FOR A PROVIDER THAT SERVES AFTER ITS COOLDOWN: a call served once the hold ended is the provider '
        + 'coming back, so a refusal after it is not a refusal right after the cooldown',
      fn: async ctx => {
        using rig = rigOn({
          sinon: ctx.sinon,
          alternative: 'neuralwatt',
        },);
        await consoleLinesDuring({
          run: async function serveAfterEachCooldown(): Promise<void> {
            for (const _refusal of Array.from({ length: REFUSALS, },)
              .keys()) {
              // oxlint-disable-next-line no-await-in-loop -- each refusal follows the reading the cooldown ended
              await rig.refuseAndRead({},);
              rig.budgets.markServed({ provider: 'synthetic', },);
            }
          },
          sinon: ctx.sinon,
        },);

        expect(waitsIn({ lines: rig.lines, },).length,).toBe(REFUSALS,);
      },
    },),

    it({
      name: 'KEEPS THE HANDLING OF A STATED RETURN: a refusal that names when the provider comes back is not a cooldown, '
        + 'so it is waited for every time however often the provider refuses',
      fn: async ctx => {
        using rig = rigOn({
          sinon: ctx.sinon,
          alternative: 'neuralwatt',
        },);
        await consoleLinesDuring({
          run: async function refuseWithAReturnEachTime(): Promise<void> {
            for (const _refusal of Array.from({ length: REFUSALS, },)
              .keys()) {
              // oxlint-disable-next-line no-await-in-loop -- each refusal follows the reading the hold ended
              await rig.refuseAndRead({ statedWaitMs: STATED_MS, },);
            }
          },
          sinon: ctx.sinon,
        },);

        expect(waitsIn({ lines: rig.lines, },).length,).toBe(REFUSALS,);
      },
    },),

    it({
      name: 'STARTS A SECOND ENTRY FRESH: the provider the first entry gave up on is waited for once in the second, '
        + 'which then gives it up on its own evidence',
      fn: async ctx => {
        using rig = rigOn({
          sinon: ctx.sinon,
          alternative: 'neuralwatt',
        },);
        await consoleLinesDuring({
          run: async function refuseInTwoEntries(): Promise<void> {
            for (const entryId of [
              'XingZ6020',
              'XingZ6021',
            ]) {
              rig.budgets.beginEntry({ entryId, },);
              for (const _refusal of Array.from({ length: REFUSALS, },)
                .keys()) {
                // oxlint-disable-next-line no-await-in-loop -- each refusal follows the reading the cooldown ended
                await rig.refuseAndRead({},);
              }
            }
          },
          sinon: ctx.sinon,
        },);

        expect(waitsIn({ lines: rig.lines, },).length,).toBe(2,);
      },
    },),
  ],
},);
