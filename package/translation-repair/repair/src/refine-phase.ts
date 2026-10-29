import type { Logger, } from '@monochromatic-dev/logger-core/ts';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

import type { ModelClient, } from '@monochromatic-dev/translation-repair-provider/ts';
import type { ChunkPair, } from '@monochromatic-dev/translation-repair-slice-reading/ts';
import {
  collectDefinitions,
  parseDocument,
} from '@monochromatic-dev/translation-repair-shared-contract/ts';
import {
  assertOverlap,
  mapOverlapped,
  type StageSettings,
} from '@monochromatic-dev/translation-repair-stage/ts';
import {
  assertCheckerIndependence,
  assertCheckerQuorumReachable,
  type ChunkRepairOutcome,
  type RepairModels,
  repairReplacements,
  type RepairSliceSeating,
} from '@monochromatic-dev/translation-repair-assembly/ts';
import { inSliceLogContext, } from '@monochromatic-dev/translation-repair-typography/ts';
import { settleRefinePhaseSlice, } from './refine-phase-slice.ts';
import { refineRunShape, } from './refine-slice-key.ts';
import type { RefinedSliceSettlement, } from './refine-slice-settle.ts';
import type { SliceCache, } from '@monochromatic-dev/translation-repair-translate-floor/ts';
import { spliceSlices, } from '@monochromatic-dev/translation-repair-page-pass/ts';

//region Refinement phase
// Naturalness is a second per-slice phase after every accuracy outcome settles.
// It stays outside `repairChunk`: accuracy exits early when no defect validates,
// while awkward but accurate text is this lane's primary input.
//
// THE PHASE CACHES ITS OWN QUESTIONS. Accuracy persists before naturalness runs,
// so sharing its cache would conflate two settlements and omitting this cache
// rebought every rewrite after an accuracy resume. Definitions from assembled
// accuracy text belong to every key because another slice can change references
// this slice's rewriter and guards resolve.
//
// OVERLAP CHANGES WHEN SLICES SETTLE, NEVER AGGREGATION ORDER. Settlements come
// back in outcome order, and `askedRewriters` is true when any freshly bought
// slice reached a rewriter. Resumed slices always report false for this run.

/**
 Outcomes after refinement, with phase telemetry.

 @example
 ```ts
 const result: RefinePhaseResult = {
   outcomes,
   findings: [],
   askedRewriters: true,
 };
 ```
 */
export type RefinePhaseResult = {
  /**
   Final per-slice outcomes in input order.
   */
  readonly outcomes: readonly ChunkRepairOutcome[];

  /**
   Phase telemetry in input-slice order.
   */
  readonly findings: readonly string[];

  /**
   Whether current run asked any rewriter rather than only resuming or finding
   no eligible paragraph.
   */
  readonly askedRewriters: boolean;
};

/**
 Names an accuracy outcome's slice on the lines the overlapped map writes
 about a failure it leaves unthrown.

 @param item - accuracy outcome, named by the slice it settled

 @returns Slice by its index

 @example
 ```ts
 outcomeSliceName({ item: outcome, },); // 'slice 3'
 ```
 */
function outcomeSliceName({ item, }: { readonly item: { readonly sliceIndex: number; }; },): string {
  return `slice ${String(item.sliceIndex,)}`;
}

/**
 Runs naturalness lane over every accuracy-settled slice.

 @param client - injected model client

 @param targetText - archive translation used to assemble accuracy text

 @param slices - prepared pairs in document order

 @param outcomes - accuracy settlements in aggregation order

 @param models - role roster; absent refiners turn lane off

 @param reseat - reads the checker seating as of now, so the recheck and the
 rewrite probe run on the bench a hold that began inside the lane re-seats
 (class one hundred thirteen); the standing seating when absent

 @param identityContext - declared names and handles model prompts preserve

 @param referenceContext - what the pages the original cites say, with
 their rule, when the original cites any (class forty-one)

 @param declaredNames - exact declarations deterministic guard preserves

 @param declaredSubjectForms - declared forms among them that name the subject, which may stand in for one another

 @param refineCache - naturalness namespace for resume and persistence

 @param signal - caller abort honored by exchanges and persistence guard

 @param perCallTimeoutMs - deadline per exchange

 @param overlap - most slices in flight; one reproduces former loop

 @param l - pipeline logger

 @param stageSettings - straggler windows and contest ledger the entry file
 read for this launch (`StageSettings`)

 @returns Final outcomes, ordered findings, and current-run purchase signal

 @throws OverlapRefusedError when overlap is fractional or below one, even
 when lane is configured off

 @throws UnpreparedSliceError when an outcome names no prepared slice

 @throws Whatever model, cache, or caller abort throws

 @example
 ```ts
 const phase = await runRefinePhase({
   client,
   targetText,
   slices,
   outcomes,
   models,
   declaredNames,
   signal,
   perCallTimeoutMs,
   overlap: 4,
   l,
   stageSettings,
 },);
 ```
 */
export async function runRefinePhase(
  {
    client,
    targetText,
    slices,
    outcomes,
    models,
    reseat,
    identityContext,
    referenceContext,
    declaredNames,
    declaredSubjectForms,
    refineCache,
    signal,
    perCallTimeoutMs,
    overlap = 1,
    l,
    stageSettings,
  }: ForeignBorrowed<{
    readonly client: ModelClient;
    readonly targetText: string;
    readonly slices: readonly ChunkPair[];
    readonly outcomes: readonly ChunkRepairOutcome[];
    readonly models: RepairModels;
    readonly reseat?: () => Promise<RepairSliceSeating>;
    readonly identityContext?: string;
    readonly referenceContext?: string;
    readonly declaredNames: readonly string[];
    readonly declaredSubjectForms?: readonly string[];
    readonly refineCache?: SliceCache<RefinedSliceSettlement>;
    readonly signal: AbortSignal;
    readonly perCallTimeoutMs: number;
    readonly overlap?: number;
    readonly l: Logger;
    readonly stageSettings: StageSettings;
  }>,
): Promise<RefinePhaseResult> {
  // A disabled lane must not make invalid caller configuration valid.
  assertOverlap({ overlap, },);

  /**
   Rewriters, empty when lane is configured off.
   */
  const refinerModelIds = models.refinerModelIds ?? [];
  if (refinerModelIds.length === 0) {
    return {
      outcomes,
      findings: [],
      askedRewriters: false,
    };
  }

  assertCheckerIndependence({
    editorModelIds: models.editorModelIds,
    refinerModelIds,
    checkerModelIds: models.checkerModelIds,
    selfCertificationPermitted: models.checkerSelfCertificationPermitted ?? false,
  },);
  assertCheckerQuorumReachable({ checkerModelIds: models.checkerModelIds, },);

  /**
   References from assembled accuracy text, shared by every slice question.
   */
  const definitions = collectDefinitions({
    document: parseDocument({
      text: spliceSlices({
        targetText,
        slices,
        replacements: repairReplacements({ outcomes, },),
      },),
    },),
  },);

  /**
   Model-facing governance shared by every refinement key.
   */
  const runShape = refineRunShape({
    refinerModelIds,
    judgeModelIds: models.judgeModelIds,
    checkerModelIds: models.checkerModelIds,
    ...(identityContext === undefined ? {} : { identityContext, }),
    ...(referenceContext === undefined ? {} : { referenceContext, }),
  },);

  /**
   Every refinement settlement in accuracy-outcome order.
   */
  const settlements = await mapOverlapped({
    items: outcomes,
    overlap,
    signal,
    l,
    nameOf: outcomeSliceName,
    oneItem: async function settleOne({ item: outcome, },) {
      // UNDER THE SLICE'S LOG CONTEXT, as the accuracy pass settles each slice
      // (ledger A11): refinement runs slices side by side under overlap, and
      // its round lines named no slice until ledger L12.
      return await inSliceLogContext({
        lane: 'repair',
        sliceIndex: outcome.sliceIndex,
        run: async function settleRefineInContext() {
          return await settleRefinePhaseSlice({
            client,
            outcome,
            slices,
            models,
            ...((reseat === undefined) ? {} : { reseat, }),
            refinerModelIds,
            runShape,
            definitions,
            ...(identityContext === undefined ? {} : { identityContext, }),
            ...(referenceContext === undefined ? {} : { referenceContext, }),
            declaredNames,
            ...((declaredSubjectForms === undefined) ? {} : { declaredSubjectForms, }),
            ...((refineCache === undefined) ? {} : { refineCache, }),
            signal,
            perCallTimeoutMs,
            l,
            stageSettings,
          },);
        },
      },);
    },
  },);

  return {
    outcomes: settlements.map(function toOutcome(settlement,) {
      return settlement.outcome;
    },),
    findings: settlements.flatMap(function toFindings(settlement,) {
      return settlement.findings;
    },),
    askedRewriters: settlements.some(function askedThisRun(settlement,) {
      return settlement.asked;
    },),
  };
}

//endregion Refinement phase
