import { tagged, } from '@monochromatic-dev/module-logger/ts';

import { contextRoot, } from '@monochromatic-dev/translation-repair-typography/ts';
import type { BedrockClient, } from './bedrock-client.ts';
import {
  bedrockIsDry,
  bedrockMeterLevel,
  deepSeekIsDry,
  deepSeekMeterLevel,
  hyperIsDry,
  hyperMeterLevel,
  openRouterIsDry,
  openRouterMeterLevel,
  syntheticIsDry,
  syntheticMeterLevel,
  xiaomiIsDry,
  xiaomiMeterLevel,
} from './budget-routing.ts';
import type { ModelClient, } from './chat-contract.ts';
import type { DeepSeekClient, } from './deepseek-client.ts';
import { exchangeFailureLogText, } from '@monochromatic-dev/translation-repair-provider-stream/ts';
import type { HyperClient, } from './hyper-client.ts';
import type { XiaomiClient, } from './xiaomi-client.ts';
import type { NeuralWattClient, } from './neuralwatt-client.ts';
import {
  neuralWattDrynessOf,
  type NeuralWattDryness,
} from './neuralwatt-return-hold.ts';
import {
  neuralWattIsDry,
  neuralWattMeterLevel,
} from './neuralwatt-meter.ts';
import type { OpenRouterClient, } from './openrouter-client.ts';
import type {
  ProviderName,
  ProviderRecord,
} from '@monochromatic-dev/translation-repair-shared-contract/ts';

//region Provider meters
// WHAT EACH PROVIDER'S METER SAID, read once for everyone: the three states a
// reading can be in, the record one answers with, and the read of every
// meter together. Split out of `provider-budget.ts` on 2026-09-07 when the
// fourth provider put that file over the line budget; the budget layer keeps
// the cache, the holds and the routing view, and asks this file for the
// numbers.

/**
 Logger root for the meter readers.
 */
const l = contextRoot({ tag: 'translation-repair', },);

/**
 What one provider's meter said, keeping a meter that could not be read
 distinct from one that answered.

 THREE STATES RATHER THAN A BOOLEAN, because routing and measurement want
 different things out of the same read. Routing needs one bit, spend here or
 do not, and an unreachable meter has to fall on the spendable side of it for
 the reason `drynessOf` was written with. Measurement needs to know that the
 bit was a guess: a duty cycle counting an unreadable meter as an available
 provider reports an outage as uptime, which is backwards for the one number
 it exists to produce.

 @internal
 */
export type MeterState = 'wet' | 'dry' | 'unreadable';

/**
 Whether a meter state stops us spending on that provider.

 ONLY A METER THAT ANSWERED AND SAID DRY holds a provider out, so this file's
 routing policy is unchanged by the third state existing.

 @param state - what the meter said, or that it said nothing

 @returns Whether the router should treat this provider as out of budget

 @example
 ```ts
 const dry = routesAsDry({ state: 'unreadable', },);
 // => false
 ```

 @internal
 */
export function routesAsDry(
  { state, }: { readonly state: MeterState; },
): boolean {
  return state === 'dry';
}

/**
 What one meter answered: the verdict, and the numbers it was drawn from.

 BOTH COME OUT OF ONE READ so they cannot disagree. A verdict rendered from
 one snapshot beside a level rendered from a later one would record a moment
 that never happened, which is worse evidence than recording no level at all.

 @internal
 */
export type MeterLevel = {
  /**
   Whether this reading holds the provider out of spending.
   */
  readonly dry: boolean;

  /**
   `key=value` tokens naming what was read, no value carrying a space.
   */
  readonly fields: readonly string[];

  /**
   Why a dry reading is dry, and for NeuralWatt's spent day when it is over (`neuralWattDrynessOf`). Absent for every
   other provider's reading.
   */
  readonly dryness?: NeuralWattDryness;
};

/**
 One meter as the availability record should carry it.

 @internal
 */
export type MeterRecord = {
  /**
   What the meter said, or that it said nothing.
   */
  readonly state: MeterState;

  /**
   Numbers behind the state, in the order they should be written.

   EMPTY IS NOT AN ABSENCE SENTINEL HERE. A meter that did not answer has no
   numbers to report, and `state` already carries the fact that it did not,
   so nothing is being encoded twice and nothing is lost.
   */
  readonly fields: readonly string[];

  /**
   Why a dry reading is dry, when the meter said so (`MeterLevel.dryness`).
   */
  readonly dryness?: NeuralWattDryness;
};

/**
 Reads one provider's meter, naming an unreachable meter rather than
 flattening it into the answer a working meter would have given.

 @param name - provider being read, for the log line

 @param readLevel - meter read, which may reject

 @param signal - signal the read honours, whose own reason is the caller's
 stop rather than a meter that could not be read

 @returns What that meter said and was reading, or that it could not be read,
 a read the caller stopped included

 @example
 ```ts
 const meter = await meterRecordOf({ name: 'hyper', readLevel, signal, },);
 ```

 @internal
 */
export async function meterRecordOf(
  {
    name,
    readLevel,
    signal,
  }: {
    readonly name: ProviderName;
    readonly readLevel: () => Promise<MeterLevel>;
    readonly signal: AbortSignal;
  },
): Promise<MeterRecord> {
  /**
   Logger pre-tagged with this function's name.
   */
  const rl = tagged({
    tag: meterRecordOf.name,
    l,
  },);

  try {
    /**
     Verdict and numbers, both off the same read.
     */
    const level = await readLevel();

    return {
      state: level.dry
        ? 'dry'
        : 'wet',
      fields: level.fields,
      // Conditional spread keeps an absent cause absent instead of undefined.
      ...((level.dryness === undefined) ? {} : { dryness: level.dryness, }),
    };
  } catch (error) {
    // A monitoring failure must not become an outage: the router's failover
    // still recovers a real refusal, and a false dry stops calls that work.
    //
    // THE CALLER'S OWN STOP IS NAMED AS ONE, not as a meter that could not
    // be read: the meter said nothing because nobody waited for it (ledger
    // B403 left this open). It still records `unreadable`, since the reading
    // is shared and its first caller's stop resolves it for every sharer
    // (`takeReading` in `provider-budget.ts`).
    if (signal.aborted && (error === signal.reason)) {
      rl.warn(`${name}: budget not read, since the caller stopped the reading; treating as spendable`,);
      return {
        state: 'unreadable',
        fields: [],
      };
    }
    // Rendered as a stage's lost voice is, so a refusal keeps its status and
    // the provider's words, the key masked where the transport built it.
    rl.warn(
      `${name}: budget unreadable, treating as spendable (${
        exchangeFailureLogText({ error, },)
      })`,
    );
    return {
      state: 'unreadable',
      fields: [],
    };
  }
}

/**
 Meter of a provider that was never configured: dry, with nothing to report.

 DRY RATHER THAN UNREADABLE, because absence is known rather than failed: the
 router must never send a call to a provider with no key, and a seat reader
 must count that provider as unable to serve.
 */
export const UNCONFIGURED_METER: MeterRecord = {
  state: 'dry',
  fields: [],
};

/**
 Reads every configured provider's meter together, so one slow endpoint
 does not serialise behind another, and names a provider that was never
 configured as dry with nothing to report.

 @param neuralwatt - first provider's energy reader, absent when unconfigured

 @param synthetic - second provider's quota reader, absent when unconfigured

 @param hyper - third provider's balance reader, absent when unconfigured

 @param bedrock - fourth-added provider's ledger reader, absent when unconfigured

 @param deepseek - DeepSeek's balance reader, absent when unconfigured

 @param xiaomi - Xiaomi's spend reader, absent when unconfigured

 @param openrouter - fourth provider's credits reader, absent when unconfigured

 @param wall - wall clock and the margin a spent NeuralWatt day's return carries, absent
 where the cause is not asked for

 @param signal - abort signal of whichever call started this reading

 @returns Every provider's meter record, keyed by name

 @example
 ```ts
 const meters = await readEveryMeter({ neuralwatt, synthetic, hyper, bedrock, deepseek, xiaomi, openrouter, signal, },);
 ```
 */
export async function readEveryMeter(
  {
    neuralwatt,
    synthetic,
    hyper,
    bedrock,
    deepseek,
    xiaomi,
    openrouter,
    wall,
    signal,
  }: {
    readonly neuralwatt?: Pick<NeuralWattClient, 'energy'>;
    readonly synthetic?: Pick<ModelClient, 'quotas'>;
    readonly hyper?: Pick<HyperClient, 'credits'>;
    readonly bedrock?: Pick<BedrockClient, 'credits'>;
    readonly deepseek?: Pick<DeepSeekClient, 'credits'>;
    readonly xiaomi?: Pick<XiaomiClient, 'spend'>;
    readonly openrouter?: Pick<OpenRouterClient, 'credits'>;
    readonly wall?: {
      readonly now: () => Date;
      readonly marginMs: number
    };
    readonly signal: AbortSignal;
  },
): Promise<ProviderRecord<MeterRecord>> {
  /**
   Every meter, read together so one slow endpoint does not serialise
   behind another.
   */
  const [
    neuralwattMeter,
    syntheticMeter,
    hyperMeter,
    bedrockMeter,
    deepseekMeter,
    xiaomiMeter,
    openrouterMeter,
  ] = await Promise.all([
    (neuralwatt === undefined)
      ? Promise.resolve(UNCONFIGURED_METER,)
      : meterRecordOf({
        name: 'neuralwatt',
        signal,
        readLevel: async function readEnergy(): Promise<MeterLevel> {
          /**
           Ledger reading the verdict and the numbers are both drawn from.
           */
          const energy = await neuralwatt.energy({ signal, },);
          /**
           Why the reading is dry, and for a spent day how long until it starts again.
           */
          const dryness = (wall === undefined)
            ? undefined
            : neuralWattDrynessOf({
              energy,
              now: wall.now(),
              marginMs: wall.marginMs,
            },);

          return {
            dry: neuralWattIsDry({ energy, },),
            fields: neuralWattMeterLevel({ energy, },),
            // Conditional spread keeps an absent cause absent instead of undefined.
            ...((dryness === undefined) ? {} : { dryness, }),
          };
        },
      },),
    (synthetic === undefined)
      ? Promise.resolve(UNCONFIGURED_METER,)
      : meterRecordOf({
        name: 'synthetic',
        signal,
        readLevel: async function readQuota(): Promise<MeterLevel> {
          /**
           Snapshot the verdict and the numbers are both drawn from.
           */
          const quota = await synthetic.quotas({ signal, },);

          return {
            dry: syntheticIsDry({ quota, },),
            fields: syntheticMeterLevel({ quota, },),
          };
        },
      },),
    (hyper === undefined)
      ? Promise.resolve(UNCONFIGURED_METER,)
      : meterRecordOf({
        name: 'hyper',
        signal,
        readLevel: async function readCredits(): Promise<MeterLevel> {
          /**
           Balance the verdict and the number are both drawn from.
           */
          const credits = await hyper.credits({ signal, },);

          return {
            dry: hyperIsDry({ credits, },),
            fields: hyperMeterLevel({ credits, },),
          };
        },
      },),
    (bedrock === undefined)
      ? Promise.resolve(UNCONFIGURED_METER,)
      : meterRecordOf({
        name: 'bedrock',
        signal,
        readLevel: async function readBedrockCredits(): Promise<MeterLevel> {
          /**
           Ledger reading the verdict and the number are both drawn from.
           */
          const credits = await bedrock.credits({ signal, },);

          return {
            dry: bedrockIsDry({ credits, },),
            fields: bedrockMeterLevel({ credits, },),
          };
        },
      },),
    (deepseek === undefined)
      ? Promise.resolve(UNCONFIGURED_METER,)
      : meterRecordOf({
        name: 'deepseek',
        signal,
        readLevel: async function readDeepSeekBalance(): Promise<MeterLevel> {
          /**
           Balance the verdict and the number are both drawn from.
           */
          const balance = await deepseek.credits({ signal, },);

          return {
            dry: deepSeekIsDry({ balance, },),
            fields: deepSeekMeterLevel({ balance, },),
          };
        },
      },),
    (xiaomi === undefined)
      ? Promise.resolve(UNCONFIGURED_METER,)
      : meterRecordOf({
        name: 'xiaomi',
        signal,
        readLevel: async function readXiaomiSpend(): Promise<MeterLevel> {
          /**
           Spend the (always wet) verdict and the number are both drawn from.
           */
          const spend = await xiaomi.spend({ signal, },);

          return {
            dry: xiaomiIsDry({ spend, },),
            fields: xiaomiMeterLevel({ spend, },),
          };
        },
      },),
    (openrouter === undefined)
      ? Promise.resolve(UNCONFIGURED_METER,)
      : meterRecordOf({
        name: 'openrouter',
        signal,
        readLevel: async function readOpenRouterCredits(): Promise<MeterLevel> {
          /**
           Credits the verdict and the number are both drawn from.
           */
          const credits = await openrouter.credits({ signal, },);

          return {
            dry: openRouterIsDry({ credits, },),
            fields: openRouterMeterLevel({ credits, },),
          };
        },
      },),
  ],);

  return {
    neuralwatt: neuralwattMeter,
    synthetic: syntheticMeter,
    hyper: hyperMeter,
    bedrock: bedrockMeter,
    deepseek: deepseekMeter,
    xiaomi: xiaomiMeter,
    openrouter: openrouterMeter,
  };
}

//endregion Provider meters
