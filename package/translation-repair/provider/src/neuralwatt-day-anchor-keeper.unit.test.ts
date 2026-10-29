/**
 Tests that a NeuralWatt day whose start could not be learned is learned later: the owner wants the free energy used
 "front and center", and one transient quota failure at the start of a run must not send every call of the run to a
 paid provider. Only a KNOWN anchor is kept; an unknown one is tried again from a fresh quota reading, no sooner than
 60 seconds after the last failed attempt on the injected clock, and the outage is warned about once.

 Fixtures are cat-themed invention; the clock is the hand-moved one of the scripted client.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { neuralWattIsDry, } from '../dist/final/node/index.mjs';
import { consoleLinesDuring, } from './neuralwatt-ledger.test-fixture.ts';
import {
  keyAllowanceOf,
  quotaExchangesOf,
  quotaReply,
  scriptedClient,
  SIGNAL,
} from './neuralwatt-scripted-client.test-fixture.ts';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';

/**
 The instant the run starts at, after the day began: 15:15 EDT in America/New_York is 19:15 UTC.
 */
const START = '2026-10-20T23:15:00.000Z';

/**
 Start of the subscription period the quota reports.
 */
const PERIOD_START = '2026-10-20T19:15:08Z';

/**
 A quota reply that carries the period start.
 */
const GOOD = quotaReply({ kwhUsed: 1, periodStart: PERIOD_START, },);

/**
 A quota reply whose period start is null.
 */
const NO_PERIOD = quotaReply({ kwhUsed: 1, periodStartAbsent: true, },);

/**
 A quota reply that is a server error.
 */
const DOWN = { status: 503, bodyText: 'down', };

await describe({
  name: 'NeuralWatt day anchor retry',
  concurrency: 1,
  children: [
    it({
      name: 'LEARNS THE DAY AFTER A QUOTA READ THAT FAILED AT THE START: NeuralWatt is refused until the retry, then '
        + 'usable, the retry a fresh quota read and not the failed baseline',
      fn: async ctx => {
        await using scratch = await scratchDir({ prefix: 'whiskers-neuralwatt-retry-', },);
        const { client, clock, exchanges, } = scriptedClient({
          dir: scratch.path,
          clockStart: START,
          quota: [DOWN, GOOD,],
        },);
        /**
         The readings before and after the retry is due.
         */
        const readings: Awaited<ReturnType<typeof client.energy>>[] = [];
        await consoleLinesDuring({
          run: async function readings_(): Promise<void> {
            readings.push(await client.energy({ signal: SIGNAL, },),);
            clock.setTo('2026-10-20T23:16:01.000Z',);
            readings.push(await client.energy({ signal: SIGNAL, },),);
          },
          sinon: ctx.sinon,
        },);
        expect(readings[0] === undefined ? undefined : neuralWattIsDry({ energy: readings[0], },),).toBe(true,);
        expect(readings[1]?.dayStart,).toBe('2026-10-20T19:15:08.000Z',);
        expect(readings[1] === undefined ? undefined : neuralWattIsDry({ energy: readings[1], },),).toBe(false,);
        expect(quotaExchangesOf({ exchanges, },).length,).toBe(2,);
      },
    },),
    it({
      name: 'MAKES NO QUOTA READ INSIDE THE 60 SECONDS AFTER A FAILED ATTEMPT, and reads again at 60 seconds exactly',
      fn: async ctx => {
        await using scratch = await scratchDir({ prefix: 'whiskers-neuralwatt-retry-', },);
        const { client, clock, exchanges, } = scriptedClient({
          dir: scratch.path,
          clockStart: START,
          quota: [DOWN, DOWN, GOOD,],
        },);
        /**
         Quota reads seen after each reading.
         */
        const reads: number[] = [];
        await consoleLinesDuring({
          run: async function readings(): Promise<void> {
            await client.energy({ signal: SIGNAL, },);
            reads.push(quotaExchangesOf({ exchanges, },).length,);
            clock.setTo('2026-10-20T23:15:30.000Z',);
            await client.energy({ signal: SIGNAL, },);
            reads.push(quotaExchangesOf({ exchanges, },).length,);
            clock.setTo('2026-10-20T23:15:59.999Z',);
            await client.energy({ signal: SIGNAL, },);
            reads.push(quotaExchangesOf({ exchanges, },).length,);
            clock.setTo('2026-10-20T23:16:00.000Z',);
            await client.energy({ signal: SIGNAL, },);
            reads.push(quotaExchangesOf({ exchanges, },).length,);
            clock.setTo('2026-10-20T23:16:59.999Z',);
            await client.energy({ signal: SIGNAL, },);
            reads.push(quotaExchangesOf({ exchanges, },).length,);
            clock.setTo('2026-10-20T23:17:00.000Z',);
            await client.energy({ signal: SIGNAL, },);
            reads.push(quotaExchangesOf({ exchanges, },).length,);
          },
          sinon: ctx.sinon,
        },);
        expect(reads,).toEqual([1, 1, 1, 2, 2, 3,],);
      },
    },),
    it({
      name: 'KEEPS RETRYING A REPLY WITH NO current_period_start AT THE SPACING, warns once for the outage, and logs '
        + 'where the day starts when it is finally learned',
      fn: async ctx => {
        await using scratch = await scratchDir({ prefix: 'whiskers-neuralwatt-retry-', },);
        const { client, clock, exchanges, } = scriptedClient({
          dir: scratch.path,
          clockStart: START,
          quota: [NO_PERIOD, NO_PERIOD, NO_PERIOD, GOOD,],
        },);
        /**
         The day start each reading named, none where it named none.
         */
        const dayStarts: string[] = [];
        const lines = await consoleLinesDuring({
          run: async function readings(): Promise<void> {
            dayStarts.push((await client.energy({ signal: SIGNAL, },)).dayStart ?? 'none',);
            clock.setTo('2026-10-20T23:16:01.000Z',);
            dayStarts.push((await client.energy({ signal: SIGNAL, },)).dayStart ?? 'none',);
            clock.setTo('2026-10-20T23:17:02.000Z',);
            dayStarts.push((await client.energy({ signal: SIGNAL, },)).dayStart ?? 'none',);
            clock.setTo('2026-10-20T23:18:03.000Z',);
            dayStarts.push((await client.energy({ signal: SIGNAL, },)).dayStart ?? 'none',);
          },
          sinon: ctx.sinon,
        },);
        expect(dayStarts,).toEqual(['none', 'none', 'none', '2026-10-20T19:15:08.000Z',],);
        expect(quotaExchangesOf({ exchanges, },).length,).toBe(4,);
        expect(lines.filter(function warnsOfUnknownDay(line,): boolean {
          return line.includes('subscription.current_period_start',) && line.includes('the day start is unknown',);
        },).length,).toBe(1,);
        expect(lines.filter(function logsLearnedDay(line,): boolean {
          return line.includes('NeuralWatt day starts at 19:15:08.000 UTC',) && line.includes('learned on a retry',);
        },).length,).toBe(1,);
      },
    },),
    it({
      name: 'TAKES THE KEY\'S CAP FROM THE RETRY THAT LEARNED THE DAY, a blocked cap holding NeuralWatt dry without a '
        + 'further quota read',
      fn: async ctx => {
        await using scratch = await scratchDir({ prefix: 'whiskers-neuralwatt-retry-', },);
        const { client, clock, exchanges, } = scriptedClient({
          dir: scratch.path,
          clockStart: START,
          quota: [
            DOWN,
            quotaReply({
              kwhUsed: 1,
              periodStart: PERIOD_START,
              keyAllowance: keyAllowanceOf({ remainingUsd: 0, blocked: true, },),
            },),
          ],
        },);
        /**
         The reading once the retry is due.
         */
        const reading = await (async function readAfterRetry() {
          await consoleLinesDuring({
            run: async function first(): Promise<void> {
              await client.energy({ signal: SIGNAL, },);
            },
            sinon: ctx.sinon,
          },);
          clock.setTo('2026-10-20T23:16:01.000Z',);
          return await client.energy({ signal: SIGNAL, },);
        })();
        expect(reading.dayStart,).toBe('2026-10-20T19:15:08.000Z',);
        expect(reading.keyBlockedUntil,).toBe('2026-10-21T19:15:08.000Z',);
        expect(neuralWattIsDry({ energy: reading, },),).toBe(true,);
        expect(quotaExchangesOf({ exchanges, },).length,).toBe(2,);
      },
    },),
    it({
      name: 'SHARES ONE RETRY AMONG THE CALLS THAT ARRIVE WHILE IT IS IN FLIGHT: two readings at once make one quota read',
      fn: async ctx => {
        await using scratch = await scratchDir({ prefix: 'whiskers-neuralwatt-retry-', },);
        const { client, clock, exchanges, } = scriptedClient({
          dir: scratch.path,
          clockStart: START,
          quota: [DOWN, GOOD,],
        },);
        await consoleLinesDuring({
          run: async function readings(): Promise<void> {
            await client.energy({ signal: SIGNAL, },);
            clock.setTo('2026-10-20T23:16:01.000Z',);
            await Promise.all([client.energy({ signal: SIGNAL, },), client.energy({ signal: SIGNAL, },),],);
          },
          sinon: ctx.sinon,
        },);
        expect(quotaExchangesOf({ exchanges, },).length,).toBe(2,);
      },
    },),
    it({
      name: 'WARNS ONCE FOR AN OUTAGE OF THE QUOTA that lasts through several retries, naming the missing field',
      fn: async ctx => {
        await using scratch = await scratchDir({ prefix: 'whiskers-neuralwatt-retry-', },);
        const { client, clock, } = scriptedClient({
          dir: scratch.path,
          clockStart: START,
          quota: [DOWN,],
        },);
        const lines = await consoleLinesDuring({
          run: async function readings(): Promise<void> {
            await client.energy({ signal: SIGNAL, },);
            clock.setTo('2026-10-20T23:16:01.000Z',);
            await client.energy({ signal: SIGNAL, },);
            clock.setTo('2026-10-20T23:17:02.000Z',);
            await client.energy({ signal: SIGNAL, },);
          },
          sinon: ctx.sinon,
        },);
        expect(lines.filter(function warnsOfUnknownDay(line,): boolean {
          return line.includes('the day start is unknown',);
        },).length,).toBe(1,);
      },
    },),
    it({
      name: 'REPORTS THE DECIDED ANCHOR WITHOUT DECIDING ONE: unknown before any reading and while the quota is down, '
        + 'then the day start\'s time of day once a reading has learned it, and no quota read is made by asking',
      fn: async ctx => {
        await using scratch = await scratchDir({ prefix: 'whiskers-neuralwatt-retry-', },);
        const { client, clock, exchanges, } = scriptedClient({
          dir: scratch.path,
          clockStart: START,
          quota: [DOWN, GOOD,],
        },);
        /**
         What the client reported it had decided, before any reading, after a failed one and after a good one.
         */
        const reported: ReturnType<typeof client.dayAnchor>[] = [client.dayAnchor(),];
        await consoleLinesDuring({
          run: async function readings(): Promise<void> {
            await client.energy({ signal: SIGNAL, },);
            reported.push(client.dayAnchor(),);
            clock.setTo('2026-10-20T23:16:01.000Z',);
            await client.energy({ signal: SIGNAL, },);
            reported.push(client.dayAnchor(),);
          },
          sinon: ctx.sinon,
        },);
        expect(reported,).toEqual([
          { known: false, },
          { known: false, },
          {
            known: true,
            msOfDay: 69_308_000,
          },
        ],);
        expect(quotaExchangesOf({ exchanges, },).length,).toBe(2,);
      },
    },),
  ],
},);
