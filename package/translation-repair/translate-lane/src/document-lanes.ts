import {
  archiveDisputesOfRepair,
  logArchiveDisputes,
} from '@monochromatic-dev/translation-repair-archive/ts';
import {
  type Logger,
  tagged,
} from '@monochromatic-dev/logger-core/ts';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

import type { ModelClient, } from '@monochromatic-dev/translation-repair-provider/ts';
import { logAlignmentFindings, } from './alignment-findings-log.ts';
import type {
  ChunkPair,
  PairedReading,
} from '@monochromatic-dev/translation-repair-slice-reading/ts';
import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';
import {
  assertDeliveryAgreesWithDocument,
  type IdentifiedDeliveryLedger,
} from '@monochromatic-dev/translation-repair-lane-contest/ts';
import {
  type InsertionAdmission,
  preparationIdentity,
  type PreparedDocumentPair,
} from '@monochromatic-dev/translation-repair-corpus-source/ts';
import type {
  ChunkRepairOutcome,
  RepairModels,
  RepairSliceSeating,
  RepairTranslationResult,
} from '@monochromatic-dev/translation-repair-assembly/ts';
import {
  type RefinedSliceSettlement,
  repairPreparedDocument,
} from '@monochromatic-dev/translation-repair-repair/ts';
import {
  assertRostersConfigured,
  type StageSettings,
} from '@monochromatic-dev/translation-repair-stage/ts';
import {
  buildSliceDelivery,
  type LaneSliceText,
  type SliceCache,
  type SliceDeliveryRecord,
} from '@monochromatic-dev/translation-repair-translate-floor/ts';
import type { SliceReplacement, } from '@monochromatic-dev/translation-repair-page-pass/ts';
import type {
  TranslateDocumentResult,
  TranslateModels,
  TranslateSliceRecord,
  TranslateSliceSeating,
} from './translate-document-contract.ts';
import { translateDocument, } from './translate-document.ts';

//region Document lanes
// Both lanes over ONE preparation, arbitrating nothing.
//
// The two lanes answer different questions about the same document. The repair
// lane keeps the archive's English and mends it where critics find defects; the
// translate lane renders every slice afresh from the Chinese and lets judges
// choose between what it wrote and what the archive already had. Which of those
// two documents should ship is Question 5 in
// `doc/planning/translation-repair-open-decisions.md`, and it is the user's to
// answer.
//
// So this driver returns both and picks neither. It exists to make the two
// comparable: one preparation means one slicing, one alignment and one identity
// block, so a difference between the outputs is a difference between the lanes
// rather than between two runs of the aligner.
//
// It costs both lanes. A caller that wants one lane calls that lane.

/**
 What running both lanes over one preparation produced.

 Deliberately has no winner, no preferred lane and no merged text. Adding one
 would answer Question 5 in code, and it would answer it invisibly: every
 later count would inherit the choice without anything recording that a choice
 had been made.

 @example
 ```ts
 const lanes: DocumentLanesResult = await runDocumentLanes({ ... },);
 ```
 */
export type DocumentLanesResult = {
  /**
   Alignment findings from the shared preparation, reported once.

   They belong to the preparation rather than to either lane, and both lanes
   ran over the same one, so counting them per lane would count one defect in
   the archive twice. The repair result repeats them inside its own findings,
   which is that lane's existing contract and is left alone here.
   */
  readonly alignmentFindings: readonly string[];

  /**
   What the repair lane returned, exactly as it returned it.
   */
  readonly repair: RepairTranslationResult;

  /**
   What the translate lane returned, exactly as it returned it.
   */
  readonly translate: TranslateDocumentResult;

  /**
   Every prepared slice as the repair lane delivered it.

   DERIVED, not decided: one row per slice saying what was translated, what
   the archive had, what the lane chose and what its document carries. Adding
   it answers no question about which lane wins, since each ledger describes
   its own lane's document and neither mentions the other.

   CARRIES THE SLICING IT WAS BUILT OVER, stamped here rather than by whoever
   reads it later. This driver is the only place that holds the preparation
   and the rows at once; a name applied downstream is the applier's claim
   about the rows, and a consumer that stamps both ledgers itself makes every
   later identity check agree with itself by construction.
   */
  readonly repairDelivery: IdentifiedDeliveryLedger;

  /**
   Every prepared slice as the translate lane delivered it.
   */
  readonly translateDelivery: IdentifiedDeliveryLedger;
};

/**
 Builds one lane's ledger and checks it against that lane's own document.

 CHECKED HERE rather than trusted, because the ledger joins what the lane
 DECIDED to what its index sets say the document CARRIES, and those are two
 derivations. They agree by construction today; a driver holding the document
 is the first place able to say so rather than assume it.

 @param slices - preparation both lanes ran over

 @param incumbentText - archive's own translation, which both lanes wrote into

 @param documentText - text this lane returned

 @param wordings - what this lane decided per slice

 @param changedSliceIndices - slices this lane's document carries a change for

 @param withdrawnSliceIndices - slices whose change assembly took back

 @param trimmedReplacements - shipped slices whose text the assembly guard
 trimmed, with the text the document carries

 @param blocked - whether this lane refused the whole document before assembly

 @returns One row per prepared slice, in document order

 @throws {@link SliceDeliveryError} when the lane's own reports cannot
 describe one delivery, and {@link DeliveryInvariantError} when the rows do
 not describe the returned document

 @example
 ```ts
 const ledger = laneDelivery({ slices, incumbentText, documentText, wordings, ... },);
 ```
 */
function laneDelivery(
  {
    slices,
    incumbentText,
    documentText,
    wordings,
    changedSliceIndices,
    withdrawnSliceIndices,
    trimmedReplacements,
    blocked,
  }: {
    readonly slices: readonly ChunkPair[];
    readonly incumbentText: string;
    readonly documentText: string;
    readonly wordings: readonly LaneSliceText[];
    readonly changedSliceIndices: readonly number[];
    readonly withdrawnSliceIndices: readonly number[];
    readonly trimmedReplacements: readonly SliceReplacement[];
    readonly blocked: boolean;
  },
): readonly SliceDeliveryRecord[] {
  /**
   Rows joining this lane's three reports, one per prepared slice.
   */
  const ledger = buildSliceDelivery({
    slices,
    wordings,
    changedSliceIndices,
    withdrawnSliceIndices,
    trimmedReplacements,
    blocked,
  },);
  assertDeliveryAgreesWithDocument({
    ledger,
    slices,
    incumbentText,
    documentText,
    changedSliceIndices,
  },);
  return ledger;
}

/**
 Runs both lanes over one prepared pair and returns both outputs.

 Sequential rather than concurrent, and repair first. Concurrency would put
 two fanned-out stages over the same provider capacity, which buys nothing:
 the quota spent is identical and both lanes already serialize their own
 slices for that reason. Repair runs first because its naturalness lane
 settles AFTER the slice loop and nothing persists what it produced, while the
 translate lane caches every slice as it finishes. Under a deadline that cuts
 the entry, running the uncheckpointed phase first is the order that loses
 less of what was bought.

 No abort check sits between the lanes on purpose. Both drivers let a fully
 cached lane finish after an abort, because resuming cached slices buys
 nothing, and a gate here would refuse that.

 @param client - injected model client, shared by both lanes

 @param prepared - one preparation both lanes read

 @param repairModels - roster for the repair lane

 @param translateModels - roster for the translate lane

 @param reseatTranslate - re-reads the translate lane's roster when the lane
 is about to start, since the repair lane has spent minutes by then and a
 provider may have been held out meanwhile (the thirteenth class); the
 roster given is used when absent

 @param beforeSlice - awaited before each slice of either lane starts, told
 which lane, so a caller can hold the slice back while a named provider hold
 keeps that lane's bench from quorum (the thirteenth class's second face);
 a roster it returns for the repair lane seats that slice (class one hundred
 three, where the checker bench read at the lanes boundary ran on one voice
 after a dry-out inside the lane), and one for the translate lane seats that
 lane's slice (ledger H5)


 @param signal - entry abort both lanes honor

 @param perCallTimeoutMs - deadline per exchange, passed to both lanes rather
 than left to each lane's default, which differ

 @param overlap - most slices either lane keeps in flight; lanes remain
 sequential relative to each other

 @param repairSliceCache - repair lane's cache, in its own namespace

 @param refineSliceCache - naturalness lane's cache, in its own namespace; it
 runs after the repair lane has persisted, so it cannot share that one

 @param translateSliceCache - translate lane's cache, in its own namespace

 @param translateInsertionAdmission - production evidence deciding which
 source-only slices translation may fill

 @param l - logger both lanes tag under, so one entry reads as one run

 @param stageSettings - straggler windows and contest ledger the entry file
 read for this launch (`StageSettings`)

 @returns Both lane results and the preparation's alignment findings

 @throws Whatever either lane throws, the caller's abort reason included; a
 failure in the first lane means the second never runs

 @example
 ```ts
 const lanes = await runDocumentLanes({
   client,
   prepared,
   repairModels,
   translateModels,
   signal,
   perCallTimeoutMs,
   l,
   stageSettings,
 },);
 ```
 */
export async function runDocumentLanes(
  {
    client,
    prepared,
    repairModels,
    translateModels,
    reseatTranslate,
    beforeSlice,
    pictureReadings,
    signal,
    perCallTimeoutMs,
    overlap = 1,
    repairSliceCache,
    refineSliceCache,
    translateSliceCache,
    translateInsertionAdmission,
    l,
    stageSettings,
  }: ForeignBorrowed<{
    readonly client: ModelClient;
    readonly prepared: PreparedDocumentPair;
    readonly repairModels: RepairModels;
    readonly translateModels: TranslateModels;
    readonly reseatTranslate?: () => Promise<TranslateModels>;
    readonly beforeSlice?: (
      args: { readonly lane: 'repair' | 'translate'; },
    ) => Promise<RepairSliceSeating & TranslateSliceSeating>;

    /**
     What each of this document's pictures was read as, gathered before either
     lane starts.

     THE TRANSLATE LANE ONLY, today. The repair lane edits the archive in
     place against critic claims, and none of its stages asks what a picture
     says; the translate lane writes each slice fresh from the source, which is
     where a passage transcribing a picture has no source without this.
     */
    readonly pictureReadings?: ReadonlyMap<string, PairedReading>;
    readonly signal: AbortSignal;
    readonly perCallTimeoutMs: number;
    readonly overlap?: number;
    readonly repairSliceCache?: SliceCache<ChunkRepairOutcome>;
    readonly refineSliceCache?: SliceCache<RefinedSliceSettlement>;
    readonly translateSliceCache?: SliceCache<TranslateSliceRecord>;
    readonly translateInsertionAdmission?: InsertionAdmission;
    readonly l: Logger;
    readonly stageSettings: StageSettings;
  }>,
): Promise<DocumentLanesResult> {
  // BOTH rosters before EITHER lane starts, in ONE check. Each driver checks
  // its own, which is what makes the check unbypassable, but repair runs first
  // and would otherwise spend an entire document before an unconfigured
  // translate roster was discovered. Neither of these is the enforcement; both
  // are the courtesy.
  //
  // ONE call rather than two, because two stop at the repair lane: an operator
  // with a role empty in EACH lane would pay a preflight per lane to learn
  // that, which is the cost this check exists to spare them.
  //
  // The role names carry their lane, and that is required rather than tidy.
  // Both lanes have a `judgeModelIds`, one object cannot hold that key twice,
  // and an unprefixed map would quietly report on one lane's judges only.
  assertRostersConfigured({
    lane: 'repair and translate',
    roles: {
      'repair.criticModelIds': repairModels.criticModelIds,
      'repair.panelModelIds': repairModels.panelModelIds,
      'repair.editorModelIds': repairModels.editorModelIds,
      'repair.judgeModelIds': repairModels.judgeModelIds,
      'repair.checkerModelIds': repairModels.checkerModelIds,
      'translate.translatorModelIds': translateModels.translatorModelIds,
      'translate.judgeModelIds': translateModels.judgeModelIds,
    },
  },);

  /**
   Logger tagged with this driver, which both lanes then tag under.
   */
  const dl = tagged({
    tag: runDocumentLanes.name,
    l,
  },);
  dl.info(
    `both lanes over ${String(prepared.slices
      .length,)} ${
      wordForCount({
        count: prepared.slices
          .length,
        one: 'slice',
        many: 'slices',
      },)
    }, repair first`,
  );
  logAlignmentFindings({
    findings: prepared.alignmentFindings,
    l: dl,
  },);

  /**
   Repair lane's answer: the archive's English, mended where critics found
   defects.
   */
  const repair = await repairPreparedDocument({
    client,
    prepared,
    models: repairModels,
    signal,
    perCallTimeoutMs,
    overlap,
    ...((repairSliceCache === undefined)
      ? {}
      : { sliceCache: repairSliceCache, }),
    ...((refineSliceCache === undefined)
      ? {}
      : { refineCache: refineSliceCache, }),
    ...((beforeSlice === undefined)
      ? {}
      : {
        beforeSlice: async function beforeRepairSlice(): Promise<RepairSliceSeating> {
          /**
           What the hook hands this chunk, either lane's roster in it.
           */
          const seating = await beforeSlice({ lane: 'repair', },);
          return (seating.repairModels === undefined) ? {} : { repairModels: seating.repairModels, };
        },
      }),
    parentLogger: dl,
    stageSettings,
  },);

  /**
   Translate lane's roster as of now: re-read when the caller can, since the
   repair lane has just spent minutes and a provider held out meanwhile
   would refuse every writer it serves in the same millisecond.
   */
  const translateSeats = (reseatTranslate === undefined) ? translateModels : await reseatTranslate();
  if (reseatTranslate !== undefined) {
    /**
     Writers the re-seating kept, for the line.
     */
    const writers = translateSeats.translatorModelIds
      .join(', ',);
    dl.info(`translate lane re-seated before starting: ${writers} writing`,);
  }

  /**
   Slices whose archive rendering the repair lane's adjudicators found to
   add what the original never states; the repair text stands in for the
   archive on the translate lane (class one hundred seven).
   */
  const archiveDisputes = archiveDisputesOfRepair({ repair, },);
  logArchiveDisputes({
    disputes: archiveDisputes,
    l: dl,
  },);

  /**
   Translate lane's answer: every slice rendered afresh, with the archive's
   own English standing as one candidate.
   */
  const translate = await translateDocument({
    client,
    prepared,
    models: translateSeats,
    ...((archiveDisputes.size === 0) ? {} : { archiveDisputes, }),
    ...((pictureReadings === undefined)
      ? {}
      : { pictureReadings, }),
    signal,
    perCallTimeoutMs,
    overlap,
    ...((translateSliceCache === undefined)
      ? {}
      : { sliceCache: translateSliceCache, }),
    ...((translateInsertionAdmission === undefined)
      ? {}
      : { insertionAdmission: translateInsertionAdmission, }),
    ...((beforeSlice === undefined)
      ? {}
      : {
        beforeSlice: async function beforeTranslateSlice(): Promise<TranslateSliceSeating> {
          /**
           What the hook hands this slice, either lane's roster in it.
           */
          const seating = await beforeSlice({ lane: 'translate', },);
          return {
            ...((seating.translateModels === undefined) ? {} : { translateModels: seating.translateModels, }),
            ...((seating.selectReachable === undefined) ? {} : { selectReachable: seating.selectReachable, }),
          };
        },
      }),
    l: dl,
    stageSettings,
  },);
  dl.info(
    `repair ${repair.status}, translate changed ${
      String(translate.changedSliceCount,)
    }/${String(prepared.slices
      .length,)} ${
      wordForCount({
        count: prepared.slices
          .length,
        one: 'slice',
        many: 'slices',
      },)
    }; neither was chosen over the other`,
  );

  /**
   Name the slicing both lanes ran over gives itself, computed once and
   stamped onto both ledgers.

   ONE COMPUTATION FOR TWO LEDGERS is exactly right here and would be wrong
   anywhere else: both were built from `prepared` in this function, so one
   name is a fact rather than an assumption. A later consumer holding two
   ledgers cannot say that, which is why the name travels with them.
   */
  const slicing = preparationIdentity({ prepared, },);

  return {
    alignmentFindings: prepared.alignmentFindings,
    repair,
    translate,
    repairDelivery: {
      preparationIdentity: slicing,
      records: laneDelivery({
        slices: prepared.slices,
        incumbentText: prepared.targetText,
        documentText: repair.repairedText,
        wordings: repair.sliceTexts,
        changedSliceIndices: repair.changedSliceIndices,
        withdrawnSliceIndices: repair.withdrawnSliceIndices,
        trimmedReplacements: repair.trimmedReplacements,
        // The dominance refusal, which returns the archive untouched however
        // many slices had decided a repair by the time it fired. Without this
        // the rows for those slices would read as a contradiction rather than
        // as what they are: decisions the document was refused before it could
        // carry.
        blocked: repair.status === 'blocked-non-translation',
      },),
    },
    translateDelivery: {
      preparationIdentity: slicing,
      records: laneDelivery({
        slices: prepared.slices,
        incumbentText: prepared.targetText,
        documentText: translate.translatedText,
        wordings: translate.sliceTexts,
        changedSliceIndices: translate.changedSliceIndices,
        withdrawnSliceIndices: translate.withdrawnSliceIndices,
        trimmedReplacements: translate.trimmedReplacements,
        // The translate lane has no whole-document refusal: it assembles what
        // its slices decided, and the only thing that takes a decision back is
        // the assembly guard, which the withdrawn set already names.
        blocked: false,
      },),
    },
  };
}

//endregion Document lanes
