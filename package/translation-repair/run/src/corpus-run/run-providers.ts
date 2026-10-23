import { tagged, } from '@monochromatic-dev/module-logger/ts';

import {
  type BedrockClient,
  type ClaudeClient,
  bedrockLedgerFromEnv,
  createBedrockClient,
  createClaudeClient,
  createDeepSeekClient,
  createHyperClient,
  createNeuralWattClient,
  createOpenRouterClient,
  createProviderBudgets,
  createSyntheticClient,
  createXiaomiClient,
  createCredentialHolds,
  type DeepSeekClient,
  type CredentialHolds,
  type Decider,
  describePlanSpend,
  type HyperClient,
  hyperRequestsPerHour,
  type NeuralWattClient,
  neuralWattDayStartFrom,
  neuralWattLedgerFromEnv,
  type OpenRouterClient,
  type ProviderBudgetsWithStates,
  type ModelClient,
  type XiaomiClient,
} from '@monochromatic-dev/translation-repair-provider/ts';
import type { ModelTransport, } from '@monochromatic-dev/translation-repair-provider-stream/ts';
import type { ProviderName, } from '@monochromatic-dev/translation-repair-shared-contract/ts';
import { createDecisionsClient, } from '../decisions-client.ts';
import {
  CLAUDE_KEY_VARIABLE,
  DEEPSEEK_KEY_VARIABLE,
  XIAOMI_KEY_VARIABLE,
} from './direct-key-variables.ts';
import {
  isProviderOff,
  type ProviderOff,
  PROVIDERS_LEFT_OFF,
} from './providers-off.ts';
import { RunConfigError, } from './run-config-error.ts';
import { registerRunEndReport, } from './run-end-report.ts';

//region Run providers
// WHICH PROVIDERS A RUN HAS, read off the environment: one client per key
// present, and the shared budget view over them. Split out of
// `run-config.ts` on 2026-09-07 when the fourth provider put that file over
// the line budget. Any number of keys may be absent; the owner, 2026-09-07:
// "any number of provider being dry or lacking API key is expected and normal
// operation". Only no key at all is a refusal.

/**
 Milliseconds the end-of-run quota read may take before the report gives up.
 */
const PLAN_ENERGY_TIMEOUT_MS = 15_000;

/**
 Clients a run may route to, each present when its key is, plus the budget
 view over all of them.

 @example
 ```ts
 const { budgets, } = configureProviders({
   env: process.env,
   transport: fetchTransport,
   maxHoldMs: 25_200_000,
 },);
 ```
 */
export type ConfiguredProviders = {
  /**
   Fifth provider client, first in routing order, when its key is set: the
   free daily energy the owner wants used first.
   */
  readonly neuralwatt?: NeuralWattClient;

  /**
   First provider client, when its key is set.
   */
  readonly synthetic?: ModelClient;

  /**
   Second provider client, when its key is set.
   */
  readonly hyper?: HyperClient;

  /**
   Fourth provider client, when its key is set.
   */
  readonly bedrock?: BedrockClient;

  /**
   DeepSeek's own API client, when its key is set; it serves one model.
   */
  readonly deepseek?: DeepSeekClient;

  /**
   Xiaomi's own API client, when its key is set; it serves one model.
   */
  readonly xiaomi?: XiaomiClient;

  /**
   The owner's Claude key client (Anthropic's own API), when its key is set; it serves Sonnet 5.5 and Haiku 5.5.
   */
  readonly anthropic?: ClaudeClient;

  /**
   Third provider client, when its key is set.
   */
  readonly openrouter?: OpenRouterClient;

  /**
   Client of OpenRouter's decisions endpoint, on the same key, for the
   decision-only seats.
   */
  readonly decisions?: Decider;

  /**
   Providers whose credential a call found refused, held out for the rest of
   the run. Built here, beside the budgets, so the `METERS` line the budgets
   print reads a held provider as dry as the routed view does.
   */
  readonly credentialHolds: CredentialHolds;

  /**
   Shared budget view every provider is routed by.
   */
  readonly budgets: ProviderBudgetsWithStates;
};

/**
 Builds every configured provider's client and the budget view over them.

 @param env - environment the keys, the Hyper pace and the Bedrock and NeuralWatt
 ledgers' places are read from: `process.env` in a run, a test's own otherwise

 @param transport - HTTP every client calls over: `fetchTransport` in a run.
 Both are REQUIRED (ledger M43, M68): a test that took the process's
 environment would build clients on the keys the suite inherits and a
 Bedrock client over the real spend ledger

 @param off - providers left off, whose keys are read as unset: no client, a meter that reads
 dry, and no call. The owner's rulings by default (`PROVIDERS_LEFT_OFF`); a test that exercises
 an off provider's client hands an empty list

 @param maxHoldMs - longest a wait a refusal states for itself may hold a provider out (REQUIRED); the pass hands its
 hard cap, the longest a hold can matter, and a probe hands the probe bound of `run-probe-client.ts`

 @returns Clients present per key, plus the budgets

 @throws {@link RunConfigError} when every provider key variable is unset or empty

 @example
 ```ts
 const providers = configureProviders({
   env: process.env,
   transport: fetchTransport,
   maxHoldMs: 25_200_000,
 },);
 ```
 */
export function configureProviders(
  {
    env,
    transport,
    off = PROVIDERS_LEFT_OFF,
    maxHoldMs,
  }: {
    readonly env: Readonly<NodeJS.ProcessEnv>;
    readonly transport: ModelTransport;
    readonly off?: readonly ProviderOff[];
    readonly maxHoldMs: number;
  },
): ConfiguredProviders {
  /**
   Reads one provider's key, as unset where the provider is left off.

   @param provider - provider the key belongs to

   @param key - the environment's value, empty when unset

   @returns The key, or empty for a provider that is off or has none

   @example
   ```ts
   const bedrockKey = keyUnlessOff({ provider: 'bedrock', key: env.TRANSLATION_REPAIR_AMAZON_BEDROCK_API_KEY ?? '', },);
   ```
   */
  function keyUnlessOff(
    {
      provider,
      key,
    }: {
      readonly provider: ProviderName;
      readonly key: string;
    },
  ): string {
    if (isProviderOff({
      off,
      provider,
    },))
      return '';
    return key;
  }

  /**
   NeuralWatt API key, the fifth provider and the first in routing order.
   */
  const neuralWattKey = keyUnlessOff({
    provider: 'neuralwatt',
    key: env.TRANSLATION_REPAIR_NEURALWATT_API_KEY ?? '',
  },);
  /**
   Synthetic API key, resolved by name from the environment handed over.
   */
  const apiKey = keyUnlessOff({
    provider: 'synthetic',
    key: env.TRANSLATION_REPAIR_SYNTHETIC_API_KEY ?? '',
  },);
  /**
   Second provider key,
   independently optional because either provider may run alone.
   */
  const hyperKey = keyUnlessOff({
    provider: 'hyper',
    key: env.TRANSLATION_REPAIR_CHARM_HYPER_API_KEY ?? '',
  },);
  /**
   Third provider key, the paid fallback, optional for the same reason.
   */
  const openRouterKey = keyUnlessOff({
    provider: 'openrouter',
    key: env.TRANSLATION_REPAIR_OPENROUTER_API_KEY ?? '',
  },);
  /**
   Fourth provider key, the prepaid per-token provider, optional likewise.
   READ AS UNSET WHILE BEDROCK IS OFF (`providers-off.ts`), however the environment holds it.
   */
  const bedrockKey = keyUnlessOff({
    provider: 'bedrock',
    key: env.TRANSLATION_REPAIR_AMAZON_BEDROCK_API_KEY ?? '',
  },);
  /**
   DeepSeek key, the owner's own name for it. Optional: a run without it routes the one model it serves
   elsewhere, and never to another provider's key in its place.
   */
  const deepSeekKey = keyUnlessOff({
    provider: 'deepseek',
    key: env[DEEPSEEK_KEY_VARIABLE] ?? '',
  },);
  /**
   Xiaomi key, optional likewise.
   */
  const xiaomiKey = keyUnlessOff({
    provider: 'xiaomi',
    key: env[XIAOMI_KEY_VARIABLE] ?? '',
  },);
  /**
   The owner's Claude key, optional likewise: a run without it seats no Claude card, and never borrows another
   provider's key in its place.
   */
  const claudeKey = keyUnlessOff({
    provider: 'anthropic',
    key: env[CLAUDE_KEY_VARIABLE] ?? '',
  },);
  /**
   Whether no provider at all is configured, which nothing can run on.
   */
  const noKeyAtAll = (neuralWattKey === '')
    && (apiKey === '')
    && (hyperKey === '')
    && (bedrockKey === '')
    && (deepSeekKey === '')
    && (xiaomiKey === '')
    && (claudeKey === '')
    && (openRouterKey === '');
  if (noKeyAtAll) {
    throw new RunConfigError({
      variable: 'TRANSLATION_REPAIR_NEURALWATT_API_KEY, TRANSLATION_REPAIR_SYNTHETIC_API_KEY, '
        + 'TRANSLATION_REPAIR_CHARM_HYPER_API_KEY, TRANSLATION_REPAIR_AMAZON_BEDROCK_API_KEY, '
        + `${DEEPSEEK_KEY_VARIABLE}, ${XIAOMI_KEY_VARIABLE}, ${CLAUDE_KEY_VARIABLE} `
        + 'or TRANSLATION_REPAIR_OPENROUTER_API_KEY',
    },);
  }

  // THE INFO LINES COME AFTER THE NO-KEY REFUSAL, so a launch with no key at all leaves stdout empty and says its
  // refusal alone.
  /**
   Logger pre-tagged with this function's name.
   */
  const rl = tagged({ tag: configureProviders.name, },);
  if (deepSeekKey === '')
    rl.info(
      `${DEEPSEEK_KEY_VARIABLE} is not set, so DeepSeek is unconfigured: its meter reads dry and no call goes there`,
    );
  if (xiaomiKey === '')
    rl.info(
      `${XIAOMI_KEY_VARIABLE} is not set, so Xiaomi is unconfigured: its meter reads dry and no call goes there`,
    );
  if (claudeKey === '')
    rl.info(
      `${CLAUDE_KEY_VARIABLE} is not set, so the Claude route is unconfigured: its meter reads dry and no call goes there`,
    );

  /**
   Fifth provider client when configured, over the durable energy ledger the
   environment names. Its end-of-run plan-energy report is registered here, so
   the command that ends the run prints it (`run-end-report.ts`); a run that
   makes no NeuralWatt call reads no quota and says so.
   */
  const neuralwatt = (neuralWattKey === '')
    ? undefined
    : createNeuralWattClient({
      apiKey: neuralWattKey,
      ledger: neuralWattLedgerFromEnv({ env, },),
      dayStartSetting: neuralWattDayStartFrom({ env, },),
      transport,
    },);
  if (neuralwatt !== undefined) {
    registerRunEndReport({
      key: 'neuralwatt-plan-energy',
      report: async function reportPlanEnergy(): Promise<string> {
        return describePlanSpend({
          spend: await neuralwatt.planSpend({ signal: AbortSignal.timeout(PLAN_ENERGY_TIMEOUT_MS,), },),
        },);
      },
    },);
  }

  /**
   First provider client when configured.
   */
  const synthetic = (apiKey === '')
    ? undefined
    : createSyntheticClient({
      apiKey,
      transport,
    },);

  /**
   Second provider client when configured.
   */
  const hyper = (hyperKey === '')
    ? undefined
    : createHyperClient({
      apiKey: hyperKey,
      requestsPerHour: hyperRequestsPerHour({ env, },),
      transport,
    },);

  /**
   Third provider client when configured.
   */
  const openrouter = (openRouterKey === '')
    ? undefined
    : createOpenRouterClient({
      apiKey: openRouterKey,
      transport,
    },);

  /**
   Decisions client on the OpenRouter key when configured; the endpoint
   draws on the same credit balance.
   */
  const decisions = (openRouterKey === '')
    ? undefined
    : createDecisionsClient({
      apiKey: openRouterKey,
      transport,
    },);

  /**
   Fourth provider client when configured, over the durable spend ledger
   the environment names.
   */
  const bedrock = (bedrockKey === '')
    ? undefined
    : createBedrockClient({
      apiKey: bedrockKey,
      ledger: bedrockLedgerFromEnv({ env, },),
      transport,
    },);

  /**
   DeepSeek client when configured, which reads its balance over the same key.
   */
  const deepseek = (deepSeekKey === '')
    ? undefined
    : createDeepSeekClient({
      apiKey: deepSeekKey,
      transport,
    },);

  /**
   Xiaomi client when configured; it paces itself to the documented request rate.
   */
  const xiaomi = (xiaomiKey === '')
    ? undefined
    : createXiaomiClient({
      apiKey: xiaomiKey,
      transport,
    },);

  /**
   The Claude client when configured; it paces itself to the key's stated request rate and logs its limits once.
   */
  const anthropic = (claudeKey === '')
    ? undefined
    : createClaudeClient({
      apiKey: claudeKey,
      transport,
    },);

  /**
   Providers whose credential a call found refused, held out for the rest of
   the run.
   */
  const credentialHolds = createCredentialHolds();

  /**
   Shared budget view every provider is routed by.
   */
  const budgets = createProviderBudgets({
    ...((neuralwatt === undefined) ? {} : { neuralwatt, }),
    ...((synthetic === undefined) ? {} : { synthetic, }),
    ...((hyper === undefined) ? {} : { hyper, }),
    ...((bedrock === undefined) ? {} : { bedrock, }),
    ...((deepseek === undefined) ? {} : { deepseek, }),
    ...((xiaomi === undefined) ? {} : { xiaomi, }),
    ...((anthropic === undefined) ? {} : { anthropic, }),
    ...((openrouter === undefined) ? {} : { openrouter, }),
    maxHoldMs,
    heldOut: function credentialRefused(provider,): boolean {
      return credentialHolds.isHeld({ provider, },);
    },
  },);

  return {
    ...((neuralwatt === undefined) ? {} : { neuralwatt, }),
    ...((synthetic === undefined) ? {} : { synthetic, }),
    ...((hyper === undefined) ? {} : { hyper, }),
    ...((bedrock === undefined) ? {} : { bedrock, }),
    ...((deepseek === undefined) ? {} : { deepseek, }),
    ...((xiaomi === undefined) ? {} : { xiaomi, }),
    ...((anthropic === undefined) ? {} : { anthropic, }),
    ...((openrouter === undefined) ? {} : { openrouter, }),
    ...((decisions === undefined) ? {} : { decisions, }),
    credentialHolds,
    budgets,
  };
}

//endregion Run providers
