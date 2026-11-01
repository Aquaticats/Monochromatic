import {
  bothHalvesInserted,
  type ChunkPair,
  containerHalfPairs,
} from '@monochromatic-dev/translation-repair-slice-reading/ts';
import type { SliceReplacement, } from '@monochromatic-dev/translation-repair-page-pass/ts';

//region Assembly container halves
// A HALF NEVER SHIPS WITHOUT ITS PARTNER. Run before the assembly guard reads
// the page, at both assemblies (the translate lane's and the composed page's),
// so the guard never meets a closing tag whose opening the page does not
// carry. The guard's counterfactual withdraws one slice at a time and XingZ607
// carried two such tags, so nothing it tried repaired the page and it withdrew
// every replacement; naming the pair here withdraws the one half and says
// which slice left it alone.

/**
 Finding prefix a withheld half is recorded under.

 @example
 ```ts
 findings.some((finding) => finding.startsWith(CONTAINER_HALF_WITHHELD_FINDING));
 ```
 */
export const CONTAINER_HALF_WITHHELD_FINDING: string = 'assembly-container-half-withheld';

/**
 One half to withhold, beside the partner slice that ships nothing.
 */
type LoneHalf = {
  /**
   Slice whose replacement is withheld.
   */
  readonly half: number;

  /**
   Slice that ships nothing, leaving the half alone.
   */
  readonly partner: number;
};

/**
 Withholds every container half whose partner half ships nothing.

 Only pairs whose halves are both source-only slices are read: a half the
 archive already carries stands on the archive's own tag when its replacement
 is withheld or absent, so the page keeps its structure either way.

 @param slices - prepared slices, whose source names the pairs

 @param replacements - what the assembly would write, one per changed slice

 @returns Replacements less the withheld halves, their indices and findings

 @example
 ```ts
 const halves = withholdLoneContainerHalves({ slices, replacements, },);
 ```
 */
export function withholdLoneContainerHalves(
  {
    slices,
    replacements,
  }: {
    readonly slices: readonly ChunkPair[];
    readonly replacements: readonly SliceReplacement[];
  },
): {
  readonly replacements: readonly SliceReplacement[];
  readonly withheld: readonly number[];
  readonly findings: readonly string[];
} {
  /**
   Slices the assembly writes text into.
   */
  const written = new Set(replacements
    .filter(function carriesText(replacement,): boolean {
      return replacement.replacementText !== '';
    },)
    .map(function toIndex(replacement,): number {
      return replacement.sliceIndex;
    },),);
  /**
   Halves to withhold, each beside the partner that ships nothing.
   */
  const lone = containerHalfPairs({ slices, },)
    .filter(function inserted(pair,): boolean {
      return bothHalvesInserted({ pair, },);
    },)
    .flatMap(function unpartnered(pair,): readonly LoneHalf[] {
      /**
       Slice index of the opening half.
       */
      const openIndex = pair.open
        .sliceIndex;
      /**
       Slice index of the closing half.
       */
      const closeIndex = pair.close
        .sliceIndex;
      /**
       Whether the opening half's slice writes text.
       */
      const openWritten = written.has(openIndex,);
      /**
       Whether the closing half's slice writes text.
       */
      const closeWritten = written.has(closeIndex,);
      if (openWritten === closeWritten)
        return [];
      if (openWritten)
        return [{
          half: openIndex,
          partner: closeIndex,
        },];
      return [{
        half: closeIndex,
        partner: openIndex,
      },];
    },);
  /**
   Indices withheld, in the order the pairs close.
   */
  const withheld = lone.map(function toIndex(entry,): number {
    return entry.half;
  },);
  return {
    replacements: replacements.filter(function keeps(replacement,): boolean {
      return !withheld.includes(replacement.sliceIndex,);
    },),
    withheld,
    findings: lone.map(function toFinding(entry,): string {
      return `${CONTAINER_HALF_WITHHELD_FINDING} (slice ${String(entry.half,)} beside slice ${
        String(entry.partner,)
      }: one container's halves ship together, and slice ${String(entry.partner,)} ships nothing)`;
    },),
  };
}

//endregion Assembly container halves
