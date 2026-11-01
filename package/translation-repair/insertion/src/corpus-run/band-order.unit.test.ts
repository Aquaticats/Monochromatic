/**
 Tests for the band ordering a corpus pass starts entries in.

 This module had no test at all, and it decides which entries a pass reaches
 first. A defect here does not crash anything: it quietly fills one band
 faster than the others, so the stratified sample drawn later is biased toward
 whichever band the ordering favored, and the precision number the milestone
 gate reads is measured on the wrong population.

 The rank offset is the subtle part and gets the most attention in this file. Without
 it every run restarts each band at zero, the within-rank tiebreak hands every
 run to the same band, and the starvation this ordering exists to prevent
 comes back.

 Fixtures are cat-themed invention.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  assertSourceBytes,
  classifyBand,
  MEDIUM_BAND_MAX_BYTES,
  type SizeBand,
  SMALL_BAND_MAX_BYTES,
} from '@monochromatic-dev/translation-repair-house-rule';
import {
  bandOf,
  countSettledPerBand,
  MEDIUM_PAGE_BYTES,
  rankWithinBands,
  type SizedEntry,
  SMALL_PAGE_BYTES,
} from '../../dist/final/node/index.mjs';

/**
 Bands a raw byte count, stating the unit explicitly.

 This case sweeps byte counts around the band cuts, which are byte
 counts by definition and cannot be produced from text, so the
 assertion is the honest way to reach `classifyBand`.

 @param count - UTF-8 byte length under test

 @returns Band that count falls in

 @example
 ```ts
 expect(bandAt(1_842,),).toBe('small',);
 ```
 */
function bandAt(count: number,): SizeBand {
  assertSourceBytes(count,);
  return classifyBand({ sourceBytes: count, },);
}


/**
 Builds an entry reduced to what ordering reads.

 @param id - corpus entry id

 @param sourceBytes - page source size in UTF-8 bytes

 @returns Sized entry

 @example
 ```ts
 const entry = sized({ id: 'Mittens', sourceBytes: 900, },);
 ```
 */
function sized(
  {
    id,
    sourceBytes,
  }: {
    readonly id: string;
    readonly sourceBytes: number;
  },
): SizedEntry {
  return {
    id,
    sourceBytes,
  };
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: bandOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'places a page below the small cut in the small band, and treats '
            + 'the cut itself as MEDIUM, since the bound is exclusive on the small '
            + 'side',
          fn: async () => {
            expect(bandOf({ sourceBytes: SMALL_PAGE_BYTES - 1, },),).toBe('small',);
            expect(bandOf({ sourceBytes: SMALL_PAGE_BYTES, },),).toBe('medium',);
          },
        },),

        it({
          name: 'treats the medium cut itself as LARGE for the same reason, so no '
            + 'page size falls in two bands and none falls in none',
          fn: async () => {
            expect(bandOf({ sourceBytes: MEDIUM_PAGE_BYTES - 1, },),).toBe('medium',);
            expect(bandOf({ sourceBytes: MEDIUM_PAGE_BYTES, },),).toBe('large',);
          },
        },),

        it({
          name: 'classifies a zero-byte page as small rather than refusing, since '
            + 'an empty page is a real corpus state and must land somewhere',
          fn: async () => {
            expect(bandOf({ sourceBytes: 0, },),).toBe('small',);
          },
        },),

        it({
          name: 'AGREES WITH classifyBand at every boundary. The two are separate '
            + 'implementations of one rule, kept aligned only by a doc comment, and '
            + 'if they drift the pass fills different bands than the sample '
            + 'stratifies over: the graded sheet would then be drawn from a '
            + 'population the accumulation never balanced',
          fn: async () => {
            expect(SMALL_PAGE_BYTES,).toBe(SMALL_BAND_MAX_BYTES,);
            expect(MEDIUM_PAGE_BYTES,).toBe(MEDIUM_BAND_MAX_BYTES,);

            for (const sourceBytes of [
              0,
              1,
              SMALL_PAGE_BYTES - 1,
              SMALL_PAGE_BYTES,
              SMALL_PAGE_BYTES + 1,
              MEDIUM_PAGE_BYTES - 1,
              MEDIUM_PAGE_BYTES,
              MEDIUM_PAGE_BYTES + 1,
              1_000_000,
            ])
              expect(bandOf({ sourceBytes, },),).toBe(
                bandAt(sourceBytes,),
              );
          },
        },),
      ],
    },),

    describe({
      name: countSettledPerBand.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'reports a count for EVERY band including the ones with no settled '
            + 'entries, so the rank offset reads zero rather than undefined for a '
            + 'band a pass has not reached yet',
          fn: async () => {
            /**
             Settled counts across a slate holding only large entries.
             */
            const counts = countSettledPerBand({
              entries: [
                sized({
                  id: 'Marmalade',
                  sourceBytes: 9_000,
                },),
                sized({
                  id: 'Clementine',
                  sourceBytes: 8_000,
                },),
              ],
            },);

            expect(counts.get('large',),).toBe(2,);
            expect(counts.get('medium',),).toBe(0,);
            expect(counts.get('small',),).toBe(0,);
          },
        },),

        it({
          name: 'counts each band independently across a mixed slate',
          fn: async () => {
            /**
             Settled counts across one entry of each band plus an extra small.
             */
            const counts = countSettledPerBand({
              entries: [
                sized({
                  id: 'Mittens',
                  sourceBytes: 900,
                },),
                sized({
                  id: 'Biscuit',
                  sourceBytes: 1_000,
                },),
                sized({
                  id: 'Pumpkin',
                  sourceBytes: 2_000,
                },),
                sized({
                  id: 'Marmalade',
                  sourceBytes: 9_000,
                },),
              ],
            },);

            expect(counts.get('small',),).toBe(2,);
            expect(counts.get('medium',),).toBe(1,);
            expect(counts.get('large',),).toBe(1,);
          },
        },),
      ],
    },),

    describe({
      name: rankWithinBands.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'ranks each band from zero independently when nothing has settled, '
            + 'so the first entry of every band ties and the bands interleave '
            + 'instead of one draining before the next starts',
          fn: async () => {
            /**
             Ranks over one slate holding two entries in each of two bands.
             */
            const ranks = rankWithinBands({
              entries: [
                sized({
                  id: 'Mittens',
                  sourceBytes: 900,
                },),
                sized({
                  id: 'Biscuit',
                  sourceBytes: 1_000,
                },),
                sized({
                  id: 'Marmalade',
                  sourceBytes: 9_000,
                },),
                sized({
                  id: 'Clementine',
                  sourceBytes: 8_000,
                },),
              ],
              settledPerBand: new Map(),
            },);

            expect(ranks.get('Mittens',),).toBe(0,);
            expect(ranks.get('Biscuit',),).toBe(1,);
            expect(ranks.get('Marmalade',),).toBe(0,);
            expect(ranks.get('Clementine',),).toBe(1,);
          },
        },),

        it({
          name: 'OFFSETS each band by what already settled, which is the whole '
            + 'point: ranking runs over the remaining entries only, so without the '
            + 'offset every run restarts each band at zero and the tiebreak hands '
            + 'run after run to the same band, reproducing the starvation this '
            + 'ordering exists to prevent',
          fn: async () => {
            /**
             Ranks when the small band is three ahead and the large band is not.
             */
            const ranks = rankWithinBands({
              entries: [
                sized({
                  id: 'Mittens',
                  sourceBytes: 900,
                },),
                sized({
                  id: 'Marmalade',
                  sourceBytes: 9_000,
                },),
              ],
              settledPerBand: new Map([
                [
                  'small',
                  3,
                ],
              ],),
            },);

            // The small entry is fourth in its band's whole fill order; the large
            // entry is first in its own. So the band that is behind leads.
            expect(ranks.get('Mittens',),).toBe(3,);
            expect(ranks.get('Marmalade',),).toBe(0,);
            expect(
              (ranks.get('Marmalade',) ?? 0) < (ranks.get('Mittens',) ?? 0),
            ).toBe(true,);
          },
        },),

        it({
          name: 'treats a band absent from the settled map as zero settled, so a '
            + 'partial map from an earlier run cannot shift a band it never '
            + 'mentioned',
          fn: async () => {
            /**
             Ranks with a settled map naming only the medium band.
             */
            const ranks = rankWithinBands({
              entries: [
                sized({
                  id: 'Mittens',
                  sourceBytes: 900,
                },),
              ],
              settledPerBand: new Map([
                [
                  'medium',
                  5,
                ],
              ],),
            },);

            expect(ranks.get('Mittens',),).toBe(0,);
          },
        },),

        it({
          name: 'ranks every entry exactly once and leaves none unranked, since an '
            + 'entry missing from the map would sort by the comparator fallback '
            + 'rather than by its band position',
          fn: async () => {
            /**
             Slate spanning all three bands.
             */
            const entries = [
              sized({
                id: 'Mittens',
                sourceBytes: 900,
              },),
              sized({
                id: 'Pumpkin',
                sourceBytes: 2_000,
              },),
              sized({
                id: 'Marmalade',
                sourceBytes: 9_000,
              },),
            ];

            /**
             Ranks over that slate.
             */
            const ranks = rankWithinBands({
              entries,
              settledPerBand: new Map(),
            },);

            expect(ranks.size,).toBe(entries.length,);
            for (const entry of entries)
              expect(ranks.has(entry.id,),).toBe(true,);
          },
        },),

        it({
          name: 'returns an empty map for an empty slate rather than throwing',
          fn: async () => {
            expect(
              rankWithinBands({
                entries: [],
                settledPerBand: new Map(),
              },).size,
            ).toBe(0,);
          },
        },),
      ],
    },),
  ],
},);
