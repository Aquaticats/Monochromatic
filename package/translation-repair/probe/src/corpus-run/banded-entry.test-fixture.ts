import type { GradingCandidate, } from '@monochromatic-dev/translation-repair-house-rule';
import type { BandedEntry, } from '../../dist/final/node/index.mjs';

//region Banded entry
// ONE SETTLED ENTRY WITH A NUMBER OF CANDIDATES, for the cases of the draw
// that read how a pool divides.
//
// TEST SUPPORT, NOT PACKAGE SOURCE.

/**
 Builds one entry banded as the draw's reader builds it.

 @param id - entry id

 @param band - size band of the entry's source

 @param count - how many accepted issues the entry contributes

 @returns Entry with that many candidates

 @example
 ```ts
 const entry = bandedEntryOf({ id: 'mittens', band: 'small', count: 2, },);
 ```
 */
export function bandedEntryOf(
  {
    id,
    band,
    count,
  }: {
    readonly id: string;
    readonly band: BandedEntry['band'];
    readonly count: number;
  },
): BandedEntry {
  return {
    id,
    band,
    candidates: Array.from(
      { length: count, },
      function candidateAt(
        _unused,
        index,
      ): GradingCandidate {
        return {
          entryId: id,
          band,
          issueId: `${id}/issue-${String(index,)}`,
          category: 'accuracy/omission',
          severity: 'minor',
          summary: 'A purr is dropped from the greeting.',
          sourceAnchor: 'unanchored',
          sourceQuotes: [],
          targetQuotes: [],
        };
      },
    ),
  };
}

//endregion Banded entry
