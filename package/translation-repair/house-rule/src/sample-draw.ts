import { createHash, } from 'node:crypto';

import { compareCodePoints, } from '@monochromatic-dev/translation-repair-markup/ts';
import {
  type BandQuota,
  type GradingCandidate,
  SIZE_BANDS,
  type SizeBand,
} from './sample-grading.ts';

//region Stratified sample draw
// The seeded, deterministic draw over grading candidates: allocate a total
// sample size across the size bands as evenly as availability allows, then
// fill each band spread across its entries so one issue-heavy entry never
// dominates a band's slots. Order is a pure function of the seed, so a draw is
// reproducible and auditable for a fixed candidate pool.

/**
 Allocates a total sample size across the size bands as evenly as
 availability allows. Slots go one at a time round-robin in band order to any
 band with a candidate to spare, so the result is the most even
 availability-capped split and is deterministic. When candidates are scarce
 the total falls to what exists rather than over-drawing a band.

 @param available - candidate count per band

 @param size - total slots desired across all bands, a whole number

 @returns Slots allocated per band, summing to `min(size, total available)`

 @throws {@link RangeError} when the size is not a whole number, zero or more

 @example
 ```ts
 const quota = allocateBandQuota({
   available: { small: 200, medium: 180, large: 90, },
   size: 50,
 },); // { small: 17, medium: 17, large: 16 }
 ```
 */
export function allocateBandQuota(
  {
    available,
    size,
  }: {
    readonly available: BandQuota;
    readonly size: number;
  },
): BandQuota {
  // A WHOLE NUMBER OF SLOTS OR NOTHING. The round-robin stops on `remaining`
  // reaching exactly zero, which a fraction steps over: two and a half slots
  // asked of bands holding ten each handed out nine, one from every band on
  // every pass (ledger B127).
  if ((!Number.isSafeInteger(size,)) || (size < 0))
    throw new RangeError(
      `A sample size is a whole number of slots, zero or more; received ${String(size,)}.`,
    );

  /**
   Slots handed out so far, mutated in place as the round-robin proceeds.
   */
  const quota: Record<SizeBand, number> = {
    small: 0,
    medium: 0,
    large: 0,
  };

  /**
   Slots still to hand out, capped at what the bands can collectively hold.
   */
  let remaining = Math.min(
    size,
    SIZE_BANDS.reduce(
      function addAvailable(
        sum,
        band,
      ) {
        return sum + available[band];
      },
      0,
    ),
  );

  /**
   Passes never outlive the slots to place: a pass that runs places at
   least one while any remain, the `Math.min` cap guaranteeing a band
   below its own. The ceiling is the termination proof the no-progress
   break was a stopgap for.
   */
  const passes = remaining;
  for (let pass = 0; pass < passes; pass += 1) {
    for (const band of SIZE_BANDS) {
      if (remaining === 0)
        break;
      if (quota[band] < available[band]) {
        quota[band] += 1;
        remaining -= 1;
      }
    }
    if (remaining === 0)
      break;
  }

  return quota;
}

/**
 Deterministic shuffle key for an id under a seed: the sha256 hex of
 `seed:kind:id`. Lexicographic order over the hex gives a stable
 pseudo-random order that is a pure function of the seed, matching the
 codebase's `createHash` idiom without a stateful generator.

 @param seed - draw seed

 @param kind - namespace separating entry keys from issue keys

 @param id - the entry or issue id

 @returns Hex sha256 digest to sort by
 */
function shuffleKey(
  {
    seed,
    kind,
    id,
  }: {
    readonly seed: string;
    readonly kind: string;
    readonly id: string;
  },
): string {
  return createHash('sha256',)
    .update(`${seed}:${kind}:${id}`,)
    .digest('hex',);
}

/**
 One candidate paired with its issue shuffle key, before ranking.
 */
type KeyedCandidate = {
  /**
   The candidate under consideration.
   */
  readonly candidate: GradingCandidate;

  /**
   Shuffle key of the candidate's issue.
   */
  readonly issueKey: string;
};

/**
 One candidate decorated with the keys the stratified draw sorts by.
 */
type RankedCandidate = {
  /**
   The candidate under consideration.
   */
  readonly candidate: GradingCandidate;

  /**
   Shuffle order of this issue within its entry (0 is picked first).
   */
  readonly rank: number;

  /**
   Shuffle key of the candidate's entry, breaking ties between entries.
   */
  readonly entryKey: string;
};

/**
 Selects `count` candidates from one band, spreading across its entries: each
 entry's issues are shuffle-ordered and ranked, then selection takes every
 entry's rank-0 issue before any rank-1 issue. So one issue-heavy entry never
 dominates the band's slots, and the order is a pure function of the seed.

 @param candidates - every candidate in the band

 @param count - slots to fill from the band

 @param seed - draw seed

 @returns The selected candidates in round-robin draw order
 */
function selectFromBand(
  {
    candidates,
    count,
    seed,
  }: {
    readonly candidates: readonly GradingCandidate[];
    readonly count: number;
    readonly seed: string;
  },
): readonly GradingCandidate[] {
  /**
   Candidates grouped by entry, so each entry's issues can be ranked among
   themselves before the round-robin.
   */
  const byEntry = new Map<string, GradingCandidate[]>();
  for (const candidate of candidates) {
    /**
     The entry's bucket, created on first sighting.
     */
    const bucket = byEntry.get(candidate.entryId,) ?? [];
    bucket.push(candidate,);
    byEntry.set(
      candidate.entryId,
      bucket,
    );
  }

  /**
   Every candidate decorated with its within-entry rank and its entry's
   shuffle key, ready for the global round-robin sort.
   */
  const ranked: readonly RankedCandidate[] = [
    ...byEntry.values(),
  ]
    .flatMap(function rankEntry(
      bucket: readonly GradingCandidate[],
    ): readonly RankedCandidate[] {
      return bucket
        .map(function withIssueKey(candidate,): KeyedCandidate {
          return {
            candidate,
            issueKey: shuffleKey({
              seed,
              kind: 'issue',
              id: candidate.issueId,
            },),
          };
        },)
        .toSorted(function byIssueKey(
          a,
          b,
        ) {
          return compareCodePoints({
            left: a.issueKey,
            right: b.issueKey,
          },);
        },)
        .map(function withRank(
          entry,
          rank,
        ): RankedCandidate {
          return {
            candidate: entry.candidate,
            rank,
            entryKey: shuffleKey({
              seed,
              kind: 'entry',
              id: entry.candidate
                .entryId,
            },),
          };
        },);
    },);

  return ranked
    .toSorted(function byRoundRobin(
      a,
      b,
    ) {
      if (a.rank !== b.rank)
        return a.rank - b.rank;
      // Entry-key ordering, breaking rank ties between entries. Two
      // candidates of one rank come from different entries, since a rank is
      // a position inside one entry's bucket and `byEntry` keeps one bucket
      // per entry, so their entry keys differ and nothing is left to an
      // issue key or to arrival order (ledger T8, 2026-10-04).
      return compareCodePoints({
        left: a.entryKey,
        right: b.entryKey,
      },);
    },)
    .slice(
    0,
    count,
    )
    .map(function toCandidate(entry,) {
      return entry.candidate;
    },);
}

/**
 Draws a stratified sample of grading candidates. Slots split across bands by
 {@link allocateBandQuota}, then each band fills by round-robin across its
 entries, so the draw is representative across size bands and spread across
 entries within each band. Fully deterministic in the seed.

 @param candidates - the full accepted-issue pool

 @param size - total sample size desired

 @param seed - draw seed

 @returns The sampled candidates, small band first

 @example
 ```ts
 const sample = drawStratifiedSample({
   candidates: pool,
   size: DEFAULT_SAMPLE_SIZE,
   seed: DEFAULT_SAMPLE_SEED,
 },);
 ```
 */
export function drawStratifiedSample(
  {
    candidates,
    size,
    seed,
  }: {
    readonly candidates: readonly GradingCandidate[];
    readonly size: number;
    readonly seed: string;
  },
): readonly GradingCandidate[] {
  /**
   Candidate count present in each band.
   */
  const available: BandQuota = {
    small: candidates
      .filter(function inSmall(candidate,) {
        return candidate.band === 'small';
      },)
      .length,
    medium: candidates
      .filter(function inMedium(candidate,) {
        return candidate.band === 'medium';
      },)
      .length,
    large: candidates
      .filter(function inLarge(candidate,) {
        return candidate.band === 'large';
      },)
      .length,
  };

  /**
   Slots allocated to each band under the availability cap.
   */
  const quota = allocateBandQuota({
    available,
    size,
  },);

  return SIZE_BANDS.flatMap(function drawBand(band,) {
    return selectFromBand({
      candidates: candidates
        .filter(function inBand(candidate,) {
          return candidate.band === band;
        },),
      count: quota[band],
      seed,
    },);
  },);
}

//endregion Stratified sample draw
