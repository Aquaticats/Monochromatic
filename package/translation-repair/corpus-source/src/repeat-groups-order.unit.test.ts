/**
 Tests for the order repeat groups are listed in: first occurrence by start,
 then a maximal repeat before a refrain, then the first occurrence ending
 sooner, then the later occurrences, then the repeat with more of them, then
 a refrain's prefix and suffix as code points. Fixtures are cat-themed
 invention only.

 @module
 */

import {
  caught,
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { compareFoundRepeats, } from '../dist/final/node/index.mjs';

/**
 Builds a found repeat from the places its occurrences start and end.

 @param kind - kind of repeat
 @param places - start and end offset of each occurrence, in page order
 @param prefix - prefix of a refrain, empty for a maximal repeat
 @param suffix - suffix of a refrain, empty for a maximal repeat

 @returns Found repeat with those occurrences

 @example
 ```ts
 const repeat = foundRepeat({ kind: 'maximal', places: [[0, 12,], [20, 32,],], prefix: '', suffix: '', },);
 ```
 */
function foundRepeat(
  {
    kind,
    places,
    prefix,
    suffix,
  }: {
    readonly kind: 'maximal' | 'refrain';
    readonly places: readonly (readonly [number, number])[];
    readonly prefix: string;
    readonly suffix: string;
  },
) {
  return {
    kind,
    prefix,
    suffix,
    length: 0,
    spans: places.map(([startOffset, endOffset,],) => ({
      startOffset,
      endOffset,
      quoted: false,
      width: endOffset - startOffset,
      slotStartOffset: startOffset,
      slotEndOffset: endOffset,
    }),),
  };
}

/**
 Builds a maximal repeat from the places its occurrences start and end.

 @param places - start and end offset of each occurrence, in page order

 @returns Maximal found repeat with those occurrences

 @example
 ```ts
 const repeat = maximalAt({ places: [[0, 12,], [20, 32,],], },);
 ```
 */
function maximalAt({ places, }: { readonly places: readonly (readonly [number, number])[]; },) {
  return foundRepeat({
    kind: 'maximal',
    places,
    prefix: '',
    suffix: '',
  },);
}

/**
 Builds a refrain from the places its occurrences start and end.

 @param places - start and end offset of each occurrence, in page order
 @param prefix - prefix shared by the occurrences
 @param suffix - suffix shared by the occurrences

 @returns Refrain found repeat with those occurrences

 @example
 ```ts
 const repeat = refrainAt({ places: [[0, 11,], [20, 31,],], prefix: '猫咪窗台', suffix: '阳盹儿', },);
 ```
 */
function refrainAt(
  {
    places,
    prefix,
    suffix,
  }: {
    readonly places: readonly (readonly [number, number])[];
    readonly prefix: string;
    readonly suffix: string;
  },
) {
  return foundRepeat({
    kind: 'refrain',
    places,
    prefix,
    suffix,
  },);
}

/**
 Sign of the order of two found repeats.

 @param first - one found repeat
 @param second - other found repeat

 @returns Minus one, zero or one

 @example
 ```ts
 const sign = orderOf({ first, second, },);
 ```
 */
function orderOf(
  {
    first,
    second,
  }: {
    readonly first: ReturnType<typeof maximalAt>;
    readonly second: ReturnType<typeof maximalAt>;
  },
): number {
  return Math.sign(compareFoundRepeats({
    first,
    second,
  },),);
}

await describe({
  name: compareFoundRepeats.name,
  concurrency: DEFAULT_CONCURRENCY,
  children: [
    it({
      name: 'Lists the repeat whose first occurrence starts sooner first, whatever the kind',
      fn: async () => {
        /** Maximal repeat that starts later than the refrain. */
        const later = maximalAt({ places: [[5, 17,], [30, 42,],], },);
        /** Refrain that starts sooner than the maximal repeat. */
        const sooner = refrainAt({
          places: [[2, 13,], [30, 41,],],
          prefix: '甲',
          suffix: '乙',
        },);

        expect(orderOf({ first: sooner, second: later, },),).toBe(-1,);
        expect(orderOf({ first: later, second: sooner, },),).toBe(1,);
      },
    },),

    it({
      name: 'Lists a maximal repeat before a refrain whose first occurrence starts at the same offset',
      fn: async () => {
        /** Maximal repeat that ends later than the refrain's first clause. */
        const maximal = maximalAt({ places: [[0, 23,], [30, 53,],], },);
        /** Refrain whose first clause starts where the maximal repeat does and ends sooner. */
        const refrain = refrainAt({
          places: [[0, 11,], [30, 41,],],
          prefix: '甲',
          suffix: '乙',
        },);

        expect(orderOf({ first: maximal, second: refrain, },),).toBe(-1,);
        expect(orderOf({ first: refrain, second: maximal, },),).toBe(1,);
      },
    },),

    it({
      name: 'Lists the repeat whose first occurrence ends sooner first when kind and start match',
      fn: async () => {
        /** Maximal repeat of twelve elements. */
        const shorter = maximalAt({ places: [[0, 12,], [30, 42,], [60, 72,],], },);
        /** Maximal repeat of fifteen elements from the same start. */
        const longer = maximalAt({ places: [[0, 15,], [30, 45,],], },);

        expect(orderOf({ first: shorter, second: longer, },),).toBe(-1,);
        expect(orderOf({ first: longer, second: shorter, },),).toBe(1,);
      },
    },),

    it({
      name: 'Orders repeats whose first occurrences match by the start of the next occurrence that differs',
      fn: async () => {
        /** Repeat whose second occurrence starts sooner. */
        const sooner = maximalAt({ places: [[0, 12,], [30, 42,], [90, 102,],], },);
        /** Repeat whose second occurrence starts later. */
        const later = maximalAt({ places: [[0, 12,], [40, 52,],], },);

        expect(orderOf({ first: sooner, second: later, },),).toBe(-1,);
        expect(orderOf({ first: later, second: sooner, },),).toBe(1,);
      },
    },),

    it({
      name: 'Lists the repeat with more occurrences first when the other repeat\'s starts continue as its own do',
      fn: async () => {
        /** Repeat with three occurrences. */
        const three = maximalAt({ places: [[0, 12,], [30, 42,], [60, 72,],], },);
        /** Repeat with the first two of them. */
        const two = maximalAt({ places: [[0, 12,], [30, 42,],], },);

        expect(orderOf({ first: three, second: two, },),).toBe(-1,);
        expect(orderOf({ first: two, second: three, },),).toBe(1,);
      },
    },),

    it({
      name: 'Orders refrains that match everywhere else by prefix then suffix, as code points',
      fn: async () => {
        /** Refrain whose prefix is a fullwidth tilde, code point U+FF5E. */
        const tilde = refrainAt({
          places: [[0, 11,], [30, 41,],],
          prefix: '～',
          suffix: '乙',
        },);
        /** Refrain whose prefix is an ideograph beyond the first plane, whose UTF-16 units sort before the tilde. */
        const astral = refrainAt({
          places: [[0, 11,], [30, 41,],],
          prefix: '𠀀',
          suffix: '乙',
        },);
        /** Refrain with the tilde's prefix and a suffix that sorts later. */
        const laterSuffix = refrainAt({
          places: [[0, 11,], [30, 41,],],
          prefix: '～',
          suffix: '戊',
        },);

        expect(orderOf({ first: tilde, second: astral, },),).toBe(-1,);
        expect(orderOf({ first: astral, second: tilde, },),).toBe(1,);
        expect(orderOf({ first: laterSuffix, second: tilde, },),).toBe(1,);
        expect(orderOf({ first: tilde, second: laterSuffix, },),).toBe(-1,);
      },
    },),

    it({
      name: 'Returns zero for two repeats that match in every part the order reads',
      fn: async () => {
        expect(compareFoundRepeats({
          first: maximalAt({ places: [[0, 12,], [30, 42,],], },),
          second: maximalAt({ places: [[0, 12,], [30, 42,],], },),
        },),).toBe(0,);
      },
    },),

    it({
      name: 'Gives the same list for the same repeats whichever order they come in',
      fn: async () => {
        /** Refrain, listed last of the four. */
        const refrain = refrainAt({
          places: [[0, 11,], [30, 41,],],
          prefix: '甲',
          suffix: '乙',
        },);
        /** Maximal repeat whose first occurrence ends latest, listed third. */
        const widest = maximalAt({ places: [[0, 23,], [30, 53,],], },);
        /** Maximal repeat whose second occurrence starts later, listed second. */
        const lateSecond = maximalAt({ places: [[0, 12,], [40, 52,],], },);
        /** Maximal repeat whose second occurrence starts sooner, listed first. */
        const earlySecond = maximalAt({ places: [[0, 12,], [30, 42,],], },);
        /** The order the comparison gives them in. */
        const expected = [earlySecond, lateSecond, widest, refrain,];

        for (const given of [
          [refrain, widest, lateSecond, earlySecond,],
          [earlySecond, lateSecond, widest, refrain,],
          [lateSecond, refrain, earlySecond, widest,],
        ])
          expect(given.toSorted((first, second,) => compareFoundRepeats({ first, second, },),),).toEqual(expected,);
      },
    },),

    it({
      name: 'Refuses a repeat that holds no occurrence, which no finder builds',
      fn: async () => {
        /** Repeat with no occurrence at all. */
        const empty = maximalAt({ places: [], },);
        /** What the comparison throws. */
        const refusal = caught(function compareEmpty(): unknown {
          return compareFoundRepeats({
            first: empty,
            second: maximalAt({ places: [[0, 12,],], },),
          },);
        },);

        expect(refusal,).toBeInstanceOf(Error,);
        expect(String(refusal,),).toBe(
          'Error: unreachable: a found repeat holds no occurrence, though both finders keep at least two',
        );
      },
    },),
  ],
},);
