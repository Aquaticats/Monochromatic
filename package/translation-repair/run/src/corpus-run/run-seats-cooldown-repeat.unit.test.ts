/**
 Tests the seat reader against the XingZ6020 shape: a provider whose meter reads unreadable (so the budgets count it
 spendable) refuses every call, each refusal sets the 300 s refusal cooldown, and the reader waits the cooldown out each
 time only to be refused at once.

 The budgets are the real ones over a stub meter that cannot be read, the seat reader is the real one, and one fake
 clock moves both the budgets' hold arithmetic and the reader's sleeps, so no case waits. The cooldown is the real
 300 s, which the clock passes in an instant. A second provider reads wet, since a reader left with no provider at
 all refuses with every provider dry, which is another case's subject.

 Fixtures are cat-themed invention; every meter is a stub and no HTTP is made.

 @module
 */

import {
  describe,
  type DisposableSandbox,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import { createProviderBudgets, } from '@monochromatic-dev/translation-repair-provider';
import { consoleLinesDuring, } from '@monochromatic-dev/translation-repair-provider/ts/neuralwatt-ledger.test-fixture.ts';
import { TEST_MAX_HOLD_MS, } from '@monochromatic-dev/translation-repair-provider/ts/budget-hold.test-fixture.ts';
import { levelCapturingLogger, } from '@monochromatic-dev/translation-repair-test-fixture/ts/capturing-logger.test-fixture.ts';
import { readPhaseSeats, } from '../../dist/final/node/index.mjs';

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
};

/**
 Builds the rig: a fake clock, the real budgets over a Synthetic meter that cannot be read and a Bedrock meter that reads
 wet, and the real seat reader over them.

 @param sinon - case's own sandbox, which fakes the timers

 @returns The rig, and the disposable that puts the real timers back

 @example
 ```ts
 using installed = rigOn({ sinon: ctx.sinon, });
 ```
 */
function rigOn({ sinon, }: { readonly sinon: DisposableSandbox; },): Rig & Disposable {
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
   The real budgets over the stub meters, on the fake clock.
   */
  const budgets = createProviderBudgets({
    maxHoldMs: TEST_MAX_HOLD_MS,
    synthetic: {
      quotas: function quotas(): Promise<never> {
        return Promise.reject(new Error('Synthetic /quotas body violated expectations',),);
      },
    },
    bedrock: {
      credits: function credits() {
        return Promise.resolve({
          creditUsd: 100,
          spentUsd: 10,
          reckonedUsd: 0,
          remainingUsd: 90,
          calls: 4,
        },);
      },
    },
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
  return {
    budgets,
    lines,
    refuseAndRead: async function refuseAndRead({ statedWaitMs, },): Promise<void> {
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
      // The reading's answer is not read, only what it said.
      await reading;
    },
    [Symbol.dispose]: function removeClock(): void {
      clock.restore();
    },
  };
}

/**
 The readings that waited for a hold to end.

 @param lines - lines the seat reader wrote

 @returns The waiting lines

 @example
 ```ts
 expect(waitsIn({ lines, },).length,).toBe(1,);
 ```
 */
function waitsIn({ lines, }: { readonly lines: readonly string[]; },): readonly string[] {
  return lines.filter(function isWait(line,): boolean {
    return line.startsWith('warn JUDGE SEATS phase=translate lane short of quorum',)
      && line.includes('waiting ',);
  },);
}

await describe({
  name: 'the seat reader and a provider that refuses at once after its cooldown',
  // One case at a time: each installs a fake clock and stubs the console, which are one for the whole process.
  concurrency: 1,
  children: [
    it({
      name: 'WAITS OUT THE COOLDOWN ONCE AND NO MORE when the provider refuses again right after it, so the entry '
        + 'spends one cooldown and not one for every slice that is refused',
      fn: async ctx => {
        using rig = rigOn({ sinon: ctx.sinon, },);
        await consoleLinesDuring({
          run: async function refuseAtOnceAfterEachCooldown(): Promise<void> {
            for (const _refusal of Array.from({ length: REFUSALS, },)
              .keys()) {
              // oxlint-disable-next-line no-await-in-loop -- each refusal follows the reading the cooldown ended
              await rig.refuseAndRead({},);
            }
          },
          sinon: ctx.sinon,
        },);

        expect(waitsIn({ lines: rig.lines, },).length,).toBe(1,);
      },
    },),

    it({
      name: 'LOGS THE DECISION with the provider, the entry and the cooldown that was waited out, once however often the '
        + 'provider refuses again',
      fn: async ctx => {
        using rig = rigOn({ sinon: ctx.sinon, },);
        rig.budgets.beginEntry({ entryId: 'XingZ6020', },);
        const lines = await consoleLinesDuring({
          run: async function refuseAtOnceAfterEachCooldown(): Promise<void> {
            for (const _refusal of Array.from({ length: REFUSALS, },)
              .keys()) {
              // oxlint-disable-next-line no-await-in-loop -- each refusal follows the reading the cooldown ended
              await rig.refuseAndRead({},);
            }
          },
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
        using rig = rigOn({ sinon: ctx.sinon, },);
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
        using rig = rigOn({ sinon: ctx.sinon, },);
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
        using rig = rigOn({ sinon: ctx.sinon, },);
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
