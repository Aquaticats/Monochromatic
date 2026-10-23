/**
 Tests for the provider decision.

 THESE CASES ARE THE OWNER'S POLICY, stated as a table. Synthetic first,
 overflowing to the next provider with budget when a provider's per-model
 concurrency limit is taken (no live provider states one today, so those cases
 inject a saturated record); Synthetic's weekly budget emptying is a reason to
 fail over, and so is an empty five-hour window on a reply that still carries
 one; Hyper dry too sends the call to OpenRouter (2026-09-03); every
 provider empty at once ends the run.

 THE ASYMMETRY IS DELIBERATE and worth reading twice: a model that some
 providers do not serve loses its voice when the ones that do are dry, and
 the run continues. Only every budget being empty throws, because only then
 is nothing buyable at all.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import {
  EveryProviderDryError,
  hyperIsDry,
  hyperMeterLevel,
  type NeuralWattDryness,
  NO_PROVIDER,
  providerServing,
  routeProviderFor,
  syntheticIsDry,
  syntheticMeterLevel,
} from '../dist/final/node/index.mjs';

/**
 Budget reading with room left on both of Synthetic's limits.
 */
const roomyQuota = {
  fiveHour: {
    remaining: 750,
    max: 750,
    limited: false,
    nextTickAt: '2026-08-24T22:55:29.000Z',
  },
  weekly: {
    percentRemaining: 99.8,
    nextRegenAt: '2026-08-25T00:12:58.000Z',
  },
} as const;

/**
 Budget reading with no five-hour window, the shape of Synthetic's reply since its rolling limit went away (2026-10-19).
 */
const weeklyOnlyQuota = {
  weekly: {
    percentRemaining: 99.8,
    nextRegenAt: '2026-10-20T00:12:58.000Z',
  },
} as const;

/**
 Every provider reachable, which is true of the shared seats since OpenRouter
 serves the whole roster.
 */
const everyReach = {
  neuralwatt: false,
  synthetic: true,
  hyper: true,
  bedrock: true,
  deepseek: false,
  xiaomi: false,
  anthropic: false,
  openrouter: true,
} as const;

/**
 Nobody dry.
 */
const allWet = {
  neuralwatt: false,
  synthetic: false,
  hyper: false,
  bedrock: false,
  deepseek: false,
  xiaomi: false,
  anthropic: false,
  openrouter: false,
} as const;

/**
 Nobody saturated.
 */
const noneSaturated = allWet;

/**
 Every provider dry by its meter or a refusal.
 */
const allDry = {
  neuralwatt: true,
  synthetic: true,
  hyper: true,
  bedrock: true,
  deepseek: true,
  xiaomi: true,
  anthropic: true,
  openrouter: true,
} as const;

/**
 Message the router's run-ending refusal carries for one reading.

 @param dry - which providers read dry, budget and wire taken together

 @returns The refusal's message

 @throws Error when the reading leaves a provider buyable, so no refusal is raised

 @example
 ```ts
 const message = allDryMessageOf({ dry: allDry, },);
 ```
 */
function allDryMessageOf({ dry, }: { readonly dry: Readonly<Record<keyof typeof allDry, boolean>>; },): string {
  try {
    routeProviderFor({ reach: everyReach, dry, saturated: noneSaturated, },);
  } catch (error) {
    if (error instanceof EveryProviderDryError)
      return error.message;
    throw error;
  }
  throw new Error('the reading was meant to end the run',);
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: syntheticIsDry.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'READS a roomy budget as not dry',
          fn: async () => {
            expect(syntheticIsDry({ quota: roomyQuota, },),).toBe(false,);
          },
        },),

        it({
          name: 'READS AN EMPTY WEEKLY BUDGET AS DRY, which is the limit that actually emptied and '
            + 'cost 866 of 875 lost voices one HTTP 429 each',
          fn: async () => {
            expect(syntheticIsDry({
              quota: {
                ...roomyQuota,
                weekly: {
                  ...roomyQuota.weekly,
                  percentRemaining: 0,
                },
              },
            },),).toBe(true,);
          },
        },),

        it({
          name: 'READS an empty five-hour window as dry, since the owner corrected that this provider '
            + 'could have two limits and either one stops a call',
          fn: async () => {
            expect(syntheticIsDry({
              quota: {
                ...roomyQuota,
                fiveHour: {
                  ...roomyQuota.fiveHour,
                  remaining: 0,
                },
              },
            },),).toBe(true,);
          },
        },),

        it({
          name: 'READS A REPLY WITH NO FIVE-HOUR WINDOW BY ITS WEEKLY BUDGET ALONE, not dry with credit left and dry with none, '
            + 'since Synthetic dropped its rolling limit and the weekly budget always decides',
          fn: async () => {
            expect([
              syntheticIsDry({ quota: weeklyOnlyQuota, },),
              syntheticIsDry({
                quota: {
                  weekly: {
                    ...weeklyOnlyQuota.weekly,
                    percentRemaining: 0,
                  },
                },
              },),
            ],).toEqual([false, true,],);
          },
        },),

        it({
          name: 'READS the provider saying it is throttling as dry, even with credit left, because '
            + 'that is the provider stating the answer directly',
          fn: async () => {
            expect(syntheticIsDry({
              quota: {
                ...roomyQuota,
                fiveHour: {
                  ...roomyQuota.fiveHour,
                  limited: true,
                },
              },
            },),).toBe(true,);
          },
        },),
      ],
    },),

    describe({
      name: hyperIsDry.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'READS the measured balance as not dry',
          fn: async () => {
            expect(hyperIsDry({ credits: { balance: 249, }, },),).toBe(false,);
          },
        },),

        it({
          name: 'READS a spent-out balance as dry, at zero and below it',
          fn: async () => {
            expect(hyperIsDry({ credits: { balance: 0, }, },),).toBe(true,);
            expect(hyperIsDry({ credits: { balance: -1, }, },),).toBe(true,);
          },
        },),

        it({
          name: 'READS any positive balance as not dry, since what one call costs was never measured '
            + 'and a cushion would be a number nobody established',
          fn: async () => {
            expect(hyperIsDry({ credits: { balance: 0.01, }, },),).toBe(false,);
          },
        },),
      ],
    },),

    describe({
      name: providerServing.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'NAMES the first provider in spending order that serves the model and reads wet, or none',
          fn: async () => {
            expect(providerServing({
              reach: everyReach,
              dry: allWet,
            },),).toBe('synthetic',);
            expect(providerServing({
              reach: everyReach,
              dry: {
                ...allWet,
                synthetic: true,
              },
            },),).toBe('bedrock',);
            expect(providerServing({
              reach: everyReach,
              dry: {
                ...allWet,
                synthetic: true,
                bedrock: true,
              },
            },),).toBe('hyper',);
            expect(providerServing({
              reach: everyReach,
              dry: {
                ...allWet,
                synthetic: true,
                bedrock: true,
                hyper: true,
              },
            },),).toBe('openrouter',);
            expect(providerServing({
              reach: {
                ...everyReach,
                synthetic: false,
              },
              dry: allWet,
            },),).toBe('bedrock',);
            expect(providerServing({
              reach: everyReach,
              dry: {
                neuralwatt: true,
                synthetic: true,
                hyper: true,
                bedrock: true,
                deepseek: true,
                xiaomi: true,
                anthropic: true,
                openrouter: true,
              },
            },),).toBe(NO_PROVIDER,);
          },
        },),
      ],
    },),

    describe({
      name: routeProviderFor.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'SENDS TO SYNTHETIC FIRST while its per-model limit has room, which is the owner '
            + 'policy for maximum speed',
          fn: async () => {
            expect(routeProviderFor({
              reach: everyReach,
              dry: allWet,
              saturated: noneSaturated,
            },),).toEqual({ kind: 'synthetic', },);
          },
        },),

        it({
          name: 'OVERFLOWS TO BEDROCK, next in the order since 2026-09-07, once that limit is taken, '
            + 'because this provider has no per-model concurrency limit and waiting would cost the speed '
            + 'the split is for',
          fn: async () => {
            expect(routeProviderFor({
              reach: everyReach,
              dry: allWet,
              saturated: {
                ...noneSaturated,
                synthetic: true,
              },
            },),).toEqual({ kind: 'bedrock', },);
          },
        },),

        it({
          name: 'SWITCHES TO BEDROCK when Synthetic is dry, whether or not its concurrency had room, '
            + 'since budget outranks saturation, and to Hyper when Bedrock is dry too',
          fn: async () => {
            for (const synthetic of [true, false,]) {
              expect(routeProviderFor({
                reach: everyReach,
                dry: {
                  ...allWet,
                  synthetic: true,
                },
                saturated: {
                  ...noneSaturated,
                  synthetic,
                },
              },),).toEqual({ kind: 'bedrock', },);
              expect(routeProviderFor({
                reach: everyReach,
                dry: {
                  ...allWet,
                  synthetic: true,
                  bedrock: true,
                },
                saturated: {
                  ...noneSaturated,
                  synthetic,
                },
              },),).toEqual({ kind: 'hyper', },);
            }
          },
        },),

        it({
          name: 'FALLS THROUGH TO OPENROUTER when Synthetic and Hyper are both dry, the owner\'s order '
            + 'of 2026-09-03: the paid provider is last, and it is where a call goes when the '
            + 'subscription and the balance are both out',
          fn: async () => {
            expect(routeProviderFor({
              reach: everyReach,
              dry: {
                neuralwatt: false,
                synthetic: true,
                hyper: true,
                bedrock: true,
                deepseek: false,
                xiaomi: false,
                anthropic: false,
                openrouter: false,
              },
              saturated: noneSaturated,
            },),).toEqual({ kind: 'openrouter', },);
          },
        },),

        it({
          name: 'KEEPS SENDING TO SYNTHETIC PAST SATURATION when nobody behind it is usable, because a '
            + 'queue behind the per-model limit is slower than the split and still buys the answer',
          fn: async () => {
            expect(routeProviderFor({
              reach: everyReach,
              dry: {
                neuralwatt: false,
                synthetic: false,
                hyper: true,
                bedrock: true,
                deepseek: true,
                xiaomi: true,
                anthropic: true,
                openrouter: true,
              },
              saturated: {
                ...noneSaturated,
                synthetic: true,
              },
            },),).toEqual({ kind: 'synthetic', },);
          },
        },),

        it({
          name: 'OVERFLOWS PAST A DRY HYPER TO OPENROUTER when Synthetic is saturated, since the walk '
            + 'prefers the first usable provider with a free slot wherever it sits in the order',
          fn: async () => {
            expect(routeProviderFor({
              reach: everyReach,
              dry: {
                neuralwatt: false,
                synthetic: false,
                hyper: true,
                bedrock: true,
                deepseek: false,
                xiaomi: false,
                anthropic: false,
                openrouter: false,
              },
              saturated: {
                ...noneSaturated,
                synthetic: true,
              },
            },),).toEqual({ kind: 'openrouter', },);
          },
        },),

        it({
          name: 'THROWS when every budget is empty at once, which ends the run at the owner instruction',
          fn: async () => {
            expect(() => {
              routeProviderFor({
                reach: everyReach,
                dry: {
                  neuralwatt: true,
                  synthetic: true,
                  hyper: true,
                  bedrock: true,
                  deepseek: true,
                  xiaomi: true,
                  anthropic: true,
                  openrouter: true,
                },
                saturated: noneSaturated,
              },);
            },).toThrow(EveryProviderDryError,);
          },
        },),

        it({
          name: 'THROWS on every budget empty even for a model no provider serves, since the '
            + 'run is over either way and the budget is the larger fact',
          fn: async () => {
            expect(() => {
              routeProviderFor({
                reach: {
                  neuralwatt: false,
                  synthetic: false,
                  hyper: false,
                  bedrock: false,
                  deepseek: false,
                  xiaomi: false,
                  anthropic: false,
                  openrouter: false,
                },
                dry: {
                  neuralwatt: true,
                  synthetic: true,
                  hyper: true,
                  bedrock: true,
                  deepseek: true,
                  xiaomi: true,
                  anthropic: true,
                  openrouter: true,
                },
                saturated: noneSaturated,
              },);
            },).toThrow(EveryProviderDryError,);
          },
        },),

        it({
          name: 'KEEPS a model with one provider on that provider past saturation, since there is '
            + 'no counterpart elsewhere to overflow to',
          fn: async () => {
            expect(routeProviderFor({
              reach: {
                neuralwatt: false,
                synthetic: true,
                hyper: false,
                bedrock: false,
                deepseek: false,
                xiaomi: false,
                anthropic: false,
                openrouter: false,
              },
              dry: allWet,
              saturated: {
                ...noneSaturated,
                synthetic: true,
              },
            },),).toEqual({ kind: 'synthetic', },);
          },
        },),

        it({
          name: 'SENDS a model Synthetic does not serve to the next provider that does, Bedrock then Hyper, '
            + 'regardless of what Synthetic is doing',
          fn: async () => {
            expect(routeProviderFor({
              reach: {
                neuralwatt: false,
                synthetic: false,
                hyper: true,
                bedrock: true,
                deepseek: false,
                xiaomi: false,
                anthropic: false,
                openrouter: true,
              },
              dry: allWet,
              saturated: noneSaturated,
            },),).toEqual({ kind: 'bedrock', },);
            expect(routeProviderFor({
              reach: {
                neuralwatt: false,
                synthetic: false,
                hyper: true,
                bedrock: false,
                deepseek: false,
                xiaomi: false,
                anthropic: false,
                openrouter: true,
              },
              dry: allWet,
              saturated: noneSaturated,
            },),).toEqual({ kind: 'hyper', },);
          },
        },),

        it({
          name: 'REPORTS a model as unreachable rather than throwing when every provider serving it '
            + 'is dry while another is wet, because that costs one panelist its voice and the run goes on',
          fn: async () => {
            expect(routeProviderFor({
              reach: {
                neuralwatt: false,
                synthetic: true,
                hyper: false,
                bedrock: false,
                deepseek: false,
                xiaomi: false,
                anthropic: false,
                openrouter: false,
              },
              dry: {
                ...allWet,
                synthetic: true,
              },
              saturated: noneSaturated,
            },).kind,).toBe('unreachable',);
          },
        },),

        it({
          name: 'SEPARATES a model no provider serves from one whose providers are merely dry, since '
            + 'the first is a roster mistake and the second is a budget state',
          fn: async () => {
            expect(routeProviderFor({
              reach: {
                neuralwatt: false,
                synthetic: false,
                hyper: false,
                bedrock: false,
                deepseek: false,
                xiaomi: false,
                anthropic: false,
                openrouter: false,
              },
              dry: allWet,
              saturated: noneSaturated,
            },),).toEqual({
              kind: 'unreachable',
              reason: 'no provider serves this model',
            },);

            expect(routeProviderFor({
              reach: {
                neuralwatt: false,
                synthetic: true,
                hyper: false,
                bedrock: false,
                deepseek: false,
                xiaomi: false,
                anthropic: false,
                openrouter: false,
              },
              dry: {
                ...allWet,
                synthetic: true,
              },
              saturated: noneSaturated,
            },),).toEqual({
              kind: 'unreachable',
              reason: 'every provider serving this model is out of budget (synthetic dry); the providers that do not '
                + 'serve it read neuralwatt wet, bedrock wet, deepseek wet, xiaomi wet, '
                + 'anthropic wet, hyper wet, openrouter wet',
            },);
          },
        },),

        it({
          name: 'DOES NOT END THE RUN for a model Xiaomi does not serve when every other provider reads dry and '
            + 'Xiaomi reads wet, because only every provider dry ends it and this model alone has lost its '
            + 'providers',
          fn: async () => {
            expect(routeProviderFor({
              reach: everyReach,
              dry: {
                ...allDry,
                xiaomi: false,
              },
              saturated: noneSaturated,
            },).kind,).toBe('unreachable',);
          },
        },),

        it({
          name: 'NAMES THE PROVIDERS THAT SERVE THE DARK MODEL AND HOW EACH READS, and how the ones that do not serve '
            + 'it read, so a read of the run tells a lost seat from the run-ending stop',
          fn: async () => {
            expect(routeProviderFor({
              reach: {
                ...everyReach,
                synthetic: false,
                bedrock: false,
                hyper: true,
                deepseek: true,
              },
              dry: {
                ...allWet,
                hyper: true,
                deepseek: true,
                openrouter: true,
              },
              saturated: noneSaturated,
            },),).toEqual({
              kind: 'unreachable',
              reason: 'every provider serving this model is out of budget (deepseek dry, hyper dry, openrouter dry); '
                + 'the providers that do not serve it read neuralwatt wet, synthetic wet, '
                + 'bedrock wet, xiaomi wet, anthropic wet',
            },);
          },
        },),

        it({
          name: 'REPORTS a Hyper-only model as unreachable while Hyper is dry and NeuralWatt and DeepSeek read wet, '
            + 'since the providers that serve the other seats are not this seat\'s',
          fn: async () => {
            expect(routeProviderFor({
              reach: {
                neuralwatt: false,
                synthetic: false,
                hyper: true,
                bedrock: false,
                deepseek: false,
                xiaomi: false,
                anthropic: false,
                openrouter: false,
              },
              dry: {
                ...allWet,
                hyper: true,
              },
              saturated: noneSaturated,
            },).kind,).toBe('unreachable',);
          },
        },),
      ],
    },),

    describe({
      name: syntheticMeterLevel.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'names both limits, including the one with room left',
          fn: async () => {
            expect(syntheticMeterLevel({ quota: roomyQuota, },),).toEqual([
              'syntheticWeekly=99.8%',
              'syntheticFiveHour=750/750',
              'syntheticThrottled=no',
            ],);
          },
        },),

        it({
          name: 'NAMES A MISSING FIVE-HOUR WINDOW as `none`, in both fields that read it, rather than dropping the fields or '
            + 'claiming the account is not throttled',
          fn: async () => {
            expect(syntheticMeterLevel({ quota: weeklyOnlyQuota, },),).toEqual([
              'syntheticWeekly=99.8%',
              'syntheticFiveHour=none',
              'syntheticThrottled=none',
            ],);
          },
        },),

        it({
          name: 'STATES THE WEEKLY RETURN when the weekly budget is spent, and only then, so a dry reading names when it ends',
          fn: async () => {
            expect(syntheticMeterLevel({
              quota: {
                weekly: {
                  percentRemaining: 0,
                  nextRegenAt: '2026-10-23T05:03:47.000Z',
                },
              },
            },),).toEqual([
              'syntheticWeekly=0%',
              'syntheticFiveHour=none',
              'syntheticThrottled=none',
              'syntheticWeeklyReturn=2026-10-23T05:03:47.000Z',
            ],);
            expect(syntheticMeterLevel({
              quota: {
                ...roomyQuota,
                weekly: {
                  percentRemaining: 0,
                  nextRegenAt: '2026-08-25T00:12:58.000Z',
                },
              },
            },),).toEqual([
              'syntheticWeekly=0%',
              'syntheticFiveHour=750/750',
              'syntheticThrottled=no',
              'syntheticWeeklyReturn=2026-08-25T00:12:58.000Z',
            ],);
          },
        },),

        it({
          name: 'PRINTS ONLY A VALIDATED INSTANT as the weekly return: a value with a space and a newline, an empty one and a '
            + 'spelling the logger does not write read `unreadable`, so no snapshot can forge a field or a line',
          fn: async () => {
            /**
             The return field printed for each weekly return a snapshot might carry.
             */
            const returned = [
              'a synthetic=wet\nINJECT',
              '',
              '2026-10-23T05:03:47Z',
              '2026-10-23T05:03:47.000Z',
            ].map(function returnFieldFor(nextRegenAt,): string {
              return syntheticMeterLevel({
                quota: { weekly: { percentRemaining: 0, nextRegenAt, }, },
              },)
                .find(function isReturn(field,): boolean {
                  return field.startsWith('syntheticWeeklyReturn',);
                },) ?? 'no return field';
            },);
            expect(returned,).toEqual([
              'syntheticWeeklyReturn=unreadable',
              'syntheticWeeklyReturn=unreadable',
              'syntheticWeeklyReturn=unreadable',
              'syntheticWeeklyReturn=2026-10-23T05:03:47.000Z',
            ],);
            // A window the parser marked unreadable carries no `nextRegenAt` at all.
            expect(syntheticMeterLevel({ quota: { weekly: { percentRemaining: 0, }, }, },),).toEqual([
              'syntheticWeekly=0%',
              'syntheticFiveHour=none',
              'syntheticThrottled=none',
              'syntheticWeeklyReturn=unreadable',
            ],);
          },
        },),

        it({
          name: 'separates active throttling from an emptied budget, which route the same way',
          fn: async () => {
            /**
             A reading whose window is full and whose account is being throttled,
             which `syntheticIsDry` calls dry for a reason the state cannot show.
             */
            const throttled = syntheticMeterLevel({
              quota: {
                ...roomyQuota,
                fiveHour: {
                  ...roomyQuota.fiveHour,
                  limited: true,
                },
              },
            },);

            expect(syntheticIsDry({
              quota: {
                ...roomyQuota,
                fiveHour: {
                  ...roomyQuota.fiveHour,
                  limited: true,
                },
              },
            },),).toBe(true,);
            expect(throttled,).toEqual([
              'syntheticWeekly=99.8%',
              'syntheticFiveHour=750/750',
              'syntheticThrottled=yes',
            ],);
          },
        },),
      ],
    },),

    describe({
      name: hyperMeterLevel.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'names the one number this provider reports',
          fn: async () => {
            expect(hyperMeterLevel({ credits: { balance: 0, }, },),).toEqual(['hyperBalance=0',],);
          },
        },),
      ],
    },),

    describe({
      name: 'the all-dry refusal names every provider (ledger P12)',
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'STATES THE TIME OF DAY THE ACCOUNT\'S NEURALWATT DAY STARTS AT, in UTC, when the run decided one, '
            + 'and never midnight: the owner ruled the day is "not 00:00 UTC"',
          fn: async () => {
            /**
             The message for an account whose day starts at 19:15:08 UTC, the XingZ6018 account.
             */
            const { message, } = new EveryProviderDryError({
              neuralwatt: {
                kind: 'at',
                msOfDay: 69_308_000,
              },
            },);
            expect(message,).toContain(
              'NeuralWatt\'s free energy returns when its day starts again, at 19:15:08 UTC, the day start this run '
                + 'decided or learned;',
            );
            expect(message,).not.toContain('00:00 UTC',);
          },
        },),

        it({
          name: 'KEEPS THE MILLISECONDS of a day start that does not fall on a whole second, which only a setting or a '
            + 'quota reply with a fraction can give',
          fn: async () => {
            /**
             The message for a day that starts half a second into 19:15:08 UTC.
             */
            const { message, } = new EveryProviderDryError({
              neuralwatt: {
                kind: 'at',
                msOfDay: 69_308_500,
              },
            },);
            expect(message,).toContain('at 19:15:08.500 UTC,',);
          },
        },),

        it({
          name: 'SAYS NEURALWATT IS OFF, naming no time of day, when the run has no NeuralWatt client',
          fn: async () => {
            /**
             The message for a run with NeuralWatt left off.
             */
            const { message, } = new EveryProviderDryError({ neuralwatt: { kind: 'off', }, },);
            expect(message,).toContain('NeuralWatt is off for this run, so it returns no energy;',);
            expect(message,).not.toContain('UTC',);
            // THE OPENING NAMES NO NEURALWATT ENERGY EITHER: an off provider has none to be out of.
            expect(message,).not.toContain('has no free daily energy',);
            expect(message.startsWith('Every provider is out of budget at once: Synthetic has no weekly credit left',),).toBe(true,);
          },
        },),

        it({
          name: 'SAYS NO DAY START WAS DECIDED, naming no time of day, when NeuralWatt is configured but its quota never '
            + 'gave one',
          fn: async () => {
            /**
             The message for a run whose NeuralWatt day start stayed unknown.
             */
            const { message, } = new EveryProviderDryError({ neuralwatt: { kind: 'unanchored', }, },);
            expect(message,).toContain(
              'NeuralWatt\'s day start was not decided for this run, so no time of day can be stated for its return;',
            );
            expect(message,).not.toContain('UTC',);
            // THE OPENING DOES NOT CLAIM THE ENERGY IS SPENT: the keeper refuses NeuralWatt until a day start is
            // learned, and the energy may well be there.
            expect(message,).not.toContain('has no free daily energy',);
            expect(message,).toContain(
              'Every provider is out of budget at once: NeuralWatt is refused until its day start is learned, Synthetic',
            );
          },
        },),

        it({
          name: 'HANDS THE ROUTER\'S NEURALWATT DAY TO THE REFUSAL it throws on an all-dry reading',
          fn: async () => {
            /**
             What the router threw on an all-dry reading with a decided day start.
             */
            const thrown = (function refusal(): unknown {
              try {
                routeProviderFor({
                  reach: {
                    neuralwatt: true,
                    synthetic: true,
                    hyper: true,
                    bedrock: true,
                    deepseek: true,
                    xiaomi: true,
                    anthropic: true,
                    openrouter: true,
                  },
                  dry: {
                    neuralwatt: true,
                    synthetic: true,
                    hyper: true,
                    bedrock: true,
                    deepseek: true,
                    xiaomi: true,
                    anthropic: true,
                    openrouter: true,
                  },
                  saturated: {
                    neuralwatt: false,
                    synthetic: false,
                    hyper: false,
                    bedrock: false,
                    deepseek: false,
                    xiaomi: false,
                    anthropic: false,
                    openrouter: false,
                  },
                  neuralwatt: {
                    kind: 'at',
                    msOfDay: 69_308_000,
                  },
                },);
                return undefined;
              }
              catch (error) {
                return error;
              }
            })();
            if (!(thrown instanceof EveryProviderDryError))
              throw new Error('an all-dry refusal by construction',);
            expect(thrown.message,).toContain('at 19:15:08 UTC',);
          },
        },),

        it({
          name: 'STATES SYNTHETIC\'S LIMIT AS THE WEEKLY CREDIT, with the five-hour window named only where a reply still carries one',
          fn: async () => {
            expect(new EveryProviderDryError().message,).toContain(
              'Synthetic has no weekly credit left (or no five-hour request left, on a reply that still carries that window), '
                + 'Charm Hyper has no balance left',
            );
          },
        },),

        it({
          name: 'LISTS BEDROCK AMONG THE PROVIDERS OUT OF BUDGET AND SAYS ITS CREDIT NEVER REFILLS, and cites the reading '
            + 'the router decided on, since '
            + 'the refusal named three providers, said they refill, and carried "no reading cited" from the router',
          fn: async () => {
            /**
             What the router threw on an all-dry reading.
             */
            const thrown = (function refusal(): unknown {
              try {
                routeProviderFor({
                  reach: {
                    neuralwatt: false,
                    synthetic: true,
                    hyper: true,
                    bedrock: true,
                    deepseek: false,
                    xiaomi: false,
                    anthropic: false,
                    openrouter: true,
                  },
                  dry: {
                    neuralwatt: true,
                    synthetic: true,
                    hyper: true,
                    bedrock: true,
                    deepseek: true,
                    xiaomi: true,
                    anthropic: true,
                    openrouter: true,
                  },
                  saturated: {
                    neuralwatt: false,
                    synthetic: false,
                    hyper: false,
                    bedrock: false,
                    deepseek: false,
                    xiaomi: false,
                    anthropic: false,
                    openrouter: false,
                  },
                },);
                return undefined;
              }
              catch (error) {
                return error;
              }
            })();
            if (!(thrown instanceof EveryProviderDryError))
              throw new Error('an all-dry refusal by construction',);

            /**
             The message cut at the sentence after the out-of-budget list, so the
             list is read apart from the sentences that name Bedrock again for its
             refill: a check over the whole message passed with Bedrock gone from
             the list.
             */
            const parts = thrown.message.split('Nothing further can be bought',);

            /**
             The list itself; the whole message only if the split found nothing,
             which the assertion's `parts` count refuses.
             */
            const [outOfBudget = '',] = parts;
            expect({
              // THE SPLIT MUST FIND ITS SENTENCE, or the "list" is the whole
              // message and the refill sentence's "Bedrock" passes it again.
              parts: parts.length,
              unlisted: ['Synthetic', 'Hyper', 'Bedrock', 'OpenRouter',].filter(function missingFromList(name,): boolean {
                return !outOfBudget.includes(name,);
              },),
              neverRefills: thrown.message.includes('never topped up',),
              cited: !thrown.message.includes('no reading cited',),
              reading: thrown.message.includes('bedrock dry',),
            },).toEqual({
              parts: 2,
              unlisted: [],
              neverRefills: true,
              cited: true,
              reading: true,
            },);
          },
        },),
      ],
    },),

    describe({
      name: 'the all-dry refusal states why NeuralWatt reads dry',
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'STATES THE DAY START AND THE DISTANCE for a spent day, and the key\'s own time for a blocked key, '
            + 'and neither a day start nor a return for energy held by calls in flight: only the true cause',
          fn: async () => {
            /**
             The message for one cause of NeuralWatt's dryness.
             */
            const messageFor = (neuralwattDryness: NeuralWattDryness,): string => new EveryProviderDryError({
              measured: 'meters read every provider dry',
              neuralwatt: { kind: 'at', msOfDay: 69_308_000, },
              neuralwattDryness,
            },).message;

            const spent = messageFor({
              kind: 'spent-day',
              dayStart: '2026-10-23T19:15:08.000Z',
              returnsInMs: 301_000,
            },);
            expect(spent,).toContain('at 19:15:08 UTC (in 5 min 1 s)',);

            const blocked = messageFor({ kind: 'key-blocked', until: '2026-10-23T05:00:00.000Z', },);
            expect(blocked,).toContain('blocked until 2026-10-23T05:00:00.000Z',);
            expect(blocked,).not.toContain('day starts again',);
            expect(blocked,).not.toContain('19:15:08',);

            const reserved = messageFor({ kind: 'reserved', },);
            expect(reserved,).toContain('held by calls in flight',);
            expect(reserved,).not.toContain('day starts again',);
            expect(reserved,).not.toContain('19:15:08',);
          },
        },),
      ],
    },),

    describe({
      name: 'the all-dry refusal says why Xiaomi reads dry',
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'SAYS A PAYMENT REFUSAL when Xiaomi was refused for payment, and reads it dry in the measured clause',
          fn: async () => {
            const message = allDryMessageOf({ dry: allDry, },);
            expect(message,).toContain('Xiaomi has refused a call for payment',);
            expect(message,).toContain('xiaomi dry',);
          },
        },),

        it({
          name: 'SAYS XIAOMI READS WET BUT SERVES NO SEAT THAT WRITES when it is the one provider not dry and only '
            + 'checker seats are Xiaomi\'s, and ends the run even for a call Xiaomi would serve: nothing that writes '
            + 'a page can be bought, so the run cannot go on; with a writer seat on Xiaomi it routes there instead',
          fn: async () => {
            /**
             Every provider dry but Xiaomi, and a model only Xiaomi serves.
             */
            const reach = {
              ...allWet,
              xiaomi: true,
            };
            /**
             What the router raises when Xiaomi serves nothing that writes.
             */
            const message = ((): string => {
              try {
                routeProviderFor({
                  reach,
                  dry: { ...allDry, xiaomi: false, },
                  saturated: noneSaturated,
                  writing: { ...allDry, xiaomi: false, },
                },);
              } catch (error) {
                return (error as Error).message;
              }
              return 'no refusal';
            })();
            expect(message,).toContain('Xiaomi reads wet but serves no seat that writes',);
            expect(message,).toContain('xiaomi wet (serves no seat that writes)',);
            expect(routeProviderFor({
              reach,
              dry: { ...allDry, xiaomi: false, },
              saturated: noneSaturated,
              writing: allDry,
            },),).toEqual({ kind: 'xiaomi', },);
          },
        },),
      ],
    },),
  ],
},);
