import {
  type Logger,
  tagged,
} from '@monochromatic-dev/module-logger/ts';

import {
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
  neuralWattDayStartFrom,
  neuralWattLedgerFromEnv,
} from '@monochromatic-dev/translation-repair-provider/ts';
import { PROVIDER_ORDER, } from '@monochromatic-dev/translation-repair-shared-contract/ts';
import type { ModelTransport, } from '@monochromatic-dev/translation-repair-provider-stream/ts';
import { readProviderKeys, } from './budget-sample-keys.ts';
import { taskPathOf, } from './command-owners.ts';
import { PROBE_MAX_HOLD_MS, } from './run-probe-client.ts';

//region Budget sample run
// Takes ONE reading of every provider's meter and leaves it in the log.
//
// WHY THIS EXISTS SEPARATELY FROM A RUN. The budget layer reads the meters when
// something asks to spend, so the availability record is dense while a pass is
// running and empty otherwise. That is the right denominator for a duty cycle,
// which prices a seat by availability WHEN WE WERE ASKING. It is the wrong one
// for the other half of the question: an outage that stops a pass also stops
// the readings, so nothing observes when the provider came back, and every
// outage that ended a run reads as open-ended forever.
//
// This closes that. Run it between passes, or on a timer, and the record gains
// readings during the quiet stretches where the recovery actually happened.
//
// SPENDS NO GENERATION. It reads the five meters the router reads at most once
// a minute while working (ledger D16): the Synthetic, Hyper and OpenRouter
// endpoints, and the Bedrock and NeuralWatt ledgers on disk, where those
// providers' credit and daily energy are kept (NeuralWatt's quota endpoint is
// not read: it reports the plan, not the free daily energy). No model is called, no token is produced, and nothing is written to
// a run directory.
//
// THE READING IS THE OUTPUT. It goes to the log as a `METERS` line, which is
// the same line a pass leaves and the same line `meter-report` reads back.
// Capture both streams: the reading is at info and an unreadable meter warns.

/**
 How long one sample may take before it is abandoned.

 SET TO THE FRESHNESS WINDOW rather than picked. A reading is trusted for
 sixty seconds, so one that takes longer than that to arrive has aged out
 before it could be used, and a sampler that waits past it is measuring the
 endpoint's latency rather than the provider's budget.
 */
const SAMPLE_TIMEOUT_MS = 60_000;

/**
 Reads every provider's meter once and leaves the reading in the log.

 Returns nothing: the `METERS` line IS the output.

 @param env - environment the keys and the Bedrock and NeuralWatt ledgers' places are read from: `process.env` in a run

 @param transport - HTTP every meter is read over: `fetchTransport` in a run

 @param l - logger the summary line is written to, tagged here

 @throws {@link StatedRefusalError} when any provider's key is absent, since
 a sample of some providers cannot answer a question about the others

 @example
 ```ts
 await sampleBudgets({ env: process.env, transport: fetchTransport, l, },);
 ```
 */
export async function sampleBudgets(
  {
    env,
    transport,
    l,
  }: {
    readonly env: Readonly<NodeJS.ProcessEnv>;
    readonly transport: ModelTransport;
    readonly l: Logger;
  },
): Promise<void> {
  /**
   Logger pre-tagged with this function's name.
   */
  const rl = tagged({
    tag: sampleBudgets.name,
    l,
  },);

  /**
   Every provider's key.
   */
  const keys = readProviderKeys({ env, },);

  /**
   Budget view over every meter, which logs what it reads.

   ITS CACHE CANNOT INTERFERE. A fresh view has never read anything, so the
   first call always reaches the wire, and this process makes exactly one.
   */
  const budgets = createProviderBudgets({
    // A SINGLE READING MARKS NO REFUSAL, so no hold is ever set and this bound never cuts; the budgets require one.
    maxHoldMs: PROBE_MAX_HOLD_MS,
    neuralwatt: createNeuralWattClient({
      apiKey: keys.neuralwatt,
      ledger: neuralWattLedgerFromEnv({ env, },),
      dayStartSetting: neuralWattDayStartFrom({ env, },),
      transport,
    },),
    synthetic: createSyntheticClient({
      apiKey: keys.synthetic,
      transport,
    },),
    hyper: createHyperClient({
      apiKey: keys.hyper,
      transport,
    },),
    bedrock: createBedrockClient({
      apiKey: keys.bedrock,
      ledger: bedrockLedgerFromEnv({ env, },),
      transport,
    },),
    // THE TWO DIRECT PROVIDERS ARE SAMPLED WHEN KEYED. An unset key leaves one unconfigured, which the
    // line records as dry, the word every unconfigured provider gets.
    ...((keys.deepseek === '')
      ? {}
      : {
        deepseek: createDeepSeekClient({
          apiKey: keys.deepseek,
          transport,
        },),
      }),
    ...((keys.xiaomi === '')
      ? {}
      : {
        xiaomi: createXiaomiClient({
          apiKey: keys.xiaomi,
          transport,
        },),
      }),
    // THE CLAUDE KEY LIKEWISE, when set: its meter is the run's own priced spend and a billing flag, so the sample
    // reads it wet, and the line carries `anthropicUsdSpent`.
    ...((keys.anthropic === '')
      ? {}
      : {
        anthropic: createClaudeClient({
          apiKey: keys.anthropic,
          transport,
        },),
      }),
    openrouter: createOpenRouterClient({
      apiKey: keys.openrouter,
      transport,
    },),
  },);

  /**
   The routed view, whose real product is the line the read leaves behind.
   */
  const view = await budgets.read({ signal: AbortSignal.timeout(SAMPLE_TIMEOUT_MS,), },);

  /**
   What routing would do with each provider, for the summary.
   */
  const verdicts = PROVIDER_ORDER.map(function verdictOf(provider,): string {
    return `${view[provider] ? 'avoid' : 'use'} ${provider}`;
  },);
  rl.info(
    `SAMPLED: routing would ${verdicts.join(', ',)}. The reading this command logged is the record; `
      + `read a collection of them with \`mise run ${taskPathOf({ command: 'meter-report', },)}\``,
  );
}

//endregion Budget sample run
