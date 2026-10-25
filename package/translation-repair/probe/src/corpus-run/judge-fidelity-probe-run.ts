import {
  mkdir,
  mkdtemp,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import type { Logger, } from '@monochromatic-dev/module-logger/ts';
import type { ModelClient, } from '@monochromatic-dev/translation-repair-provider/ts';
import type { CorpusPin, } from '@monochromatic-dev/translation-repair-corpus-source/ts';
import { hashContent, } from '@monochromatic-dev/translation-repair-document-structure/ts';
import { FidelityReferenceError, } from '../fidelity-reference-error.ts';
import type {
  FidelityReferenceSpec,
  ReviewedFidelityReference,
} from '../fidelity-reference-model.ts';
import { reviewedFidelityRequest, } from '../fidelity-reference-request.ts';
import { StatedRefusalError, } from '@monochromatic-dev/translation-repair-glossary/ts';
import { reviewedFidelityTrials, } from '../fidelity-reference-trials.ts';
import {
  type CommandLineOf,
  runFidelityTrial,
} from '@monochromatic-dev/translation-repair-run/ts';
import {
  mapOverlapped,
  type StageSettings,
} from '@monochromatic-dev/translation-repair-stage/ts';
import { readFidelityArguments, } from './judge-fidelity-args.ts';
import {
  boundFidelityMatrix,
  fidelityJudgedLine,
  fidelityKeptLine,
  fidelityKeptRun,
  fidelityPartialWarning,
  fidelityPlan,
  fidelityPreflightLine,
  fidelityRowResult,
  type FidelityRowResult,
} from './judge-fidelity-probe-plan.ts';
import {
  probeRosterWith,
  readCandidateIds,
  readCandidatesAlone,
} from './probe-candidates.ts';
import { persistProbeRun, } from './probe-store.ts';
import type { RunnerClosure, } from '@monochromatic-dev/translation-repair-insertion/ts';

//region Judge fidelity probe run
// Source-reviewed judge calibration, once the process has handed it the
// things only a real process has: the command line, the corpus pin, the
// reference reader, the runs directory, the client builder, the clock, the
// build's identity, the logger and the standard output.
//
// An unchanged archive is not automatically correct. Review locks the
// reference and every intentional delta.

/**
 Where the run's one JSON document goes, which is standard output in a run.

 @example
 ```ts
 const out: FidelityProbeSink = process.stdout;
 ```
 */
export type FidelityProbeSink = {
  /**
   Writes the text whole.
   */
  readonly write: (text: string,) => unknown;
};

/**
 Checks what the command line asked for against the reviewed references, and
 restates a refusal as the operator's.

 THE OPERATOR'S TO MEND, SO STATED. The inputs `reviewedFidelityRequest`
 checks are ones the operator typed or set: an entry filter, `--context`, the
 cap, the judges named, the damage families, and the corpus pin the
 environment can move. `FidelityReferenceError` is a fault elsewhere, where a
 reference read from the clone fails its reviewed hashes, so it is restated
 here, at the one site where it is the operator's, with its own sentence and
 itself as the cause. Until 2026-10-10 it printed as a fault of the command at
 exit 5, under frames (ledger B382). One refusal under the same operation is
 not the operator's: an empty checked-in manifest, or one repeating an id,
 which `selectReviewedFidelitySpecs` refuses as well and this restates too.

 @internal

 @param request - what `reviewedFidelityRequest` checks

 @returns The selected reviewed specifications

 @throws StatedRefusalError when the request is refused, its sentence the
 reviewed reference's own

 @example
 ```ts
 const specs = requestedSpecs({ request: { corpusSha, onlyEntryIds: [], damageKinds, judgeModelIds, cap: 0, withContext: false, }, },);
 ```
 */
export function requestedSpecs(
  { request, }: { readonly request: Parameters<typeof reviewedFidelityRequest>[0]; },
): readonly FidelityReferenceSpec[] {
  try {
    return reviewedFidelityRequest(request,);
  } catch (error) {
    if (error instanceof FidelityReferenceError) {
      throw new StatedRefusalError({
        says: error.message,
        cause: error,
      },);
    }
    // Every refusal of `reviewedFidelityRequest` and of the
    // `selectReviewedFidelitySpecs` it calls is a `FidelityReferenceError`,
    // and what else they do (comparisons, a `Set`, a `structuredClone` of the
    // checked-in manifest) throws nothing for any request.
    throw new Error(
      'unreachable: checking a fidelity request threw something other than its own refusal, though every refusal it gives is a FidelityReferenceError',
      { cause: error, },
    );
  }
}

/**
 Names a planned row on the lines the overlapped map writes about a failure it
 leaves unthrown, by the trial it runs.

 @param item - planned row, named by its trial

 @returns The trial by its handle

 @example
 ```ts
 plannedTrialName({ item: row, },); // 'trial tabby-nap-clean-first'
 ```
 */
function plannedTrialName(
  { item, }: { readonly item: { readonly trial: { readonly trialId: string; }; }; },
): string {
  return `trial ${item.trial
    .trialId}`;
}

/**
 Runs only source-reviewed, hash-locked comparisons through the production
 selector. Individual ballots are retained; a singleton judge's underweight
 merged verdict is not a quality score. This never changes role admission itself.

 @param line - the probe's command line, read whole by `reportingRefusals`

 @param pin - corpus commit and clone the reviewed references are read at

 @param readReferences - reads the selected references out of the pinned clone,
 which is the one step that touches the corpus

 @param resolveRuns - names the runs directory, called only once a run has
 something to keep, so a request that is refused or a zero cap needs none

 @param newClient - builds the client the judges are asked through, called only
 after the plan is on disk so a refused request needs no key

 @param now - the clock, read for the start and for the finish

 @param pipelineDigest - digest of the built output, read when the process started

 @param runnerClosure - chunks the executing entry imports, read when the process started

 @param perCallTimeoutMs - deadline of every exchange

 @param log - progress and diagnostic logger

 @param out - where the rows document is written

 @param stageSettings - straggler windows and contest ledger the entry file
 read for this launch (`StageSettings`)

 @throws {@link FidelityReferenceError} for reference drift in what the clone holds

 @throws StatedRefusalError when the command line asks for something the probe does not run, or
 for a request the reviewed references refuse

 @example
 ```ts
 await runFidelityProbe({ line, pin, readReferences, resolveRuns, newClient, now, pipelineDigest, runnerClosure, perCallTimeoutMs, log, out, stageSettings, },);
 ```
 */
export async function runFidelityProbe(
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
    readonly line: CommandLineOf<'judge-fidelity-probe'>;
    readonly pin: CorpusPin;
    readonly readReferences: (
      input: {
        readonly pin: CorpusPin;
        readonly specs: readonly FidelityReferenceSpec[];
      },
    ) => Promise<readonly ReviewedFidelityReference[]>;
    readonly resolveRuns: () => Promise<string>;
    readonly newClient: (options: { readonly promptPayloadDir: string; },) => ModelClient;
    readonly now: () => string;
    readonly pipelineDigest: string;
    readonly runnerClosure: RunnerClosure;
    readonly perCallTimeoutMs: number;
    readonly log: Logger;
    readonly out: FidelityProbeSink;
    readonly stageSettings: StageSettings;
  },
): Promise<void> {
  /**
   Existing flags retain their names; unreviewed context cannot become new gold evidence.
   */
  const {
    onlyIds,
    cap,
    damageKinds,
    withContext,
  } = readFidelityArguments({ line, },);
  /**
   Approved candidates can be measured without acquiring a production seat.
   */
  const judgeModelIds = probeRosterWith({
    candidates: readCandidateIds({ line, },),
    alone: readCandidatesAlone({ line, },),
  },);
  /**
   Request and authorship checks precede all corpus and provider activity.
   */
  const specs = requestedSpecs({
    request: {
      corpusSha: pin.commitSha,
      onlyEntryIds: onlyIds,
      damageKinds,
      judgeModelIds,
      cap,
      withContext,
    },
  },);
  log.info(`judges: ${judgeModelIds.join(', ',)}`,);
  if (cap === 0) {
    log.info(fidelityPreflightLine({ selected: specs.length, },),);
    return;
  }
  /**
   When the run began, read before its first work, the corpus read, so the plan
   and the kept run date the whole run, as the sibling probes read theirs
   (`coverage-probe-run.ts`, `recall-benchmark-run.ts`,
   `rendering-audit-settled-drive.ts`); until 2026-10-10 it was read after the
   references and the run directory, so their time was left out.
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
   Fixed complete matrix, bounded in attempted rows before any calls begin.
   */
  const matrix = reviewedFidelityTrials({
    references,
    damageKinds,
  },);
  /**
   Completeness describes the requested population, not automatic role eligibility.
   */
  const bound = boundFidelityMatrix({
    matrix,
    cap,
  },);
  if (!bound.complete)
    log.warn(fidelityPartialWarning({
      planned: bound.planned
        .length,
      total: matrix.length,
    },),);
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
   Fresh per-invocation state cannot replay old ballots or synthetic planner responses.
   */
  const runDir = await mkdtemp(join(
    runsDir,
    'judge-fidelity-',
  ),);
  /**
   Completed payloads remain available for explicit audit after interruption.
   */
  const promptPayloadDir = join(
    runDir,
    'payloads',
  );
  /**
   Every exchange has the existing measured per-call deadline.
   */
  const controller = new AbortController();
  /**
   A durable plan cannot be mistaken for a completed result if execution is interrupted.
   */
  const plan = fidelityPlan({
    startedAt,
    pipelineDigest,
    runnerClosure,
    referenceManifestDigest: hashContent({ content: JSON.stringify(specs,), },),
    specs,
    judgeModelIds,
    matrix,
    bound,
  },);
  await writeFile(
    join(
      runDir,
      'plan.json',
    ),
    JSON.stringify(
      plan,
      undefined,
      2,
    ),
    {
      mode: 0o600,
      flag: 'wx',
    },
  );
  /**
   Client construction follows all input verification and durable plan publication.
   */
  const client = newClient({ promptPayloadDir, },);
  /**
   Sequential native driver retains every outcome and its actual ballots.
   */
  const rows = await mapOverlapped({
    items: bound.planned,
    overlap: 1,
    // The probe never stops its own exchanges, so this signal never aborts;
    // one row runs at a time, so no failure is left unthrown to be written.
    signal: controller.signal,
    l: log,
    nameOf: plannedTrialName,
    oneItem: async function runReviewedTrial({
      item: row,
      position,
    },): Promise<FidelityRowResult> {
      /**
       Existing selector and unchanged criteria evaluate the reviewed comparison.
       */
      const outcome = await runFidelityTrial({
        client,
        trial: row.trial,
        judgeModelIds,
        signal: controller.signal,
        perCallTimeoutMs,
        l: log,
        stageSettings,
      },);
      /**
       A completed row remains readable even if a later row interrupts the invocation.
       */
      const result = fidelityRowResult({
        row,
        outcome,
      },);
      await writeFile(
        join(
          runDir,
          `row-${String(position,)}.json`,
        ),
        JSON.stringify(
          result,
          undefined,
          2,
        ),
        {
          mode: 0o600,
          flag: 'wx',
        },
      );
      return result;
    },
  },);
  log.info(fidelityJudgedLine({ count: rows.length, },),);
  /**
   Full reviewed provenance accompanies model outcomes without corpus passages in stdout metadata.
   */
  const keptAt = await persistProbeRun({
    runsDir: runDir,
    probeName: 'judge-fidelity-probe',
    run: fidelityKeptRun({
      plan,
      rows,
      finishedAt: now(),
      promptPayloadDir,
      corpusPin: pin.commitSha,
      entriesRequested: onlyIds,
      trialCap: cap,
      damageKinds,
    },),
  },);
  log.info(fidelityKeptLine({
    count: rows.length,
    keptAt,
  },),);
  // Model reasons can quote source material, so operational callers redirect this output privately.
  out.write(`${JSON.stringify(
    { rows, },
    undefined,
    2,
  )}\n`,);
}

//endregion Judge fidelity probe run
