import { wallClockNow, } from '@monochromatic-dev/translation-repair-markup/ts';
import {
  contextRoot,
  installRunLog,
  openRunLog,
  printRunLine,
} from '@monochromatic-dev/translation-repair-typography/ts';
import { CLAUDE_SCHEMA_PRODUCERS, } from '../claude-schema-producers.ts';
import {
  declinedEntryIds,
  digestPipeline,
  readAttemptMap,
  StatedRefusalError,
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
    wallClock: passClock = wallClockNow,
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

  // THE RUN'S RECORDS, opened once the lock is held so a pass another pass refuses leaves nothing behind, and before
  // anything is printed or logged: everything the pass logs from here on, and the lines it prints, are recorded in
  // `<runsDir>/log/<day>/*.log.jsonl` through the shared logger's file sink (`typography/src/run-log.ts`), which no
  // retention sweep ever touches. A run that cannot record itself does not start: its evidence is the point.
  /**
   The run's log, flushed when the pass returns or throws, before the lock is let go.
   */
  await using runLog = await openRunLog({ runDir: runsDir, },);
  if (!runLog.recording)
    throw new StatedRefusalError({
      says: `the pass cannot write its records to ${runLog.directory}, so it would leave no record of itself and has `
        + 'not started. Make that directory writable, or name another runs directory with TRANSLATION_REPAIR_RUNS_DIR, '
        + 'then start the pass again.',
    },);
  /**
   Install of that log for every context root, put back when the pass returns.
   */
  using _recording = installRunLog({ log: runLog, },);

  /**
   Every path this pass reads and writes under its runs dir; the artifacts
   directory and the published tree are created now so a pass that settles
   nothing still leaves what it promised (`runs-layout.ts`).
   */
  const {
    artifactsDir,
    publishDir,
    declinedDir,
    interimDir,
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
    wallClock: passClock,
  },);

  /**
   The line the pass opens its run with, which a census tells a pass's records by.
   */
  const startLine = passStartLine({
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
    wallClock: passClock,
  },);
  printRunLine({ line: startLine, },);

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
    printRunLine({ line: launchLine, },);

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
    printRunLine({ line: requiredLine, },);

  // SAID AT EVERY LAUNCH, whether or not its key is in the environment: the client this run builds reads an off
  // provider's key as unset, so no call goes to it, and the line is where the run says so.
  for (const offLine of passProvidersOffLines({ off: PROVIDERS_LEFT_OFF, },))
    printRunLine({ line: offLine, },);

  /**
   Gate ending the pass when the router finds every provider dry (`pass-dry-stop.ts`); the pass never waits for
   NeuralWatt's energy to return (the owner's answer of 2026-10-29, "Never wait, relaunch").
   */
  const dryStop = createPassDryStop();

  /**
   Shared client using measured production provider concurrency, guarded so a dry refusal stops the pass.
   */
  const client = dryStop.guard({ client: input.newClient({
    promptPayloadDir,
    maxHoldMs: hardCapMs,
  },), },);

  if (line.switched('plan',)) {
    printRunLine({
      line: passPlanLine({
        tip,
        pipelineDigest,
        pendingIds: pending.map(function toPlanId(entry,): string {
          return entry.id;
        },),
        previewCount: PLAN_PREVIEW_COUNT,
      },),
    },);
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
    wallClock: passClock,
    settle: function settleOne(
      {
        entry,
        attempt,
        wallClock,
      },
    ): Promise<EntryOutcome> {
      return input.settle({
        client,
        entry,
        attempt,
        wallClock,
        artifactsDir,
        publishDir,
        declinedDir,
        interimDir,
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
  printRunLine({
    line: passDoneLine({
      processed,
      pending: pending.length,
      pairs,
      settledIds,
      declinedIds,
      elapsedMs: now() - start,
      wallClock: passClock,
    },),
  },);
}

//endregion Corpus pass run
