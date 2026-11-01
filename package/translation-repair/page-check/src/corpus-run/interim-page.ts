import type { Logger, } from '@monochromatic-dev/logger-core/ts';

import type { ArtifactConsolidateSlice, } from '@monochromatic-dev/translation-repair-artifact/ts';
import type { ArtifactComparisonRow, } from '@monochromatic-dev/translation-repair-artifact-pool/ts';
import type { ArtifactLaneSelection, } from '@monochromatic-dev/translation-repair-lane-contest/ts';
import type { ArchiveOriginalSpan, } from '@monochromatic-dev/translation-repair-corpus-source/ts';
import type { ChunkPair, } from '@monochromatic-dev/translation-repair-slice-reading/ts';
import { isInsertionChunk, } from '@monochromatic-dev/translation-repair-shared-contract/ts';
import { guardPageAssembly, } from './page-assembly-guard.ts';
import { publishFixedPage, } from './publish-fixed.ts';
import type { DestinationCheck, } from './dropped-destinations.ts';
import type { PublishDefect, } from './publish-defects.ts';
import {
  type WouldShipSource,
  wouldShipTextPerSlice,
} from './would-ship-text.ts';

//region Interim page
// THE PAGE A PASS SHIPS FOR AN ENTRY IT HAS TO LEAVE OPEN. The owner's answer of 2026-10-29 ("Ship interim, resumable"):
// when every provider goes dry in the middle of a page, the run ships the settled slices, plus the archive text or the
// original under its note elsewhere, and keeps the entry open so a later run replaces it (as the owner ruled on
// 2026-10-23 for source-text slices).
//
// NO SECOND ASSEMBLER. The page is composed by the reading every publisher and checker already derives a page from
// (`would-ship-text.ts`): a slice with a consolidation record reads as the stages left it, and a slice without one is
// marked unsettled, which reads as the archive's text (the owner, 2026-09-27, "Keep archive, ship", for a slice with no
// valid wording) and, where the archive has none, as the original under the note that it has not been translated yet
// (`source-ships.ts`, the owner, 2026-10-23). Writing, checking and the content defects are `publishFixedPage`'s.
//
// A SLICE COUNTS AS SETTLED WHEN IT HAS A CONSOLIDATION RECORD (the lead's reading of the owner's answer, recorded in
// ledger B1006). A slice the lanes agreed on or the contest decided, with no record yet, ships as the archive or the
// original, so bought work stays on disk in the caches for the later run and not on the interim page.

/**
 What an interim page was composed from.

 @example
 ```ts
 const stopped: InterimStop = { consolidateSlices: [], };
 ```
 */
export type InterimStop = {
  /**
   Comparison rows of the lanes, present once both lanes returned; absent when the stop came before, where every slice
   reads as neither lane having reached it.
   */
  readonly comparison?: readonly ArtifactComparisonRow[];

  /**
   The contest's records, present once the contest returned; absent before.
   */
  readonly laneSelection?: ArtifactLaneSelection;

  /**
   Consolidation records written before the stop, which are the slices that count as settled.
   */
  readonly consolidateSlices: readonly ArtifactConsolidateSlice[];

  /**
   Slices that read as the archive's text although a consolidation record exists, for a reading of what ships that
   refused them (`UnansweredContestSliceError`, B1222); none when absent.
   */
  readonly keepArchiveAt?: readonly number[];

  /**
   Whether the passages the archive lacks (slices whose archive side is an insertion) ship nothing instead of their
   original under the note. True before the admission of the source-only passages has been read: the archive may already
   render such a passage in a neighbour, so the note and the original would repeat it (B1244). Absent or false once the
   fold of carried passages has run, when a passage that is still a gap is one the lanes owe.
   */
  readonly silenceInsertions?: boolean;
};

/**
 The page already standing where an interim page would go, as far as the pass can tell.

 @example
 ```ts
 const standing: PageStanding = { kind: 'interim', settledSlices: 40, };
 ```
 */
export type PageStanding =
  | {
    /**
     No page stands there.
     */
    readonly kind: 'none';
  }
    | {
    /**
     A settled page stands there, whose artifact is not written (or this attempt wrote it), so it is the better page.
     */
      readonly kind: 'settled';
    }
    | {
    /**
     An interim page stands there that carries this many settled slices.
     */
      readonly kind: 'interim';

    /**
     Slices of it that carry a wording the stages settled.
     */
      readonly settledSlices: number;
    };

/**
 Whether the page standing is better than one that settled this many slices, which then is not written.

 AN INTERIM PAGE NEVER REPLACES A SETTLED PAGE OR AN INTERIM PAGE WITH MORE SETTLED SLICES (B1242, B1243). A later attempt
 can stop earlier than an earlier one, and an attempt that failed after its settled page was written would otherwise be
 followed by one that replaces that page. As many settled slices replaces: the later page is the fresher.

 @param standing - page already there

 @param settledSlices - slices the candidate page settled

 @returns True when the standing page stays

 @example
 ```ts
 const stays = standingStays({ standing: { kind: 'settled', }, settledSlices: 3, },); // => true
 ```
 */
export function standingStays(
  {
    standing,
    settledSlices,
  }: {
    readonly standing: PageStanding;
    readonly settledSlices: number;
  },
): boolean {
  if (standing.kind === 'settled')
    return true;
  return (standing.kind === 'interim') && (settledSlices < standing.settledSlices);
}

/**
 The row of one slice neither lane reached: the archive as it stands, or nothing where the archive has none.

 @param slice - pair the entry was prepared into

 @returns Comparison row that records no lane outcome

 @example
 ```ts
 const row = archiveOnlyRow({ slice, },);
 ```
 */
function archiveOnlyRow({ slice, }: { readonly slice: ChunkPair; },): ArtifactComparisonRow {
  /**
   The translation-side chunk, which says whether the archive renders this slice.
   */
  const { target, } = slice;
  /**
   Whether the archive holds a rendering of this slice, read off the prepared chunk and not off its text.
   */
  const present = !isInsertionChunk(target,);
  /**
   What a document that no lane changed carries here.
   */
  const carried = present ? target.text : '';
  return {
    sliceIndex: target.sliceIndex,
    incumbentKind: present ? 'present' : 'absent',
    incumbentText: target.text,
    repairText: carried,
    translateText: carried,
    laneRelation: present ? 'archive-stands' : 'gap-remains',
    repairOutcome: { kind: 'not-evaluated', },
    translateOutcome: { kind: 'not-evaluated', },
    decisionComparison: {
      kind: 'not-comparable',
      undecidedLanes: [
        'repair',
        'translate',
      ],
    },
    repairDelivery: present ? { kind: 'incumbent-retained', } : { kind: 'gap-remains', },
    translateDelivery: present ? { kind: 'incumbent-retained', } : { kind: 'gap-remains', },
  };
}

/**
 Composes the reading an interim page is written from.

 @param stop - what the entry had settled when the stop came

 @param slices - pairs the entry was prepared into

 @param archiveText - English page the slices address, after preparation

 @returns Source for `publishFixedPage`, every slice without a consolidation record marked unsettled

 @example
 ```ts
 const source = interimSource({ stop, slices, archiveText, },);
 ```
 */
function interimSource(
  {
    stop,
    slices,
    archiveText,
  }: {
    readonly stop: InterimStop;
    readonly slices: readonly ChunkPair[];
    readonly archiveText: string;
  },
): WouldShipSource {
  /**
   Rows of the page, the lanes' where they returned.
   */
  const comparison = stop.comparison
    ?? slices.map(function rowOf(slice,): ArtifactComparisonRow {
      return archiveOnlyRow({ slice, },);
    },);
  /**
   Consolidation records written before the stop.
   */
  const { consolidateSlices, } = stop;
  /**
   Slices the caller keeps at the archive whatever their records say.
   */
  const keptAtArchive = new Set(stop.keepArchiveAt,);
  /**
   Slices a consolidation record settled, less those kept at the archive.
   */
  const settled = new Set(consolidateSlices
    .map(function indexOf({ sliceIndex, },): number {
      return sliceIndex;
    },)
    .filter(function isKept(sliceIndex,): boolean {
      return !keptAtArchive.has(sliceIndex,);
    },),);
  return {
    comparison,
    consolidation: (consolidateSlices.length === 0)
      ? { kind: 'not-run', }
      : {
        kind: 'settled',
        slices: consolidateSlices,
      },
    laneSelection: stop.laneSelection ?? { kind: 'pending-human-decision', },
    preparation: { archiveText, },
    lanes: {
      translate: {
        // A SLICE WITH NO RECORD OF ITS ORIGINAL READS AS SILENT (`withoutSourceRecordAt`), so a source-only passage
        // whose admission nobody has read ships the archive's blank and not the original under the note (B1244).
        delivery: slices
          .filter(function carriesOriginal({ target, },): boolean {
            return !((stop.silenceInsertions === true) && isInsertionChunk(target,));
          },)
          .map(function sourceOf({
            source,
            target,
          },) {
            return {
              sliceIndex: target.sliceIndex,
              sourceText: source.text,
            };
          },),
      },
    },
    interim: {
      unsettledSlices: new Set(comparison
        .map(function indexOf({ sliceIndex, },): number {
          return sliceIndex;
        },)
        .filter(function isUnsettled(sliceIndex,): boolean {
          return !settled.has(sliceIndex,);
        },),),
    },
  };
}

/**
 How many slices of an interim page carry each kind of text.

 @example
 ```ts
 const counts: InterimCounts = { settledSlices: 6, archiveSlices: 3, sourceSlices: 1, };
 ```
 */
export type InterimCounts = {
  /**
   Slices that carry a wording the stages settled.
   */
  readonly settledSlices: number;

  /**
   Slices that carry the archive's own text, a blank archive span included.
   */
  readonly archiveSlices: number;

  /**
   Slices that carry the original under the note that it has not been translated yet.
   */
  readonly sourceSlices: number;
};

/**
 Counts what each slice of an interim page carries, off the reading the page was written from.

 @param source - source `interimSource` composed

 @param keptArchiveAt - slices the page-grammar fallback put back to the archive, which the page then carries as the
 archive has them whatever the source says (B1246); none for a page that parsed as composed

 @param sourceDroppedAt - slices whose original the composition left off the page because no writing of it made the page
 parse, which the page then carries as the archive has them, a blank span where the archive has none (B1300); none for
 a page that kept every original

 @returns Three counts, which add up to the number of slices

 @throws {@link UnansweredContestSliceError} when the lanes differ at a settled slice the contest never answered

 @example
 ```ts
 const { settledSlices, archiveSlices, sourceSlices, } = interimCountsOf({ source, keptArchiveAt: [], sourceDroppedAt: [], },);
 ```
 */
function interimCountsOf(
  {
    source,
    keptArchiveAt,
    sourceDroppedAt,
  }: {
    readonly source: WouldShipSource;
    readonly keptArchiveAt: readonly number[];
    readonly sourceDroppedAt: readonly number[];
  },
): InterimCounts {
  /**
   Slices the page carries as the archive has them because the grammar refused their wording or no writing of their
   original made the page parse.
   */
  const putBack = new Set([
    ...keptArchiveAt,
    ...sourceDroppedAt,
  ],);
  /**
   What each slice would carry.
   */
  const readings = wouldShipTextPerSlice({ artifact: source, },);
  /**
   The slices nothing settled, none for a source composed with every slice decided.
   */
  const unsettled = source.interim
    ?.unsettledSlices
    ?? new Set<number>();
  /**
   Readings of the slices that carry the original.
   */
  const originals = readings.filter(function carriesOriginal({
    sliceIndex,
    reading,
  },): boolean {
    return (reading.kind === 'source-ships') && (!putBack.has(sliceIndex,));
  },);
  /**
   Readings of the slices that carry the archive's text, or nothing where the archive has none to carry.
   */
  const archives = readings.filter(function carriesArchive({
    sliceIndex,
    reading,
  },): boolean {
    if (putBack.has(sliceIndex,))
      return true;
    if (reading.kind === 'source-ships')
      return false;
    return unsettled.has(sliceIndex,) || ((reading.kind === 'wording') && (reading.decidedBy === 'archive'));
  },);
  /**
   Slices of the page that carry the original, and those that carry the archive.
   */
  const unsettledCount = originals.length + archives.length;
  return {
    settledSlices: readings.length - unsettledCount,
    archiveSlices: archives.length,
    sourceSlices: originals.length,
  };
}

/**
 What shipping an interim page left behind.

 @example
 ```ts
 const shipped: InterimShip = await publishInterimPage({ stop, slices, archiveText, sourceText, entryId, publishDir, l, },);
 ```
 */
export type InterimShip = InterimCounts & {
  /**
   Page as written, which the record's digest is taken over.
   */
  readonly pageText: string;

  /**
   What the page carries of the source's destinations.
   */
  readonly destinations: DestinationCheck;

  /**
   The content checks the page ships failing.
   */
  readonly defects: readonly PublishDefect[];

  /**
   Whether the page was written. False where a better page stands (`standingStays`), when `pageText` and the counts
   describe the page that was composed and not one on disk.
   */
  readonly written: boolean;
};

/**
 Writes the interim page of an entry stopped in the middle of its page.

 @param stop - what the entry had settled when the stop came

 @param slices - pairs the entry was prepared into, carrying the spans to write into

 @param archiveText - whole archive English the slices address, after preparation

 @param sourceText - whole source page, read for the destinations it links to

 @param entryId - person entry being published

 @param publishDir - root of the mirrored tree

 @param l - logger, tagged by the caller with this entry

 @param archiveOriginalSpans - spans of the archive sealed as the English original, which the page carries byte for byte

 @param readGrammar - reading of a page by the grammar, the package's by default; a case plants another to make a page
 the strict grammar refuses

 @param standing - page already standing where this one would go, none by default; a settled page, or an interim page
 with more settled slices than this one, is left as it is and `written` says so (B1242, B1243)

 @returns Page as written, what each slice of it carries, and what it carries of the destinations

 @example
 ```ts
 const shipped = await publishInterimPage({ stop, slices, archiveText, sourceText, entryId, publishDir, l, },);
 ```
 */
export async function publishInterimPage(
  {
    stop,
    slices,
    archiveText,
    sourceText,
    entryId,
    publishDir,
    l,
    archiveOriginalSpans = [],
    readGrammar,
    standing = { kind: 'none', },
  }: {
    readonly stop: InterimStop;
    readonly slices: readonly ChunkPair[];
    readonly archiveText: string;
    readonly sourceText: string;
    readonly entryId: string;
    readonly publishDir: string;
    readonly l: Logger;
    readonly archiveOriginalSpans?: readonly ArchiveOriginalSpan[];
    readonly readGrammar?: Parameters<typeof publishFixedPage>[0]['readGrammar'];
    readonly standing?: PageStanding;
  },
): Promise<InterimShip> {
  /**
   The reading before the page-level guard has weighed its page.
   */
  const unguarded = interimSource({
    stop,
    slices,
    archiveText,
  },);
  /**
   What the guard made of the page that reading would ship.
   */
  const pageAssembly = guardPageAssembly({
    artifact: unguarded,
    slices,
    sourceText,
    targetText: archiveText,
    archiveOriginalSpans,
  },);
  for (const finding of pageAssembly.findings)
    l.warn(`page assembly: ${finding}`,);
  /**
   The reading with the guard's outcome recorded, the one the page is written from.
   */
  const source: WouldShipSource = {
    ...unguarded,
    pageAssembly,
  };
  /**
   What writing the page reported.
   */
  const published = await publishFixedPage({
    artifact: source,
    slices,
    archiveText,
    sourceText,
    entryId,
    publishDir,
    l,
    archiveOriginalSpans,
    ...((readGrammar === undefined) ? {} : { readGrammar, }),
    // ASKED AFTER THE GRAMMAR HAS PUT SLICES BACK AND THE COMPOSITION HAS LEFT ORIGINALS OFF, so the settled slices counted
    // are those of the page that would ship.
    mayWrite: function mayReplaceStanding(
      {
        keptArchiveAt,
        sourceDroppedAt,
      },
    ): boolean {
      return !standingStays({
        standing,
        settledSlices: interimCountsOf({
          source,
          keptArchiveAt,
          sourceDroppedAt,
        },)
          .settledSlices,
      },);
    },
  },);
  return {
    ...interimCountsOf({
      source,
      keptArchiveAt: published.keptArchiveAt,
      sourceDroppedAt: published.sourceDroppedAt,
    },),
    pageText: published.pageText,
    destinations: published.destinations,
    defects: published.defects,
    written: published.written,
  };
}

//endregion Interim page
