/**
 Tests that the seat reader does not wait for a provider refused for payment, whose stated hold ends and whose meter
 reads wet.

 THE SHAPE: a refusal for payment (HTTP 402) keeps a provider dry until its meter moves, however wet the meter reads,
 and the same refusal may state a wait of its own, which is a hold. When that hold ends the provider is still dry, so a
 reader that counts it as returning waits for nothing. Here the wide bench stands one seat short of its quorum with
 NeuralWatt and Bedrock wet, and Hyper, whose return would reach the quorum, is refused for payment while its meter
 reads wet. The budgets are the real ones over stub meters, as the router's own are, so the states and the holds reach
 the reader the way a run's do; the hold is a few milliseconds on the real clock.

 Fixtures are cat-themed invention; every meter is a stub and no HTTP is made.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import { createProviderBudgets, } from '@monochromatic-dev/translation-repair-provider';
import { TEST_MAX_HOLD_MS, } from '@monochromatic-dev/translation-repair-provider/ts/budget-hold.test-fixture.ts';
import { levelCapturingLogger, } from '@monochromatic-dev/translation-repair-test-fixture/ts/capturing-logger.test-fixture.ts';
import { readJudgeSeats, } from '../../dist/final/node/index.mjs';

/**
 Abort signal every read here carries.
 */
const SIGNAL = new AbortController().signal;

/**
 A stated wait short enough for a case to wait out on the real clock.
 */
const SHORT_HOLD_MS = 40;

/**
 Budgets over stub meters with NeuralWatt, Bedrock and Hyper wet and every other provider unconfigured, so dry.

 @returns The real budgets, reading on the real clock

 @example
 ```ts
 const budgets = budgetsWithHyperWet();
 ```
 */
function budgetsWithHyperWet(): ReturnType<typeof createProviderBudgets> {
  return createProviderBudgets({
    maxHoldMs: TEST_MAX_HOLD_MS,
    neuralwatt: {
      energy: async () => ({
        dayStart: '2026-10-23T19:15:08.000Z',
        dailyWh: 550,
        spentWh: 0,
        reckonedWh: 0,
        reservedWh: 0,
        remainingWh: 550,
        calls: 0,
      }),
    },
    bedrock: {
      credits: async () => ({
        creditUsd: 100,
        spentUsd: 10,
        reckonedUsd: 0,
        remainingUsd: 90,
        calls: 12,
      }),
    },
    hyper: { credits: async () => ({ balance: 243, }), },
  },);
}

/**
 Refuses Hyper once, then reads the lane contest's seats and counts the dryness reads the reading took.

 @param paymentRequired - whether the refusal said the balance cannot pay for the call

 @param lines - receives what the reading logs

 @returns Dryness reads taken: 1 without a wait, 2 with one

 @example
 ```ts
 const reads = await readsAfterRefusal({ paymentRequired: true, lines: [], },);
 ```
 */
async function readsAfterRefusal(
  {
    paymentRequired,
    lines,
  }: {
    readonly paymentRequired: boolean;
    readonly lines: string[];
  },
): Promise<number> {
  /**
   The real budgets whose Hyper meter reads wet.
   */
  const budgets = budgetsWithHyperWet();
  await budgets.markRefused({
    provider: 'hyper',
    signal: SIGNAL,
    statedWaitMs: SHORT_HOLD_MS,
    paymentRequired,
  },);
  /**
   Dryness reads taken so far.
   */
  const counted = { reads: 0, };
  await readJudgeSeats({
    client: {
      providerDryness: async function providerDryness() {
        counted.reads += 1;
        return await budgets.read({ signal: SIGNAL, },);
      },
      providerMeterStates: async function providerMeterStates() {
        return await budgets.readStates({ signal: SIGNAL, },);
      },
      providerHolds: budgets.holds,
    },
    phase: 'lane contest',
    signal: SIGNAL,
    l: levelCapturingLogger({ lines, },),
    pollMs: 5,
  },);
  return counted.reads;
}

await describe({
  name: 'the seat reader and a provider refused for payment',
  concurrency: 1,
  children: [
    it({
      name: 'WAITS OUT THE STATED HOLD OF A PROVIDER REFUSED FOR A REASON OTHER THAN PAYMENT (the control): Hyper back '
        + 'makes the wide bench 4 of 8 against a quorum of 4, and its meter reads wet, so the wait buys the quorum',
      fn: async () => {
        expect(await readsAfterRefusal({ paymentRequired: false, lines: [], },),).toBe(2,);
      },
    },),

    it({
      name: 'DOES NOT WAIT FOR A PROVIDER REFUSED FOR PAYMENT whose stated hold ends and whose meter reads wet: it stays '
        + 'dry until the meter moves, so the hold ending brings nothing back and the reader seats the bench short at once',
      fn: async () => {
        /**
         What the reading logged.
         */
        const lines: string[] = [];
        expect(await readsAfterRefusal({ paymentRequired: true, lines, },),).toBe(1,);
        expect(lines.some(function waitLine(line,): boolean {
          return line.includes('waiting ',);
        },),).toBe(false,);
      },
    },),
  ],
},);
