/**
 Tests how NeuralWatt takes its place at the front of the spending order: what
 its meter says is dry, what the meter line prints, which roster models it
 reaches, and where a call goes when it is wet, dry, or refuses.

 The owner, 2026-10-20: "Use the neuralwatt free energy front and center."

 Fixtures are the roster itself and cat-themed invention.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import { NoProviderForModelError, } from '@monochromatic-dev/translation-repair-markup';
import {
  type ProviderName,
  ROSTER_MODEL_IDS,
} from '@monochromatic-dev/translation-repair-shared-contract';
import {
  createProviderBudgets,
  createRoutingClient,
  NEURALWATT_SMALLEST_ESTIMATE_WH,
  type NeuralWattEnergy,
  NeuralWattAllowanceError,
  neuralWattIsDry,
  neuralWattMeterLevel,
  reachOf,
  readEveryMeter,
  routeProviderFor,
  visionReachOf,
} from '../dist/final/node/index.mjs';
import { consoleLinesDuring, } from './neuralwatt-ledger.test-fixture.ts';
import {
  SEAT_HYPER_OPENROUTER_UNMEASURED,
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_HYPER_VISION,
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { TEST_MAX_HOLD_MS, } from './budget-hold.test-fixture.ts';

/**
 Abort signal every call here carries.
 */
const SIGNAL = new AbortController().signal;

/**
 Energy reading with a given remainder.

 @param remainingWh - what the day has left

 @returns The reading

 @example
 ```ts
 const energy = energyOf({ remainingWh: 12.5, },);
 ```
 */
function energyOf({ remainingWh, }: { readonly remainingWh: number; },): NeuralWattEnergy {
  return {
    dayStart: '2026-10-20T19:15:08.000Z',
    dailyWh: 550,
    spentWh: 550 - remainingWh,
    reckonedWh: 0.5,
    reservedWh: 0,
    remainingWh,
    calls: 3,
  };
}

/**
 Every provider serving, none dry, none full.
 */
const ALL_REACH = {
  neuralwatt: true,
  synthetic: true,
  bedrock: true,
  deepseek: false,
  xiaomi: false,
  anthropic: false,
  hyper: true,
  openrouter: true,
} as const;

/**
 No provider dry.
 */
const NONE_DRY = {
  neuralwatt: false,
  synthetic: false,
  bedrock: false,
  deepseek: false,
  xiaomi: false,
  anthropic: false,
  hyper: false,
  openrouter: false,
} as const;

/**
 No provider's slots all taken.
 */
const NONE_FULL = NONE_DRY;

/**
 A routing client over NeuralWatt and Hyper whose NeuralWatt caller refuses
 its first call as too large for the energy left (an estimate of 120 Wh
 against 100 Wh) and takes every call after it.

 @returns The client and the providers it asked, in order

 @example
 ```ts
 const { called, client, } = routedWithOneLargeRefusal();
 ```
 */
function routedWithOneLargeRefusal(): {
  readonly called: readonly ProviderName[];
  readonly client: ReturnType<typeof createRoutingClient>;
} {
  /**
   Providers asked, in order.
   */
  const called: ProviderName[] = [];

  /**
   Calls NeuralWatt has taken, the first of which it refuses as too large.
   */
  const taken = { count: 0, };
  return {
    called,
    client: createRoutingClient({
      callers: {
        neuralwatt: {
          chatText: async () => {
            called.push('neuralwatt',);
            taken.count += 1;
            if (taken.count === 1)
              throw new NeuralWattAllowanceError({
                model: 'deepseek-v4.1-flash',
                estimateWh: 120,
                energy: energyOf({ remainingWh: 100, }),
              },);
            return { text: 'neuralwatt', };
          },
        },
        hyper: {
          chatText: async () => {
            called.push('hyper',);
            return { text: 'hyper', };
          },
        },
      },
      budgets: createProviderBudgets({
        maxHoldMs: TEST_MAX_HOLD_MS,
        neuralwatt: { energy: async () => energyOf({ remainingWh: 100, }), },
        hyper: { credits: async () => ({ balance: 10, }), },
      },),
    },),
  };
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: 'neuralwatt meter',
      children: [
        it({
          name: 'READS DRY ONCE THE DAY CANNOT HOLD EVEN THE SMALLEST CALL, wet at that figure, in watt-hours left after '
            + 'the holds in flight',
          fn: async () => {
            expect(neuralWattIsDry({ energy: energyOf({ remainingWh: 550, }), },),).toBe(false,);
            expect(neuralWattIsDry({ energy: energyOf({ remainingWh: NEURALWATT_SMALLEST_ESTIMATE_WH, }), },),).toBe(false,);
            expect(neuralWattIsDry({ energy: energyOf({ remainingWh: NEURALWATT_SMALLEST_ESTIMATE_WH / 2, }), },),).toBe(true,);
            expect(neuralWattIsDry({ energy: energyOf({ remainingWh: 0, }), },),).toBe(true,);
            expect(neuralWattIsDry({ energy: energyOf({ remainingWh: -0.4, }), },),).toBe(true,);
          },
        },),

        it({
          name: 'PRINTS WHAT IT READ AS KEY=VALUE FIELDS WITH NO SPACE IN A VALUE: the watt-hours left to three places '
            + 'the share that rests on estimates, the account figure, the day start and the key allowance, with no field '
            + 'about cost',
          fn: async () => {
            expect(neuralWattMeterLevel({ energy: energyOf({ remainingWh: 549.9856, }), },),).toEqual([
              'neuralwattWh=549.986',
              'neuralwattReckonedWh=0.500',
              'neuralwattAccountWh=none',
              'neuralwattDayStart=2026-10-20T19:15:08.000Z',
              'neuralwattKeyAllowance=none',
            ],);
            expect(neuralWattMeterLevel({ energy: energyOf({ remainingWh: 1, }), },),).toEqual([
              'neuralwattWh=1.000',
              'neuralwattReckonedWh=0.500',
              'neuralwattAccountWh=none',
              'neuralwattDayStart=2026-10-20T19:15:08.000Z',
              'neuralwattKeyAllowance=none',
            ],);
          },
        },),

        it({
          name: 'IS READ WITH THE OTHER METERS: wet or dry off the energy, and DRY WITH NOTHING TO REPORT where no key '
            + 'was set, as every unconfigured provider is',
          fn: async () => {
            /**
             Meters read with NeuralWatt configured and wet.
             */
            const wet = await readEveryMeter({
              neuralwatt: { energy: async () => energyOf({ remainingWh: 100, }), },
              signal: SIGNAL,
            },);
            expect(wet.neuralwatt,).toEqual({
              state: 'wet',
              fields: [
                'neuralwattWh=100.000',
                'neuralwattReckonedWh=0.500',
                'neuralwattAccountWh=none',
                'neuralwattDayStart=2026-10-20T19:15:08.000Z',
                'neuralwattKeyAllowance=none',
              ],
            },);
            /**
             Meters read with it dry.
             */
            const dry = await readEveryMeter({
              neuralwatt: { energy: async () => energyOf({ remainingWh: 0, }), },
              signal: SIGNAL,
            },);
            expect(dry.neuralwatt.state,).toBe('dry',);
            /**
             Meters read with no key.
             */
            const unconfigured = await readEveryMeter({ signal: SIGNAL, },);
            expect(unconfigured.neuralwatt,).toEqual({ state: 'dry', fields: [], },);
          },
        },),

        it({
          name: 'TREATS AN UNREADABLE LEDGER AS SPENDABLE, as every meter that could not be read is, and says so',
          fn: async () => {
            /**
             Meters read when the ledger will not read.
             */
            const meters = await readEveryMeter({
              neuralwatt: {
                energy: async () => {
                  throw new Error('ledger line 3 violated expectations',);
                },
              },
              signal: SIGNAL,
            },);
            expect(meters.neuralwatt.state,).toBe('unreadable',);
          },
        },),

        it({
          name: 'FEEDS THE BUDGET VIEW AND THE METERS LINE: dry when spent, wet when energy is left, its fields on the line',
          fn: async ctx => {
            const lines = await consoleLinesDuring({
              run: async function readOnce(): Promise<void> {
                /**
                 Budgets over a wet NeuralWatt meter alone.
                 */
                const budgets = createProviderBudgets({
                  maxHoldMs: TEST_MAX_HOLD_MS,
                  neuralwatt: { energy: async () => energyOf({ remainingWh: 250, }), },
                },);
                const view = await budgets.read({ signal: SIGNAL, },);
                expect(view.neuralwatt,).toBe(false,);
                expect(view.synthetic,).toBe(true,);
              },
              sinon: ctx.sinon,
            },);
            expect(lines.some(function isMeters(line,): boolean {
              return line.includes('METERS neuralwatt=wet',) && line.includes('neuralwattWh=250.000',)
                && (!line.includes('neuralwattBilled',));
            },),).toBe(true,);
          },
        },),
      ],
    },),

    describe({
      name: 'neuralwatt reach',
      children: [
        it({
          name: 'REACHES THE TWO MODELS THE FREE ENERGY COVERS AND NO OTHER, GLM-5.3-Flash\'s Synthetic withholding '
            + 'notwithstanding',
          fn: async () => {
            for (const modelId of ROSTER_MODEL_IDS) {
              /**
               Whether this model is one of the two.
               */
              const covered = (modelId === SEAT_HYPER_OPENROUTER_UNMEASURED)
                || (modelId === SEAT_HYPER_OPENROUTER_VISION_EDITOR);
              expect(reachOf({ modelId, },).neuralwatt,).toBe(covered,);
            }
          },
        },),

        it({
          name: 'SHOWS PICTURES TO THE COVERED MODELS because the listing reports image input for both, and to no other',
          fn: async () => {
            expect(visionReachOf({ modelId: SEAT_HYPER_OPENROUTER_UNMEASURED, },).neuralwatt,).toBe(true,);
            expect(visionReachOf({ modelId: SEAT_HYPER_OPENROUTER_VISION_EDITOR, },).neuralwatt,).toBe(true,);
            expect(visionReachOf({ modelId: SEAT_HYPER_VISION, },).neuralwatt,).toBe(false,);
          },
        },),
      ],
    },),

    describe({
      name: 'neuralwatt routing',
      children: [
        it({
          name: 'TAKES THE CALL FIRST WHILE IT IS WET, ahead of every provider that also serves the model',
          fn: async () => {
            expect(routeProviderFor({ reach: ALL_REACH, dry: NONE_DRY, saturated: NONE_FULL, },),).toEqual({ kind: 'neuralwatt', },);
          },
        },),

        it({
          name: 'PASSES THE CALL TO SYNTHETIC, THEN THE REST OF THE ORDER, WHEN ITS ENERGY IS SPENT',
          fn: async () => {
            expect(routeProviderFor({
              reach: ALL_REACH,
              dry: { ...NONE_DRY, neuralwatt: true, },
              saturated: NONE_FULL,
            },),).toEqual({ kind: 'synthetic', },);
            expect(routeProviderFor({
              reach: ALL_REACH,
              dry: { ...NONE_DRY, neuralwatt: true, synthetic: true, bedrock: true, },
              saturated: NONE_FULL,
            },),).toEqual({ kind: 'hyper', },);
          },
        },),

        it({
          name: 'SENDS A DEEPSEEK V4.1 FLASH CALL TO NEURALWATT WHILE IT HAS ENERGY, to Hyper when it has none, and a '
            + 'call to a model it does not serve to the provider that does',
          fn: async () => {
            /**
             Providers asked, in order.
             */
            const called: ProviderName[] = [];
            /**
             Stub caller for one provider.

             @param provider - whose caller

             @returns Caller recording its provider
             */
            const callerOf = (provider: ProviderName,) => ({
              chatText: async () => {
                called.push(provider,);
                return { text: provider, };
              },
            });
            /**
             Remainder the stub meter reads.
             */
            const meter = { remainingWh: 100, };
            const client = createRoutingClient({
              callers: {
                neuralwatt: callerOf('neuralwatt',),
                hyper: callerOf('hyper',),
              },
              budgets: createProviderBudgets({
                maxHoldMs: TEST_MAX_HOLD_MS,
                neuralwatt: { energy: async () => energyOf({ remainingWh: meter.remainingWh, }), },
                hyper: { credits: async () => ({ balance: 10, }), },
                freshForMs: 0,
                now: () => 1_000,
              },),
            },);
            const messages = [{ role: 'user' as const, content: 'Where does the cat sleep?', },];
            await client.chatText({ modelId: SEAT_HYPER_OPENROUTER_UNMEASURED, messages, signal: SIGNAL, },);
            await client.chatText({ modelId: SEAT_HYPER_OPENROUTER_VISION_EDITOR, messages, signal: SIGNAL, },);
            await client.chatText({ modelId: SEAT_HYPER_VISION, messages, signal: SIGNAL, },);
            meter.remainingWh = 0;
            await client.chatText({ modelId: SEAT_HYPER_OPENROUTER_UNMEASURED, messages, signal: SIGNAL, },);
            expect(called,).toEqual(['neuralwatt', 'neuralwatt', 'hyper', 'hyper',],);
          },
        },),

        it({
          name: 'MOVES ON TO THE NEXT PROVIDER WHEN NEURALWATT REFUSES A CALL ITS ESTIMATE DID NOT FIT, '
            + 'since that refusal is a budget refusal',
          fn: async () => {
            /**
             Providers asked, in order.
             */
            const called: ProviderName[] = [];
            const client = createRoutingClient({
              callers: {
                neuralwatt: {
                  chatText: async () => {
                    called.push('neuralwatt',);
                    throw new NeuralWattAllowanceError({
                      model: 'deepseek-v4.1-flash',
                      estimateWh: 0.3,
                      energy: energyOf({ remainingWh: 0.1, }),
                    },);
                  },
                },
                hyper: {
                  chatText: async () => {
                    called.push('hyper',);
                    return { text: 'hyper', };
                  },
                },
              },
              budgets: createProviderBudgets({
                maxHoldMs: TEST_MAX_HOLD_MS,
                neuralwatt: { energy: async () => energyOf({ remainingWh: 0.1, }), },
                hyper: { credits: async () => ({ balance: 10, }), },
              },),
              holdPollMs: 1,
            },);
            /**
             Reply the call settled on.
             */
            const reply = await client.chatText({
              modelId: SEAT_HYPER_OPENROUTER_UNMEASURED,
              messages: [{ role: 'user', content: 'Where does the cat sleep?', },],
              signal: SIGNAL,
            },);
            expect(called,).toEqual(['neuralwatt', 'hyper',],);
            expect(reply.text,).toBe('hyper',);
          },
        },),

        it({
          name: 'HOLDS NEURALWATT OUT FOR NOTHING WHEN IT REFUSES ONE LARGE CALL, so the small calls behind it still spend '
            + 'the free energy: the refused call goes to Hyper, and the next call goes to NeuralWatt again',
          fn: async () => {
            const { called, client, } = routedWithOneLargeRefusal();
            const messages = [{ role: 'user' as const, content: 'Where does the cat sleep?', },];
            await client.chatText({ modelId: SEAT_HYPER_OPENROUTER_UNMEASURED, messages, signal: SIGNAL, },);
            await client.chatText({ modelId: SEAT_HYPER_OPENROUTER_UNMEASURED, messages, signal: SIGNAL, },);
            expect(called,).toEqual(['neuralwatt', 'hyper', 'neuralwatt',],);
          },
        },),

        it({
          name: 'HOLDS NEURALWATT OUT FOR NOTHING WHEN IT REFUSES ONE LARGE RE-ASK, so the small calls behind it still '
            + 'spend the free energy: the re-ask gives up, and the next call goes to NeuralWatt again',
          fn: async () => {
            const { called, client, } = routedWithOneLargeRefusal();
            const messages = [{ role: 'user' as const, content: 'Where does the cat sleep?', },];
            /**
             What the re-ask threw, taken as data.
             */
            let refusal: unknown;
            try {
              await client.chatText({
                modelId: SEAT_HYPER_OPENROUTER_UNMEASURED,
                messages,
                signal: SIGNAL,
                otherThan: 'hyper',
              },);
            }
            catch (error) {
              refusal = error;
            }
            expect(refusal,).toBeInstanceOf(NoProviderForModelError,);
            await client.chatText({ modelId: SEAT_HYPER_OPENROUTER_UNMEASURED, messages, signal: SIGNAL, },);
            expect(called,).toEqual(['neuralwatt', 'neuralwatt',],);
          },
        },),

        it({
          name: 'LEAVES A MODEL SYNTHETIC SERVES ALONE OFF NEURALWATT: a call to a Synthetic-only model never names it',
          fn: async () => {
            expect(reachOf({ modelId: SEAT_SYNTHETIC_TEXT_EVERYWHERE, },).neuralwatt,).toBe(false,);
          },
        },),
      ],
    },),
  ],
},);
