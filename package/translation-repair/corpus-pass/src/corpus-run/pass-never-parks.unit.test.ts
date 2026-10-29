/**
 Tests that the pass never parks for NeuralWatt's day return: when the dry-stop rule says stop, the pass stops, whatever
 the return time.

 THE OWNER'S ANSWER OF 2026-10-29 is "Never wait, relaunch": "Stop at once and relaunch later." The refusals here come
 from the real budgets over stub meters and a stub wall clock, with NeuralWatt's spent day starting again 301 s later
 (past the 300 s a router call used to wait, inside what the pass had left), so the gate and the queue meet exactly
 what a run meets. The stop line still states the time of day in UTC, so the operator knows when to relaunch.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  createProviderBudgets,
  EveryProviderDryError,
  type ModelClient,
  NOBODY_REFUSED,
  readBudgetsPastHolds,
} from '@monochromatic-dev/translation-repair-provider';
import { resetRunSpend, } from '@monochromatic-dev/translation-repair-shared-contract';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';
import { rejectionOf, } from '@monochromatic-dev/translation-repair-test-fixture/ts/rejecting-call.test-fixture.ts';
import { SEAT_HYPER_OPENROUTER_VISION_EDITOR, } from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';
import { relayingConsoleLog, } from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/console-log-capture.test-fixture.ts';
import { TEST_MAX_HOLD_MS, } from '@monochromatic-dev/translation-repair-provider/ts/budget-hold.test-fixture.ts';
import {
  createPassDryStop,
  runPassQueue,
} from '../../dist/final/node/index.mjs';

/**
 Milliseconds after 00:00 UTC the account's day starts at: 19:15:08 UTC.
 */
const DAY_START_MS_OF_DAY = 69_308_000;

/**
 Generation every case's attempts run under.
 */
const GENERATION = `sha256-tree-v1:${'c'.repeat(64,)}`;

/**
 Client whose every chat call asks the real budgets for a provider and meets the refusal the router makes when every
 provider is dry and NeuralWatt's spent day starts again 301 s later.

 @returns The client, over budgets whose other providers are unconfigured and so dry

 @example
 ```ts
 const client = clientOverSpentNeuralWatt();
 ```
 */
function clientOverSpentNeuralWatt(): ModelClient {
  /**
   The real budgets over a NeuralWatt meter that reads the spent day.
   */
  const budgets = createProviderBudgets({
    maxHoldMs: TEST_MAX_HOLD_MS,
    neuralwatt: {
      energy: async () => ({
        dayStart: '2026-10-22T19:15:08.000Z',
        dailyWh: 550,
        spentWh: 550,
        reckonedWh: 0,
        reservedWh: 0,
        remainingWh: 0,
        calls: 40,
      }),
      dayAnchor: () => ({ known: true, msOfDay: DAY_START_MS_OF_DAY, }),
    },
    wallNow: () => new Date(Date.UTC(
      2_026,
      9,
      23,
      0,
      0,
      0,
      DAY_START_MS_OF_DAY - 301_000,
    ),),
    returnMarginMs: 0,
  },);
  /**
   The call every method makes.

   @param signal - the call's abort handle

   @returns Never

   @throws {@link EveryProviderDryError} always
   */
  async function askRouter({ signal, }: { readonly signal: AbortSignal; },): Promise<never> {
    await readBudgetsPastHolds({
      budgets,
      modelId: 'glm-5.3',
      signal,
      refused: NOBODY_REFUSED,
      pollMs: 1_000,
    },);
    throw new Error('the router found a provider with budget',);
  }
  return {
    chatText: async request => await askRouter({ signal: request.signal, },),
    chatJson: async request => await askRouter({ signal: request.signal, },),
    quotas: async () => {
      throw new Error('quotas unused',);
    },
  };
}

/**
 Makes one chat call through a client and returns the failure it raises.

 @param client - client to call
 @param signal - call's abort handle

 @returns Failure the call raised

 @example
 ```ts
 const failure = await dryCallOf({ client, signal: dryStop.signal, },);
 ```
 */
async function dryCallOf(
  {
    client,
    signal,
  }: {
    readonly client: ModelClient;
    readonly signal: AbortSignal;
  },
): Promise<unknown> {
  return await rejectionOf(async function dryCall(): Promise<unknown> {
    return await client.chatText({
      modelId: SEAT_HYPER_OPENROUTER_VISION_EDITOR,
      messages: [],
      signal,
    },);
  },);
}

await describe({
  name: 'the pass never parks for NeuralWatt\'s return',
  concurrency: 1,
  children: [
    it({
      name: 'STOPS THE PASS AT ONCE ON THE REFUSAL, STATING THE RETURN TIME: every provider dry and NeuralWatt 301 s '
        + 'from its day start, with an hour left in the pass, trips the gate on the first refusal and the stop note '
        + 'names the time of day in UTC',
      fn: async () => {
        const dryStop = createPassDryStop({
          passLeftMs: () => 3_600_000,
          now: () => 1_000,
          waitMs: async function noWait(): Promise<void> {
            await Promise.resolve();
          },
        },);
        const client = dryStop.guard({ client: clientOverSpentNeuralWatt(), },);
        const refusal = await dryCallOf({ client, signal: dryStop.signal, },);
        expect(refusal,).toBeInstanceOf(EveryProviderDryError,);
        expect(dryStop.tripped(),).toBe(true,);
        expect(dryStop.signal.reason,).toBe(refusal,);
        const [note = '',] = dryStop.takeStopNotes();
        expect(note,).toContain('at 19:15:08 UTC',);
      },
    },),

    it({
      name: 'ATTEMPTS NO ENTRY AGAIN AFTER THE REFUSAL AND STARTS NONE BEHIND IT: the entry that met the dry router is '
        + 'attempted once, the entry behind it never starts, and the stop line names the return time',
      fn: async ctx => {
        resetRunSpend();
        using printed = relayingConsoleLog({ sinon: ctx.sinon, },);
        await using scratch = await scratchDir({ prefix: 'pass-never-parks-', },);
        const dryStop = createPassDryStop({
          passLeftMs: () => 3_600_000,
          now: () => 1_000,
          waitMs: async function noWait(): Promise<void> {
            await Promise.resolve();
          },
        },);
        const client = dryStop.guard({ client: clientOverSpentNeuralWatt(), },);
        /**
         Entries attempted.
         */
        const made: string[] = [];
        await runPassQueue({
          pending: [
            {
              id: 'tabby',
              sourceText: '猫睡觉。\n',
              targetText: 'The cat naps.\n',
            },
            {
              id: 'mittens',
              sourceText: '猫睡觉。\n',
              targetText: 'The cat naps.\n',
            },
          ],
          attempts: new Map(),
          attemptsPath: join(
            scratch.path,
            'attempts.json',
          ),
          sliceCacheDir: join(
            scratch.path,
            'slice-cache',
          ),
          pipelineDigest: GENERATION,
          dryStop,
          softBudgetMs: HANG_STOP_MS,
          spendCeilingUsd: 20,
          start: 0,
          now: () => 0,
          settle: async function settle({ entry, },) {
            made.push(entry.id,);
            await dryCallOf({ client, signal: dryStop.signal, },);
            return { kind: 'stopped', };
          },
        },);
        expect(made,).toEqual(['tabby',],);
        expect(printed.lines.filter(function isStop(line,): boolean {
          return line.startsWith('STOP ',) && line.includes('at 19:15:08 UTC',);
        },).length,).toBe(1,);
      },
    },),
  ],
},);
