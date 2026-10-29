import type { ArchiveDispute, } from '@monochromatic-dev/translation-repair-archive/ts';
import {
  type Logger,
  tagged,
} from '@monochromatic-dev/logger-core/ts';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

import type { ModelClient, } from '@monochromatic-dev/translation-repair-provider/ts';
import type { PairedReading, } from '@monochromatic-dev/translation-repair-slice-reading/ts';
import {
  admitInsertions,
  type InsertionAdmission,
  type PreparedDocumentPair,
} from '@monochromatic-dev/translation-repair-corpus-source/ts';
import {
  inSliceLogContext,
  sliceTagged,
} from '@monochromatic-dev/translation-repair-typography/ts';
import {
  assertRostersConfigured,
  mapOverlapped,
  type StageSettings,
  type TwinMemo,
} from '@monochromatic-dev/translation-repair-stage/ts';
import { attestedDetailLines, } from '@monochromatic-dev/translation-repair-reference/ts';
import type { SliceCache, } from '@monochromatic-dev/translation-repair-translate-floor/ts';
import { assembleTranslation, } from './translate-assemble.ts';
import type {
  TranslateDocumentResult,
  TranslateModels,
  TranslateSliceRecord,
  TranslateSliceSeating,
  UnfilledSlice,
} from './translate-document-contract.ts';
import { resumeReuseOrBuyTranslateSlice, } from './translate-slice-settle.ts';
import { translateRunShape, } from './translate-slice-key.ts';

//region Translate document
// The lane's document driver: every prepared slice is translated, judged, and
// settled into one record, and the accepted text is assembled back into a whole
// translation.
//
// EVERY SLICE, UNCONDITIONALLY. The repair driver returns early on several
// paths, and those paths are exactly the slices translation exists to recover:
// a fluent but mediocre translation that no critic complains about is never
// repaired and must still be translated. So nothing here decides whether a
// slice is worth visiting.
//
// A run that cannot finish THROWS rather than returning what it has. A result
// reporting unvisited slices as unchanged is indistinguishable from a document
// that needed no translation, and the settled slices are already in the cache
// for the next attempt.
//
// DIRECT-LIBRARY ADMISSION may still return deterministic `not-corroborated`
// unfilled row. Production admission pauses unresolved placement before lanes,
// and absent-passage translation continuously repairs or pauses operationally.
//
// OVERLAP CHANGES WHEN INDEPENDENT SLICES RUN, NOT WHAT THE ARTIFACT SAYS.
// `mapOverlapped` returns settlements in slice order, and every finding comes
// back with its slice, so aggregation is byte-stable at any overlap. The twin
// memo coordinates identical questions already in flight and exposes only
// records the cache would expose to a warm run.

/**
 Names a prepared slice on the lines the overlapped map writes about a failure
 it leaves unthrown.

 @param item - slice pair, named by its placement in the translation

 @returns The slice by its index

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
 Translates every slice of a prepared document pair and reassembles it.

 @param client - injected model client

 @param prepared - slices, governance and declared names, shared with any
 other lane running over same document

 @param models - translator and judge rosters

 @param pictureReadings - what document pictures were read as

 @param signal - entry deadline and caller abort, honored by every exchange

 @param perCallTimeoutMs - deadline per exchange

 @param sliceCache - resumable per-slice cache, absent when caller wants no
 resumption

 @param insertionAdmission - caller's authoritative production evidence for
 source-only slices; absent uses deterministic page shortfall

 @param overlap - most slices in flight; one reproduces former sequential loop

 @param l - pipeline logger

 @param stageSettings - straggler windows and contest ledger the entry file
 read for this launch (`StageSettings`)

 @returns Reassembled translation, its `status`, one settled record per
 filled slice, and every passage this run left missing

 @throws {@link Error} when roster cannot seat a stage, overlap is invalid,
 or caller aborts while this lane is buying

 @throws Whatever `signal.reason` carries once caller aborts with slices still
 unbought; nothing settled under that abort is cached

 @example
 ```ts
 const result = await translateDocument({
   client,
   prepared,
   models,
   signal,
   perCallTimeoutMs,
   overlap: 4,
   l,
   stageSettings,
 },);
 ```
 */
export async function translateDocument(
  {
    client,
    prepared,
    models,
    pictureReadings = new Map(),
    signal,
    perCallTimeoutMs,
    sliceCache,
    insertionAdmission,
    overlap = 1,
    beforeSlice,
    archiveDisputes,
    l,
    stageSettings,
  }: ForeignBorrowed<{
    readonly client: ModelClient;
    readonly prepared: PreparedDocumentPair;
    readonly models: TranslateModels;

    /**
     Slices whose archive rendering the repair lane's adjudicators disputed,
     keyed by slice index; each carries the repair text that stands in for
     the archive as this lane's incumbent (class one hundred seven).
     */
    readonly archiveDisputes?: ReadonlyMap<number, ArchiveDispute>;
    readonly pictureReadings?: ReadonlyMap<string, PairedReading>;
    readonly signal: AbortSignal;
    readonly perCallTimeoutMs: number;
    readonly sliceCache?: SliceCache<TranslateSliceRecord>;
    readonly insertionAdmission?: InsertionAdmission;
    readonly overlap?: number;

    /**
     Awaited before each slice starts, so a caller can hold the slice back
     while a named provider hold keeps the bench from quorum; a roster it
     returns seats that slice (ledger H5, as class one hundred three seats
     the repair lane's chunks).
     */
    readonly beforeSlice?: () => Promise<TranslateSliceSeating>;
    readonly l: Logger;
    readonly stageSettings: StageSettings;
  }>,
): Promise<TranslateDocumentResult> {
  // FIRST, before run shape and cache lookup: a fully cached document must not
  // make an invalid configuration valid. A stage with nobody in it settles
  // exactly like one whose voices all failed, and only one is operator error.
  assertRostersConfigured({
    lane: 'translate',
    roles: {
      translatorModelIds: models.translatorModelIds,
      judgeModelIds: models.judgeModelIds,
    },
  },);

  /**
   Logger tagged with this driver.
   */
  const tl = sliceTagged({
    l: tagged({
      tag: translateDocument.name,
      l,
    },),
  },);

  /**
   Attested lines the translators are shown, none when nothing was attested
   (class thirty-nine).
   */
  const attestedLines = attestedDetailLines({ details: prepared.attestedDetails ?? [], },);
  /**
   What a run on one roster asks, folded into every key it settles, so a
   slice seated on a re-read roster is keyed by the roster that judged it.

   @param seated - roster the slice runs on

   @returns Run shape for that roster
   */
  function shapeFor({ seated, }: { readonly seated: TranslateModels; },): string {
    return translateRunShape({
      models: seated,
      ...((prepared.identityContext === undefined)
        ? {}
        : { identityContext: prepared.identityContext, }),
      lifeEnded: prepared.lifeEnded === true,
      ...((prepared.subjectNames === undefined) ? {} : { subjectNames: prepared.subjectNames, }),
      ...((prepared.referenceContext === undefined)
        ? {}
        : { referenceContext: prepared.referenceContext, }),
      ...((attestedLines.length === 0) ? {} : { attestedLines, }),
    },);
  }

  /**
   What this run asks on the lane's own roster, folded into every key.
   */
  const runShape = shapeFor({ seated: models, },);

  /**
   Slices with no translation beside them that page has room to be missing.

   Computed once because shortfall belongs to whole page rather than one
   section, and spending it per section admits more than page is missing.
   */
  const admission = insertionAdmission ?? {
    positions: admitInsertions({
      slices: prepared.slices,
      sourceText: prepared.sourceText,
      targetText: prepared.targetText,
    },),
    carried: [],
    findings: [],
  };
  /**
   Prepared positions whose source is fully rendered elsewhere, including
   those folded into the neighbour that renders them (class one hundred ten).
   */
  const carriedPositions = new Set([
    ...(admission.carried ?? []),
    ...(admission.folded ?? []),
  ]
    .map(function carriedPosition(carried,): number {
      return carried.position;
    },),);

  /**
   Purchases in this run by shared key, exposing only persisted records.
   */
  const twins: TwinMemo<TranslateSliceRecord> = new Map();

  /**
   Every slice's outcome and findings, returned in slice order.
   */
  const settlements = await mapOverlapped({
    items: prepared.slices,
    overlap,
    signal,
    l: tl,
    nameOf: pairedSliceName,
    oneItem: async function settleOne({
      item: slice,
      position: slicePosition,
    },) {
      return await inSliceLogContext({
        lane: 'translate',
        sliceIndex: slice.target
          .sliceIndex,
        run: async function settleTranslateInContext() {
          /**
           What the hook hands this slice: nothing, or a roster read since
           the lane started.
           */
          const seating: TranslateSliceSeating = (beforeSlice === undefined) ? {} : await beforeSlice();
          /**
           Roster this slice runs on.
           */
          const sliceModels = seating.translateModels ?? models;
          if (seating.translateModels !== undefined) {
            assertRostersConfigured({
              lane: 'translate',
              roles: {
                translatorModelIds: sliceModels.translatorModelIds,
                judgeModelIds: sliceModels.judgeModelIds,
              },
            },);
          }
          /**
           Whether production evidence permits filling this source-only slice.
           */
          const insertionAdmitted = admission
            .positions
            .has(slicePosition,);
          /**
           Whether source-only passage needs no local rendering.
           */
          const insertionCarried = carriedPositions.has(slicePosition,);
          /**
           Dispute over this slice's archive rendering, absent for most slices.
           */
          const archiveDispute = archiveDisputes?.get(slice.target
            .sliceIndex,);
          return await resumeReuseOrBuyTranslateSlice({
            client,
            prepared,
            models: sliceModels,
            slice,
            slicePosition,
            insertionAdmitted,
            insertionCarried,
            pictureReadings,
            runShape: (seating.translateModels === undefined) ? runShape : shapeFor({ seated: sliceModels, },),
            ...((sliceCache === undefined) ? {} : { sliceCache, }),
            twins,
            ...((seating.selectReachable === undefined) ? {} : { selectReachable: seating.selectReachable, }),
            ...((archiveDispute === undefined) ? {} : { archiveDispute, }),
            signal,
            perCallTimeoutMs,
            l: tl,
            stageSettings,
          },);
        },
      },);
    },
  },);

  /**
   Settled records in document order.
   */
  const settled = settlements.flatMap(function toSettled(
    { outcome, },
  ): readonly TranslateSliceRecord[] {
    return (outcome.kind === 'settled')
      ? [outcome.record,]
      : [];
  },);

  /**
   Passages this run reached but left missing, in document order.
   */
  const unfilled = settlements.flatMap(function toUnfilled(
    { outcome, },
  ): readonly UnfilledSlice[] {
    return (outcome.kind === 'unfilled')
      ? [outcome.unfilled,]
      : [];
  },);

  /**
   Slice indexes whose source is already rendered elsewhere.
   */
  const carriedChunkIndices = settlements.flatMap(function toCarried(
    { outcome, },
  ): readonly number[] {
    return (outcome.kind === 'carried')
      ? [outcome.sliceIndex,]
      : [];
  },);

  /**
   Slices recovered from disk, excluding twins reused within this run.
   */
  const resumedSliceCount = settlements
    .filter(function resumedFromDisk({ outcome, },): boolean {
      return (outcome.kind === 'settled') && outcome.resumedFromDisk;
    },)
    .length;

  /**
   Cache refusals and unfilled evidence, grouped in document order.
   */
  const findings = [
    ...admission.findings,
    ...settlements.flatMap(function toFindings(
      settlement,
    ): readonly string[] {
      return [
        ...settlement.refusedCacheFindings,
        ...settlement.unfilledFindings,
      ];
    },),
  ];

  return assembleTranslation({
    prepared,
    settled,
    unfilled,
    carriedChunkIndices,
    resumedSliceCount,
    findings,
    l: tl,
  },);
}

//endregion Translate document
