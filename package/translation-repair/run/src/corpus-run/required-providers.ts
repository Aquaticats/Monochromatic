import { tagged, } from '@monochromatic-dev/logger-core/ts';
import { runLogger, } from '@monochromatic-dev/translation-repair-typography/ts';

import { allInInputOrder, } from '@monochromatic-dev/translation-repair-document-structure/ts';
import {
  bedrockIsDry,
  bedrockLedgerFromEnv,
  createBedrockClient,
  createHyperClient,
  createNeuralWattClient,
  createOpenRouterClient,
  createSyntheticClient,
  hyperIsDry,
  neuralWattIsDry,
  neuralWattDayStartFrom,
  neuralWattLedgerFromEnv,
  openRouterIsDry,
  syntheticIsDry,
} from '@monochromatic-dev/translation-repair-provider/ts';
import {
  exchangeFailureLogText,
  type ModelTransport,
} from '@monochromatic-dev/translation-repair-provider-stream/ts';
import { isProviderName, } from '@monochromatic-dev/translation-repair-shared-contract/ts';
import { StatedRefusalError, } from '@monochromatic-dev/translation-repair-glossary/ts';
import {
  isProviderOff,
  type ProviderOff,
  PROVIDERS_LEFT_OFF,
} from './providers-off.ts';
import { idListFlag, } from './command-flags.ts';
import {
  isRequirable,
  REQUIRABLE_PROVIDERS,
  type RequiredProvider,
} from './requirable-providers.ts';
import type { ReadsFlag, } from './command-line-types.ts';

//region Required providers for measured arms

/**
 CLI token selecting required provider set.
 */
const REQUIRED_PROVIDERS_FLAG = '--require-providers';

/**
 Environment variable carrying each provider's key.
 */
const KEY_VARIABLES: Readonly<Record<RequiredProvider, string>> = {
  neuralwatt: 'TRANSLATION_REPAIR_NEURALWATT_API_KEY',
  synthetic: 'TRANSLATION_REPAIR_SYNTHETIC_API_KEY',
  hyper: 'TRANSLATION_REPAIR_CHARM_HYPER_API_KEY',
  bedrock: 'TRANSLATION_REPAIR_AMAZON_BEDROCK_API_KEY',
  openrouter: 'TRANSLATION_REPAIR_OPENROUTER_API_KEY',
};

/**
 Raised before model calls when measured arm provider requirement is not wet.

 @example
 ```ts
 throw new RequiredProviderError({ provider: 'hyper', reason: 'key missing', });
 ```
 */
export class RequiredProviderError extends StatedRefusalError {
  /**
   Declares message safe because provider and reason are closed vocabulary.
   */
  override readonly messageNamesOnly: true = true;

  /**
   Constructs provider requirement refusal.

   @param provider - required provider

   @param reason - closed launch reason

   @example
   ```ts
   new RequiredProviderError({ provider, reason: 'budget dry', });
   ```
   */
  public constructor(
    {
      provider,
      reason,
    }: {
      readonly provider: RequiredProvider;
      readonly reason: 'budget dry' | 'key missing' | 'left off' | 'meter unavailable';
    },
  ) {
    super({ says: `required provider ${provider} is not ready: ${reason}`, },);
    this.name = 'RequiredProviderError';
  }
}

/**
 Parses measured-arm provider requirement from CLI.

 @param line - pass's command line, read whole by `reportingRefusals`

 @returns Required providers in caller order without duplicates

 @throws {@link StatedRefusalError} when flag value names no provider or an
 unknown one

 @example
 ```ts
 const required = readRequiredProviders({ line, },);
 ```
 */
export function readRequiredProviders(
  { line, }: { readonly line: ReadsFlag<'require-providers'>; },
): readonly RequiredProvider[] {
  /**
   Parsed provider names before stable deduplication.
   */
  const parsedProviders = idListFlag({
    asked: line.flag('require-providers',),
    naming: `provider of ${REQUIRABLE_PROVIDERS.join(', ',)}`,
  },)
    .map(function parseProvider(provider,): RequiredProvider {
      if (isRequirable(provider,))
        return provider;
      if (isProviderName(provider,))
        throw new StatedRefusalError({
          says: `${REQUIRED_PROVIDERS_FLAG} accepts only ${REQUIRABLE_PROVIDERS.join(', ',)}, and ${
            JSON.stringify(provider,)
          } is an optional provider that no arm requires wet`,
        },);
      throw new StatedRefusalError({
        says: `${REQUIRED_PROVIDERS_FLAG} accepts only ${REQUIRABLE_PROVIDERS.join(', ',)}, and ${
          JSON.stringify(provider,)
        } is none of them`,
      },);
    },);
  return parsedProviders.filter(function unique(
    provider,
    index,
  ): boolean {
    return parsedProviders.indexOf(provider,) === index;
  },);
}

/**
 Reads one provider's meter and refuses when it is dry or unreadable.

 @param provider - provider being gated

 @param readDry - live meter read answering whether the provider is dry

 @param signal - caller's signal the meter read honours, whose own reason is
 the caller's stop rather than a meter that could not be read

 @throws {@link RequiredProviderError} when the meter reads dry or cannot be read

 @throws The caller's reason unchanged, with no line, when the read ended in
 it, since the caller stopping the gate says nothing about the meter

 @example
 ```ts
 await gateProvider({ provider: 'hyper', readDry, signal, },);
 ```
 */
async function gateProvider(
  {
    provider,
    readDry,
    signal,
  }: {
    readonly provider: RequiredProvider;
    readonly readDry: () => Promise<boolean>;
    readonly signal: AbortSignal;
  },
): Promise<void> {
  /**
   Whether the live meter reads dry, or that it could not be read.
   */
  const dry = await (async function read(): Promise<boolean | 'unreadable'> {
    try {
      return await readDry();
    } catch (error) {
      // THE CALLER'S OWN STOP IS NO METER FAILURE (ledger B403 left this
      // open): written as one, it read as a meter that could not be read and
      // refused the arm as `meter unavailable` in place of the stop. It goes
      // on unchanged and unwritten, as a stage call's does. A failure of the
      // read's own, a deadline or a refusal that landed as the stop arrived,
      // is still written with the provider's words and refuses the arm.
      if (signal.aborted && (error === signal.reason))
        throw error;
      // Every meter failure refuses the arm alike, and the refusal names only
      // the provider, so what failed is written to the log here (ledger T8).
      /**
       Logger tagged with the gate's name.
       */
      const rl = tagged({ tag: gateProvider.name, l: runLogger, },);
      rl.warn(`${provider} meter could not be read: ${exchangeFailureLogText({ error, },)}`,);
      return 'unreadable';
    }
  })();
  if (dry === 'unreadable')
    throw new RequiredProviderError({
      provider,
      reason: 'meter unavailable',
    },);
  if (dry)
    throw new RequiredProviderError({
      provider,
      reason: 'budget dry',
    },);
}

/**
 Requires selected provider keys and live non-dry meters before model calls.

 Ordinary runs pass empty requirement and retain any-provider behavior.
 Validation and performance arms name the providers they require explicitly.

 @param required - providers measured arm requires wet

 @param env - environment the keys and the Bedrock and NeuralWatt ledgers' places are read
 from: `process.env` in a run, a test's own otherwise

 @param transport - HTTP the meters are read over: `fetchTransport` in a run.
 Both are REQUIRED, since each reaches past the process (ledger M43, M68)

 @param signal - meter cancellation

 @param off - providers left off, which no run asks and so none can be required wet; the owner's
 rulings by default (`PROVIDERS_LEFT_OFF`)

 @throws {@link RequiredProviderError} before model call when requirement fails

 @throws The caller's reason when `signal` aborts and the first required
 provider in order ends its meter read in it; a meter read that fails of its
 own as the stop lands is written with the provider's words and refuses as
 any unreadable meter does

 @example
 ```ts
 await assertRequiredProvidersReady({ required: ['synthetic', 'hyper'], env: process.env, transport: fetchTransport, signal, });
 ```
 */
export async function assertRequiredProvidersReady(
  {
    required,
    env,
    transport,
    signal,
    off = PROVIDERS_LEFT_OFF,
  }: {
    readonly required: readonly RequiredProvider[];
    readonly env: Readonly<NodeJS.ProcessEnv>;
    readonly transport: ModelTransport;
    readonly signal: AbortSignal;
    readonly off?: readonly ProviderOff[];
  },
): Promise<void> {
  if (required.length === 0)
    return;
  // A PROVIDER LEFT OFF IS REFUSED BEFORE ITS KEY OR ITS METER IS LOOKED AT: the run client reads its key as
  // unset and asks it nothing, so a gate that passed it would print `status=wet` for a provider the run never asks.
  for (const provider of required) {
    if (isProviderOff({
      off,
      provider,
    },))
      throw new RequiredProviderError({
        provider,
        reason: 'left off',
      },);
  }
  /**
   Each required provider's key, read without exposing its value.
   */
  const keys = required.map(function keyOf(provider,): {
    readonly provider: RequiredProvider;
    readonly key: string;
  } {
    return {
      provider,
      key: env[KEY_VARIABLES[provider]] ?? '',
    };
  },);
  for (
    const {
      provider,
      key,
    } of keys
  ) {
    if (key === '')
      throw new RequiredProviderError({
        provider,
        reason: 'key missing',
      },);
  }
  await allInInputOrder({
    members: keys.map(async function checkProvider(
      {
        provider,
        key,
      },
    ): Promise<void> {
      if (provider === 'neuralwatt') {
        /**
         Required NeuralWatt meter client, over the energy ledger the environment names.
         */
        const client = createNeuralWattClient({
          apiKey: key,
          ledger: neuralWattLedgerFromEnv({ env, },),
          dayStartSetting: neuralWattDayStartFrom({ env, },),
          transport,
        },);
        await gateProvider({
          provider,
          readDry: async function readEnergy(): Promise<boolean> {
            return neuralWattIsDry({ energy: await client.energy({ signal, },), },);
          },
          signal,
        },);
        return;
      }
      if (provider === 'synthetic') {
        /**
         Required Synthetic meter client.
         */
        const client = createSyntheticClient({
          apiKey: key,
          transport,
        },);
        await gateProvider({
          provider,
          readDry: async function readQuota(): Promise<boolean> {
            return syntheticIsDry({ quota: await client.quotas({ signal, },), },);
          },
          signal,
        },);
        return;
      }
      if (provider === 'hyper') {
        /**
         Required Hyper meter client.
         */
        const client = createHyperClient({
          apiKey: key,
          transport,
        },);
        await gateProvider({
          provider,
          readDry: async function readCredits(): Promise<boolean> {
            return hyperIsDry({ credits: await client.credits({ signal, },), },);
          },
          signal,
        },);
        return;
      }
      if (provider === 'bedrock') {
        /**
         Required Bedrock meter client, over the ledger the environment names.
         */
        const client = createBedrockClient({
          apiKey: key,
          ledger: bedrockLedgerFromEnv({ env, },),
          transport,
        },);
        await gateProvider({
          provider,
          readDry: async function readBedrockCredits(): Promise<boolean> {
            return bedrockIsDry({ credits: await client.credits({ signal, },), },);
          },
          signal,
        },);
        return;
      }
      /**
       Required OpenRouter meter client.
       */
      const client = createOpenRouterClient({
        apiKey: key,
        transport,
      },);
      await gateProvider({
        provider,
        readDry: async function readOpenRouterCredits(): Promise<boolean> {
          return openRouterIsDry({ credits: await client.credits({ signal, },), },);
        },
        signal,
      },);
    },),
  },);
}

//endregion Required providers for measured arms
