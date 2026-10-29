import type { ArchiveOriginalSpan, } from '@monochromatic-dev/translation-repair-corpus-source/ts';
import type { ChunkPair, } from '@monochromatic-dev/translation-repair-slice-reading/ts';
import type { ArtifactPageAssembly, } from '@monochromatic-dev/translation-repair-artifact/ts';
import {
  rowsChangingArchive,
  settlePageRounds,
} from './page-assembly-rounds.ts';
import { shippableReplacements, } from './shippable-replacements.ts';
import type { WouldShipSource, } from './would-ship-text.ts';

//region Page assembly guard
// THE COMPOSED PAGE THROUGH THE SAME GUARD EACH LANE RUNS. What the polish, the
// consolidation and the contest chose per slice is spliced over the archive
// the way the page will be, and the assembly guard trims an orphan definition
// out of a definitions-only slice, withdraws a slice that breaks the footnote
// graph or the parse, and names what it did. The outcome is recorded in the
// artifact (`artifact-two-lane-page-assembly.ts`) rather than acted on here, so
// the page a reader composes from the artifact is the page the guard settled.

/**
 Runs the assembly guard over the page the artifact would ship.

 @param artifact - artifact as composed before the guard, carrying no page
 assembly yet

 @param slices - preparation defining replacement spans

 @param sourceText - original document, whose headings set how many
 distinct headings the page owes (class forty-five)

 @param targetText - archive text the replacement spans address

 @param archiveOriginalSpans - spans sealed as the English original, which
 the Canadian forms pass leaves as the archive has them

 @returns What the guard trimmed, withdrew and found

 @example
 ```ts
 const pageAssembly = guardPageAssembly({ artifact, slices, sourceText, targetText, },);
 ```
 */
export function guardPageAssembly(
  {
    artifact,
    slices,
    sourceText,
    targetText,
    archiveOriginalSpans = [],
  }: {
    readonly artifact: WouldShipSource;
    readonly slices: readonly ChunkPair[];
    readonly sourceText: string;
    readonly targetText: string;
    readonly archiveOriginalSpans?: readonly ArchiveOriginalSpan[];
  },
): ArtifactPageAssembly {
  /**
   Archive text of every slice, by index.
   */
  const incumbentBySlice = new Map(slices.map(function toEntry(slice,) {
    return [
      slice.target
        .sliceIndex,
      slice.target
        .text,
    ] as const;
  },),);
  /**
   Replacements the page would write, less any that repeat the archive's own
   wording.
   */
  const replacements = rowsChangingArchive({
    replacements: shippableReplacements({
      artifact,
      includeSource: false,
    },),
    incumbentBySlice,
  },);
  /**
   Rounds of the passes and the guard until one takes nothing back, each over
   the rows the ones before left (ledger K5).
   */
  const {
    final,
    takenBack,
    earlierFindings,
  } = settlePageRounds({
    slices,
    sourceText,
    targetText,
    replacements,
    archiveOriginalSpans,
    incumbentBySlice,
  },);
  /**
   The settling round's passes and guard.
   */
  const {
    halves,
    passes,
    guarded,
  } = final;
  /**
   Rewritten rows the footnote guard neither trimmed nor withdrew, which ride
   the same override a trimmed slice does: the page carries this text. A slice
   an earlier round took back is among them where a pass rewrote its archive
   text, and that row wins over its withdrawal.
   */
  const restoredOnly = [...passes.restored
    .values(),]
    .filter(function untouchedByGuard(row,): boolean {
      /**
       Whether the guard already owns this slice's override.
       */
      const trimmedByGuard = guarded.trimmed
        .some(function namesIt(trimmed,): boolean {
          return trimmed.sliceIndex === row.sliceIndex;
        },);
      return (!trimmedByGuard) && (!guarded.revertedChunkIndices
        .includes(row.sliceIndex,));
    },);
  return {
    trimmed: [
      ...guarded.trimmed,
      ...restoredOnly,
    ],
    withdrawn: takenBack,
    findings: [
      ...earlierFindings,
      ...halves.findings,
      ...passes.findings,
      ...guarded.findings,
    ],
  };
}

//endregion Page assembly guard
