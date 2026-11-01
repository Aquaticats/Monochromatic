import type { ChunkPair, } from '@monochromatic-dev/translation-repair-slice-reading/ts';
import type { CarriedInsertion, } from '@monochromatic-dev/translation-repair-corpus-source/ts';
import {
  assertCarriedInsertionsRemain,
  assertPageFootnotesIntact,
  type WouldShipSource,
} from '@monochromatic-dev/translation-repair-page-check/ts';

//region Pass page guards
// The guards the pass asks of the would-ship page before it tallies, in one
// place so `pass-entry.ts` keeps to its line budget: the carried insertions
// still carried, and the footnote graph the page composes intact.

/**
 Asks every page-level guard of the would-ship page.

 @param artifact - final stage decisions used for publication

 @param slices - preparation defining replacement spans

 @param targetText - archive text the replacement spans address

 @param carried - source-only passages proven rendered elsewhere before lanes

 @throws {@link TranslationRepairInterruptedError} from whichever guard
 refuses

 @example
 ```ts
 assertPageGuards({ artifact, slices: prepared.slices, targetText: settledArchiveText, carried, },);
 ```
 */
export function assertPageGuards(
  {
    artifact,
    slices,
    targetText,
    carried,
  }: {
    readonly artifact: WouldShipSource;
    readonly slices: readonly ChunkPair[];
    readonly targetText: string;
    readonly carried: readonly CarriedInsertion[];
  },
): void {
  assertCarriedInsertionsRemain({
    artifact,
    slices,
    targetText,
    carried,
  },);
  assertPageFootnotesIntact({
    artifact,
    slices,
    targetText,
  },);
}

//endregion Pass page guards
