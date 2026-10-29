import { tagged, } from '@monochromatic-dev/logger-core/ts';
import { readReviewedFidelityReferences, } from '../fidelity-reference-read.ts';
import { fetchTransport, } from '@monochromatic-dev/translation-repair-provider-stream/ts';
import {
  type CommandLineOf,
  readRunCorpusPinSetting,
  reportingRefusals,
  resolveRunsDir,
  RUN_PER_CALL_TIMEOUT_MS,
} from '@monochromatic-dev/translation-repair-run/ts';
import { cardsClientFrom, } from './editor-fidelity-client.ts';
import { runEditorFidelityProbe, } from './editor-fidelity-probe-run.ts';
import { digestPipeline, } from '@monochromatic-dev/translation-repair-glossary/ts';
import { nowAsIso, } from './probe-run-clock.ts';
import { readRunnerClosure, } from '@monochromatic-dev/translation-repair-insertion/ts';
import { readStageSettings, } from '@monochromatic-dev/translation-repair-stage/ts';
import type {
  ClaudeAttemptAsk,
  ModelClient,
} from '@monochromatic-dev/translation-repair-provider/ts';
import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract/ts';

//region Editor fidelity probe
// Wiring only: the command line, the build's identity, the corpus pin, the reference reader, the runs directory, the
// client builder, the clock, the logger and standard output go to `editor-fidelity-probe-run.ts`, which holds the whole
// run. The client is built over the Claude key alone and guarded to the named cards, so a launch that holds every
// provider's key still has one route.

/**
 Builds the client the cards are asked through: the Claude key alone, guarded to the cards named.

 @param promptPayloadDir - where the client keeps request payloads for the run

 @param cardIds - cards the run seats

 @param admitClaudeAttempt - hook the client calls before each attempt it makes, which may throw to refuse it

 @returns Client that refuses any model that is not one of the cards

 @example
 ```ts
 const client = newCardsClient({ promptPayloadDir, cardIds, admitClaudeAttempt, },);
 ```
 */
function newCardsClient(
  {
    promptPayloadDir,
    cardIds,
    admitClaudeAttempt,
  }: {
    readonly promptPayloadDir: string;
    readonly cardIds: readonly RosterModelId[];
    readonly admitClaudeAttempt: (attempt: ClaudeAttemptAsk,) => void;
  },
): ModelClient {
  return cardsClientFrom({
    env: process.env,
    transport: fetchTransport,
    promptPayloadDir,
    cardIds,
    admitClaudeAttempt,
  },);
}

/**
 Hands the run what only a real process has.

 @param line - probe's command line, read whole by `reportingRefusals`

 @example
 ```ts
 await main({ line, },);
 ```
 */
async function main({ line, }: { readonly line: CommandLineOf<'editor-fidelity-probe'>; },): Promise<void> {
  /**
   Corpus clone and commit this run reads, after any environment override, read inside the refusal boundary.
   */
  const { pin, } = readRunCorpusPinSetting({ env: process.env, },);

  /**
   Digest over built output, read before anything runs: the identity that answers for a run is the build present at
   startup, whatever is rebuilt while it is in flight.
   */
  const { digest: pipelineDigest, } = await digestPipeline({ dir: import.meta.dirname, },);
  return runEditorFidelityProbe({
    line,
    pin,
    readReferences: readReviewedFidelityReferences,
    resolveRuns: resolveRunsDir,
    newClient: newCardsClient,
    now: nowAsIso,
    pipelineDigest,
    runnerClosure: await readRunnerClosure({ entryPath: line.script, },),
    perCallTimeoutMs: RUN_PER_CALL_TIMEOUT_MS,
    log: tagged({ tag: 'editor-fidelity-probe', },),
    out: process.stdout,
    stageSettings: readStageSettings({ env: process.env, },),
  },);
}

if (import.meta.main)
  await reportingRefusals({
    what: 'editor-fidelity-probe',
    argv: process.argv,
    env: process.env,
    run: main,
  },);

//endregion Editor fidelity probe
