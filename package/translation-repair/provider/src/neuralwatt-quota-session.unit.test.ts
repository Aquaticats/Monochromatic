/**
 Tests the two quota readings of one NeuralWatt run: a baseline taken lazily,
 once, before the first call, and the reading at the end, whose difference is
 the plan energy the run spent (`neuralwatt-quota.ts` says what that figure
 can and cannot see, `https://docs.neuralwatt.com/api/quota`, fetched
 2026-10-20). A reading that fails never blocks a call; it is warned, and the
 end report says why it has nothing to compare.

 REACHED THROUGH THE CLIENT, because `createNeuralWattQuotaSession` is not on
 the package index, which exports only what the siblings and their tests
 reach, and a test imports the built package and not the source. The client
 takes the baseline before its first call and hands the end report on as
 `planSpend`, so the cases ask a question and read the quota exchanges the
 scripted transport saw. No case reaches NeuralWatt or reads a key, the retry
 policy allows no retry so no case waits, and logging goes through the case's
 own sandbox. Fixtures are cat-themed invention.

 A BASELINE THAT FAILED IS TRIED AGAIN, no sooner than 60 seconds after the failure on the injected clock (the pace of
 the day anchor keeper), and a baseline taken that way is labelled as starting at the later reading. Those cases reach
 the session directly, over a scripted transport and a hand-moved clock, since only the session knows when it was asked.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type {
  TransportExchange,
  TransportReply,
} from '@monochromatic-dev/translation-repair-provider-stream';
import { createNeuralWattQuotaSession, } from '../dist/final/node/index.mjs';
import {
  consoleLinesDuring,
  handClock,
} from './neuralwatt-ledger.test-fixture.ts';
import {
  ask,
  BASE_URL,
  chatExchangesOf,
  NO_SUBSCRIPTION_REPLY,
  quotaExchangesOf,
  quotaReply,
  scriptedClient,
  SIGNAL,
  UNREPORTED_QUOTA_REPLY,
} from './neuralwatt-scripted-client.test-fixture.ts';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';

/**
 The day-start setting of the runs whose quota gives no period start: without a day a run sends nothing, and these
 cases are about the quota readings of a run that does.
 */
const MIDNIGHT_SETTING = { msOfDay: 0, } as const;

/**
 What the quota route does on a read: answers, or drops the connection.
 */
type QuotaReply = Error | TransportReply;

/**
 A quota reply that is a server error.
 */
const DOWN = { status: 503, bodyText: 'down', };

/**
 The instant the direct cases start at.
 */
const START = '2026-10-20T12:00:00.000Z';

/**
 A quota session over a scripted transport and a hand-moved clock.

 @param replies - what the quota route does on each read, in order, the last repeating

 @returns Session, reads the transport saw and clock

 @example
 ```ts
 const { session, reads, clock, } = sessionOver({ replies: [DOWN, quotaReply({ kwhUsed: 14, },),], },);
 ```
 */
function sessionOver(
  { replies, }: { readonly replies: readonly QuotaReply[]; },
): {
  readonly session: ReturnType<typeof createNeuralWattQuotaSession>;
  readonly reads: readonly TransportExchange[];
  readonly clock: ReturnType<typeof handClock>;
} {
  /**
   Every read the transport was handed.
   */
  const reads: TransportExchange[] = [];

  /**
   Clock the session reads, which a case moves by hand.
   */
  const clock = handClock({ start: START, },);
  return {
    reads,
    clock,
    session: createNeuralWattQuotaSession({
      headers: {},
      baseUrl: BASE_URL,
      now: clock.now,
      retryPolicy: {
        limit: 0,
        baseMs: 1,
      },
      transport: function answer(exchange,): Promise<TransportReply> {
        reads.push(exchange,);

        /**
         What this read is scripted to get.
         */
        const scripted = replies[Math.min(
          reads.length - 1,
          replies.length - 1,
        )] ?? DOWN;
        return Error.isError(scripted,) ? Promise.reject(scripted,) : Promise.resolve(scripted,);
      },
    },),
  };
}

/**
 Quota replies that fail, with the kind of failure each is warned as: a refused status, a body that is not the
 documented shape and a dropped connection.
 */
const FAILURES = [
  [{ status: 403, bodyText: 'Forbidden', }, 'ProviderHttpError',],
  [{ status: 200, bodyText: '[1]', }, 'NeuralWattQuotaShapeError',],
  [new Error('connection dropped',), 'Error',],
] as const satisfies readonly (readonly [QuotaReply, string,])[];

await describe({
  name: 'NeuralWatt quota session',
  concurrency: 1,
  children: [
    describe({
      name: 'the baseline',
      children: [
        it({
          name: 'READS THE QUOTA ONCE BEFORE THE FIRST CALL, a GET to the quota route with the key and the signal '
            + 'the call carries, ahead of the first chat request, and says what the plan had charged at the start of '
            + 'the run',
          fn: async ctx => {
            await using scratch = await scratchDir({ prefix: 'whiskers-neuralwatt-quota-', },);
            const { client, exchanges, } = scriptedClient({ dir: scratch.path, },);
            /**
             Lines the call logged.
             */
            const lines = await consoleLinesDuring({
              run: async function call(): Promise<void> {
                await ask({ client, },);
              },
              sinon: ctx.sinon,
            },);
            expect(exchanges.map(function shape(exchange,): string {
              return `${exchange.method} ${exchange.url.split('?',)[0] ?? ''}`;
            },),).toEqual([
              `GET ${BASE_URL}/quota`,
              `GET ${BASE_URL}/models`,
              `GET ${BASE_URL}/usage/by-model`,
              `POST ${BASE_URL}/chat/completions`,
            ],);
            expect(exchanges[0]?.label,).toBe('quota',);
            expect(exchanges[0]?.headers.Authorization,).toBe('Bearer test-key',);
            expect(exchanges[0]?.signal,).toBe(SIGNAL,);
            expect(lines.some(function says(line,): boolean {
              return line.includes('NeuralWatt quota at the start of the run: 13.9023 kWh charged this period',);
            },),).toBe(true,);
          },
        },),

        it({
          name: 'SAYS NO ENERGY CHARGED TO A PLAN IS REPORTED when the account has no subscription or its reply leaves '
            + 'the charged figure out, since an unreported figure is not a zero',
          fn: async ctx => {
            /**
             Takes the baseline of one run whose quota reply is given.

             @param reply - what the quota route answers
             */
            async function takeBaseline(reply: QuotaReply,): Promise<void> {
              await using scratch = await scratchDir({ prefix: 'whiskers-neuralwatt-quota-', },);
              const { client, } = scriptedClient({
                dir: scratch.path,
                quota: [reply,],
                dayStartSetting: MIDNIGHT_SETTING,
              },);
              await ask({ client, },);
            }
            /**
             Lines the two runs logged.
             */
            const lines = await consoleLinesDuring({
              run: async function runs(): Promise<void> {
                await takeBaseline(NO_SUBSCRIPTION_REPLY,);
                await takeBaseline(UNREPORTED_QUOTA_REPLY,);
              },
              sinon: ctx.sinon,
            },);
            expect(lines.filter(function says(line,): boolean {
              return line.includes('NeuralWatt quota at the start of the run: no energy charged to a plan is reported',);
            },).length,).toBe(2,);
            expect(lines.some(function says(line,): boolean {
              return line.includes('kWh charged this period',);
            },),).toBe(false,);
          },
        },),

        it({
          name: 'TAKES ONE READING FOR EVERY CALL OF THE RUN, a second call and two calls made at the same time '
            + 'awaiting the first reading rather than reading again',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'whiskers-neuralwatt-quota-', },);
            const { client, exchanges, } = scriptedClient({ dir: scratch.path, },);
            await Promise.all([ask({ client, },), ask({ client, },),],);
            await ask({ client, },);
            expect(quotaExchangesOf({ exchanges, },).length,).toBe(1,);
            expect(chatExchangesOf({ exchanges, },).length,).toBe(3,);
          },
        },),

        it({
          name: 'NEVER BLOCKS A CALL FOR A READ THAT FAILED, a refused status, a body that is not the documented shape '
            + 'and a dropped connection alike: the call is sent, the failure is warned with its kind, and the read is '
            + 'not tried again while the clock has not moved',
          fn: async ctx => {
            /**
             Runs two calls on a run whose quota read fails.

             @param reply - what the quota route does

             @returns Quota reads and chat requests the transport saw
             */
            async function failingRun(reply: QuotaReply,): Promise<readonly number[]> {
              await using scratch = await scratchDir({ prefix: 'whiskers-neuralwatt-quota-', },);
              const { client, exchanges, } = scriptedClient({
                dir: scratch.path,
                quota: [reply,],
                dayStartSetting: MIDNIGHT_SETTING,
              },);
              expect((await ask({ client, },)).text,).toBe('{"spot":"sunbeam"}',);
              await ask({ client, },);
              return [quotaExchangesOf({ exchanges, },).length, chatExchangesOf({ exchanges, },).length,];
            }
            /**
             Reads and requests of each run.
             */
            const counts: (readonly number[])[] = [];
            /**
             Lines the failed reads logged.
             */
            const lines = await consoleLinesDuring({
              run: async function runs(): Promise<void> {
                /**
                 Reads and requests of every run, made at once.
                 */
                const made = await Promise.all(FAILURES.map(async function run([reply,],) {
                  return await failingRun(reply,);
                },),);
                counts.push(...made,);
              },
              sinon: ctx.sinon,
            },);
            expect(counts,).toEqual([[1, 2,], [1, 2,], [1, 2,],],);
            for (const [, kind,] of FAILURES) {
              expect(lines.some(function warns(line,): boolean {
                return line.includes(`NeuralWatt quota could not be read before the first call (${kind})`,)
                  && line.includes('read again no sooner than 60 seconds after this failure',)
                  && line.includes('no starting reading unless one of those reads succeeds',)
                  && (!line.includes('will have no starting reading',));
              },),).toBe(true,);
            }
          },
        },),
      ],
    },),

    describe({
      name: 'the end report',
      children: [
        it({
          name: 'TAKES NO READING FOR A RUN THAT MADE NO NEURALWATT CALL, and says no starting reading was taken',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'whiskers-neuralwatt-quota-', },);
            const { client, exchanges, } = scriptedClient({ dir: scratch.path, },);
            expect(await client.planSpend({ signal: SIGNAL, },),).toEqual({
              kind: 'not-comparable',
              reason: 'the run made no NeuralWatt call, so no starting reading was taken',
            },);
            expect(exchanges.length,).toBe(0,);
          },
        },),

        it({
          name: 'DOES NOT READ AGAIN WHEN THE BASELINE COULD NOT BE TAKEN AND NO TIME HAS PASSED, and says so with '
            + 'the number of attempts and when the last was, since there is nothing to compare a later reading with',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'whiskers-neuralwatt-quota-', },);
            const { client, exchanges, } = scriptedClient({
              dir: scratch.path,
              quota: [{ status: 403, bodyText: 'Forbidden', }, quotaReply({ kwhUsed: 14, }),],
              dayStartSetting: MIDNIGHT_SETTING,
            },);
            await ask({ client, },);
            expect(await client.planSpend({ signal: SIGNAL, },),).toEqual({
              kind: 'not-comparable',
              reason: 'the starting reading could not be taken: every attempt to read the quota for it failed '
                + '(1 attempt, the last at 2026-10-20T12:00:00.000Z)',
            },);
            expect(quotaExchangesOf({ exchanges, },).length,).toBe(1,);
          },
        },),

        it({
          name: 'REPORTS THE DIFFERENCE OF THE TWO READINGS as plan energy spent, the second read a GET to the same '
            + 'route as the first',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'whiskers-neuralwatt-quota-', },);
            const { client, exchanges, } = scriptedClient({
              dir: scratch.path,
              quota: [quotaReply({ kwhUsed: 13.9023, }), quotaReply({ kwhUsed: 13.9523, }),],
            },);
            await ask({ client, },);
            /**
             Difference of the two readings.
             */
            const spend = await client.planSpend({ signal: SIGNAL, },);
            expect(spend.kind,).toBe('measured',);
            if (spend.kind === 'measured') {
              expect(spend.kwh,).toBeCloseTo(0.05, 9,);
              expect(spend.inOverage,).toBe(false,);
            }
            expect(quotaExchangesOf({ exchanges, },).map(function urlOf(exchange,): string {
              return exchange.url;
            },),).toEqual([`${BASE_URL}/quota`, `${BASE_URL}/quota`,],);
          },
        },),

        it({
          name: 'DECLINES TO COMPARE READINGS THAT CANNOT BE COMPARED, a billing period that rolled over between them, '
            + 'and says why',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'whiskers-neuralwatt-quota-', },);
            const { client, } = scriptedClient({
              dir: scratch.path,
              quota: [
                quotaReply({ kwhUsed: 19, }),
                quotaReply({ kwhUsed: 0.2, periodStart: '2026-11-11T05:05:25Z', }),
              ],
            },);
            await ask({ client, },);
            /**
             Comparison of the start with the reading of the next period.
             */
            const spend = await client.planSpend({ signal: SIGNAL, },);
            expect(spend.kind,).toBe('not-comparable',);
            if (spend.kind === 'not-comparable')
              expect(spend.reason,).toContain('billing period',);
          },
        },),

        it({
          name: 'SAYS THE READING AT THE END COULD NOT BE TAKEN when it fails, a refused status, a body that is not '
            + 'the documented shape and a dropped connection alike, warning with the kind of failure and never throwing',
          fn: async ctx => {
            /**
             Reports the end of a run whose second quota read fails.

             @param reply - what the quota route does the second time

             @returns What the end report said
             */
            async function endOfRun(reply: QuotaReply,) {
              await using scratch = await scratchDir({ prefix: 'whiskers-neuralwatt-quota-', },);
              const { client, } = scriptedClient({
                dir: scratch.path,
                quota: [quotaReply({ kwhUsed: 13.9023, }), reply,],
              },);
              await ask({ client, },);
              return await client.planSpend({ signal: SIGNAL, },);
            }
            /**
             What each run's end report said.
             */
            const spends: unknown[] = [];
            /**
             Lines the failed reads logged.
             */
            const lines = await consoleLinesDuring({
              run: async function runs(): Promise<void> {
                /**
                 End report of every run, made at once.
                 */
                const reported = await Promise.all(FAILURES.map(async function run([reply,],) {
                  return await endOfRun(reply,);
                },),);
                spends.push(...reported,);
              },
              sinon: ctx.sinon,
            },);
            expect(spends,).toEqual(FAILURES.map(function said(): unknown {
              return { kind: 'not-comparable', reason: 'the reading at the end of the run could not be taken', };
            },),);
            for (const [, kind,] of FAILURES) {
              expect(lines.some(function warns(line,): boolean {
                return line.includes(`NeuralWatt quota could not be read at the end of the run (${kind})`,);
              },),).toBe(true,);
            }
          },
        },),
      ],
    },),

    describe({
      name: 'a baseline that failed',
      children: [
        it({
          name: 'IS TRIED AGAIN NO SOONER THAN 60 SECONDS AFTER THE FAILURE, and a baseline taken by the retry is '
            + 'labelled as starting at that later reading: its time is given, and the first read is not the start',
          fn: async ctx => {
            const { session, reads, clock, } = sessionOver({
              replies: [DOWN, quotaReply({ kwhUsed: 14, },), quotaReply({ kwhUsed: 14.5, },),],
            },);
            /**
             What the baseline said at each asking.
             */
            const takenAt: boolean[] = [];
            /**
             Reads seen after each asking.
             */
            const readsAt: number[] = [];
            const lines = await consoleLinesDuring({
              run: async function asks(): Promise<void> {
                for (const instant of [START, '2026-10-20T12:00:59.999Z', '2026-10-20T12:01:00.000Z',]) {
                  clock.setTo(instant,);
                  /* oxlint-disable no-await-in-loop -- each asking reads the clock the one before moved */
                  takenAt.push((await session.ensureBaseline({ signal: SIGNAL, },)).taken,);
                  /* oxlint-enable no-await-in-loop */
                  readsAt.push(reads.length,);
                }
              },
              sinon: ctx.sinon,
            },);
            expect(takenAt,).toEqual([false, false, true,],);
            expect(readsAt,).toEqual([1, 1, 2,],);
            clock.setTo('2026-10-20T12:30:00.000Z',);
            /**
             The end report.
             */
            const spend = await session.planSpend({ signal: SIGNAL, },);
            expect(spend,).toEqual({ kind: 'measured', kwh: 0.5, inOverage: false, laterStartAt: '2026-10-20T12:01:00.000Z', },);
            expect(reads.length,).toBe(3,);
            expect(lines.some(function says(line,): boolean {
              return line.includes('NeuralWatt quota read at 2026-10-20T12:01:00.000Z',)
                && line.includes('the first read of the run failed',)
                && line.includes('starts at this later reading',)
                && line.includes('energy spent before it is not counted',);
            },),).toBe(true,);
            expect(lines.some(function saysStartOfRun(line,): boolean {
              return line.includes('NeuralWatt quota at the start of the run',);
            },),).toBe(false,);
          },
        },),

        it({
          name: 'IS NOT TRIED AGAIN INSIDE THE 60 SECONDS, however often it is asked for, and says the first '
            + 'attempt failed',
          fn: async () => {
            const { session, reads, clock, } = sessionOver({ replies: [DOWN, quotaReply({ kwhUsed: 14, },),], },);
            expect((await session.ensureBaseline({ signal: SIGNAL, },)).taken,).toBe(false,);
            for (const instant of ['2026-10-20T12:00:01.000Z', '2026-10-20T12:00:30.000Z', '2026-10-20T12:00:59.999Z',]) {
              clock.setTo(instant,);
              /* oxlint-disable no-await-in-loop -- each asking reads the clock the one before moved */
              expect((await session.ensureBaseline({ signal: SIGNAL, },)).taken,).toBe(false,);
              /* oxlint-enable no-await-in-loop */
            }
            expect(reads.length,).toBe(1,);
          },
        },),

        it({
          name: 'IS TRIED ONCE WHEN TWO CALLS ASK AT THE SAME MOMENT AFTER THE 60 SECONDS, both awaiting the one retry',
          fn: async () => {
            const { session, reads, clock, } = sessionOver({ replies: [DOWN, quotaReply({ kwhUsed: 14, },),], },);
            await session.ensureBaseline({ signal: SIGNAL, },);
            clock.setTo('2026-10-20T12:02:00.000Z',);
            /**
             Both askings, made at once.
             */
            const outcomes = await Promise.all([
              session.ensureBaseline({ signal: SIGNAL, },),
              session.ensureBaseline({ signal: SIGNAL, },),
            ],);
            expect(outcomes.map(function takenOf(outcome,): boolean {
              return outcome.taken;
            },),).toEqual([true, true,],);
            expect(reads.length,).toBe(2,);
          },
        },),

        it({
          name: 'BECOMES THE BASELINE WHEN A LATER READ OF THE QUOTA SUCCEEDS, the day anchor\'s retry and the key '
            + 'allowance\'s refresh read it through readNow and are not asked twice, and the comparison is labelled as '
            + 'starting at that reading',
          fn: async () => {
            const { session, reads, clock, } = sessionOver({
              replies: [DOWN, quotaReply({ kwhUsed: 14, },), quotaReply({ kwhUsed: 14.5, },),],
            },);
            await session.ensureBaseline({ signal: SIGNAL, },);
            clock.setTo('2026-10-20T12:00:05.000Z',);
            await session.readNow({ signal: SIGNAL, },);
            clock.setTo('2026-10-20T12:10:00.000Z',);
            expect(await session.planSpend({ signal: SIGNAL, },),).toMatchObject({
              kind: 'measured',
              laterStartAt: '2026-10-20T12:00:05.000Z',
            },);
            expect(reads.length,).toBe(3,);
          },
        },),

        it({
          name: 'IS NOT BEGUN BY A READ MADE BEFORE ANY ATTEMPT, since a run that never asked for a baseline made no '
            + 'NeuralWatt call and says so',
          fn: async () => {
            const { session, } = sessionOver({ replies: [quotaReply({ kwhUsed: 14, },),], },);
            await session.readNow({ signal: SIGNAL, },);
            expect(await session.planSpend({ signal: SIGNAL, },),).toEqual({
              kind: 'not-comparable',
              reason: 'the run made no NeuralWatt call, so no starting reading was taken',
            },);
          },
        },),

        it({
          name: 'STAYS NOT COMPARABLE WHEN EVERY READ FAILS, and says how many attempts failed and when the last was, '
            + 'warning once for the outage and not at each retry',
          fn: async ctx => {
            const { session, reads, clock, } = sessionOver({ replies: [DOWN,], },);
            const lines = await consoleLinesDuring({
              run: async function asks(): Promise<void> {
                for (const instant of [START, '2026-10-20T12:01:00.000Z', '2026-10-20T12:02:00.000Z',]) {
                  clock.setTo(instant,);
                  /* oxlint-disable no-await-in-loop -- each asking reads the clock the one before moved */
                  await session.ensureBaseline({ signal: SIGNAL, },);
                  /* oxlint-enable no-await-in-loop */
                }
              },
              sinon: ctx.sinon,
            },);
            clock.setTo('2026-10-20T12:03:00.000Z',);
            expect(await session.planSpend({ signal: SIGNAL, },),).toEqual({
              kind: 'not-comparable',
              reason: 'the starting reading could not be taken: every attempt to read the quota for it failed '
                + '(3 attempts, the last at 2026-10-20T12:02:00.000Z)',
            },);
            expect(reads.length,).toBe(3,);
            expect(lines.filter(function warns(line,): boolean {
              return line.includes('NeuralWatt quota could not be read before the first call',);
            },).length,).toBe(1,);
          },
        },),

        it({
          name: 'LEAVES A BASELINE TAKEN ON THE FIRST TRY AS IT WAS, with no later start in the end report, and reads '
            + 'the quota no more however much time passes',
          fn: async () => {
            const { session, reads, clock, } = sessionOver({
              replies: [quotaReply({ kwhUsed: 14, },), quotaReply({ kwhUsed: 14.5, },),],
            },);
            expect((await session.ensureBaseline({ signal: SIGNAL, },)).taken,).toBe(true,);
            clock.setTo('2026-10-20T14:00:00.000Z',);
            expect((await session.ensureBaseline({ signal: SIGNAL, },)).taken,).toBe(true,);
            expect(reads.length,).toBe(1,);
            expect(await session.planSpend({ signal: SIGNAL, },),).toEqual({ kind: 'measured', kwh: 0.5, inOverage: false, },);
          },
        },),

        it({
          name: 'IS LEARNED BY THE DAY ANCHOR\'S RETRY THROUGH A CLIENT: the first read fails, the retry 61 seconds later '
            + 'reads the quota, and the end report compares from that later reading',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'whiskers-neuralwatt-quota-', },);
            const { client, clock, exchanges, } = scriptedClient({
              dir: scratch.path,
              quota: [DOWN, quotaReply({ kwhUsed: 14, },), quotaReply({ kwhUsed: 14.5, },),],
            },);
            await client.energy({ signal: SIGNAL, },);
            clock.setTo('2026-10-20T12:01:01.000Z',);
            await client.energy({ signal: SIGNAL, },);
            clock.setTo('2026-10-20T12:30:00.000Z',);
            expect(await client.planSpend({ signal: SIGNAL, },),).toMatchObject({
              kind: 'measured',
              kwh: 0.5,
              laterStartAt: '2026-10-20T12:01:01.000Z',
            },);
            expect(quotaExchangesOf({ exchanges, },).length,).toBe(3,);
          },
        },),

        it({
          name: 'IS LEARNED BY THE KEY ALLOWANCE\'S REFRESH THROUGH A CLIENT: the first read fails with the day given by '
            + 'the setting, a call is sent, and the next meter reading takes the baseline',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'whiskers-neuralwatt-quota-', },);
            const { client, clock, exchanges, } = scriptedClient({
              dir: scratch.path,
              quota: [DOWN, quotaReply({ kwhUsed: 14, },), quotaReply({ kwhUsed: 14.5, },),],
              dayStartSetting: MIDNIGHT_SETTING,
            },);
            await ask({ client, },);
            clock.setTo('2026-10-20T12:05:00.000Z',);
            await client.energy({ signal: SIGNAL, },);
            clock.setTo('2026-10-20T12:30:00.000Z',);
            expect(await client.planSpend({ signal: SIGNAL, },),).toMatchObject({
              kind: 'measured',
              kwh: 0.5,
              laterStartAt: '2026-10-20T12:05:00.000Z',
            },);
            expect(quotaExchangesOf({ exchanges, },).length,).toBe(3,);
          },
        },),
      ],
    },),
  ],
},);
