import {
  type AttemptMap,
  attemptsOf,
} from '@monochromatic-dev/translation-repair-glossary/ts';
import {
  bandOf,
  countSettledPerBand,
  rankWithinBands,
  type SizedEntry,
} from '@monochromatic-dev/translation-repair-insertion/ts';
import type { CorpusPair, } from '@monochromatic-dev/translation-repair-slice-cache/ts';

//region Corpus pass order
// The order a corpus pass works its pending entries in. Split out of the
// procedure so each tier of the comparison is read by a case of its own.

/**
 How far each size band leads within one rank: the larger band goes first,
 because a large entry may need a second run to settle, so starting it
 earlier costs nothing and lets it resume sooner.
 */
const BAND_LEAD = {
  large: 0,
  medium: 1,
  small: 2,
} as const;

/**
 One number an entry was given while the pass computed its order.

 @param numbers - what was computed, by entry id, over the very entries being ordered

 @param id - entry whose number to read

 @param what - what the number is, for the message of a failure

 @returns Its number

 @throws {@link Error} when the entry has none, which cannot happen since every table is computed
 over the same entries that are being ordered

 @example
 ```ts
 const rank = numberOfEntry({ numbers: ranks, id: 'tabby', what: 'within-band rank', },);
 ```
 */
function numberOfEntry(
  {
    numbers,
    id,
    what,
  }: {
    readonly numbers: ReadonlyMap<string, number>;
    readonly id: string;
    readonly what: string;
  },
): number {
  /**
   Number the entry holds.
   */
  const found = numbers.get(id,);
  if (found === undefined) {
    throw new Error(
      `unreachable: entry ${JSON.stringify(id,)} has no ${what} although it was computed over `
        + 'the same entries that are being ordered',
    );
  }
  return found;
}

/**
 Orders the entries a pass still has to run: cached progress resumes first,
 then the bands interleave by within-band rank so coverage fills evenly, then
 the larger band leads within one rank. Within one band the entries are ranked
 fewest attempts first, so flaky entries deprioritize.

 @param eligible - complete unsettled pairs, in walk order

 @param settled - already-settled entries with their sizes, which offset each band's ranks so a
 run does not restart every band at rank zero

 @param resumableIds - entries holding cached slices from an aborted run

 @param attempts - attempt counts from earlier runs

 @returns The same pairs in run order

 @example
 ```ts
 const pending = orderPendingEntries({ eligible, settled, resumableIds, attempts, },);
 ```
 */
export function orderPendingEntries(
  {
    eligible,
    settled,
    resumableIds,
    attempts,
  }: {
    readonly eligible: readonly CorpusPair[];
    readonly settled: readonly SizedEntry[];
    readonly resumableIds: ReadonlySet<string>;
    readonly attempts: AttemptMap;
  },
): readonly CorpusPair[] {
  /**
   Encoder measuring page-source byte size once per entry.
   */
  const sizer = new TextEncoder();

  /**
   Every eligible entry reduced to its id and page-source byte size, measured
   once so ordering never re-encodes text on a compare, and held with the
   fewest attempts first: each band's ranks follow this order, so an entry that
   keeps failing waits behind the ones not yet tried instead of keeping its
   place in the walk and being retried first on every run.
   */
  const sized = eligible
    .map(function toSized(entry,): SizedEntry {
      return {
        id: entry.id,
        sourceBytes: sizer.encode(entry.sourceText,)
          .length,
      };
    },)
    .toSorted(function byFewestAttempts(
      a,
      b,
    ): number {
      return attemptsOf({
        attempts,
        id: a.id,
      },) - attemptsOf({
        attempts,
        id: b.id,
      },);
    },);

  /**
   Each entry's rank within its own size band, so ordering interleaves the
   bands instead of draining one before starting the next.
   */
  const ranks = rankWithinBands({
    entries: sized,
    settledPerBand: countSettledPerBand({ entries: settled, },),
  },);

  /**
   How far each entry's band leads within one rank.
   */
  const leads = new Map(sized.map(function toLead(entry,) {
    return [
      entry.id,
      BAND_LEAD[bandOf({ sourceBytes: entry.sourceBytes, },)],
    ] as const;
  },),);

  return eligible.toSorted(function byResumeThenBandThenAttempts(
    a,
    b,
  ) {
    /**
     Negative when only `a` has cached progress (so it resumes first),
     positive when only `b` does; zero when neither or both do.
     */
    const resumeDelta = Number(resumableIds.has(b.id,),)
      - Number(resumableIds.has(a.id,),);
    if (resumeDelta !== 0)
      return resumeDelta;

    /**
     Difference in within-band rank. Interleaving on this fills every band
     at the same pace, so the tenth entry of each band arrives at roughly
     the same time rather than one band starving.
     */
    const rankDelta = numberOfEntry({
      numbers: ranks,
      id: a.id,
      what: 'within-band rank',
    },) - numberOfEntry({
      numbers: ranks,
      id: b.id,
      what: 'within-band rank',
    },);
    if (rankDelta !== 0)
      return rankDelta;

    // Within one rank the larger band goes first, then the medium, then the small. Two entries
    // of one band and one rank are one entry, so nothing is left to decide after this.
    return numberOfEntry({
      numbers: leads,
      id: a.id,
      what: 'band',
    },) - numberOfEntry({
      numbers: leads,
      id: b.id,
      what: 'band',
    },);
  },);
}

//endregion Corpus pass order
