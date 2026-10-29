/**
 Tests for the order a corpus pass works its pending entries in: cached
 progress first, then the size bands interleaved by within-band rank, then the
 larger band, then fewest attempts.

 A defect here does not crash anything: it quietly starts the wrong entry
 first, so one band fills faster than the others or a large entry waits behind
 a medium one that could have waited.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  MEDIUM_PAGE_BYTES,
  SMALL_PAGE_BYTES,
} from '@monochromatic-dev/translation-repair-insertion';
import type { AttemptMap, } from '@monochromatic-dev/translation-repair-glossary';
import { orderPendingEntries, } from '../../dist/final/node/index.mjs';

/**
 One pair whose original page is the given number of bytes, which is what
 decides its band.

 @param id - entry id
 @param bytes - size of the original page in UTF-8 bytes, all of it ASCII

 @returns Pair

 @example
 ```ts
 const pair = pairOfSize({ id: 'tabby', bytes: 100, },);
 ```
 */
function pairOfSize(
  {
    id,
    bytes,
  }: {
    readonly id: string;
    readonly bytes: number;
  },
): {
  readonly id: string;
  readonly sourceText: string;
  readonly targetText: string;
} {
  return {
    id,
    sourceText: 'n'.repeat(bytes,),
    targetText: 'The cat naps.',
  };
}

/**
 Ids of a list of pairs, in order.

 @param pairs - pairs to read

 @returns Their ids

 @example
 ```ts
 const ids = idsOf({ pairs, },);
 ```
 */
function idsOf(
  { pairs, }: { readonly pairs: readonly { readonly id: string; }[]; },
): readonly string[] {
  return pairs.map(function toId(pair,): string {
    return pair.id;
  },);
}

/**
 Page size that falls in the small band.
 */
const SMALL = SMALL_PAGE_BYTES - 1;

/**
 Page size that falls in the medium band.
 */
const MEDIUM = SMALL_PAGE_BYTES;

/**
 Page size that falls in the large band.
 */
const LARGE = MEDIUM_PAGE_BYTES;

await describe({
  name: orderPendingEntries.name,
  children: [
    it({
      name: 'ORDERS NOTHING when nothing is eligible',
      fn: async () => {
        expect(orderPendingEntries({
          eligible: [],
          settled: [],
          resumableIds: new Set<string>(),
          attempts: new Map(),
        },),).toEqual([],);
      },
    },),
    it({
      name: 'KEEPS ONE ENTRY as it is, and leaves the list it was handed unsorted',
      fn: async () => {
        /**
         Two entries handed over in the order they were walked.
         */
        const eligible = [
          pairOfSize({
            id: 'mittens',
            bytes: LARGE,
          },),
          pairOfSize({
            id: 'tabby',
            bytes: SMALL,
          },),
        ];
        orderPendingEntries({
          eligible,
          settled: [],
          resumableIds: new Set<string>(),
          attempts: new Map(),
        },);
        expect(idsOf({ pairs: eligible, },),).toEqual([
          'mittens',
          'tabby',
        ],);
      },
    },),
    it({
      name: 'RESUMES CACHED PROGRESS FIRST, ahead of an entry of a lower rank and a larger band, and keeps the '
        + 'two cached ones in their own order',
      fn: async () => {
        expect(idsOf({ pairs: orderPendingEntries({
          eligible: [
            pairOfSize({
              id: 'biscuit',
              bytes: LARGE,
            },),
            pairOfSize({
              id: 'tabby',
              bytes: SMALL,
            },),
            pairOfSize({
              id: 'mittens',
              bytes: SMALL,
            },),
          ],
          settled: [],
          resumableIds: new Set([
            'mittens',
            'tabby',
          ],),
          attempts: new Map(),
        },), },),).toEqual([
          'tabby',
          'mittens',
          'biscuit',
        ],);
      },
    },),
    it({
      name: 'INTERLEAVES THE BANDS BY RANK, so the second entry of each band comes after the first of every band',
      fn: async () => {
        expect(idsOf({ pairs: orderPendingEntries({
          eligible: [
            pairOfSize({
              id: 'small-1',
              bytes: SMALL,
            },),
            pairOfSize({
              id: 'small-2',
              bytes: SMALL,
            },),
            pairOfSize({
              id: 'large-1',
              bytes: LARGE,
            },),
            pairOfSize({
              id: 'large-2',
              bytes: LARGE,
            },),
          ],
          settled: [],
          resumableIds: new Set<string>(),
          attempts: new Map(),
        },), },),).toEqual([
          'large-1',
          'small-1',
          'large-2',
          'small-2',
        ],);
      },
    },),
    it({
      name: 'OFFSETS A BAND BY WHAT IT ALREADY SETTLED, so the band furthest behind leads',
      fn: async () => {
        expect(idsOf({ pairs: orderPendingEntries({
          eligible: [
            pairOfSize({
              id: 'large-1',
              bytes: LARGE,
            },),
            pairOfSize({
              id: 'small-1',
              bytes: SMALL,
            },),
          ],
          settled: [
            {
              id: 'large-done-1',
              sourceBytes: LARGE,
            },
            {
              id: 'large-done-2',
              sourceBytes: LARGE,
            },
          ],
          resumableIds: new Set<string>(),
          attempts: new Map(),
        },), },),).toEqual([
          'small-1',
          'large-1',
        ],);
      },
    },),
    it({
      name: 'PUTS THE LARGE BAND BEFORE THE MEDIUM AND THE SMALL within one rank, whatever order they were walked in',
      fn: async () => {
        expect(idsOf({ pairs: orderPendingEntries({
          eligible: [
            pairOfSize({
              id: 'small',
              bytes: SMALL,
            },),
            pairOfSize({
              id: 'medium',
              bytes: MEDIUM,
            },),
            pairOfSize({
              id: 'large',
              bytes: LARGE,
            },),
          ],
          settled: [],
          resumableIds: new Set<string>(),
          attempts: new Map(),
        },), },),).toEqual([
          'large',
          'medium',
          'small',
        ],);
      },
    },),
    it({
      name: 'PUTS THE FEWEST ATTEMPTS FIRST within one band, so an entry that keeps failing waits behind the '
        + 'ones not yet tried, and keeps walk order on a tie',
      fn: async () => {
        /**
         Attempt counts: the first walked has been tried twice, the second once, the rest never.
         */
        const attempts: AttemptMap = new Map([
          [
            'tabby',
            2,
          ],
          [
            'biscuit',
            1,
          ],
        ],);
        expect(idsOf({ pairs: orderPendingEntries({
          eligible: [
            pairOfSize({
              id: 'tabby',
              bytes: SMALL,
            },),
            pairOfSize({
              id: 'biscuit',
              bytes: SMALL,
            },),
            pairOfSize({
              id: 'mittens',
              bytes: SMALL,
            },),
            pairOfSize({
              id: 'whiskers',
              bytes: SMALL,
            },),
          ],
          settled: [],
          resumableIds: new Set<string>(),
          attempts,
        },), },),).toEqual([
          'mittens',
          'whiskers',
          'biscuit',
          'tabby',
        ],);
      },
    },),
    it({
      name: 'DOES NOT LET A FEW ATTEMPTS MOVE AN ENTRY OUT OF ITS BAND\'S TURN: the tried large entry still comes '
        + 'before the untried small one of the same rank',
      fn: async () => {
        expect(idsOf({ pairs: orderPendingEntries({
          eligible: [
            pairOfSize({
              id: 'small',
              bytes: SMALL,
            },),
            pairOfSize({
              id: 'large',
              bytes: LARGE,
            },),
          ],
          settled: [],
          resumableIds: new Set<string>(),
          attempts: new Map([
            [
              'large',
              3,
            ],
          ],),
        },), },),).toEqual([
          'large',
          'small',
        ],);
      },
    },),
  ],
},);
