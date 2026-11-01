import { SIZE_BANDS, } from '@monochromatic-dev/translation-repair-house-rule/ts';
import type {
  BandedEntry,
  EntryContribution,
} from './draw-entry-load.ts';

//region Draw sample bands
// The lines a draw prints about the size bands of its pool.

/**
 Lines saying, for each size band, how many entries it holds, how many of them
 contribute a candidate, how many issues they bring and how those divide.

 @param entries - every settled entry banded with its candidates

 @returns One `POOL band=` line per size band, in band order

 @example
 ```ts
 for (const line of poolBandLines({ entries, },))
   console.log(line,);
 ```
 */
export function poolBandLines(
  { entries, }: { readonly entries: readonly BandedEntry[]; },
): readonly string[] {
  return SIZE_BANDS.map(function lineOfBand(band,): string {
    /**
     Entries whose size band is the current band.
     */
    const bandEntries = entries.filter(function inBand(entry,) {
      return entry.band === band;
    },);
    /**
     Accepted issues those entries contribute to the pool.
     */
    const bandAccepted = bandEntries.reduce(
      function addCandidates(
        sum,
        entry,
      ) {
        return sum
          + entry.candidates
            .length;
      },
      0,
    );
    /**
     Entries actually contributing a candidate.

     An entry that settled `unchanged` accepts nothing, so it raises the entry
     count while adding no candidate and no spread. Reading readiness off the
     raw count would credit it for coverage it does not provide.
     */
    const contributing = bandEntries.filter(function hasCandidates(
      { candidates, },
    ) {
      return candidates.length > 0;
    },);
    /**
     Per-entry candidate counts, heaviest first.

     Printed because the band totals hide how lopsided a band is: the draw
     round-robins across entries, so a band's spread comes from how many
     entries contribute, not from how many candidates they brought. Seeing the
     shape here is what keeps that distinction from being guessed at.
     */
    const composition = contributing
      .map(function toCount(
        {
          id,
          candidates,
        },
      ): EntryContribution {
        return {
          id,
          count: candidates.length,
        };
      },)
      .toSorted(function byCountDescending(
        a: EntryContribution,
        b: EntryContribution,
      ) {
        return b.count - a.count;
      },)
      .map(function toLabel(
        {
          id,
          count,
        },
      ) {
        return `${id}:${String(count,)}`;
      },);
    return `POOL band=${band} entries=${String(bandEntries.length,)} contributing=${
      String(contributing.length,)
    } accepted=${String(bandAccepted,)} perEntry=${composition.join(',',)}`;
  },);
}

//endregion Draw sample bands
