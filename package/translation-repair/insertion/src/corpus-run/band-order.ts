import {
  MEDIUM_BAND_MAX_BYTES,
  SMALL_BAND_MAX_BYTES,
} from '@monochromatic-dev/translation-repair-house-rule/ts';

//region Band ordering
// Decides the order a corpus pass starts entries in, so coverage fills every
// size band at the same pace.
//
// Sorting the small band LAST was tried and does not serve the stated coverage
// bar. The corpus holds 31 small, 32 medium, and 29 large pairs, so
// deprioritizing small means all 61 non-small entries must settle before the
// first small one does; at the observed half-hour-plus per entry that is over
// a day of compute before the small band opens at all. An earlier accumulation
// only reached a stocked small band because those entries had settled in
// previous passes, which a fresh pass does not inherit.
//
// Interleaving serves the same intent the deprioritization was reaching for.
// Its purpose was to stop early settling over-representing the small band,
// because small entries finish inside one run while large ones consume it.
// Round-robin across bands prevents that over-representation directly and
// symmetrically, and reaches an even ten per band in about thirty entries
// rather than seventy-one.

/**
 Page-source byte size below which an entry sits in the small band. The
 corpus page.md sizes fall into rough tertiles with the lower cut near
 1.8 KiB.

 Takes its value from `sample-grading.ts` rather than restating it. The bands
 a pass FILLS have to be the bands a sample STRATIFIES over, or the graded
 sheet is drawn from a population the accumulation never balanced. Two
 literals kept equal by a doc comment is the arrangement that fails silently,
 because nothing about a wrong number looks wrong.
 */
export const SMALL_PAGE_BYTES: typeof SMALL_BAND_MAX_BYTES = SMALL_BAND_MAX_BYTES;

/**
 Page-source byte size at or above which an entry sits in the large band, the
 upper tertile cut.

 Shares its definition with the sampler for the reason given on
 {@link SMALL_PAGE_BYTES}.
 */
export const MEDIUM_PAGE_BYTES: typeof MEDIUM_BAND_MAX_BYTES = MEDIUM_BAND_MAX_BYTES;

/**
 Size bands in the order they lead within one rank. The larger band goes
 first because a large entry may need a second run to settle, so starting it
 earlier costs nothing and lets it resume sooner.
 */
const BANDS = [
  'large',
  'medium',
  'small',
] as const;

/**
 One entry reduced to what ordering needs.

 @example
 ```ts
 const sized: SizedEntry = { id: 'Kitten', sourceBytes: 1_920, };
 ```
 */
export type SizedEntry = {
  /**
   Corpus entry id.
   */
  readonly id: string;

  /**
   Page source size in UTF-8 bytes.
   */
  readonly sourceBytes: number;
};

/**
 Band an entry's page source falls in.

 @param sourceBytes - page source size in UTF-8 bytes

 @returns Band name

 @example
 ```ts
 const band = bandOf({ sourceBytes: 1_920, },); // 'medium'
 ```
 */
export function bandOf(
  { sourceBytes, }: { readonly sourceBytes: number; },
): typeof BANDS[number] {
  if (sourceBytes < SMALL_PAGE_BYTES)
    return 'small';
  if (sourceBytes < MEDIUM_PAGE_BYTES)
    return 'medium';
  return 'large';
}

/**
 Ranks every entry within its own size band, so a comparator can interleave
 the bands by rank instead of draining one before starting the next.

 @param entries - eligible entries with their page sizes

 @returns Entry id to zero-based rank within its band

 @example
 ```ts
 const ranks = rankWithinBands({ entries, settledPerBand, },);
 ```
 */
export function rankWithinBands(
  {
    entries,
    settledPerBand,
  }: {
    readonly entries: readonly SizedEntry[];
    readonly settledPerBand: ReadonlyMap<string, number>;
  },
): ReadonlyMap<string, number> {
  return new Map(
    BANDS.flatMap(function rankBand(band,) {
      /**
       How many of this band already settled. Ranking runs over the
       REMAINING entries only, so without this offset every run restarts each
       band at zero and the within-rank tiebreak hands every run to the same
       band, reproducing the starvation this ordering exists to prevent.
       Offsetting by the settled count makes the rank an entry's position in
       its band's whole fill order, so the band that is furthest behind
       always leads.
       */
      const settled = settledPerBand.get(band,) ?? 0;
      return entries
        .filter(function inBand(entry,) {
          return bandOf({ sourceBytes: entry.sourceBytes, },) === band;
        },)
        .map(function toRank(
          entry,
          index,
        ) {
          return [
            entry.id,
            settled + index,
          ] as const;
        },);
    },),
  );
}

/**
 Counts settled entries per band, the offset {@link rankWithinBands} needs to
 keep ordering fair across runs.

 @param entries - already-settled entries with their page sizes

 @returns Band name to settled count

 @example
 ```ts
 const settled = countSettledPerBand({ entries: done, },);
 ```
 */
export function countSettledPerBand(
  { entries, }: { readonly entries: readonly SizedEntry[]; },
): ReadonlyMap<string, number> {
  return new Map(
    BANDS.map(function countBand(band,) {
      return [
        band,
        entries.filter(function inBand(entry,) {
          return bandOf({ sourceBytes: entry.sourceBytes, },) === band;
        },)
          .length,
      ] as const;
    },),
  );
}

//endregion Band ordering
