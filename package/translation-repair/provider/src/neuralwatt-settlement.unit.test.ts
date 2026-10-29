/**
 Tests what a finished NeuralWatt call tells the meter: the stream's two
 comment lines (`neuralwatt-comments.ts`) settle the reservation the call
 held, so the measured energy replaces the estimate, a missing energy comment
 leaves the estimate standing and marked, and a cost above zero is reported on
 the SPEND line and nowhere else, because NeuralWatt reports a nominal cost on
 free-energy requests too (`https://docs.neuralwatt.com/energy-methodology.md`,
 fetched 2026-10-20; the owner's dashboard, 2026-10-20, counted a 0.0167 Wh
 request that reported 0.000107 USD wholly against the free energy).

 REACHED THROUGH THE CLIENT, because `settleNeuralWattCall` is not on the
 package index, which exports only what the siblings and their tests reach,
 and a test imports the built package and not the source. Each case answers one
 question with a scripted stream and reads what the settlement did off the
 ledger file, the lines it logged and the run's USD meter. One branch cannot be
 reached that way: the singular `token` of the log line, which needs a prompt of
 exactly one token and no request body is that short. No case reaches
 NeuralWatt or reads a key, the clock is the case's own, and logging goes
 through the case's own sandbox. Fixtures are cat-themed invention.

 @module
 */

import {
  chmod,
  readFile,
  writeFile,
} from 'node:fs/promises';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  estimateNeuralWattWh,
  promptTokensOfBody,
} from '../dist/final/node/index.mjs';
import { runSpendUsd, } from '@monochromatic-dev/translation-repair-shared-contract';
import type { TransportExchange, } from '@monochromatic-dev/translation-repair-provider-stream';
import {
  consoleLinesDuring,
  jsonRecordOf,
  MIDNIGHT_DAY,
} from './neuralwatt-ledger.test-fixture.ts';
import {
  ask,
  chatExchangesOf,
  ENERGY_COMMENT,
  FREE_COST_COMMENT,
  NOMINAL_COST_COMMENT,
  scriptedClient,
  streamOf,
} from './neuralwatt-scripted-client.test-fixture.ts';
import { rejectionOf, } from '@monochromatic-dev/translation-repair-test-fixture/ts/rejecting-call.test-fixture.ts';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';

/**
 Model the seat every case asks is spelled as on the wire.
 */
const SERVED = 'deepseek-v4.1-flash';

/**
 Prompt tokens and energy estimate the call the transport saw stood for.

 @param exchanges - everything the transport saw

 @returns Prompt tokens of its body and watt-hours held for them

 @example
 ```ts
 const { promptTokens, estimateWh, } = heldFor({ exchanges, },);
 ```
 */
function heldFor(
  { exchanges, }: { readonly exchanges: readonly TransportExchange[]; },
): { readonly promptTokens: number; readonly estimateWh: number; } {
  /**
   Prompt tokens the body's bytes stand for.
   */
  const promptTokens = promptTokensOfBody({
    bodyBytes: Buffer.byteLength(chatExchangesOf({ exchanges, },)[0]?.bodyJson ?? '',),
  },);
  return {
    promptTokens,
    estimateWh: estimateNeuralWattWh({ servedId: SERVED, promptTokens, },).estimateWh,
  };
}

await describe({
  name: 'NeuralWatt settlement',
  concurrency: 1,
  children: [
    it({
      name: 'SETTLES AT THE MEASURED ENERGY: 0.000015 kWh is written as 0.015 Wh, unmarked, the log says the measured '
        + 'figure against the estimate and prints the SPEND line, and a free request adds nothing to the run\'s USD '
        + 'meter or to the warnings',
      fn: async ctx => {
        await using scratch = await scratchDir({ prefix: 'whiskers-neuralwatt-settle-', },);
        const { client, ledger, exchanges, } = scriptedClient({
          dir: scratch.path,
          chat: [{ status: 200, bodyText: streamOf({ comments: ENERGY_COMMENT + FREE_COST_COMMENT, }), },],
        },);
        /**
         USD the run meter held before the call.
         */
        const before = runSpendUsd({ provider: 'neuralwatt', },);
        /**
         Lines the call logged.
         */
        const lines = await consoleLinesDuring({
          run: async function call(): Promise<void> {
            await ask({ client, },);
          },
          sinon: ctx.sinon,
        },);
        const { promptTokens, estimateWh, } = heldFor({ exchanges, },);
        /**
         Line the call wrote.
         */
        const line = jsonRecordOf({ text: (await readFile(ledger.path, 'utf8',)).trim(), },);
        expect(line.wh,).toBeCloseTo(0.015, 12,);
        expect(line.estimated,).toBeUndefined();
        expect('billedUsd' in line,).toBe(false,);
        expect(lines.some(function isSpend(entry,): boolean {
          return entry.includes(`SPEND provider=neuralwatt model=${SERVED} prompt=90 completion=10 cost=0`,);
        },),).toBe(true,);
        expect(lines.some(function says(entry,): boolean {
          return entry.includes(
            `NeuralWatt energy ${SERVED}: 0.0150 Wh measured against ${estimateWh.toFixed(4,)} Wh estimated for ${
              String(promptTokens,)
            } prompt tokens`,
          );
        },),).toBe(true,);
        expect(lines.some(function warns(entry,): boolean {
          return entry.includes('BILLED',);
        },),).toBe(false,);
        expect(runSpendUsd({ provider: 'neuralwatt', },),).toBe(before,);
        expect(await ledger.read({ day: MIDNIGHT_DAY, },),).toMatchObject({ reservedWh: 0, calls: 1, },);
      },
    },),

    it({
      name: 'LEAVES THE ESTIMATE STANDING WHEN THE STREAM CARRIED NO ENERGY COMMENT: the line is written at the estimate '
        + 'and marked, and the log says the estimate stands while a cost the stream did report is still counted',
      fn: async ctx => {
        await using scratch = await scratchDir({ prefix: 'whiskers-neuralwatt-settle-', },);
        const { client, ledger, exchanges, } = scriptedClient({
          dir: scratch.path,
          chat: [{ status: 200, bodyText: streamOf({ comments: FREE_COST_COMMENT, }), },],
        },);
        /**
         Lines the call logged.
         */
        const lines = await consoleLinesDuring({
          run: async function call(): Promise<void> {
            await ask({ client, },);
          },
          sinon: ctx.sinon,
        },);
        const { promptTokens, estimateWh, } = heldFor({ exchanges, },);
        /**
         Line the call wrote.
         */
        const line = jsonRecordOf({ text: (await readFile(ledger.path, 'utf8',)).trim(), },);
        expect(line.wh,).toBeCloseTo(estimateWh, 12,);
        expect(line.estimated,).toBe('unreported-estimate',);
        expect(lines.some(function says(entry,): boolean {
          return entry.includes(
            `NeuralWatt energy ${SERVED}: not reported, ${estimateWh.toFixed(4,)} Wh estimated stands for ${
              String(promptTokens,)
            } prompt tokens`,
          );
        },),).toBe(true,);
        expect(lines.some(function isSpend(entry,): boolean {
          return entry.includes('prompt=90 completion=10 cost=0',);
        },),).toBe(true,);
      },
    },),

    it({
      name: 'SETTLES WITH NOTHING WHEN THE STREAM CARRIED NO COMMENT AT ALL: the SPEND line has no cost field, the '
        + 'ledger line has no cost field, the estimate stands marked, and nothing is said about a bill',
      fn: async ctx => {
        await using scratch = await scratchDir({ prefix: 'whiskers-neuralwatt-settle-', },);
        const { client, ledger, } = scriptedClient({
          dir: scratch.path,
          chat: [{ status: 200, bodyText: streamOf({}), },],
        },);
        /**
         Lines the call logged.
         */
        const lines = await consoleLinesDuring({
          run: async function call(): Promise<void> {
            await ask({ client, },);
          },
          sinon: ctx.sinon,
        },);
        /**
         Line the call wrote.
         */
        const line = jsonRecordOf({ text: (await readFile(ledger.path, 'utf8',)).trim(), },);
        expect(line.estimated,).toBe('unreported-estimate',);
        expect('billedUsd' in line,).toBe(false,);
        expect(lines.some(function isSpend(entry,): boolean {
          return entry.includes('SPEND provider=neuralwatt',) && entry.includes('prompt=90 completion=10',)
            && (!entry.includes('cost=',));
        },),).toBe(true,);
        expect(lines.some(function warns(entry,): boolean {
          return entry.includes('BILLED',);
        },),).toBe(false,);
      },
    },),

    it({
      name: 'REPORTS A COST ABOVE ZERO ON THE SPEND LINE AND ADDS IT TO THE RUN\'S USD METER, and nothing else: the '
        + 'ledger line is the measured energy alone, no line is logged as billed or as stopping calls, and the day '
        + 'keeps its remainder',
      fn: async ctx => {
        await using scratch = await scratchDir({ prefix: 'whiskers-neuralwatt-settle-', },);
        const { client, ledger, } = scriptedClient({
          dir: scratch.path,
          chat: [{ status: 200, bodyText: streamOf({ comments: ENERGY_COMMENT + NOMINAL_COST_COMMENT, }), },],
        },);
        /**
         USD the run meter held before the call.
         */
        const before = runSpendUsd({ provider: 'neuralwatt', },);
        /**
         Lines the costed call logged.
         */
        const lines = await consoleLinesDuring({
          run: async function call(): Promise<void> {
            await ask({ client, },);
          },
          sinon: ctx.sinon,
        },);
        /**
         Line the call wrote.
         */
        const line = jsonRecordOf({ text: (await readFile(ledger.path, 'utf8',)).trim(), },);
        expect(Object.keys(line,).toSorted(),).toEqual(['at', 'model', 'promptTokens', 'wh',],);
        expect(line.wh,).toBeCloseTo(0.015, 12,);
        expect(lines.some(function isSpend(entry,): boolean {
          return entry.includes(`SPEND provider=neuralwatt model=${SERVED} prompt=90 completion=10 cost=0.00001`,);
        },),).toBe(true,);
        expect(lines.some(function warns(entry,): boolean {
          return entry.toLowerCase()
            .includes('billed',) || entry.includes('stop until',);
        },),).toBe(false,);
        expect(runSpendUsd({ provider: 'neuralwatt', },) - before,).toBeCloseTo(0.00001, 12,);
        /**
         The day after the costed call.
         */
        const energy = await ledger.read({ day: MIDNIGHT_DAY, },);
        expect(energy.remainingWh,).toBeCloseTo(550 - 0.015, 12,);
        expect('billedToday' in energy,).toBe(false,);
      },
    },),

    it({
      name: 'LETS A LEDGER LINE THAT CANNOT BE WRITTEN REACH THE CALLER, after the energy was said, and releases the '
        + 'hold, so a failed write leaves no phantom spend in flight',
      fn: async ctx => {
        await using scratch = await scratchDir({ prefix: 'whiskers-neuralwatt-settle-', },);
        const { client, ledger, } = scriptedClient({
          dir: scratch.path,
          chat: [{ status: 200, bodyText: streamOf({ comments: ENERGY_COMMENT + FREE_COST_COMMENT, }), },],
        },);
        // THE FILE READS BUT CANNOT BE APPENDED TO: read-only, so the write fails after the reservation was granted.
        await writeFile(ledger.path, '', 'utf8',);
        await chmod(ledger.path, 0o444,);
        /**
         What the call throws.
         */
        const outcome: { refusal?: unknown; } = {};
        /**
         Lines the call logged before it threw.
         */
        const lines = await consoleLinesDuring({
          run: async function call(): Promise<void> {
            outcome.refusal = await rejectionOf(async function writing(): Promise<void> {
              await ask({ client, },);
            },);
          },
          sinon: ctx.sinon,
        },);
        expect(outcome.refusal,).toBeInstanceOf(Error,);
        expect(lines.some(function says(entry,): boolean {
          return entry.includes('0.0150 Wh measured against',);
        },),).toBe(true,);
        expect(await ledger.read({ day: MIDNIGHT_DAY, },),).toMatchObject({ reservedWh: 0, calls: 0, },);
      },
    },),
  ],
},);
