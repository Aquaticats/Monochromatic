import {
  type Logger,
  tagged,
} from '@monochromatic-dev/logger-core/ts';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

import {
  contextRoot,
  inSliceLogContext,
  sliceTagged,
  wordForCount,
} from '@monochromatic-dev/translation-repair-typography/ts';
import type { ModelClient, } from '@monochromatic-dev/translation-repair-provider/ts';
import type { PreparedDocumentPair, } from '@monochromatic-dev/translation-repair-corpus-source/ts';
import {
  assessNonTranslationDominance,
  nonTranslationDominanceFinding,
  sliceAnchorsTranslation,
} from '@monochromatic-dev/translation-repair-archive/ts';
import {
  assertRostersConfigured,
  mapOverlapped,
  type StageSettings,
  type TwinMemo,
} from '@monochromatic-dev/translation-repair-stage/ts';
import {
  assembleRepair,
  type ChunkRepairOutcome,
  type RepairModels,
  type RepairSliceSeating,
  type RepairTranslationResult,
} from '@monochromatic-dev/translation-repair-assembly/ts';
import { refineSettledSlices, } from './repair-refine-step.ts';
import { repairRunShape, } from './repair-slice-key.ts';
import { settleRepairSlice, } from './repair-slice-settle.ts';
import type { RefinedSliceSettlement, } from './refine-slice-settle.ts';
import type { SliceCache, } from '@monochromatic-dev/translation-repair-translate-floor/ts';


//region Repair translation
// The batch driver over an already prepared document: every slice settles under
// bounded overlap, then dominance is reported, naturalness runs and final text
// is assembled. Everything degrades slice by slice, never document-wide.
//
// NOTHING BLOCKS THE DOCUMENT. Ensemble-agreed critical non-translation used
// to end the run and discard every slice already repaired. Question 3 answer B
// keeps critics as evidence and takes away every early return they owned. See
// `doc/decision/translation-repair-question-answers.md`.
//
// OVERLAP CHANGES WHEN INDEPENDENT SLICES RUN, NOT ARTIFACT ORDER. Each slice
// returns its outcome and driver finding as one settlement; `mapOverlapped`
// returns those settlements in slice order however providers answered.

/**
 Logger root for repair pipeline.
 */
const l = contextRoot({ tag: 'translation-repair-pipeline', },);

/**
 One slice's accuracy settlement beside the slice it settled, so the
 dominance reading takes each outcome with its own slice rather than finding
 it by position.
 */
type SettledSlice = {
  readonly slice: PreparedDocumentPair['slices'][number];
  readonly settlement: Awaited<ReturnType<typeof settleRepairSlice>>;
};

/**
 Names a prepared slice on the lines the overlapped map writes about a failure
 it leaves unthrown.

 @param item - slice pair, named by its placement in the translation

 @returns Slice by its index

 @example
 ```ts
 pairedSliceName({ item: slice, },); // 'slice 3'
 ```
 */
function pairedSliceName(
  { item, }: { readonly item: { readonly target: { readonly sliceIndex: number; }; }; },
): string {
  return `slice ${String(item.target
    .sliceIndex,)}`;
}

/**
 Repairs one already prepared document pair.

 @param client - injected model client

 @param prepared - slices, governance, declared names and alignment findings

 @param models - repair role roster

 @param signal - caller abort honored by every exchange

 @param perCallTimeoutMs - deadline per exchange

 @param sliceCache - optional cross-run cache for accuracy outcomes

 @param refineCache - optional cross-run cache for naturalness settlements

 @param overlap - most repair or refinement slices in flight; one reproduces former loops

 @param parentLogger - logger this lane tags under

 @param stageSettings - straggler windows and contest ledger the entry file
 read for this launch (`StageSettings`)

 @returns Repaired candidate plus adjudicated issues and completion status

 @throws Whatever `signal.reason` carries once caller aborts with slices still
 unbought; nothing settled under that abort is cached

 @example
 ```ts
 const result = await repairPreparedDocument({
   client,
   prepared,
   models,
   signal,
   perCallTimeoutMs,
   overlap: 4,
   stageSettings,
 },);
 ```
 */
export async function repairPreparedDocument(
  {
    client,
    prepared,
    models,
    signal,
    perCallTimeoutMs,
    sliceCache,
    refineCache,
    overlap = 1,
    beforeSlice,
    parentLogger = l,
    stageSettings,
  }: ForeignBorrowed<{
    readonly client: ModelClient;
    readonly prepared: PreparedDocumentPair;
    readonly models: RepairModels;
    readonly signal: AbortSignal;
    readonly perCallTimeoutMs: number;
    readonly sliceCache?: SliceCache<ChunkRepairOutcome>;
    readonly refineCache?: SliceCache<RefinedSliceSettlement>;
    readonly overlap?: number;

    /**
     Awaited before each slice starts, so a caller can hold the slice back
     while a named provider hold keeps the bench from quorum; a roster it
     returns seats that slice, since a bench read once at the lane's start
     ran ten of twelve checker rounds on one voice after a provider ran dry
     inside the lane (class one hundred three, zheermao7, 2026-09-23).
     */
    readonly beforeSlice?: () => Promise<RepairSliceSeating>;
    readonly parentLogger?: Logger;
    readonly stageSettings: StageSettings;
  }>,
): Promise<RepairTranslationResult> {
  // A fully cached document must not make invalid configuration valid. Refiner
  // stays absent because an empty refiner list deliberately turns that lane off.
  assertRostersConfigured({
    lane: 'repair',
    roles: {
      criticModelIds: models.criticModelIds,
      panelModelIds: models.panelModelIds,
      editorModelIds: models.editorModelIds,
      judgeModelIds: models.judgeModelIds,
      checkerModelIds: models.checkerModelIds,
    },
  },);

  /**
   Logger pre-tagged with this function's name.
   */
  const rl = sliceTagged({
    l: tagged({
      tag: repairPreparedDocument.name,
      l: parentLogger,
    },),
  },);

  /**
   Translation under repair.
   */
  const { targetText, } = prepared;

  /**
   Alignment findings in scorecard-stable wording.
   */
  const { alignmentFindings, } = prepared;

  /**
   Paragraph-bound slice pairs in document order.
   */
  const { slices, } = prepared;

  /**
   Slices governed by line-structure rule.
   */
  const lineStructuredSlices = prepared.lineStructuredSliceIndices;
  rl.info(
    `${String(prepared.alignmentPairCount,)} aligned ${
      wordForCount({
        count: prepared.alignmentPairCount,
        one: 'unit',
        many: 'units',
      },)
    }, ${
      String(slices.length,)
    } ${
      wordForCount({
        count: slices.length,
        one: 'slice',
        many: 'slices',
      },)
    }, ${String(alignmentFindings.length,)} alignment ${
      wordForCount({
        count: alignmentFindings.length,
        one: 'finding',
        many: 'findings',
      },)
    }`,
  );

  /**
   Identity context spread into run shape and naturalness call.
   */
  const identityFragment = (prepared.identityContext === undefined)
    ? {}
    : { identityContext: prepared.identityContext, };
  /**
   Reference block as a fragment, absent when the original links nowhere.
   */
  const referenceFragment = (prepared.referenceContext === undefined)
    ? {}
    : { referenceContext: prepared.referenceContext, };

  /**
   Model-facing governance folded into every repair key.
   */
  const runShape = repairRunShape({
    models,
    ...identityFragment,
    ...referenceFragment,
  },);

  /**
   Cache-eligible purchases in this run, shared by every slice.
   */
  const twins: TwinMemo<ChunkRepairOutcome> = new Map();

  /**
   Every accuracy settlement beside the slice it settled, returned in slice
   order, so nothing downstream finds one by position.
   */
  const settled = await mapOverlapped({
    items: slices,
    overlap,
    signal,
    l: rl,
    nameOf: pairedSliceName,
    oneItem: async function settleOne({
      item: slice,
      position: slicePosition,
    },): Promise<SettledSlice> {
      /**
       This slice's settlement.
       */
      const settlement = await inSliceLogContext({
        lane: 'repair',
        sliceIndex: slice.target
          .sliceIndex,
        run: async function settleRepairInContext() {
          /**
           Seating the caller read for this slice, empty when the given roster stands.
           */
          const seating: RepairSliceSeating = (beforeSlice === undefined) ? {} : await beforeSlice();
          return await settleRepairSlice({
            client,
            prepared,
            models: seating.repairModels ?? models,
            ...((beforeSlice === undefined) ? {} : { reseat: beforeSlice, }),
            slice,
            slicePosition,
            runShape,
            ...((sliceCache === undefined) ? {} : { sliceCache, }),
            twins,
            signal,
            perCallTimeoutMs,
            l: rl,
            stageSettings,
          },);
        },
      },);
      return {
        slice,
        settlement,
      };
    },
  },);

  /**
   Accuracy outcomes in document order.
   */
  const outcomes = settled.map(function toOutcome(
    { settlement, },
  ): ChunkRepairOutcome {
    return settlement.outcome;
  },);

  /**
   Cached outcomes refused for contradicting their text, in document order.
   */
  const refusedCacheFindings = settled.flatMap(function toFindings(
    { settlement, },
  ): readonly string[] {
    return settlement.refusedCacheFindings;
  },);

  /**
   Non-translation dominance over whole run, reported and never deciding.
   */
  const dominance = assessNonTranslationDominance({
    slices: settled.map(function toTally(
      {
        slice,
        settlement: { outcome, },
      },
    ) {
      return {
        targetChars: slice.target
          .text
          .length,
        votesStand: outcome.nonTranslationStanding,
        anchorsTranslation: sliceAnchorsTranslation({ outcome, },),
      };
    },),
  },);
  if (dominance.blocked) {
    rl.warn(
      `${nonTranslationDominanceFinding(dominance,)}; reported, not blocking`,
    );
  }

  /**
   Dominance finding, present only when reading crossed.
   */
  const dominanceFindings = dominance.blocked
    ? [nonTranslationDominanceFinding(dominance,),]
    : [];

  /**
   Naturalness lane over every accuracy-settled slice.
   */
  const phase = await refineSettledSlices({
    client,
    targetText,
    slices,
    outcomes,
    models,
    // THE SAME HOOK THE PROOF STAGE READS (class one hundred thirteen), so a
    // hold that began inside the lane re-seats the recheck and the rewrite
    // probe too.
    ...((beforeSlice === undefined) ? {} : { reseat: beforeSlice, }),
    declaredNames: prepared.declaredNames,
    ...((prepared.declaredSubjectForms === undefined) ? {} : { declaredSubjectForms: prepared.declaredSubjectForms, }),
    ...identityFragment,
    ...referenceFragment,
    ...((refineCache === undefined) ? {} : { refineCache, }),
    signal,
    perCallTimeoutMs,
    overlap,
    l: rl,
    stageSettings,
  },);

  /**
   Final outcomes after optional naturalness rewrites.
   */
  const finalOutcomes = phase.outcomes;

  return assembleRepair({
    targetText,
    slices,
    outcomes: finalOutcomes,
    lineStructuredSlices,
    findings: [
      ...alignmentFindings,
      ...refusedCacheFindings,
      ...dominanceFindings,
      ...finalOutcomes.flatMap(function toFindings(outcome,) {
        return outcome.findings;
      },),
      ...phase.findings,
    ],
    l: rl,
  },);
}

//endregion Repair translation
