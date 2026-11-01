import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';
import type { ModelReading, } from './image-reading-pair.ts';
import { trigramOverlap, } from '@monochromatic-dev/translation-repair-markup/ts';

//region Most carried reading

/**
 One reading with how much of it the other readings carry.
 */
type CarriedReading = {
  /**
   Reading scored.
   */
  readonly reading: ModelReading;

  /**
   Mean trigram overlap with every other reading, zero for a lone reading.
   */
  readonly carried: number;
};

/**
 Picks the one transcript of a corroborated picture that the other readers
 carry most: the reading whose mean trigram overlap with every other reading
 is highest, the longer one on a tie (the overlap divides by the smaller
 side, so a fuller transcript the others vouch for scores the same as the
 shorter one it extends), and the earliest on a tie of length.

 The archive block review used to receive every corroborating reader's
 transcript. On 2026-09-16 (Mio13) seven of twelve reviewers spent their
 whole completion cap reasoning about one chat translation against six
 transcripts of its screenshots and sent no content; the owner chose one
 transcript per picture (`doc/decision/translation-repair-archive-review-one-transcript-2026-09-16.md`).

 @param readings - transcripts of one picture, at least one

 @returns Reading the others agree with most

 @throws Error when no reading is given

 @example
 ```ts
 const chosen = mostCarriedReading({ readings: paired.readings, },);
 ```
 */
export function mostCarriedReading(
  { readings, }: { readonly readings: readonly ModelReading[]; },
): ModelReading {
  /**
   Each reading with how much of it the others carry, in reading order.
   */
  const scored = readings.map(function score(
    reading,
    at,
  ): CarriedReading {
    /**
     Overlaps against every other reading.
     */
    const overlaps = readings
      .filter(function isOther(
        _other,
        otherAt,
      ): boolean {
        return otherAt !== at;
      },)
      .map(function overlapWith(other,): number {
        return trigramOverlap({
          left: reading.text,
          right: other.text,
        },);
      },);
    if (overlaps.length === 0) {
      return {
        reading,
        carried: 0,
      };
    }
    /**
     Overlaps summed.
     */
    const total = overlaps.reduce(
      function addOverlap(
        sum,
        one,
      ): number {
        return sum + one;
      },
      0,
    );
    return {
      reading,
      carried: total / overlaps.length,
    };
  },);
  /**
   Earliest reading, which every later one must beat.
   */
  const first = nonNullishOrThrow(scored[0],);
  /**
   Reading the others carry most: more carried wins, then the longer text,
   then the earlier reading.
   */
  const chosen = scored.reduce(
    function betterCarried(
      best,
      candidate,
    ): CarriedReading {
      if (candidate.carried > best.carried)
        return candidate;
      if (candidate.carried < best.carried)
        return best;
      /**
       Characters the candidate transcribed.
       */
      const candidateLength = candidate.reading
        .text
        .length;
      /**
       Characters the best so far transcribed.
       */
      const bestLength = best.reading
        .text
        .length;
      return (candidateLength > bestLength) ? candidate : best;
    },
    first,
  );
  return chosen.reading;
}

//endregion Most carried reading
