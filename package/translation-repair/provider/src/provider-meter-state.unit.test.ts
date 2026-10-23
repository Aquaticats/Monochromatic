/**
 Tests for the three states a provider meter can be in, and for which of them
 stops us spending.

 THE POINT OF THE THIRD STATE IS THAT IT DOES NOT CHANGE ROUTING. An
 unreachable meter has always routed as spendable and still does; what it
 gains here is a name, so a later reader of the log cannot mistake a
 monitoring failure for a provider that was up. Both halves are pinned in this file,
 because a change breaking either would be silent.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';
import {
  parseQuotaSnapshot,
  ProviderHttpError,
} from '@monochromatic-dev/translation-repair-provider-stream';
import { recordedTransport, } from '@monochromatic-dev/translation-repair-provider-stream/ts/recorded-transport.test-fixture.ts';
import {
  createProviderBudgets,
  createSyntheticClient,
  meterRecordOf,
  readEveryMeter,
  routesAsDry,
} from '../dist/final/node/index.mjs';
import { warnLinesDuring, } from '@monochromatic-dev/translation-repair-typography/ts/console-warn-lines.test-fixture.ts';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';
import { TEST_MAX_HOLD_MS, } from './budget-hold.test-fixture.ts';

/**
 Error a meter read rejects with when its endpoint cannot be reached.
 */
class MeterUnreachableError extends Error {
  public override readonly name = 'MeterUnreachableError';
}

/**
 Body of the live reading of 2026-10-23 at 03:28:16 UTC (HTTP 200, 269 characters): the weekly block is named
 `planWeeklyLimit`, spent to 0 %, and the body holds no `weeklyTokenLimit`. The values of `planWeeklyLimit` are the measured
 ones; the other blocks hold the measured key types with invented numbers.
 */
const SPENT_RENAMED_WEEKLY_BODY = JSON.stringify({
  subscription: { limit: 640, requests: 3, renewsAt: '2026-10-24T03:00:00.000Z', },
  search: { hourly: { limit: 200, requests: 0, renewsAt: '2026-10-23T04:00:00.000Z', }, },
  planWeeklyLimit: { nextRegenAt: '2026-10-23T05:03:47.000Z', percentRemaining: 0, nextRegenPercent: 2, },
},);

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: meterRecordOf.name,
      // ONE CASE AT A TIME: two cases divert `console.warn` across an await to
      // read what the package's root logger wrote.
      concurrency: 1,
      children: [
        it({
          name: 'reads a meter reporting budget left as wet',
          fn: async () => {
            /**
             Record a meter answering "there is budget" produces.
             */
            const meter = await meterRecordOf({
              name: 'synthetic',
              signal: AbortSignal.timeout(HANG_STOP_MS,),
              readLevel: async () => ({
                dry: false,
                fields: ['syntheticWeekly=97%',],
              }),
            },);

            expect(meter.state,).toBe('wet',);
          },
        },),

        it({
          name: 'reads a meter reporting nothing left as dry',
          fn: async () => {
            /**
             Record a meter answering "there is nothing left" produces.
             */
            const meter = await meterRecordOf({
              name: 'synthetic',
              signal: AbortSignal.timeout(HANG_STOP_MS,),
              readLevel: async () => ({
                dry: true,
                fields: ['syntheticWeekly=0%',],
              }),
            },);

            expect(meter.state,).toBe('dry',);
          },
        },),

        it({
          name: 'FORWARDS the numbers the meter was read from, so a dry verdict can be checked',
          fn: async () => {
            /**
             Record carrying what the meter actually said, which is what
             separates an empty budget from a threshold that was wrong about a
             budget that was not.
             */
            const meter = await meterRecordOf({
              name: 'hyper',
              signal: AbortSignal.timeout(HANG_STOP_MS,),
              readLevel: async () => ({
                dry: true,
                fields: ['hyperBalance=0',],
              }),
            },);

            expect(meter.fields,).toEqual(['hyperBalance=0',],);
          },
        },),

        it({
          name: 'REFUSES to guess for a meter that could not be read, naming it instead',
          fn: async () => {
            /**
             Record an unreachable endpoint produces, which is the whole reason
             this type has a third member.
             */
            const meter = await meterRecordOf({
              name: 'hyper',
              signal: AbortSignal.timeout(HANG_STOP_MS,),
              readLevel: async () => {
                throw new MeterUnreachableError('endpoint refused the connection',);
              },
            },);

            expect(meter.state,).toBe('unreadable',);
          },
        },),

        it({
          name: 'reports no numbers for a meter that never answered',
          fn: async () => {
            /**
             Record of a read that rejected, which has nothing to report a level
             from and must not invent one.
             */
            const meter = await meterRecordOf({
              name: 'hyper',
              signal: AbortSignal.timeout(HANG_STOP_MS,),
              readLevel: async () => {
                throw new MeterUnreachableError('endpoint refused the connection',);
              },
            },);

            expect(meter.fields,).toEqual([],);
          },
        },),

        it({
          name: 'WARNS A METER THE PROVIDER REFUSED with its status and the provider\'s words, and records it '
            + 'unreadable with no numbers',
          fn: async () => {
            const { result: meter, warned, } = await warnLinesDuring({
              run: async function readRefusedMeter() {
                return await meterRecordOf({
                  name: 'hyper',
                  signal: AbortSignal.timeout(HANG_STOP_MS,),
                  readLevel: async () => {
                    throw new ProviderHttpError({
                      provider: 'hyper',
                      status: 401,
                      bodyText: 'this cat has no collar tag',
                    },);
                  },
                },);
              },
            },);

            expect(warned,).toEqual([
              '[translation-repair] [meterRecordOf] hyper: budget unreadable, treating as spendable (refused by '
                + 'ProviderHttpError from hyper with HTTP 401 (the provider said: "this cat has no collar tag"))',
            ],);
            expect(meter,).toEqual({
              state: 'unreadable',
              fields: [],
            },);
          },
        },),

        it({
          name: 'WARNS A QUOTAS BODY THAT VIOLATES EXPECTATIONS WITH THE KEYS IT HELD AND THE JSON TYPE OF EACH, never a value: an '
            + 'empty body, a body that lacks the weekly block, and a body whose weekly block holds other fields, whose values a reader '
            + 'of the log must not see',
          fn: async () => {
            /**
             The warning a body comes to when the meter reads it.
             */
            async function warnedFor({ bodyText, }: { readonly bodyText: string; },): Promise<readonly string[]> {
              const { warned, } = await warnLinesDuring({
                run: async function readBody() {
                  return await meterRecordOf({
                    name: 'synthetic',
                    signal: AbortSignal.timeout(HANG_STOP_MS,),
                    readLevel: async () => {
                      parseQuotaSnapshot({ bodyText, },);
                      throw new MeterUnreachableError('a body that parses is not reached by this case',);
                    },
                  },);
                },
              },);
              return warned;
            }
            /**
             A body of the older shape, with values an account would not want in a log.
             */
            const older = JSON.stringify({
              subscription: {
                limit: 135,
                requests: 41,
                renewsAt: '2026-09-21T14:36:14.288Z',
                plan: 'whiskers-private-plan',
              },
              freeToolCalls: { limit: 0, },
            },);
            /**
             A body with no five-hour block whose weekly block holds other fields, with values an account would not want in a log.
             */
            const reshaped = JSON.stringify({
              subscription: { limit: 135, },
              weeklyTokenLimit: {
                percentLeft: 64.5,
                plan: 'whiskers-private-plan',
              },
            },);
            const lines = [
              await warnedFor({ bodyText: '{}', },),
              await warnedFor({ bodyText: older, },),
              await warnedFor({ bodyText: reshaped, },),
            ];
            expect(lines,).toEqual([
              [
                '[translation-repair] [meterRecordOf] synthetic: budget unreadable, treating as spendable (Synthetic /quotas body '
                  + 'violated expectations: neither weeklyTokenLimit nor planWeeklyLimit is an object; received {})',
              ],
              [
                '[translation-repair] [meterRecordOf] synthetic: budget unreadable, treating as spendable (Synthetic /quotas body '
                  + 'violated expectations: neither weeklyTokenLimit nor planWeeklyLimit is an object; received subscription:object, freeToolCalls:object)',
              ],
              [
                '[translation-repair] [meterRecordOf] synthetic: budget unreadable, treating as spendable (Synthetic /quotas body '
                  + 'violated expectations: weeklyTokenLimit.percentRemaining is not a number; received subscription:object, '
                  + 'weeklyTokenLimit:object; weeklyTokenLimit holds percentLeft:number, plan:string)',
              ],
            ],);
            /**
             The warnings as one text.
             */
            const joined = lines.flat()
              .join('\n',);
            expect(['135', '41', '64.5', 'whiskers-private-plan', '2026-09-21',].filter(function appears(value,): boolean {
              return joined.includes(value,);
            },),).toEqual([],);
          },
        },),

        it({
          name: 'WARNS A METER READ THE CALLER STOPPED AS STOPPED, not as a meter that could not be read, and still '
            + 'records it unreadable with no numbers, which every sharer of the reading routes as spendable',
          fn: async () => {
            /**
             Why the caller stopped the reading.
             */
            const reason = new Error('the owner called the cats in',);
            /**
             The caller's steering, stopped before the meter answers.
             */
            const steering = new AbortController();
            steering.abort(reason,);
            const { result: meter, warned, } = await warnLinesDuring({
              run: async function readStoppedMeter() {
                return await meterRecordOf({
                  name: 'synthetic',
                  signal: steering.signal,
                  readLevel: async () => {
                    throw reason;
                  },
                },);
              },
            },);

            expect(warned,).toEqual([
              '[translation-repair] [meterRecordOf] synthetic: budget not read, since the caller stopped the '
                + 'reading; treating as spendable',
            ],);
            expect(meter,).toEqual({
              state: 'unreadable',
              fields: [],
            },);
          },
        },),
      ],
    },),

    describe({
      name: 'the Synthetic meter on the body that renamed the weekly block',
      // ONE CASE AT A TIME: one case diverts `console.warn` across an await to swallow the refusal it provokes.
      concurrency: 1,
      children: [
        it({
          name: 'READS DRY, NOT UNREADABLE, from a real client on the measured body, and STATES the return the body names',
          fn: async () => {
            const { transport, } = recordedTransport({ replies: [{ status: 200, bodyText: SPENT_RENAMED_WEEKLY_BODY, },], },);
            const meters = await readEveryMeter({
              synthetic: createSyntheticClient({ apiKey: 'test-key', transport, },),
              signal: AbortSignal.timeout(HANG_STOP_MS,),
            },);

            expect(meters.synthetic,).toEqual({
              state: 'dry',
              fields: [
                'syntheticWeekly=0%',
                'syntheticFiveHour=none',
                'syntheticThrottled=none',
                'syntheticWeeklyReturn=2026-10-23T05:03:47.000Z',
              ],
            },);
          },
        },),

        it({
          name: 'READS WET, with no weekly return stated, from a real client on the renamed block while weekly budget is left',
          fn: async () => {
            const { transport, } = recordedTransport({
              replies: [{
                status: 200,
                bodyText: SPENT_RENAMED_WEEKLY_BODY.replace('"percentRemaining":0', '"percentRemaining":12.5',),
              },],
            },);
            const meters = await readEveryMeter({
              synthetic: createSyntheticClient({ apiKey: 'test-key', transport, },),
              signal: AbortSignal.timeout(HANG_STOP_MS,),
            },);

            expect(meters.synthetic,).toEqual({
              state: 'wet',
              fields: ['syntheticWeekly=12.5%', 'syntheticFiveHour=none', 'syntheticThrottled=none',],
            },);
          },
        },),

        it({
          name: 'READS DRY WITH THE RETURN UNREADABLE, never a forged field, from a real client on a renamed block whose return is '
            + 'no instant as the logger writes one (a forged string, no milliseconds, a number, a missing key), and logs no value',
          fn: async () => {
            /**
             The measured body with the `nextRegenAt` member replaced by each text, the empty text dropping the member.
             */
            const bodies = [
              String.raw`"nextRegenAt":"a synthetic=wet\nINJECT",`,
              '"nextRegenAt":"2026-10-23T05:03:47Z",',
              '"nextRegenAt":5,',
              '',
            ].map(function bodyWith(member,): string {
              return SPENT_RENAMED_WEEKLY_BODY.replace('"nextRegenAt":"2026-10-23T05:03:47.000Z",', member,);
            },);
            /**
             The meter record read from each body by a real client, beside every warning written meanwhile.
             */
            const { result: records, warned, } = await warnLinesDuring({
              run: async function readBodiesWithoutInstant() {
                return await Promise.all(bodies.map(async function readBody(bodyText,) {
                  const { transport, } = recordedTransport({ replies: [{ status: 200, bodyText, },], },);
                  const meters = await readEveryMeter({
                    synthetic: createSyntheticClient({ apiKey: 'test-key', transport, },),
                    signal: AbortSignal.timeout(HANG_STOP_MS,),
                  },);
                  return meters.synthetic;
                },),);
              },
            },);

            expect(records,).toEqual(bodies.map(function dryWithUnreadableReturn() {
              return {
                state: 'dry',
                fields: [
                  'syntheticWeekly=0%',
                  'syntheticFiveHour=none',
                  'syntheticThrottled=none',
                  'syntheticWeeklyReturn=unreadable',
                ],
              };
            },),);
            expect(warned,).toEqual(bodies.map(function warningOfBody() {
              return '[translation-repair] [weeklyWindowOf] Synthetic /quotas planWeeklyLimit.nextRegenAt is missing or no ISO-8601 '
                + 'instant as the logger writes one; the return reads unreadable';
            },),);
          },
        },),

        it({
          name: 'READS DRY WITH THE RETURN UNREADABLE from a real client on a body carrying both weekly names that agree on the '
            + 'percent and differ in the return, and warns once naming both blocks and the field',
          fn: async () => {
            /**
             The measured body with the old name added beside the new one, spent to 0 % in both and returning at another instant.
             */
            const bodyText = SPENT_RENAMED_WEEKLY_BODY.replace(
              '"planWeeklyLimit"',
              '"weeklyTokenLimit":{"nextRegenAt":"2026-10-23T06:00:00.000Z","percentRemaining":0},"planWeeklyLimit"',
            );
            const { transport, } = recordedTransport({ replies: [{ status: 200, bodyText, },], },);
            const { result: meters, warned, } = await warnLinesDuring({
              run: async function readDisagreeingBlocks() {
                return await readEveryMeter({
                  synthetic: createSyntheticClient({ apiKey: 'test-key', transport, },),
                  signal: AbortSignal.timeout(HANG_STOP_MS,),
                },);
              },
            },);

            expect(meters.synthetic,).toEqual({
              state: 'dry',
              fields: [
                'syntheticWeekly=0%',
                'syntheticFiveHour=none',
                'syntheticThrottled=none',
                'syntheticWeeklyReturn=unreadable',
              ],
            },);
            expect(warned,).toEqual([
              '[translation-repair] [weeklyWindowFromBody] Synthetic /quotas weeklyTokenLimit and planWeeklyLimit agree on '
                + 'percentRemaining and differ in nextRegenAt; the return reads unreadable',
            ],);
          },
        },),

        it({
          name: 'ROUTES SYNTHETIC AS DRY and reports the dry state to the seat reader, from the budget layer over a real client',
          fn: async () => {
            const { transport, } = recordedTransport({ replies: [{ status: 200, bodyText: SPENT_RENAMED_WEEKLY_BODY, },], },);
            const budgets = createProviderBudgets({
              maxHoldMs: TEST_MAX_HOLD_MS,
              synthetic: createSyntheticClient({ apiKey: 'test-key', transport, },),
            },);
            const signal = AbortSignal.timeout(HANG_STOP_MS,);

            expect((await budgets.read({ signal, },)).synthetic,).toBe(true,);
            expect((await budgets.readStates({ signal, },)).synthetic,).toBe('dry',);
          },
        },),
      ],
    },),

    describe({
      name: routesAsDry.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'holds out only a meter that answered and said dry',
          fn: async () => {
            expect(routesAsDry({ state: 'dry', },),).toBe(true,);
          },
        },),

        it({
          name: 'ACCEPTS a wet meter for spending',
          fn: async () => {
            expect(routesAsDry({ state: 'wet', },),).toBe(false,);
          },
        },),

        it({
          name: 'ACCEPTS an unreadable meter for spending, so lost monitoring is not an outage',
          fn: async () => {
            expect(routesAsDry({ state: 'unreadable', },),).toBe(false,);
          },
        },),
      ],
    },),
  ],
},);
