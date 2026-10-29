/**
 Tests the NeuralWatt day and what is counted against it. The owner, 2026-10-20: "The stop logic should obviously
 count total free energy used per day. Every neuralwatt day starts at `15:15`", which is 19:15 UTC for an owner in
 America/New_York (EDT) and is the time of day of the subscription's `current_period_start` (`GET /v1/quota`
 answered `2026-10-20T19:15:08Z`). The free energy spent in that day is the account's, across every key
 (`GET /v1/usage/by-model`), and the ledger this pipeline keeps is the floor under it.

 REACHED THROUGH THE CLIENT AND THE LEDGER, because the day session is not on the package index, which exports only
 what the siblings and their tests reach, and a test imports the built package and not the source. The clock is the
 case's own, the transport is scripted, and no case reaches NeuralWatt or reads a key. Fixtures are cat-themed
 invention.

 @module
 */

import { appendFile, } from 'node:fs/promises';
import { join, } from 'node:path';

import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  NeuralWattAllowanceError,
  neuralWattIsDry,
  neuralWattMeterLevel,
} from '../dist/final/node/index.mjs';
import {
  consoleLinesDuring,
  lineOf,
} from './neuralwatt-ledger.test-fixture.ts';
import {
  ask,
  chatExchangesOf,
  keyAllowanceOf,
  quotaReply,
  scriptedClient,
  SIGNAL,
  usageExchangesOf,
  usageReply,
  usageRowOf,
} from './neuralwatt-scripted-client.test-fixture.ts';
import { rejectionOf, } from '@monochromatic-dev/translation-repair-test-fixture/ts/rejecting-call.test-fixture.ts';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';

/**
 Start of the subscription period the quota reports: 15:15 EDT in America/New_York is 19:15 UTC.
 */
const PERIOD_START = '2026-10-20T19:15:08Z';

/**
 Instant after the day began, the one the owner's probe ran at.
 */
const AFTER_ANCHOR = '2026-10-20T23:15:00.741Z';

/**
 The day that instant is in.
 */
const TODAY_START = '2026-10-20T19:15:08.000Z';

/**
 Model every ledger line is for.
 */
const MODEL = 'deepseek-v4.1-flash';

/**
 Reads the energy of one client.

 @param client - client under test

 @returns Reading
 */
async function energyOf(
  { client, }: { readonly client: ReturnType<typeof scriptedClient>['client']; },
): ReturnType<ReturnType<typeof scriptedClient>['client']['energy']> {
  return await client.energy({ signal: SIGNAL, },);
}

await describe({
  name: 'NeuralWatt day',
  concurrency: 1,
  children: [
    describe({
      name: 'where the day starts',
      children: [
        it({
          name: 'STARTS THE DAY AT THE LATEST INSTANT AT OR BEFORE NOW WHOSE UTC TIME OF DAY IS THE PERIOD START\'S, '
            + 'on both sides of the anchor time and on the instant itself',
          fn: async () => {
            /**
             Instants the reading is taken at, and the day each is in.
             */
            const cases = [
              [AFTER_ANCHOR, '2026-10-20T19:15:08.000Z',],
              ['2026-10-20T19:15:07.999Z', '2026-10-19T19:15:08.000Z',],
              ['2026-10-20T19:15:08.000Z', '2026-10-20T19:15:08.000Z',],
              ['2026-10-21T01:00:00.000Z', '2026-10-20T19:15:08.000Z',],
              ['2026-10-20T00:00:00.000Z', '2026-10-19T19:15:08.000Z',],
            ] as const;
            /**
             The day start each reading named.
             */
            const named = await Promise.all(cases.map(async function readAt([now,],) {
              await using scratch = await scratchDir({ prefix: 'whiskers-neuralwatt-day-', },);
              const { client, } = scriptedClient({
                dir: scratch.path,
                clockStart: now,
                quota: [quotaReply({ kwhUsed: 1, periodStart: PERIOD_START, }),],
              },);
              return (await energyOf({ client, },)).dayStart;
            },),);
            expect(named,).toEqual(cases.map(function expectedOf([, expected,],): string {
              return expected;
            },),);
          },
        },),

        it({
          name: 'TAKES ONLY THE TIME OF DAY from the period start, so a period that began in another month anchors the '
            + 'same day, and a UTC midnight start is the old UTC day',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'whiskers-neuralwatt-day-', },);
            const { client, } = scriptedClient({
              dir: scratch.path,
              clockStart: AFTER_ANCHOR,
              quota: [quotaReply({ kwhUsed: 1, periodStart: '2026-09-20T19:15:08Z', }),],
            },);
            expect((await energyOf({ client, },)).dayStart,).toBe(TODAY_START,);
            await using other = await scratchDir({ prefix: 'whiskers-neuralwatt-day-', },);
            const midnight = scriptedClient({
              dir: other.path,
              clockStart: AFTER_ANCHOR,
              quota: [quotaReply({ kwhUsed: 1, periodStart: '2026-10-11T00:00:00Z', }),],
            },);
            expect((await energyOf({ client: midnight.client, },)).dayStart,).toBe('2026-10-20T00:00:00.000Z',);
          },
        },),

        it({
          name: 'SAYS ONCE AT RUN START THAT THE ANCHOR CAME FROM THE QUOTA\'S CURRENT_PERIOD_START, and the day it began',
          fn: async ctx => {
            await using scratch = await scratchDir({ prefix: 'whiskers-neuralwatt-day-', },);
            const { client, } = scriptedClient({
              dir: scratch.path,
              clockStart: AFTER_ANCHOR,
              quota: [quotaReply({ kwhUsed: 1, periodStart: PERIOD_START, }),],
            },);
            /**
             Lines the two readings logged.
             */
            const lines = await consoleLinesDuring({
              run: async function reading(): Promise<void> {
                await energyOf({ client, },);
                await energyOf({ client, },);
              },
              sinon: ctx.sinon,
            },);
            expect(lines.filter(function names(line,): boolean {
              return line.includes(TODAY_START,) && line.includes('current_period_start',);
            },).length,).toBe(1,);
          },
        },),

        it({
          name: 'SAYS AT RUN START THAT THE ANCHOR CAME FROM THE SETTING when the quota gave none, and the day it began',
          fn: async ctx => {
            await using scratch = await scratchDir({ prefix: 'whiskers-neuralwatt-day-', },);
            const { client, } = scriptedClient({
              dir: scratch.path,
              clockStart: AFTER_ANCHOR,
              quota: [quotaReply({ kwhUsed: 1, periodStartAbsent: true, }),],
              dayStartSetting: { msOfDay: ((19 * 60) + 15) * 60_000, },
            },);
            /**
             Lines the first reading logged.
             */
            const lines = await consoleLinesDuring({
              run: async function reading(): Promise<void> {
                await energyOf({ client, },);
              },
              sinon: ctx.sinon,
            },);
            expect(lines.some(function names(line,): boolean {
              return line.includes('2026-10-20T19:15:00.000Z',) && line.includes('TRANSLATION_REPAIR_NEURALWATT_DAY_START',);
            },),).toBe(true,);
          },
        },),
      ],
    },),

    describe({
      name: 'when the quota does not give the anchor',
      children: [
        it({
          name: 'FALLS BACK TO THE SETTING when the quota reply carries no period start or cannot be read',
          fn: async () => {
            /**
             What the quota route does in each run: no period start, a dropped connection, a refused status.
             */
            const quotas = [
              [quotaReply({ kwhUsed: 1, periodStartAbsent: true, }),],
              [new Error('connection dropped',),],
              [{ status: 503, bodyText: 'down', },],
            ] as const;
            /**
             The day start each run named.
             */
            const named = await Promise.all(quotas.map(async function readWith(quota,) {
              await using scratch = await scratchDir({ prefix: 'whiskers-neuralwatt-day-', },);
              const { client, } = scriptedClient({
                dir: scratch.path,
                clockStart: AFTER_ANCHOR,
                quota,
                dayStartSetting: { msOfDay: ((19 * 60) + 15) * 60_000, },
              },);
              return (await energyOf({ client, },)).dayStart;
            },),);
            expect(named,).toEqual(quotas.map(function expectedOf(): string {
              return '2026-10-20T19:15:00.000Z';
            },),);
          },
        },),

        it({
          name: 'REFUSES NEURALWATT FOR THE RUN WHEN NEITHER GIVES IT, dry on the meter and refused at a call that sends '
            + 'nothing, with one warning that names the missing field and both remedies, and no silent 00:00 UTC day',
          fn: async ctx => {
            await using scratch = await scratchDir({ prefix: 'whiskers-neuralwatt-day-', },);
            const { client, exchanges, } = scriptedClient({
              dir: scratch.path,
              clockStart: AFTER_ANCHOR,
              quota: [quotaReply({ kwhUsed: 1, periodStartAbsent: true, }),],
            },);
            /**
             Reading and refusal of the run.
             */
            const outcome: { energy?: Awaited<ReturnType<typeof energyOf>>; refusal?: unknown; } = {};
            /**
             Lines the run logged.
             */
            const lines = await consoleLinesDuring({
              run: async function runs(): Promise<void> {
                outcome.energy = await energyOf({ client, },);
                await energyOf({ client, },);
                outcome.refusal = await rejectionOf(async function sent(): Promise<void> {
                  await ask({ client, },);
                },);
              },
              sinon: ctx.sinon,
            },);
            expect(outcome.energy?.dayStart,).toBeUndefined();
            expect(neuralWattIsDry({ energy: nonNullishOrThrow(outcome.energy), },),).toBe(true,);
            expect(outcome.refusal,).toBeInstanceOf(NeuralWattAllowanceError,);
            expect(chatExchangesOf({ exchanges, },).length,).toBe(0,);
            /**
             Warnings that name the field and the setting.
             */
            const warnings = lines.filter(function names(line,): boolean {
              return line.includes('current_period_start',) && line.includes('TRANSLATION_REPAIR_NEURALWATT_DAY_START',);
            },);
            expect(warnings.length,).toBe(1,);
            expect(neuralWattMeterLevel({ energy: nonNullishOrThrow(outcome.energy), },),).toContain('neuralwattDayStart=none',);
          },
        },),
      ],
    },),

    describe({
      name: 'the ledger inside the day',
      children: [
        it({
          name: 'DOES NOT COUNT LEDGER LINES BEFORE THE ANCHOR THOUGH THEY SHARE ITS UTC DATE, and counts the lines '
            + 'after it, the evening before included while the new day has not begun',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'whiskers-neuralwatt-day-', },);
            const { client, clock, } = scriptedClient({
              dir: scratch.path,
              clockStart: AFTER_ANCHOR,
              quota: [quotaReply({ kwhUsed: 1, periodStart: PERIOD_START, }),],
            },);
            await appendFile(
              join(scratch.path, 'neuralwatt-energy.jsonl',),
              lineOf({ at: '2026-10-20T10:00:00.000Z', model: MODEL, wh: 100, promptTokens: 1, },)
                + lineOf({ at: '2026-10-20T19:15:07.999Z', model: MODEL, wh: 50, promptTokens: 1, },)
                + lineOf({ at: '2026-10-20T19:15:08.000Z', model: MODEL, wh: 7, promptTokens: 1, },)
                + lineOf({ at: '2026-10-20T22:00:00.000Z', model: MODEL, wh: 3, promptTokens: 1, },),
              'utf8',
            );
            expect(await energyOf({ client, },),).toMatchObject({ spentWh: 10, calls: 2, },);
            clock.setTo('2026-10-20T15:00:00.000Z',);
            expect(await energyOf({ client, },),).toMatchObject({
              dayStart: '2026-10-19T19:15:08.000Z',
              spentWh: 150,
              calls: 2,
            },);
          },
        },),
      ],
    },),

    describe({
      name: 'the account-wide figure',
      children: [
        it({
          name: 'ASKS BY-MODEL FOR THE WINDOW FROM THE ANCHOR TO NOW AT EVERY READING, with full timestamps',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'whiskers-neuralwatt-day-', },);
            const { client, exchanges, } = scriptedClient({
              dir: scratch.path,
              clockStart: AFTER_ANCHOR,
              quota: [quotaReply({ kwhUsed: 1, periodStart: PERIOD_START, }),],
            },);
            await energyOf({ client, },);
            await energyOf({ client, },);
            /**
             The by-model reads the two readings made.
             */
            const reads = usageExchangesOf({ exchanges, },);
            expect(reads.length,).toBe(2,);
            /**
             Query of the first read.
             */
            const { searchParams, } = new URL(reads[0]?.url ?? '',);
            expect(searchParams.get('start_date',),).toBe(TODAY_START,);
            expect(searchParams.get('end_date',),).toBe(AFTER_ANCHOR,);
            expect(reads[0]?.method,).toBe('GET',);
            expect(reads[0]?.headers.Authorization,).toBe('Bearer test-key',);
          },
        },),

        it({
          name: 'COUNTS THE COVERED MODELS\' STANDARD AND FLEX ROWS and leaves out a model the free energy does not '
            + 'cover and the speed lanes',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'whiskers-neuralwatt-day-', },);
            const { client, } = scriptedClient({
              dir: scratch.path,
              clockStart: AFTER_ANCHOR,
              quota: [quotaReply({ kwhUsed: 1, periodStart: PERIOD_START, }),],
              usage: [usageReply({
                rows: [
                  usageRowOf({ requestedModel: 'mimo-v2.6-pro', kwh: 3.826265, },),
                  usageRowOf({ requestedModel: 'glm-5.3-flash', kwh: 0.002811, },),
                  usageRowOf({ requestedModel: 'deepseek-v4.1-flash', kwh: 0.001284, },),
                  usageRowOf({ requestedModel: 'deepseek-v4.1-flash-speed', kwh: 0.5, },),
                  usageRowOf({ requestedModel: 'glm-5.3-flash-flex', kwh: 0.001, },),
                ],
              },),],
            },);
            expect((await energyOf({ client, },)).accountWh,).toBeCloseTo(5.095, 6,);
          },
        },),

        it({
          name: 'REFUSES A REPLY WHOSE PERIOD IS NOT THE WINDOW ASKED, the date-only fallback of the route included, '
            + 'and uses the ledger alone with one warning',
          fn: async ctx => {
            /**
             What each reply's period said, and the reading it led to.
             */
            const echoes = [
              { start: '2026-10-20T00:00:00+00:00', end: '2026-10-20T00:00:00+00:00', },
              { start: '2026-10-20T19:15:08+00:00', end: '2026-10-20T22:15:00.741000+00:00', },
              { start: '2026-10-20T19:15:09+00:00', end: '2026-10-20T23:15:00.741000+00:00', },
            ];
            /**
             Readings taken, one per echo.
             */
            const energies: Awaited<ReturnType<typeof energyOf>>[] = [];
            /**
             Lines the readings logged.
             */
            const lines = await consoleLinesDuring({
              run: async function readings(): Promise<void> {
                energies.push(
                  ...await Promise.all(echoes.map(async function readWith(echo,) {
                    await using scratch = await scratchDir({ prefix: 'whiskers-neuralwatt-day-', },);
                    const { client, } = scriptedClient({
                      dir: scratch.path,
                      clockStart: AFTER_ANCHOR,
                      quota: [quotaReply({ kwhUsed: 1, periodStart: PERIOD_START, }),],
                      usage: [usageReply({ rows: [usageRowOf({ requestedModel: 'glm-5.3-flash', kwh: 0.5, },),], echo, },),],
                    },);
                    return await energyOf({ client, },);
                  },),),
                );
              },
              sinon: ctx.sinon,
            },);
            expect(energies.length,).toBe(3,);
            for (const energy of energies) {
              expect(energy.accountWh,).toBeUndefined();
              expect(energy.remainingWh,).toBe(550,);
            }
            expect(lines.filter(function names(line,): boolean {
              return line.includes('/usage/by-model',) && line.includes('period',);
            },).length,).toBe(3,);
          },
        },),

        it({
          name: 'ACCEPTS THE ECHO THE ROUTE GIVES an honoured window: the same instants in its own spelling',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'whiskers-neuralwatt-day-', },);
            const { client, } = scriptedClient({
              dir: scratch.path,
              clockStart: AFTER_ANCHOR,
              quota: [quotaReply({ kwhUsed: 1, periodStart: PERIOD_START, }),],
              usage: [usageReply({
                rows: [usageRowOf({ requestedModel: 'glm-5.3-flash', kwh: 0.002811, },),],
                echo: { start: '2026-10-20T19:15:08+00:00', end: '2026-10-20T23:15:00.741000+00:00', },
              },),],
            },);
            expect((await energyOf({ client, },)).accountWh,).toBeCloseTo(2.811, 6,);
          },
        },),

        it({
          name: 'LETS ENERGY SOMEONE ELSE SPENT ON THE ACCOUNT DRIVE THE DAY DRY though the ledger saw almost none of '
            + 'it, on the meter and at a call\'s own reservation, which is refused before anything is sent',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'whiskers-neuralwatt-day-', },);
            const { client, exchanges, } = scriptedClient({
              dir: scratch.path,
              clockStart: AFTER_ANCHOR,
              quota: [quotaReply({ kwhUsed: 1, periodStart: PERIOD_START, }),],
              usage: [usageReply({ rows: [usageRowOf({ requestedModel: 'glm-5.3-flash', kwh: 0.549999, },),], },),],
            },);
            await appendFile(
              join(scratch.path, 'neuralwatt-energy.jsonl',),
              lineOf({ at: '2026-10-20T20:00:00.000Z', model: MODEL, wh: 1, promptTokens: 1, },),
              'utf8',
            );
            /**
             The reading the account figure drives.
             */
            const energy = await energyOf({ client, },);
            expect(energy.spentWh,).toBe(1,);
            expect(energy.accountWh,).toBeCloseTo(549.999, 6,);
            expect(energy.remainingWh,).toBeCloseTo(0.001, 6,);
            expect(neuralWattIsDry({ energy, },),).toBe(true,);
            expect(await rejectionOf(async function sent(): Promise<void> {
              await ask({ client, },);
            },),).toBeInstanceOf(NeuralWattAllowanceError,);
            expect(chatExchangesOf({ exchanges, },).length,).toBe(0,);
          },
        },),

        it({
          name: 'LETS THE LEDGER DRIVE THE DAY WHEN IT IS ABOVE A LAGGING ACCOUNT FIGURE, the larger of the two and '
            + 'never their sum',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'whiskers-neuralwatt-day-', },);
            const { client, } = scriptedClient({
              dir: scratch.path,
              clockStart: AFTER_ANCHOR,
              quota: [quotaReply({ kwhUsed: 1, periodStart: PERIOD_START, }),],
              usage: [usageReply({ rows: [usageRowOf({ requestedModel: 'glm-5.3-flash', kwh: 0.1, },),], },),],
            },);
            await appendFile(
              join(scratch.path, 'neuralwatt-energy.jsonl',),
              lineOf({ at: '2026-10-20T20:00:00.000Z', model: MODEL, wh: 500, promptTokens: 1, },),
              'utf8',
            );
            /**
             The reading the ledger drives.
             */
            const energy = await energyOf({ client, },);
            expect(energy.accountWh,).toBeCloseTo(100, 6,);
            expect(energy.remainingWh,).toBeCloseTo(50, 6,);
            expect(neuralWattIsDry({ energy, },),).toBe(false,);
          },
        },),

        it({
          name: 'ADDS THE CALLS IN FLIGHT to the larger of the two figures',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'whiskers-neuralwatt-day-', },);
            const { client, ledger, } = scriptedClient({
              dir: scratch.path,
              clockStart: AFTER_ANCHOR,
              quota: [quotaReply({ kwhUsed: 1, periodStart: PERIOD_START, }),],
              usage: [usageReply({ rows: [usageRowOf({ requestedModel: 'glm-5.3-flash', kwh: 0.3, },),], },),],
            },);
            /**
             A hold taken against the day the account figure belongs to.
             */
            const outcome = await ledger.reserve({
              model: MODEL,
              promptTokens: 1,
              estimateWh: 2,
              day: { kind: 'anchored', start: new Date(TODAY_START,), },
            },);
            expect(outcome.granted,).toBe(true,);
            expect(await energyOf({ client, },),).toMatchObject({ accountWh: 300, reservedWh: 2, remainingWh: 248, },);
          },
        },),

        it({
          name: 'KEEPS THE LAST FIGURE THROUGH AN OUTAGE OF THE ROUTE WHILE IT BELONGS TO THE DAY, warning once per '
            + 'outage with the endpoint and the status, and warns again for the next outage',
          fn: async ctx => {
            await using scratch = await scratchDir({ prefix: 'whiskers-neuralwatt-day-', },);
            const rows = [usageRowOf({ requestedModel: 'glm-5.3-flash', kwh: 0.1, },),];
            const { client, } = scriptedClient({
              dir: scratch.path,
              clockStart: AFTER_ANCHOR,
              quota: [quotaReply({ kwhUsed: 1, periodStart: PERIOD_START, }),],
              usage: [
                usageReply({ rows, },),
                { status: 503, bodyText: 'down', },
                { status: 503, bodyText: 'down', },
                usageReply({ rows, },),
                { status: 503, bodyText: 'down', },
              ],
            },);
            /**
             What each reading said about the account.
             */
            const accounts: number[] = [];
            /**
             Lines the readings logged.
             */
            const lines = await consoleLinesDuring({
              run: async function readings(): Promise<void> {
                // IN ORDER, since each reading meets the next scripted answer of the route.
                accounts.push(nonNullishOrThrow((await energyOf({ client, },)).accountWh,), nonNullishOrThrow((await energyOf({ client, },)).accountWh,), nonNullishOrThrow((await energyOf({ client, },)).accountWh,), nonNullishOrThrow((await energyOf({ client, },)).accountWh,), nonNullishOrThrow((await energyOf({ client, },)).accountWh,));
              },
              sinon: ctx.sinon,
            },);
            expect(accounts.map(function rounded(wh,): number {
              return Math.round(wh,);
            },),).toEqual([100, 100, 100, 100, 100,],);
            expect(lines.filter(function names(line,): boolean {
              return line.includes('/usage/by-model',) && line.includes('503',);
            },).length,).toBe(2,);
          },
        },),

        it({
          name: 'DROPS A FIGURE FROM A PREVIOUS DAY AFTER AN OUTAGE and uses the ledger alone, one warning for the outage',
          fn: async ctx => {
            await using scratch = await scratchDir({ prefix: 'whiskers-neuralwatt-day-', },);
            const { client, clock, } = scriptedClient({
              dir: scratch.path,
              clockStart: AFTER_ANCHOR,
              quota: [quotaReply({ kwhUsed: 1, periodStart: PERIOD_START, }),],
              usage: [
                usageReply({ rows: [usageRowOf({ requestedModel: 'glm-5.3-flash', kwh: 0.5, },),], },),
                new Error('connection dropped',),
              ],
            },);
            expect((await energyOf({ client, },)).accountWh,).toBeCloseTo(500, 6,);
            clock.setTo('2026-10-21T19:15:09.000Z',);
            /**
             Reading in the next day, with the route down.
             */
            const outcome: { energy?: Awaited<ReturnType<typeof energyOf>>; } = {};
            await consoleLinesDuring({
              run: async function reading(): Promise<void> {
                outcome.energy = await energyOf({ client, },);
              },
              sinon: ctx.sinon,
            },);
            expect(outcome.energy?.accountWh,).toBeUndefined();
            expect(outcome.energy?.dayStart,).toBe('2026-10-21T19:15:08.000Z',);
            expect(outcome.energy?.remainingWh,).toBe(550,);
          },
        },),

        it({
          name: 'PRINTS THE ACCOUNT FIGURE AND THE DAY START ON THE METER LINE after the two it always printed, '
            + '\'none\' for an account figure that was not read',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'whiskers-neuralwatt-day-', },);
            const read = scriptedClient({
              dir: scratch.path,
              clockStart: AFTER_ANCHOR,
              quota: [quotaReply({ kwhUsed: 1, periodStart: PERIOD_START, }),],
              usage: [usageReply({ rows: [usageRowOf({ requestedModel: 'glm-5.3-flash', kwh: 0.002811, },),], },),],
            },);
            expect(neuralWattMeterLevel({ energy: await energyOf({ client: read.client, },), },),).toEqual([
              'neuralwattWh=547.189',
              'neuralwattReckonedWh=0.000',
              'neuralwattAccountWh=2.811',
              `neuralwattDayStart=${TODAY_START}`,
              'neuralwattKeyAllowance=none',
            ],);
            await using other = await scratchDir({ prefix: 'whiskers-neuralwatt-day-', },);
            const unread = scriptedClient({
              dir: other.path,
              clockStart: AFTER_ANCHOR,
              quota: [quotaReply({ kwhUsed: 1, periodStart: PERIOD_START, }),],
              usage: [{ status: 503, bodyText: 'down', },],
            },);
            expect(neuralWattMeterLevel({ energy: await energyOf({ client: unread.client, },), },),).toEqual([
              'neuralwattWh=550.000',
              'neuralwattReckonedWh=0.000',
              'neuralwattAccountWh=none',
              `neuralwattDayStart=${TODAY_START}`,
              'neuralwattKeyAllowance=none',
            ],);
          },
        },),
      ],
    },),

    describe({
      name: 'the key allowance',
      children: [
        it({
          name: 'READS A KEY WITH NO CAP (allowance null) AS NO ALLOWANCE: not dry, and \'none\' on the meter line',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'whiskers-neuralwatt-day-', },);
            const { client, } = scriptedClient({
              dir: scratch.path,
              clockStart: AFTER_ANCHOR,
              quota: [quotaReply({ kwhUsed: 1, periodStart: PERIOD_START, keyAllowance: null, }),],
            },);
            /**
             The reading of an uncapped key.
             */
            const energy = await energyOf({ client, },);
            expect(energy.keyAllowance,).toBeUndefined();
            expect(neuralWattIsDry({ energy, },),).toBe(false,);
            expect(neuralWattMeterLevel({ energy, },),).toContain('neuralwattKeyAllowance=none',);
          },
        },),

        it({
          name: 'PRINTS AN UNBLOCKED ALLOWANCE\'S REMAINDER AND THAT IT IS NOT BLOCKED, and stays wet',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'whiskers-neuralwatt-day-', },);
            const { client, } = scriptedClient({
              dir: scratch.path,
              clockStart: AFTER_ANCHOR,
              quota: [quotaReply({
                kwhUsed: 1,
                periodStart: PERIOD_START,
                keyAllowance: keyAllowanceOf({ remainingUsd: 3.25, blocked: false, resetAt: '2026-10-21T00:00:00Z', }),
              }),],
            },);
            /**
             The reading of a capped key with room.
             */
            const energy = await energyOf({ client, },);
            expect(neuralWattIsDry({ energy, },),).toBe(false,);
            expect(neuralWattMeterLevel({ energy, },).slice(4,),).toEqual([
              'neuralwattKeyRemainingUsd=3.25',
              'neuralwattKeyBlocked=no',
            ],);
          },
        },),

        it({
          name: 'HOLDS NEURALWATT DRY UNTIL RESET_AT WHILE THE ALLOWANCE IS BLOCKED, refuses a call before anything is '
            + 'sent, warns naming the allowance, and lets the day go on once reset_at has passed',
          fn: async ctx => {
            await using scratch = await scratchDir({ prefix: 'whiskers-neuralwatt-day-', },);
            const blocked = quotaReply({
              kwhUsed: 1,
              periodStart: PERIOD_START,
              keyAllowance: keyAllowanceOf({ remainingUsd: 0, blocked: true, resetAt: '2026-10-21T00:00:00Z', }),
            });
            const { client, clock, exchanges, } = scriptedClient({
              dir: scratch.path,
              clockStart: '2026-10-20T23:00:00.000Z',
              quota: [blocked,],
            },);
            /**
             Readings and refusal of the case.
             */
            const outcome: {
              before?: Awaited<ReturnType<typeof energyOf>>;
              after?: Awaited<ReturnType<typeof energyOf>>;
              refusal?: unknown;
            } = {};
            /**
             Lines it logged.
             */
            const lines = await consoleLinesDuring({
              run: async function runs(): Promise<void> {
                outcome.before = await energyOf({ client, },);
                outcome.refusal = await rejectionOf(async function sent(): Promise<void> {
                  await ask({ client, },);
                },);
                clock.setTo('2026-10-21T00:00:01.000Z',);
                outcome.after = await energyOf({ client, },);
              },
              sinon: ctx.sinon,
            },);
            expect(neuralWattIsDry({ energy: nonNullishOrThrow(outcome.before), },),).toBe(true,);
            expect(outcome.before?.keyBlockedUntil,).toBe('2026-10-21T00:00:00.000Z',);
            expect(outcome.refusal,).toBeInstanceOf(NeuralWattAllowanceError,);
            expect(chatExchangesOf({ exchanges, },).length,).toBe(0,);
            expect(neuralWattIsDry({ energy: nonNullishOrThrow(outcome.after), },),).toBe(false,);
            expect(lines.some(function names(line,): boolean {
              return line.includes('allowance',) && line.includes('blocked',);
            },),).toBe(true,);
            expect(neuralWattMeterLevel({ energy: nonNullishOrThrow(outcome.before), },).slice(4,),).toEqual([
              'neuralwattKeyRemainingUsd=0',
              'neuralwattKeyBlocked=yes',
            ],);
          },
        },),

        it({
          name: 'HOLDS A BLOCKED ALLOWANCE WITH NO RESET_AT DRY UNTIL THE NEXT DAY ANCHOR',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'whiskers-neuralwatt-day-', },);
            const { client, } = scriptedClient({
              dir: scratch.path,
              clockStart: '2026-10-20T23:00:00.000Z',
              quota: [quotaReply({
                kwhUsed: 1,
                periodStart: PERIOD_START,
                keyAllowance: keyAllowanceOf({ remainingUsd: 0, blocked: true }),
              }),],
            },);
            /**
             The reading of a blocked key.
             */
            const energy = await energyOf({ client, },);
            expect(neuralWattIsDry({ energy, },),).toBe(true,);
            expect(energy.keyBlockedUntil,).toBe('2026-10-21T19:15:08.000Z',);
          },
        },),

        it({
          name: 'WARNS ABOUT A MALFORMED ALLOWANCE AND IGNORES IT: not dry, \'none\' on the meter line, the quota '
            + 'baseline unharmed',
          fn: async ctx => {
            /**
             Allowances that do not read: wrong types, a flag that is not a boolean, a reset that is no instant, no object.
             */
            const malformed = [
              { remaining_usd: 'lots', blocked: 'maybe', },
              { limit_usd: 5, spent_usd: 1, remaining_usd: 4, blocked: 'yes', reset_at: null, },
              { limit_usd: 5, spent_usd: 5, remaining_usd: 0, blocked: true, reset_at: 'someday', },
              'capped',
            ];
            /**
             Readings taken, one per allowance.
             */
            const energies: Awaited<ReturnType<typeof energyOf>>[] = [];
            /**
             Lines the readings logged.
             */
            const lines = await consoleLinesDuring({
              run: async function readings(): Promise<void> {
                energies.push(
                  ...await Promise.all(malformed.map(async function readWith(keyAllowance,) {
                    await using scratch = await scratchDir({ prefix: 'whiskers-neuralwatt-day-', },);
                    const { client, } = scriptedClient({
                      dir: scratch.path,
                      clockStart: AFTER_ANCHOR,
                      quota: [quotaReply({ kwhUsed: 1, periodStart: PERIOD_START, keyAllowance, }),],
                    },);
                    return await energyOf({ client, },);
                  },),),
                );
              },
              sinon: ctx.sinon,
            },);
            expect(energies.length,).toBe(4,);
            for (const energy of energies) {
              expect(neuralWattIsDry({ energy, },),).toBe(false,);
              expect(energy.keyBlockedUntil,).toBeUndefined();
              expect(neuralWattMeterLevel({ energy, },),).toContain('neuralwattKeyAllowance=none',);
              expect(energy.dayStart,).toBe(TODAY_START,);
            }
            expect(lines.filter(function names(line,): boolean {
              return line.includes('allowance',) && line.includes('malformed',);
            },).length,).toBe(4,);
          },
        },),
      ],
    },),
  ],
},);
