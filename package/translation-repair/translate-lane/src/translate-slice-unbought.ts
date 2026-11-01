import {
  type Logger,
  tagged,
} from '@monochromatic-dev/logger-core/ts';
import { absenceFinding, } from '@monochromatic-dev/translation-repair-translate-floor/ts';
import type { TranslateSliceSettlement, } from './translate-slice-settle.ts';

//region Slices settled without a purchase
// Two endings of a slice that are known before any provider is asked, kept apart from the settlement that chooses
// between them so that file states the order in which a slice is resumed, reused or bought.

/**
 Settles a source-only passage that another slice already renders in full.

 @param sliceIndex - slice within the entry

 @param l - lane logger

 @returns Settlement that carries the passage and reports nothing

 @example
 ```ts
 const settlement = carriedElsewhere({ sliceIndex: 3, l, },);
 ```
 */
export function carriedElsewhere(
  {
    sliceIndex,
    l,
  }: {
    readonly sliceIndex: number;
    readonly l: Logger;
  },
): TranslateSliceSettlement {
  /**
   Logger pre-tagged with this function's name.
   */
  const cl = tagged({
    tag: carriedElsewhere.name,
    l,
  },);

  cl.info(`slice ${String(sliceIndex,)}: source passage already carried elsewhere`,);
  return {
    outcome: {
      kind: 'carried',
      sliceIndex,
    },
    refusedCacheFindings: [],
    unfilledFindings: [],
  };
}

/**
 Settles a source-only passage the page has no room to be missing.

 NOTHING IS BOUGHT HERE. The pairing says this original went unrendered, but the page carries at least as much English
 as its source predicts, so the likelier reading is that the passage was merged into a neighbour and writing it in
 would put a second rendering of it into the document.

 @param sliceIndex - slice within the entry

 @param l - lane logger

 @returns Unfilled settlement that names the reason

 @example
 ```ts
 const settlement = notCorroborated({ sliceIndex: 3, l, },);
 ```
 */
export function notCorroborated(
  {
    sliceIndex,
    l,
  }: {
    readonly sliceIndex: number;
    readonly l: Logger;
  },
): TranslateSliceSettlement {
  /**
   Logger pre-tagged with this function's name.
   */
  const nl = tagged({
    tag: notCorroborated.name,
    l,
  },);

  nl.warn(
    `slice ${String(sliceIndex,)}: an original with no translation beside it, but the page `
      + 'has no room to be missing it; nothing was bought and the passage stays as it is',
  );
  return {
    outcome: {
      kind: 'unfilled',
      unfilled: {
        sliceIndex,
        reason: 'not-corroborated',
        findings: [],
      },
    },
    refusedCacheFindings: [],
    unfilledFindings: [
      `${absenceFinding({ reason: 'not-corroborated', },)} chunk ${String(sliceIndex,)}`,
    ],
  };
}

//endregion Slices settled without a purchase
