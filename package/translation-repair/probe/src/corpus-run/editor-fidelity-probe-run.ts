import {
  mkdir,
  mkdtemp,
} from 'node:fs/promises';
import { join, } from 'node:path';

import type { Logger, } from '@monochromatic-dev/logger-core/ts';
import type {
  ClaudeAttemptAsk,
  ModelClient,
} from '@monochromatic-dev/translation-repair-provider/ts';
import type { CorpusPin, } from '@monochromatic-dev/translation-repair-corpus-source/ts';
import type { CommandLineOf, } from '@monochromatic-dev/translation-repair-run/ts';
import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract/ts';
import type { StageSettings, } from '@monochromatic-dev/translation-repair-stage/ts';
import type { RunnerClosure, } from '@monochromatic-dev/translation-repair-insertion/ts';
import type {
  FidelityReferenceSpec,
  ReviewedFidelityReference,
} from '../fidelity-reference-model.ts';
import { EDITOR_FIDELITY_POOL, } from '../editor-fidelity-pool-manifest.ts';
import { editorFidelityTrials, } from '../editor-fidelity-trials.ts';
import { readEditorFidelityArguments, } from './editor-fidelity-args.ts';
import { recordingAsks, } from './editor-fidelity-asks.ts';
import { editorCeilings, } from './editor-fidelity-ceiling.ts';
import {
  editorFidelityPlan,
  type EditorFidelityRow,
  editorFidelityRow,
  editorKeptRun,
} from './editor-fidelity-plan.ts';
import {
  editorKeptLine,
  editorPreflightLine,
} from './editor-fidelity-plan-lines.ts';
import {
  manifestDigest,
  openingLines,
} from './editor-fidelity-opening.ts';
import { scopedToPass, } from './editor-fidelity-pass-scope.ts';
import {
  guardTotalLine,
  spendGuardedClient,
} from './editor-fidelity-spend-guard.ts';
import { runEditorTrial, } from './editor-fidelity-trial.ts';
import {
  reportTallies,
  rowLineOf,
  seatedCardsOf,
  sheetCharsOf,
} from './editor-fidelity-probe-read.ts';
import {
  plannedLine,
  trialsLine,
} from './editor-fidelity-progress-lines.ts';
import {
  pairsOf,
  plannedAsksOf,
  walkPairs,
  writeJsonOnce,
} from './editor-fidelity-walk.ts';
import { requestedSpecs, } from './judge-fidelity-probe-run.ts';
import { persistProbeRun, } from './probe-store.ts';

//region Editor fidelity probe run
// Seats each named Claude card alone as the editor of a hash-verified seeded defect that has a known fix, asks it what
// the pipeline's editor stage asks an editor, and reads the patched text by comparison with the reference. The trials
// are the frozen pool's (`editor-fidelity-pool-manifest.ts`), the same for every arm.
// Nothing else is asked: no critic, panel, judge or checker, so the only model called is the card, and no call is made
// that the spend measured so far plus its ceiling would take past `--max-usd`. The command line,
// the corpus pin, the reference reader, the runs directory, the client builder, the clock, the build's identity, the
// logger and the standard output are handed in by `editor-fidelity-probe.ts`.
//
// An unchanged archive is not automatically correct. Review locks the reference and every intentional delta.

/**
 Where the run's one JSON document goes, which is standard output in a run.

 @example
 ```ts
 const out: EditorProbeSink = process.stdout;
 ```
 */
export type EditorProbeSink = {
  /**
   Writes the text whole.
   */
  readonly write: (text: string,) => unknown;
};

/**
 Runs each named card alone over the frozen pool's trials and reads what it did.

 @param line - probe's command line, read whole by `reportingRefusals`

 @param pin - corpus commit and clone the reviewed references are read at

 @param readReferences - reads the selected references out of the pinned clone, the one step that touches the corpus

 @param resolveRuns - names the runs directory, called only once a run has something to keep

 @param newClient - builds the client the cards are asked through, called only after the plan is on disk and handed
 the cards, so a refused request needs no key and the client is built for those cards alone; the run wraps it in the
 spend guard

 @param now - clock, read for the start and for the finish

 @param pipelineDigest - digest of the built output, read when the process started

 @param runnerClosure - chunks the executing entry imports, read when the process started

 @param perCallTimeoutMs - deadline of every exchange

 @param log - progress and diagnostic logger

 @param out - where the rows document is written

 @param stageSettings - straggler windows and contest ledger the entry file read for this launch (`StageSettings`)

 @throws {@link FidelityReferenceError} for reference drift in what the clone holds

 @throws StatedRefusalError when the command line asks for something the probe does not run, a request the reviewed
 references refuse, or an arm whose next call, with the spend measured so far, would pass `--max-usd`: the arm ends
 there, and the rows it finished stay in their files

 @example
 ```ts
 await runEditorFidelityProbe({ line, pin, readReferences, resolveRuns, newClient, now, pipelineDigest, runnerClosure, perCallTimeoutMs, log, out, stageSettings, },);
 ```
 */
export async function runEditorFidelityProbe(
  {
    line,
    pin,
    readReferences,
    resolveRuns,
    newClient,
    now,
    pipelineDigest,
    runnerClosure,
    perCallTimeoutMs,
    log,
    out,
    stageSettings,
  }: {
    readonly line: CommandLineOf<'editor-fidelity-probe'>;
    readonly pin: CorpusPin;
    readonly readReferences: (
      input: {
        readonly pin: CorpusPin;
        readonly specs: readonly FidelityReferenceSpec[];
      },
    ) => Promise<readonly ReviewedFidelityReference[]>;
    readonly resolveRuns: () => Promise<string>;
    readonly newClient: (
      options: {
        readonly promptPayloadDir: string;
        readonly cardIds: readonly RosterModelId[];
        readonly admitClaudeAttempt: (attempt: ClaudeAttemptAsk,) => void;
      },
    ) => ModelClient;
    readonly now: () => string;
    readonly pipelineDigest: string;
    readonly runnerClosure: RunnerClosure;
    readonly perCallTimeoutMs: number;
    readonly log: Logger;
    readonly out: EditorProbeSink;
    readonly stageSettings: StageSettings;
  },
): Promise<void> {
  /**
   Existing flag names are kept, and an unreviewed context cannot become new gold evidence: this probe reads none.
   */
  const {
    onlyIds,
    cap,
    trials: trialsAsked,
    passes = 1,
    damageKinds,
    maxUsd,
    benchSpentUsd,
  } = readEditorFidelityArguments({ line, },);
  /**
   Cards seated, each alone, checked before anything is read.
   */
  const cardIds = seatedCardsOf({ line, },);
  /**
   Request and authorship checks precede all corpus and provider activity.
   */
  const specs = requestedSpecs({
    request: {
      specs: EDITOR_FIDELITY_POOL,
      corpusSha: pin.commitSha,
      onlyEntryIds: onlyIds,
      damageKinds,
      judgeModelIds: cardIds,
      cap,
      withContext: false,
    },
  },);
  log.info(`editors: ${cardIds.join(', ',)}`,);
  /**
   When the run began, read before its first work, the corpus read.
   */
  const startedAt = now();
  /**
   Exact source and locally reviewed reference, never a newly discovered long archive block.
   */
  const references = await readReferences({
    pin,
    specs,
  },);
  /**
   Every trial the request materializes, before a model is asked.
   */
  const trials = editorFidelityTrials({
    references,
    damageKinds,
  },);
  /**
   Asks the passes over the pool hold.
   */
  const total = trials.length * passes;
  /**
   Leading asks the arm walks, counted in attempts for each card: the number `--trials` names, else the cap; a cap of
   zero with no `--trials` asks nothing and prices every ask of the passes, so its ceiling is what an arm over the
   pool would print.
   */
  const planned = plannedAsksOf({
    trials,
    passes,
    walked: trialsAsked ?? ((cap === 0) ? total : cap),
    log,
  },);
  /**
   The most the run can cost, printed before any client exists; the spend guard holds each call to `--max-usd`.
   */
  const ceilings = editorCeilings({
    cardIds,
    sheetChars: planned.map(function charsOf({ trial, },): number {
      return sheetCharsOf({ trial, },);
    },),
  },);
  for (const said of openingLines({
    specs,
    trials,
    ceilings,
    maxUsd,
    benchSpentUsd,
  },))
    log.info(said,);
  log.info(plannedLine({
    planned: planned.length,
    total,
    passes,
  },),);
  if (cap === 0) {
    log.info(editorPreflightLine({
      cardIds,
      selected: specs.length,
      planned: planned.length,
      total,
      passes,
    },),);
    return;
  }

  /**
   Operator-selected output root; calibration callers use a disposable directory.
   */
  const runsDir = await resolveRuns();
  await mkdir(
    runsDir,
    {
      recursive: true,
      mode: 0o700,
    },
  );
  /**
   Fresh per-invocation state cannot replay old replies.
   */
  const runDir = await mkdtemp(join(
    runsDir,
    'editor-fidelity-',
  ),);
  /**
   Completed payloads remain available for explicit audit after interruption.
   */
  const promptPayloadDir = join(
    runDir,
    'payloads',
  );
  /**
   Each card's trials together, the first card's before the second's.
   */
  const pairs = pairsOf({
    cardIds,
    planned,
  },);
  /**
   Every exchange has the existing measured per-call deadline.
   */
  const controller = new AbortController();
  /**
   A durable plan cannot be mistaken for a completed result if execution is interrupted.
   */
  const plan = editorFidelityPlan({
    startedAt,
    pipelineDigest,
    runnerClosure,
    referenceManifestDigest: manifestDigest({ specs, },),
    specs,
    cardIds,
    trials,
    pairs,
    passes,
    ceilings,
  },);
  await writeJsonOnce({
    path: join(
      runDir,
      'plan.json',
    ),
    value: plan,
  },);
  /**
   Client construction follows all input verification and durable plan publication, and is for the named cards alone,
   with no call let through that could take the arm past `--max-usd`.
   */
  const client = spendGuardedClient({
    inner: function innerFor(admitClaudeAttempt,): ModelClient {
      return newClient({
        promptPayloadDir,
        cardIds,
        admitClaudeAttempt,
      },);
    },
    maxUsd,
    controller,
  },);
  /**
   The client noted for how each ask of a trial ends, which the row of the trial states.
   */
  const asks = recordingAsks({ client, },);
  /**
   Says what the guard counted when this run ends, finished or stopped, so the read step has the figure its SPEND-line
   sum is checked against even for an arm that ended on the refusal.
   */
  using guardSaid = {
    [Symbol.dispose](): void {
      log.info(guardTotalLine({ total: client.guardTotal(), },),);
    },
  };
  /**
   The rows that finished, and the guard's sentence when it capped the arm.
   */
  const rows = await walkPairs({
    pairs,
    controller,
    log,
    oneRow: async function runPair({
      pair,
      position,
    },): Promise<EditorFidelityRow> {
      /**
       The card seated alone on the trial, asked what the editor stage asks an editor.
       */
      const reading = await runEditorTrial({
        // Each pass asks the model afresh: the pass is the reuse scope of its exchanges (`editor-fidelity-pass-scope.ts`).
        client: scopedToPass({
          client: asks.client,
          pass: pair.pass ?? 0,
        },),
        cardId: pair.cardId,
        trial: pair.trial,
        signal: controller.signal,
        perCallTimeoutMs,
        l: log,
        stageSettings,
      },);
      /**
       How each ask of the trial ended.
       */
      const stops = asks.takeEnds();
      /**
       A completed row remains readable even if a later row interrupts the invocation.
       */
      const row = editorFidelityRow({
        cardId: pair.cardId,
        trial: pair.trial,
        reading,
        stops,
        ...((pair.pass === undefined) ? {} : { pass: pair.pass, }),
      },);
      await writeJsonOnce({
        path: join(
          runDir,
          `row-${String(position,)}.json`,
        ),
        value: row,
      },);
      log.info(rowLineOf({
        position,
        total: pairs.length,
        pair,
        reading,
        stops,
      },),);
      return row;
    },
  },);
  /**
   The tally of each card, in the order the cards were named, said as it is made.
   */
  const tallies = reportTallies({
    cardIds,
    rows,
    emit: function say(text,): void {
      log.info(text,);
    },
  },);
  log.info(trialsLine({
    completed: rows.length,
    planned: pairs.length,
  },),);
  /**
   Full reviewed provenance accompanies model outcomes without corpus passages in standard output.
   */
  const keptAt = await persistProbeRun({
    runsDir: runDir,
    probeName: 'editor-fidelity-probe',
    run: editorKeptRun({
      plan,
      rows,
      tallies,
      finishedAt: now(),
      promptPayloadDir,
      corpusPin: pin.commitSha,
      entriesRequested: onlyIds,
      trialCap: cap,
      damageKinds,
      maxUsd,
    },),
  },);
  log.info(editorKeptLine({
    count: rows.length,
    keptAt,
  },),);
  // Model findings can quote source material, so operational callers redirect this output privately.
  out.write(`${JSON.stringify(
    {
      rows,
      tallies,
    },
    undefined,
    2,
  )}\n`,);
}

//endregion Editor fidelity probe run
