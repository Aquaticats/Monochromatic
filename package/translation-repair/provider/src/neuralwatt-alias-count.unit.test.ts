/**
 Tests that the account-wide count of a NeuralWatt day includes the calls made under a tracking alias
 (`https://docs.neuralwatt.com/guides/tracking-aliases`, fetched 2026-10-20): `nw-flash` points at DeepSeek V4.1 Flash,
 the owner uses that model elsewhere on the same account, and a usage row for such a call carries the alias as its
 `requested_model`. What each alias points at is read from `GET /v1/models` at each account reading, because aliases
 move.

 Fixtures are the 2026-10-20 listing's names and cat-themed invention.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { consoleLinesDuring, } from './neuralwatt-ledger.test-fixture.ts';
import {
  DEEPSEEK_LISTED_AS,
  KIMI_LISTED_AS,
  LISTED_MODELS,
  modelEntryOf,
  modelsReply,
} from './neuralwatt-models-listing.test-fixture.ts';
import {
  modelsExchangesOf,
  quotaReply,
  scriptedClient,
  SIGNAL,
  usageReply,
  usageRowOf,
} from './neuralwatt-scripted-client.test-fixture.ts';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';

/**
 Start of the subscription period the quota reports: 15:15 EDT in America/New_York is 19:15 UTC.
 */
const PERIOD_START = '2026-10-20T19:15:08Z';

/**
 Instant after the day began.
 */
const AFTER_ANCHOR = '2026-10-20T23:15:00.741Z';

/**
 Rows of one day: the covered model under its own name, the flash alias and its twin, the speed lane, and the large alias.
 */
const ROWS = [
  usageRowOf({ requestedModel: 'deepseek-v4.1-flash', kwh: 0.001, servedModelId: 'deepseek-ai/DeepSeek-V4.1-Flash', },),
  usageRowOf({ requestedModel: 'nw-flash', kwh: 0.002, servedModelId: 'deepseek-ai/DeepSeek-V4.1-Flash', },),
  usageRowOf({ requestedModel: 'nw-flash-flex', kwh: 0.0005, servedModelId: 'deepseek-ai/DeepSeek-V4.1-Flash', },),
  usageRowOf({ requestedModel: 'nw-flash-speed', kwh: 0.2, servedModelId: 'deepseek-ai/DeepSeek-V4.1-Flash', },),
  usageRowOf({ requestedModel: 'nw-large', kwh: 0.1, servedModelId: 'moonshotai/Kimi-K3', },),
];

await describe({
  name: 'NeuralWatt alias count',
  concurrency: 1,
  children: [
    it({
      name: 'COUNTS THE CALLS MADE UNDER A TRACKING ALIAS AND ITS FLEX TWIN toward the day, and not the speed lane or an '
        + 'alias of a model the free energy does not cover',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'whiskers-neuralwatt-alias-', },);
        const { client, } = scriptedClient({
          dir: scratch.path,
          clockStart: AFTER_ANCHOR,
          quota: [quotaReply({ kwhUsed: 1, periodStart: PERIOD_START, }),],
          usage: [usageReply({ rows: ROWS, },),],
        },);
        expect((await client.energy({ signal: SIGNAL, },)).accountWh,).toBeCloseTo(3.5, 6,);
      },
    },),
    it({
      name: 'READS WHAT THE ALIASES POINT AT AT EACH ACCOUNT READING and follows an alias that moved to a model the free '
        + 'energy does not cover',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'whiskers-neuralwatt-alias-', },);
        const { client, exchanges, } = scriptedClient({
          dir: scratch.path,
          clockStart: AFTER_ANCHOR,
          quota: [quotaReply({ kwhUsed: 1, periodStart: PERIOD_START, }),],
          usage: [usageReply({ rows: ROWS, },),],
          models: [
            modelsReply({ entries: LISTED_MODELS, },),
            modelsReply({
              entries: [
                modelEntryOf({ id: 'deepseek-v4.1-flash', listedAs: DEEPSEEK_LISTED_AS, },),
                modelEntryOf({ id: 'glm-5.3-flash', listedAs: 'zai-org/GLM-5.3-Flash', },),
                modelEntryOf({ id: 'nw-flash', listedAs: KIMI_LISTED_AS, },),
                modelEntryOf({ id: 'nw-flash-flex', listedAs: KIMI_LISTED_AS, },),
              ],
            }),
          ],
        },);
        expect((await client.energy({ signal: SIGNAL, },)).accountWh,).toBeCloseTo(3.5, 6,);
        expect((await client.energy({ signal: SIGNAL, },)).accountWh,).toBeCloseTo(1, 6,);
        /**
         The listing reads the two readings made.
         */
        const reads = modelsExchangesOf({ exchanges, },);
        expect(reads.length,).toBe(2,);
        expect(reads[0]?.headers.Authorization,).toBe('Bearer test-key',);
      },
    },),
    it({
      name: 'COUNTS EVERY nw- ROW A COVERED MODEL SERVED WHEN THE LISTING CANNOT BE READ, and warns once per outage '
        + 'naming the endpoint and the status',
      fn: async ctx => {
        await using scratch = await scratchDir({ prefix: 'whiskers-neuralwatt-alias-', },);
        const { client, } = scriptedClient({
          dir: scratch.path,
          clockStart: AFTER_ANCHOR,
          quota: [quotaReply({ kwhUsed: 1, periodStart: PERIOD_START, }),],
          usage: [usageReply({ rows: ROWS, },),],
          models: [{ status: 503, bodyText: 'down', },],
        },);
        /**
         What each reading said about the account.
         */
        const accounts: number[] = [];
        const lines = await consoleLinesDuring({
          run: async function readings(): Promise<void> {
            // IN ORDER, since each reading meets the next scripted answer of the routes.
            accounts.push(Number((await client.energy({ signal: SIGNAL, },)).accountWh,), Number((await client.energy({ signal: SIGNAL, },)).accountWh,));
          },
          sinon: ctx.sinon,
        },);
        expect(accounts.map(function rounded(wh,): number {
          return Math.round(wh * 1_000,);
        },),).toEqual([3_500, 3_500,],);
        expect(lines.filter(function names(line,): boolean {
          return line.includes('/models',) && line.includes('503',);
        },).length,).toBe(1,);
      },
    },),
  ],
},);
