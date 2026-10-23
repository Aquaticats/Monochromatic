import { contextRoot, } from '@monochromatic-dev/translation-repair-typography/ts';
import { CLAUDE_SCHEMA_PRODUCERS, } from '../claude-schema-producers.ts';
import {
  declinedEntryIds,
  digestPipeline,
  readAttemptMap,
} from '@monochromatic-dev/translation-repair-glossary/ts';
import {
  HARD_CAP_MINUTES,
  PLAN_PREVIEW_COUNT,
  SOFT_BUDGET_MS,
} from './corpus-pass-limits.ts';
import type { CorpusPassInput, } from './corpus-pass-input.ts';
import {
  passDoneLine,
  passLaunchLines,
  passPlanLine,
  passProvidersOffLines,
  passRequiredLines,
  passStartLine,
} from './corpus-pass-lines.ts';
import { assertPassResumable, } from './corpus-pass-guards.ts';
import { assertSchemasSendable, } from './pass-schema-check.ts';
import { runPassQueue, } from './corpus-pass-queue.ts';
import { createPassDryStop, } from './pass-dry-stop.ts';
import { selectPendingEntries, } from './corpus-pass-select.ts';
import {
  artifactBackedIds,
  entriesFinishedThisRun,
  type EntryOutcome,
  finishedEntryIds,
  republishRunPages,
} from '@monochromatic-dev/translation-repair-slice-cache/ts';
import {
  assertRequiredProvidersReady,
  lockRunsDir,
  prepareRunsLayout,
  PROVIDERS_LEFT_OFF,
  readRequiredProviders,
  RUN_CALL_CONFIG,
  RUN_PER_CALL_TIMEOUT_MS,
} from '@monochromatic-dev/translation-repair-run/ts';

//region Corpus pass run
// Runs the pipeline over every complete zh/en pair at the pinned commit,
// one entry at a time: skips entries that already have an artifact, orders the
// rest to resume cached progress first, then interleave the size bands by
// within-band rank so coverage fills evenly (then fewest-attempts-first), and
// stops starting new entries at the soft budget while a per-entry hard ceiling
// aborts an entry that overruns. Each settled
// entry writes one JSON artifact and one TALLY line. The entry file
// `corpus-pass.ts` reads what only a process has (the environment, the clock,
// the git tip, the client) and hands it to this procedure, which reads none of
// it itself.

/**
 Logger the republish lines go through.
 */
const republishLog = contextRoot({ tag: 'republish', },);

/**
 Runs one accumulation pass over the corpus, writing artifacts and TALLY lines.
 Performs model calls through the client it is handed unless `--plan` is on the
 command line, which verifies setup at zero quota and returns.

 @param input - what the pass needs from the process that runs it

 @throws {@link StatedRefusalError} for any refusal the pass states in its own words, such as a
 runs directory another pass holds, a required provider that is not ready, or a flag naming
 an entry the corpus does not hold

 @throws {@link UnsendableSchemasError} when a stage schema carries what the Claude route cannot send, before the
 lock is taken, anything is printed or any provider is asked

 @example
 ```ts
 await runCorpusPassOver({ ...input, },);
 ```
 */
export async function runCorpusPassOver(input: CorpusPassInput,): Promise<void> {
  /**
   Everything the pass reads from the process.
   */
  const {
    line,
    runsDir,
    pinSetting,
    hardCapMs,
    spendCeilingUsd,
    now,
  } = input;

  /**
   Corpus clone and commit this run reads.
   */
  const { pin, } = pinSetting;

  // FIRST OF ALL, before the lock, the guards, the START line and every provider: a stage schema the Claude route
  // cannot send is a fault in the code that no document causes, and each entry that reached its stage would otherwise
  // record one `status=ERROR` after the paid calls of the stages before it. `--plan` is checked too, since it
  // promises to verify setup at zero quota.
  assertSchemasSendable({ producers: input.schemaProducers ?? CLAUDE_SCHEMA_PRODUCERS, },);

  // Taken before anything is read, and held for the whole pass. Two passes
  // sharing one directory overwrite each other attempt counts, delete each
  // other cached slices whenever their pipelines differ, and the later write of
  // any entry replaces the earlier one, all of it looking like ordinary output.
  /**
   Exclusive claim on this runs directory, released when the pass returns.
   */
  await using _lock = await lockRunsDir({ runsDir, },);

  /**
   Every path this pass reads and writes under its runs dir; the artifacts
   directory and the published tree are created now so a pass that settles
   nothing still leaves what it promised (`runs-layout.ts`).
   */
  const {
    artifactsDir,
    publishDir,
    declinedDir,
    sliceCacheDir,
    promptPayloadDir,
    attemptsPath,
  } = await prepareRunsLayout({ runsDir, },);

  /**
   Pipeline tip recorded into every artifact.
   */
  const tip = await input.readTip();

  /**
   Identity of the built pipeline this invocation is running, taken over the
   directory the runner was loaded from.

   `tip` cannot answer this and never could: it moves for a documentation
   commit that changes nothing that runs, and stays put across an uncommitted
   edit that changes everything. Every corpus-run task builds before it runs
   and runs its built file, so the files beside the entry file ARE the pipeline.
   */
  const {
    digest: pipelineDigest,
    fileCount,
  } = await digestPipeline({ dir: input.pipelineDir, },);

  await assertPassResumable({
    artifactsDir,
    pipelineDigest,
    driftAllowed: input.driftAllowed,
  },);

  /**
   Entry ids already carrying an artifact this pass, or a decline record.
   */
  const done = await finishedEntryIds({
    artifactsDir,
    declinedDir,
  },);

  /**
   Attempt counts from prior runs, or empty on the first.
   */
  const attempts = await readAttemptMap(attemptsPath,);

  /**
   Pending entries in run order, the restriction and the pairs it could not
   read already said (`corpus-pass-select.ts`), and the complete pairs the
   walk found, which the closing line counts its artifacts and declines over.
   */
  const {
    pending,
    pairs,
  } = await selectPendingEntries({
    line,
    pin,
    done,
    attempts,
    sliceCacheDir,
  },);

  console.log(passStartLine({
    facts: {
      tip,
      pipelineDigest,
      fileCount,
      pending: pending.length,
      done: done.size,
      softBudgetMs: SOFT_BUDGET_MS,
      hardCapMs,
      perCallMs: RUN_PER_CALL_TIMEOUT_MS,
      claudeCallMs: RUN_CALL_CONFIG.claudeCallTimeoutMs,
    },
  },),);

  for (const launchLine of passLaunchLines({
    hardCapMs,
    builtInCapMinutes: HARD_CAP_MINUTES,
    perCallMs: RUN_PER_CALL_TIMEOUT_MS,
    claudeCallMs: RUN_CALL_CONFIG.claudeCallTimeoutMs,
    spendCeilingUsd,
    graceNote: input.graceNote,
    writerNote: input.writerNote,
    pinSetting,
  },))
    console.log(launchLine,);

  /**
   Providers validation or performance arm explicitly requires wet.
   */
  const requiredProviders = readRequiredProviders({ line, },);
  await assertRequiredProvidersReady({
    required: requiredProviders,
    env: input.env,
    transport: input.transport,
    signal: new AbortController().signal,
  },);

  for (const requiredLine of passRequiredLines({ providers: requiredProviders, },))
    console.log(requiredLine,);

  // SAID AT EVERY LAUNCH, whether or not its key is in the environment: the client this run builds reads an off
  // provider's key as unset, so no call goes to it, and the line is where the run says so.
  for (const offLine of passProvidersOffLines({ off: PROVIDERS_LEFT_OFF, },))
    console.log(offLine,);

  /**
   When the processing loop began, on the monotonic clock, set once the loop starts: a refusal cannot reach the gate
   before it, and the gate counts the pass unbounded until then.
   */
  const loopClock: { began?: number; } = {};

  /**
   Milliseconds the pass has left before its soft budget or hard cap, whichever is shorter; unbounded before the loop
   starts. The gate parks only for a return inside it, and the seat reader waits only for one inside it.

   @returns Milliseconds left
   */
  function passLeftMs(): number {
    return (loopClock.began === undefined)
      ? Number.POSITIVE_INFINITY
      : Math.min(
        SOFT_BUDGET_MS - (now() - loopClock.began),
        hardCapMs,
      );
  }

  /**
   Gate ending the pass when the router finds every provider dry, or parking it until NeuralWatt's energy returns when
   that fits what the pass has left (`pass-dry-stop.ts`).
   */
  const dryStop = createPassDryStop({ passLeftMs, },);

  /**
   Shared client using measured production provider concurrency, guarded so a dry refusal stops the pass.
   */
  const client = dryStop.guard({ client: input.newClient({
    promptPayloadDir,
    maxHoldMs: hardCapMs,
    passLeftMs,
  },), },);

  if (line.switched('plan',)) {
    console.log(passPlanLine({
      tip,
      pipelineDigest,
      pendingIds: pending.map(function toPlanId(entry,): string {
        return entry.id;
      },),
      previewCount: PLAN_PREVIEW_COUNT,
    },),);
    return;
  }

  // AFTER THE PLAN RETURNS, which promises no write, and after the build
  // guards, so a page is rewritten only under a build the operator let resume
  // here: every page the artifacts here say should ship differently, or that
  // is missing, is rewritten from its artifact before any entry runs, and a
  // page standing for a declined entry is removed (ledger A16c).
  await republishRunPages({
    runsDir,
    artifactsDir,
    declinedDir,
    publishDir,
    cloneDir: pin.cloneDir,
    l: republishLog,
  },);

  /**
   Start of the processing loop on the monotonic clock (ledger B78).
   */
  const start = now();
  loopClock.began = start;

  await runPassQueue({
    pending,
    attempts,
    attemptsPath,
    sliceCacheDir,
    pipelineDigest,
    dryStop,
    softBudgetMs: SOFT_BUDGET_MS,
    spendCeilingUsd,
    start,
    now,
    settle: function settleOne({ entry, },): Promise<EntryOutcome> {
      return input.settle({
        client,
        entry,
        artifactsDir,
        publishDir,
        declinedDir,
        sliceCacheDir,
        tip,
        pipelineDigest,
        pin,
        hardCapMs,
        // Aborted by the dry gate and by nothing else, so only a per-entry timeout or every provider dry ever fires.
        baseSignal: dryStop.signal,
        outsideReads: input.outsideReads,
        pictureSources: input.pictureSources,
        overlap: input.overlap,
        stageSettings: input.stageSettings,
        // The refusal that ended an entry reaches the gate even where no guarded call carried it.
        endRun: dryStop.trip,
        // Asked after the trip with the entry's own failure, so its TALLY line says parked and not failed.
        parked: dryStop.parkedFor,
      },);
    },
  },);

  /**
   Entries carrying an artifact after this run, read as the scheduler reads
   them.
   */
  const settledIds = await artifactBackedIds({ artifactsDir, },);

  /**
   Entries carrying a decline record after this run.
   */
  const declinedIds = await declinedEntryIds({ declinedDir, },);

  /**
   Entries this run finished, an artifact or a decline each.
   */
  const processed = await entriesFinishedThisRun({
    before: done,
    artifactsDir,
    declinedDir,
  },);
  console.log(passDoneLine({
    processed,
    pending: pending.length,
    pairs,
    settledIds,
    declinedIds,
    elapsedMs: now() - start,
  },),);
}

//endregion Corpus pass run
