/**
 Tests the seat reader against the XingZ6020 shape: a provider whose meter reads unreadable (so the budgets count it
 spendable) refuses every call, each refusal sets the 300 s refusal cooldown, and the reader waits the cooldown out each
 time only to be refused at once.

 The budgets are the real ones over a stub meter that cannot be read, the seat reader is the real one, and one fake
 clock moves both the budgets' hold arithmetic and the reader's sleeps, so no case waits. The cooldown is the real
 300 s, which the clock passes in an instant.

 Fixtures are cat-themed invention; every meter is a stub and no HTTP is made.

 @module
 */

import {
  describe,
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

await describe({
  name: 'the seat reader and a provider that refuses at once after its cooldown',
  // One case at a time: each installs a fake clock and stubs the console, which are one for the whole process.
  concurrency: 1,
  children: [
    it({
      name: 'WAITS OUT THE COOLDOWN ONCE AND NO MORE when the provider refuses again right after it, so the entry '
        + 'spends one cooldown and not one for every slice that is refused',
      fn: async ctx => {
        /**
         Fake clock moving the budgets' holds and the reader's sleeps together.
         */
        const clock = ctx.sinon.useFakeTimers({
          toFake: [
            'setTimeout',
            'clearTimeout',
          ],
          shouldClearNativeTimers: true,
        },);
        /**
         Puts the real timers back when the case ends, however it ends.
         */
        using _installed = {
          [Symbol.dispose]: function removeClock(): void {
            clock.restore();
          },
        };
        /**
         Lines the seat reader wrote.
         */
        const lines: string[] = [];
        /**
         The real budgets over a meter that cannot be read, on the fake clock.
         */
        const budgets = createProviderBudgets({
          maxHoldMs: TEST_MAX_HOLD_MS,
          synthetic: {
            quotas: function quotas(): Promise<never> {
              return Promise.reject(new Error('Synthetic /quotas body violated expectations',),);
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
        await consoleLinesDuring({
          run: async function refuseAtOnceAfterEachCooldown(): Promise<void> {
            for (const _refusal of Array.from({ length: REFUSALS, },)
              .keys()) {
              // oxlint-disable-next-line no-await-in-loop -- each refusal follows the reading the cooldown ended
              await budgets.markRefused({
                provider: 'synthetic',
                signal: SIGNAL,
              },);
              /**
               The reading the next slice takes, started before the clock passes the cooldown.
               */
              const reading = readPhaseSeats({
                client,
                phase: 'translate lane',
                signal: SIGNAL,
                l: levelCapturingLogger({ lines, },),
                pollMs: POLL_MS,
              },);
              // oxlint-disable-next-line no-await-in-loop -- the clock passes the cooldown, then the reading is done
              await clock.tickAsync(COOLDOWN_MS + POLL_MS,);
              // oxlint-disable-next-line no-await-in-loop -- the reading's answer is not read, only what it said
              await reading;
            }
          },
          sinon: ctx.sinon,
        },);

        /**
         The readings that waited for a hold to end.
         */
        const waits = lines.filter(function isWait(line,): boolean {
          return line.startsWith('warn JUDGE SEATS phase=translate lane short of quorum',)
            && line.includes('waiting ',);
        },);
        expect(waits.length,).toBe(1,);
      },
    },),
  ],
},);
