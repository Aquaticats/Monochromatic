import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';
import type { FidelityDamageKind, } from '@monochromatic-dev/translation-repair-repair/ts';
import type { FidelityReferenceSpec, } from '../fidelity-reference-model.ts';
import type { ReviewedFidelityTrial, } from '../fidelity-reference-trials.ts';
import type { FidelityOutcome, } from '@monochromatic-dev/translation-repair-run/ts';
import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract/ts';
import type { ProbeRun, } from './probe-store.ts';
import type { RunnerClosure, } from '@monochromatic-dev/translation-repair-insertion/ts';

//region Judge fidelity probe plan
// What the fidelity probe writes down and says, built from values it already
// holds: the bounded matrix, the durable plan, one row per completed trial,
// the run's subject and the lines it prints. Kept apart from the driver so the
// driver stays a sequence of steps and each shape here has a case of its own.

/**
 The matrix a run will attempt and whether that is all of it.

 @example
 ```ts
 const bound: BoundedFidelityMatrix = { planned: [], complete: true, };
 ```
 */
export type BoundedFidelityMatrix = {
  /**
   Leading rows of the matrix, at most the cap, in matrix order.
   */
  readonly planned: readonly ReviewedFidelityTrial[];

  /**
   Whether every row of the requested matrix is planned, as opposed to a
   bounded prefix that is exploratory evidence only.
   */
  readonly complete: boolean;
};

/**
 Bounds the matrix to the trial cap, counted in attempts.

 @param matrix - every reviewed row the request materialized

 @param cap - most rows to attempt

 @returns The leading rows within the cap and whether they are the whole matrix

 @example
 ```ts
 const { planned, complete, } = boundFidelityMatrix({ matrix, cap: 16, },);
 ```
 */
export function boundFidelityMatrix(
  {
    matrix,
    cap,
  }: {
    readonly matrix: readonly ReviewedFidelityTrial[];
    readonly cap: number;
  },
): BoundedFidelityMatrix {
  /**
   A bounded prefix remains exploratory, not a complete admission comparison.
   */
  const planned = matrix.slice(
    0,
    cap,
  );
  return {
    planned,
    complete: planned.length === matrix.length,
  };
}

/**
 Line saying a request was accepted and nothing was asked, which is what a cap
 of zero buys.

 @param selected - reviewed reference specifications the request selected

 @returns The line, without a newline

 @example
 ```ts
 log.info(fidelityPreflightLine({ selected: 3, },),);
 ```
 */
export function fidelityPreflightLine({ selected, }: { readonly selected: number; },): string {
  return `preflight only: ${String(selected,)} reviewed reference ${
    wordForCount({
      count: selected,
      one: 'specification',
      many: 'specifications',
    },)
  } selected; no corpus or model calls`;
}

/**
 Warning that a bounded prefix is not the complete requested matrix.

 @param planned - rows the run will attempt

 @param total - rows the whole requested matrix holds

 @returns The warning, without a newline

 @example
 ```ts
 log.warn(fidelityPartialWarning({ planned: 2, total: 4, },),);
 ```
 */
export function fidelityPartialWarning(
  {
    planned,
    total,
  }: {
    readonly planned: number;
    readonly total: number;
  },
): string {
  return `partial reviewed matrix: ${String(planned,)} of ${String(total,)} ${
    wordForCount({
      count: total,
      one: 'row',
      many: 'rows',
    },)
  }; not complete admission evidence`;
}

/**
 One planned row as the durable plan lists it.

 @example
 ```ts
 const row: FidelityPlannedRow = { position: 0, trialId: 'purr/deletion', direction: 'preserve', cleanFirst: true, damageKind: 'deletion', };
 ```
 */
export type FidelityPlannedRow = {
  /**
   Where the row sits among the planned ones, from zero.
   */
  readonly position: number;

  /**
   Trial the row runs.
   */
  readonly trialId: string;

  /**
   Which side of the comparison holds the reviewed reference.
   */
  readonly direction: ReviewedFidelityTrial['trial']['direction'];

  /**
   Whether the reference is listed first on the ballot.
   */
  readonly cleanFirst: boolean;

  /**
   Defect the damaged text carries.
   */
  readonly damageKind: FidelityDamageKind;
};

/**
 What a run is and what it is about to attempt, written before any model is
 asked so an interrupted run cannot be mistaken for a finished one.

 @example
 ```ts
 const plan: FidelityPlan = fidelityPlan({ startedAt, pipelineDigest, runnerClosure, referenceManifestDigest, specs, judgeModelIds, matrix, bound, },);
 ```
 */
export type FidelityPlan = {
  /**
   Always `planned`: a completed run is a different file.
   */
  readonly status: 'planned';

  /**
   When this invocation began.
   */
  readonly startedAt: string;

  /**
   Digest of the built output that ran.
   */
  readonly pipelineDigest: string;

  /**
   Chunks the executing entry imports.
   */
  readonly runnerClosure: RunnerClosure;

  /**
   Digest of the reviewed manifest this run used.
   */
  readonly referenceManifestDigest: string;

  /**
   The reviewed specifications themselves.
   */
  readonly referenceManifest: readonly FidelityReferenceSpec[];

  /**
   Judges the request named.
   */
  readonly requestedRoster: readonly RosterModelId[];

  /**
   Whether the plan holds the whole requested matrix.
   */
  readonly completeRequestedMatrix: boolean;

  /**
   Rows the whole requested matrix holds.
   */
  readonly fullMatrixRows: number;

  /**
   Rows this run will attempt.
   */
  readonly plannedRows: number;

  /**
   The attempted rows, in order.
   */
  readonly rows: readonly FidelityPlannedRow[];
};

/**
 Builds the durable plan.

 These are logical trial rows, not a claim that every configured judge was
 actually asked.

 @param startedAt - when this invocation began

 @param pipelineDigest - digest of the built output that ran

 @param runnerClosure - chunks the executing entry imports

 @param referenceManifestDigest - digest of the reviewed manifest used

 @param specs - reviewed specifications the request selected

 @param judgeModelIds - judges the request named

 @param matrix - every reviewed row the request materialized

 @param bound - the rows this run attempts

 @returns The plan, ready to serialize

 @example
 ```ts
 const plan = fidelityPlan({ startedAt, pipelineDigest, runnerClosure, referenceManifestDigest, specs, judgeModelIds, matrix, bound, },);
 ```
 */
export function fidelityPlan(
  {
    startedAt,
    pipelineDigest,
    runnerClosure,
    referenceManifestDigest,
    specs,
    judgeModelIds,
    matrix,
    bound,
  }: {
    readonly startedAt: string;
    readonly pipelineDigest: string;
    readonly runnerClosure: RunnerClosure;
    readonly referenceManifestDigest: string;
    readonly specs: readonly FidelityReferenceSpec[];
    readonly judgeModelIds: readonly RosterModelId[];
    readonly matrix: readonly ReviewedFidelityTrial[];
    readonly bound: BoundedFidelityMatrix;
  },
): FidelityPlan {
  return {
    status: 'planned',
    startedAt,
    pipelineDigest,
    runnerClosure,
    referenceManifestDigest,
    referenceManifest: specs,
    requestedRoster: judgeModelIds,
    completeRequestedMatrix: bound.complete,
    fullMatrixRows: matrix.length,
    plannedRows: bound.planned
      .length,
    rows: bound.planned
      .map(function plannedRowOf(
        row,
        position,
      ): FidelityPlannedRow {
        return {
          position,
          trialId: row.trial
            .trialId,
          direction: row.trial
            .direction,
          cleanFirst: row.trial
            .cleanFirst,
          damageKind: row.trial
            .damageKind,
        };
      },),
  };
}

/**
 One completed row: the reviewed reference's provenance beside what the
 judges said.

 @example
 ```ts
 const result: FidelityRowResult = fidelityRowResult({ row, outcome, },);
 ```
 */
export type FidelityRowResult = FidelityOutcome & {
  /**
   Reviewed reference the row compared.
   */
  readonly referenceId: string;

  /**
   Corpus entry the reference came from.
   */
  readonly entryId: string;

  /**
   Reviewed source range.
   */
  readonly sourceRange: FidelityReferenceSpec['source'];

  /**
   Original archive range.
   */
  readonly archiveRange: FidelityReferenceSpec['archive'];

  /**
   Hash of the reviewed reference text.
   */
  readonly referenceHash: string;

  /**
   Changed-content count the builder reviewed.
   */
  readonly changedChars: number;

  /**
   Safe structural description of the intentional delta.
   */
  readonly damageDetail: string;
};

/**
 Joins a completed trial to the provenance of the reference it ran.

 @param row - the planned row that ran

 @param outcome - what the judges said

 @returns The row as kept, which stays readable if a later row interrupts the run

 @example
 ```ts
 const result = fidelityRowResult({ row, outcome, },);
 ```
 */
export function fidelityRowResult(
  {
    row,
    outcome,
  }: {
    readonly row: ReviewedFidelityTrial;
    readonly outcome: FidelityOutcome;
  },
): FidelityRowResult {
  return {
    referenceId: row.spec
      .id,
    entryId: row.spec
      .entryId,
    sourceRange: row.spec
      .source,
    archiveRange: row.spec
      .archive,
    referenceHash: row.spec
      .referenceHash,
    changedChars: row.changedChars,
    damageDetail: row.damageDetail,
    ...outcome,
  };
}

/**
 Line saying how many rows were judged.

 @param count - rows judged

 @returns The line, without a newline

 @example
 ```ts
 log.info(fidelityJudgedLine({ count: 4, },),);
 ```
 */
export function fidelityJudgedLine({ count, }: { readonly count: number; },): string {
  return `fidelity: ${String(count,)} reviewed trial ${
    wordForCount({
      count,
      one: 'row',
      many: 'rows',
    },)
  }; use individual ballots for per-model calibration`;
}

/**
 Line saying how many rows were kept and where.

 @param count - rows kept

 @param keptAt - path of the kept run

 @returns The line, without a newline

 @example
 ```ts
 log.info(fidelityKeptLine({ count: 4, keptAt: '/runs/x.json', },),);
 ```
 */
export function fidelityKeptLine(
  {
    count,
    keptAt,
  }: {
    readonly count: number;
    readonly keptAt: string;
  },
): string {
  return `kept ${String(count,)} reviewed ${
    wordForCount({
      count,
      one: 'row',
      many: 'rows',
    },)
  } at ${keptAt}`;
}

/**
 Everything a finished run is kept as, apart from the clock and the build's
 identity, which the driver supplies.

 @param plan - the durable plan written before the run

 @param rows - every completed row

 @param finishedAt - when the run ended

 @param promptPayloadDir - where completed payloads were kept

 @param corpusPin - corpus commit the references were read at

 @param entriesRequested - entry filter as typed, empty for every entry

 @param trialCap - cap on attempted rows

 @param damageKinds - defect families requested

 @returns The record `persistProbeRun` writes

 @example
 ```ts
 const run = fidelityKeptRun({ plan, rows, finishedAt, promptPayloadDir, corpusPin, entriesRequested, trialCap, damageKinds, },);
 ```
 */
export function fidelityKeptRun(
  {
    plan,
    rows,
    finishedAt,
    promptPayloadDir,
    corpusPin,
    entriesRequested,
    trialCap,
    damageKinds,
  }: {
    readonly plan: FidelityPlan;
    readonly rows: readonly FidelityRowResult[];
    readonly finishedAt: string;
    readonly promptPayloadDir: string;
    readonly corpusPin: string;
    readonly entriesRequested: readonly string[];
    readonly trialCap: number;
    readonly damageKinds: readonly FidelityDamageKind[];
  },
): ProbeRun {
  return {
    startedAt: plan.startedAt,
    finishedAt,
    pipelineDigest: plan.pipelineDigest,
    runnerClosure: plan.runnerClosure,
    roster: plan.requestedRoster,
    subject: {
      status: 'completed',
      completeRequestedMatrix: plan.completeRequestedMatrix,
      fullMatrixRows: plan.fullMatrixRows,
      plannedRows: plan.plannedRows,
      completedRows: rows.length,
      referenceManifestDigest: plan.referenceManifestDigest,
      promptPayloadDir,
      corpusPin,
      referenceManifest: plan.referenceManifest,
      entriesRequested,
      trialCap,
      damageKinds,
      withContext: false,
    },
    rows,
  };
}

//endregion Judge fidelity probe plan
