import type { PageLayout, } from '@monochromatic-dev/translation-repair-consistency/ts';
import type { ChunkPair, } from '@monochromatic-dev/translation-repair-slice-reading/ts';
import type { ArchiveOriginalSpan, } from '@monochromatic-dev/translation-repair-corpus-source/ts';
import type { SliceReplacement, } from '@monochromatic-dev/translation-repair-page-pass/ts';
import type { ArtifactComparisonRow, } from '@monochromatic-dev/translation-repair-artifact-pool/ts';
import type { ArtifactContestSlice, } from '@monochromatic-dev/translation-repair-lane-contest/ts';
import {
  type ArtifactConsolidateSlice,
  NO_PAGE_ASSEMBLY,
} from '@monochromatic-dev/translation-repair-artifact/ts';
import {
  guardPageAssembly,
  shippableReplacements,
  type WouldShipSource,
} from '@monochromatic-dev/translation-repair-page-check/ts';

//region Pass consistency shipped
// WHAT A PAGE SHIPS FOR THE ENGLISH THE PASS READS, BY THE CODE THAT SHIPS IT. The consistency pass's rows enter the page
// inside `wouldShipTextFor` (`would-ship-text.ts`, `consistencyOverrideAt`), BEFORE the typography that
// `wouldShipTextPerSlice` puts every wording the archive did not decide through (`inArchiveTypography`), and BEFORE the
// page-assembly passes that `settledPageArtifact` (`pass-page-assembly.ts`) runs over the page composed with the pass's rows in
// it (`guardPageAssembly`, whose record the publishing reads first). So a mark the pass proves on the lane's English can be
// half of a quotation on the glyphs that ship, and a pass can move or take off a quotation mark beside it. This file writes
// what every slice ships when the slices carry the English a reader gives, by composing the artifact the way the settled page
// does and handing it to the same guard and the same `shippableReplacements` the publishing hands `spliceSlices`.
//
// A SLICE THAT CARRIES THE ENGLISH IT CARRIES NOW HAS NO ROW. The pass records only the slices it changes, so the English of
// a slice that equals the wording it would ship before the pass is left to the stages, as it is on the settled page.

/**
 How many pages of replacements one writer keeps, the latest ones: the pages a pass asks for repeat while the trials
 stand on the same replacements, and none is asked for again once the pass has moved on.
 */
const KEPT_PAGES = 16;

/**
 Builds the writer of what every slice ships, for one page.

 @param prepared - the preparation both lanes ran over, whose spans the replacements address and whose original the passes read

 @param comparison - the comparison row of each slice

 @param contestSlices - what the contest decided

 @param consolidateSlices - what the third rendering settled

 @param raw - the wording each slice that carries English would ship before the pass, by the slice's index

 @returns A writer that gives the replacement of every slice that writes anything for the English handed to it, the same
 list for the same English

 @example
 ```ts
 const shipped = shippedWriterOf({ prepared, comparison, contestSlices, consolidateSlices, raw, },);
 ```
 */
export function shippedWriterOf(
  {
    prepared,
    comparison,
    contestSlices,
    consolidateSlices,
    raw,
  }: {
    readonly prepared: {
      readonly slices: readonly ChunkPair[];
      readonly sourceText: string;
      readonly targetText: string;
      readonly archiveOriginalSpans?: readonly ArchiveOriginalSpan[];
    };
    readonly comparison: readonly ArtifactComparisonRow[];
    readonly contestSlices: readonly ArtifactContestSlice[];
    readonly consolidateSlices: readonly ArtifactConsolidateSlice[];
    readonly raw: ReadonlyMap<number, string>;
  },
): NonNullable<PageLayout['shipped']> {
  /**
   The latest pages written, by the English that made them, mutated only by the writer.
   */
  const kept = new Map<string, readonly SliceReplacement[]>();
  return function shipped(
    { slices, },
  ): readonly SliceReplacement[] {
    /**
     The rows the pass would write: the slices that carry English other than the wording they would ship before it.
     */
    const rows = slices
      .filter(function changesWording({
        sliceIndex,
        english,
      },): boolean {
        return raw.get(sliceIndex,) !== english;
      },)
      .map(function rowOf({
        sliceIndex,
        english,
      },): {
        readonly sliceIndex: number;
        readonly text: string;
      } {
        return {
          sliceIndex,
          text: english,
        };
      },);
    /**
     What names this English.
     */
    const key = JSON.stringify(rows,);
    /**
     The page written for it before, when the writer still holds it.
     */
    const known = kept.get(key,);
    if (known !== undefined)
      return known;
    /**
     The artifact as the stages compose it, with the pass's rows in it and no page assembly yet.
     */
    const composed: WouldShipSource = {
      comparison,
      laneSelection: {
        kind: 'contested',
        slices: contestSlices,
      },
      consolidation: {
        kind: 'settled',
        slices: consolidateSlices,
      },
      preparation: { archiveText: prepared.targetText, },
      pageAssembly: NO_PAGE_ASSEMBLY,
      consistency: {
        groups: [],
        slices: rows,
        findings: [],
      },
    };
    /**
     What the guard made of the page that artifact would ship, which the publishing reads first.
     */
    const pageAssembly = guardPageAssembly({
      artifact: composed,
      slices: prepared.slices,
      sourceText: prepared.sourceText,
      targetText: prepared.targetText,
      archiveOriginalSpans: prepared.archiveOriginalSpans ?? [],
    },);
    /**
     What every slice ships, as the publishing writes it.
     */
    const written = shippableReplacements({
      artifact: {
        ...composed,
        pageAssembly,
      },
    },);
    kept.set(
      key,
      written,
    );
    /**
     The page written longest ago.
     */
    const [oldest,] = kept.keys();
    if ((kept.size > KEPT_PAGES) && (oldest !== undefined))
      kept.delete(oldest,);
    return written;
  };
}

//endregion Pass consistency shipped
