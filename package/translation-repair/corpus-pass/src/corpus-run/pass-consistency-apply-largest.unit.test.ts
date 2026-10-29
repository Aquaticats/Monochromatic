/**
 Tests that the pass puts back the largest set of left-out gains the page can hold where few enough are left out, and the set
 the order of the page makes otherwise.

 A page is a slice of chosen sentences and one slice for each gain, which carries a repeat of the same words in the first and
 receives words of its own, with an apostrophe at one or both edges. The brute force reads the page with every subset of the
 gains made: a subset is valid when each gain mark of it is a proven apostrophe on the page and the floors pass every slice
 it touches. Eight pages (two plain, six with markup) are ones where the order of the page makes a smaller set than another
 order; they were found by a random search over thousands of pages. One of them (seed 7322, plain) holds two sets of the
 largest size, so it is the one that shows the tie rule of the exact search: the earliest set is made. The rule is pinned for
 these seeds only; where gains of one slice overlap, the check stage decides first and the later gain can stand.

 Fixtures are cat-themed invention; no corpus content appears here.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';

import {
  pageOfSlices,
  provenApostrophesInEnglish,
} from '@monochromatic-dev/translation-repair-consistency';
import {
  applyConsistency,
  consistencyFloorVerdict,
  type ConsistencySliceFacts,
  type SettledGroup,
} from '../../dist/final/node/index.mjs';
import { levelCapturingLogger, } from '@monochromatic-dev/translation-repair-test-fixture/ts/capturing-logger.test-fixture.ts';
import {
  groupIn,
  withEdit,
} from './consistency-apply-group.test-fixture.ts';
import { factsOf, } from './consistency-round-page.test-fixture.ts';

/**
 The mix of text a slice is drawn from when pages are plain.
 */
const PLAIN: readonly string[] = [
  '',
  '',
  '',
  ' He said ‘hi there. ',
  ' The friends’ bowl. ',
  ' She sang “la” loudly. ',
  ' Rock ’n’ roll. ',
  ' Then it napped. ',
  ' It was ’tis fine. ',
  ' “Open here. ',
  ' Close here.” ',
  ' He said \'quoted\' twice. ',
];

/**
 What markup adds to a plain page.
 */
const MARKUP: readonly string[] = [
  '',
  '',
  ' The *kittens*’ bowl. ',
  ' (the cats’) ',
  ' `code’s` ',
  ' A **big** one’s. ',
  ' See [it](https://cats.example/it’s) ',
  ' A ‹x› b. ',
  ' He said ʼhi ',
  ' Nested ‘a “b” c’ done. ',
  ' _x_’s bowl ',
];

/**
 The name a slice's floors refuse the replacement for.
 */
const DECLARED = ['kennel',];

/**
 One gain of a generated page: the slice it is addressed to, where its words stand there, the words and the text that replaces them,
 and the sides it brings an apostrophe in on.
 */
type Splice = {
  readonly slice: number;
  readonly start: number;
  readonly end: number;
  readonly text: string;
  readonly lead: boolean;
  readonly trail: boolean;
};

/**
 One generated page: the groups settled, the English of each slice, and the gains in the order of the page.
 */
type Page = {
  readonly settled: readonly SettledGroup[];
  readonly englishes: readonly { readonly sliceIndex: number; readonly english: string; }[];
  readonly splices: readonly Splice[];
};

/**
 The generator of a seed: a mulberry32 step, so a seed names one page.

 @param seed - seed

 @returns Draws, each in zero up to one

 @example
 ```ts
 const rand = randomOf({ seed: 918, },);
 ```
 */
function randomOf({ seed, }: { readonly seed: number; },): () => number {
  /**
   The generator's state, mutated only by the draws.
   */
  const state = { a: seed >>> 0, };
  return function draw(): number {
    state.a = (state.a + 0x6D_2B_79_F5) >>> 0;
    /**
     The scrambled state.
     */
    const scrambled = { t: state.a, };
    scrambled.t = Math.imul(
      scrambled.t ^ (scrambled.t >>> 15),
      scrambled.t | 1,
    );
    scrambled.t ^= scrambled.t + Math.imul(
      scrambled.t ^ (scrambled.t >>> 7),
      scrambled.t | 61,
    );
    return ((scrambled.t ^ (scrambled.t >>> 14)) >>> 0) / 4_294_967_296;
  };
}

/**
 Builds the page of a seed.

 @param seed - seed

 @param markup - whether the slices are drawn with markup too

 @returns Page

 @example
 ```ts
 const page = pageOf({ seed: 918, markup: false, },);
 ```
 */
function pageOf(
  {
    seed,
    markup,
  }: {
    readonly seed: number;
    readonly markup: boolean;
  },
): Page {
  const rand = randomOf({ seed, },);
  /**
   Draws one of the options.

   @param options - options

   @returns One of them
   */
  function pick<Option,>(options: readonly Option[],): Option {
    return nonNullishOrThrow(options[Math.floor(rand() * options.length,)],);
  }
  /**
   What a slice is drawn from.
   */
  const fragments = markup ? [...PLAIN, ...MARKUP,] : PLAIN;
  /**
   How many slices hold gains.
   */
  const count = 2 + Math.floor(rand() * 3,);
  /**
   The chosen sentences, which the first slice holds.
   */
  const slice0 = Array.from(
    { length: count, },
    function chosenAt(
      _slot,
      at,
    ): string {
      return `Bowl ${String(at + 1,)} of the parents’ is full.`;
    },
  )
    .join(' ',);
  /**
   The English of each slice, mutated only inside this function.
   */
  const englishes: { sliceIndex: number; english: string; }[] = [{ sliceIndex: 0, english: slice0, },];
  /**
   The groups, mutated only inside this function.
   */
  const settled: SettledGroup[] = [];
  /**
   The gains, mutated only inside this function.
   */
  const splices: Splice[] = [];
  for (let slice = 1; slice <= count; slice += 1) {
    /**
     Whether the slice holds two gains.
     */
    const two = rand() < 0.4;
    /**
     The first receiving words: the name the floors refuse only in a slice with one gain.
     */
    const first = `Bowl ${String(slice,)} of the ${pick(two ? ['family',] : ['family', 'family', 'kennel',],)}`;
    /**
     The second receiving words.
     */
    const second = `Cat ${String(slice,)} of the family`;
    /**
     The English of the slice.
     */
    const english = `${pick(fragments,)}Then ${first} ${pick(fragments,)}${two ? ` and ${second} ` : ''}is empty.${pick(fragments,)}`;
    englishes.push({
      sliceIndex: slice,
      english,
    },);
    for (const words of (two ? [first, second,] : [first,])) {
      /**
       The sides the gain brings an apostrophe in on.
       */
      const kind = pick(['trail', 'lead', 'both', 'trail', 'lead',],);
      const lead = (kind === 'lead') || (kind === 'both');
      const trail = (kind === 'trail') || (kind === 'both');
      /**
       The text that replaces the words.
       */
      const text = `${lead ? '’' : ''}${words.slice(0, words.indexOf(' of the ',),)} of the ${lead ? 'folks' : 'parents'}${trail ? '’' : ''}`;
      /**
       Where the words start in the slice.
       */
      const start = english.indexOf(words,);
      settled.push(withEdit({
        group: groupIn({
          places: [
            {
              slice: 0,
              english: slice0,
              quote: `Bowl ${String(slice,)} of the parents’`,
            },
            {
              slice,
              english,
              quote: words,
            },
          ],
        },),
        edit: {
          kind: 'splice',
          placePositions: [1,],
          sliceIndex: slice,
          start,
          end: start + words.length,
          text,
        },
      },),);
      splices.push({
        slice,
        start,
        end: start + words.length,
        text,
        lead,
        trail,
      },);
    }
  }
  return {
    settled,
    englishes,
    splices,
  };
}

/**
 The English of every slice with exactly the gains of a subset made.

 @param page - page

 @param subset - places of the gains made, in the order of the page

 @returns English of each slice

 @example
 ```ts
 const texts = textsWith({ page, subset: [0, 2,], },);
 ```
 */
function textsWith(
  {
    page,
    subset,
  }: {
    readonly page: Page;
    readonly subset: readonly number[];
  },
): readonly string[] {
  return page.englishes
    .map(function textOf({
      sliceIndex,
      english,
    },): string {
      /**
       The gains made in this slice, the last first so the earlier ones keep their places.
       */
      const mine = subset
        .map(function spliceAt(place,): Splice {
          return nonNullishOrThrow(page.splices[place],);
        },)
        .filter(function isHere({ slice, },): boolean {
          return slice === sliceIndex;
        },)
        .toSorted(function lastFirst(
          left,
          right,
        ): number {
          return right.start - left.start;
        },);
      return mine.reduce(function made(
        text,
        { start, end, text: replacement, },
      ): string {
        return `${text.slice(0, start,)}${replacement}${text.slice(end,)}`;
      }, english,);
    },);
}

/**
 Whether a subset of the gains is valid on the page: each gain mark of it is a proven apostrophe on the page made with it, and
 the floors pass every slice it touches.

 @param page - page

 @param subset - places of the gains made, in the order of the page

 @returns Whether it is valid

 @example
 ```ts
 const valid = isValid({ page, subset: [1, 2,], },);
 ```
 */
function isValid(
  {
    page,
    subset,
  }: {
    readonly page: Page;
    readonly subset: readonly number[];
  },
): boolean {
  /**
   The English of each slice with the gains made.
   */
  const texts = textsWith({
    page,
    subset,
  },);
  /**
   The page they make.
   */
  const made = pageOfSlices({
    slices: texts.map(function sliceOf(
      english,
      sliceIndex,
    ) {
      return {
        sliceIndex,
        english,
      };
    },),
  },);
  /**
   The proven apostrophes of the page.
   */
  const {
    leading,
    trailing,
  } = provenApostrophesInEnglish({ english: made.text, },);
  /**
   Whether every gain of the subset has its marks proven.
   */
  const provenAll = subset.every(function isProven(place,): boolean {
    /**
     The gain.
     */
    const gain = nonNullishOrThrow(page.splices[place],);
    /**
     How far the gains of the slice before it move it.
     */
    const shift = subset
      .map(function spliceAt(other,): Splice {
        return nonNullishOrThrow(page.splices[other],);
      },)
      .filter(function isBefore(other,): boolean {
        return (other.slice === gain.slice) && (other.start < gain.start);
      },)
      .reduce(function moved(
        total,
        other,
      ): number {
        return total + (other.text.length - (other.end - other.start));
      }, 0,);
    /**
     Where the gain's words stand on the page.
     */
    const at = (made.offsets.get(gain.slice,) ?? 0) + gain.start + shift;
    return ((!gain.lead) || leading.has(at,)) && ((!gain.trail) || trailing.has((at + gain.text.length) - 1,));
  },);
  if (!provenAll)
    return false;
  return [...new Set(subset.map(function sliceOf(place,): number {
    return nonNullishOrThrow(page.splices[place],).slice;
  },),),].every(function floorsPass(sliceIndex,): boolean {
    return consistencyFloorVerdict({
      sliceIndex,
      sourceText: '小猫打盹。',
      currentText: nonNullishOrThrow(page.englishes[sliceIndex],).english,
      candidateText: nonNullishOrThrow(texts[sliceIndex],),
      lineStructured: false,
      declared: [],
      declaredNames: DECLARED,
      disputedWordings: [],
    },)
      .kind === 'passes';
  },);
}

/**
 What the pass puts back on a page and what the brute force says of it.
 */
type Reading = {
  /**
   The places of the gains the pass made.
   */
  readonly made: readonly number[];

  /**
   Every valid subset of the largest size, in the order of the page.
   */
  readonly best: readonly (readonly number[])[];

  /**
   Whether the set the pass made is valid.
   */
  readonly madeIsValid: boolean;
};

/**
 Runs the pass over the page of a seed and reads it against every subset of the gains.

 @param seed - seed

 @param markup - whether the slices are drawn with markup too

 @returns What the pass made and the valid subsets of the largest size

 @example
 ```ts
 const reading = readingOf({ seed: 918, markup: false, },);
 ```
 */
function readingOf(
  {
    seed,
    markup,
  }: {
    readonly seed: number;
    readonly markup: boolean;
  },
): Reading {
  const page = pageOf({
    seed,
    markup,
  },);
  const applied = applyConsistency({
    settled: [...page.settled,],
    slices: factsOf({ englishes: [...page.englishes,], },) as readonly ConsistencySliceFacts[],
    declared: [],
    declaredNames: DECLARED,
    l: levelCapturingLogger({ lines: [], },),
  },);
  /**
   The places of the gains the pass made: the groups whose receiving place is replaced.
   */
  const made = applied.groups.flatMap(function replacedAt(
    group,
    place,
  ): readonly number[] {
    return (group.occurrences[1]?.outcome === 'replaced') ? [place,] : [];
  },);
  /**
   Every subset of the gains.
   */
  const subsets = Array.from(
    { length: 2 ** page.splices.length, },
    function subsetAt(
      _slot,
      mask,
    ): readonly number[] {
      return page.splices.flatMap(function placeIn(
        _splice,
        place,
      ): readonly number[] {
        return (((mask >> place) % 2) === 1) ? [place,] : [];
      },);
    },
  );
  /**
   The valid ones.
   */
  const valid = subsets.filter(function holds(subset,): boolean {
    return isValid({
      page,
      subset,
    },);
  },);
  /**
   The largest size among them.
   */
  const largest = Math.max(...valid.map(function sizeOf(subset,): number {
    return subset.length;
  },),);
  return {
    made,
    best: valid
      .filter(function isLargest(subset,): boolean {
        return subset.length === largest;
      },)
      .toSorted(function inPageOrder(
        left,
        right,
      ): number {
        return (left.find(function differs(
          place,
          at,
        ): boolean {
          return place !== right[at];
        },) ?? 0) - (right.find(function differs(
          place,
          at,
        ): boolean {
          return place !== left[at];
        },) ?? 0);
      },),
    madeIsValid: valid.some(function isMade(subset,): boolean {
      return subset.join(',',) === made.join(',',);
    },),
  };
}

/**
 The places of the gains the pass makes on a page.

 @param page - page

 @returns Places made

 @example
 ```ts
 const { made, } = madeOn({ page: pageOf({ seed: 918, markup: false, }), },);
 ```
 */
function madeOn(
  { page, }: { readonly page: Page; },
): {
  readonly made: readonly number[];
} {
  const applied = applyConsistency({
    settled: [...page.settled,],
    slices: factsOf({ englishes: [...page.englishes,], },) as readonly ConsistencySliceFacts[],
    declared: [],
    declaredNames: DECLARED,
    l: levelCapturingLogger({ lines: [], },),
  },);
  /**
   The places of the gains the pass made.
   */
  const made = applied.groups.flatMap(function replacedAt(
    group,
    place,
  ): readonly number[] {
    return (group.occurrences[1]?.outcome === 'replaced') ? [place,] : [];
  },);
  return { made, };
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: applyConsistency.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'KEEPS THE EARLIEST OF TWO SETS THE PAGE CAN HOLD ONE OF: a leading gain and a trailing gain that pair when both are '
            + 'made are each proven alone, and the set of the earlier slice is the one made',
          fn: async () => {
            /**
             The chosen sentences, which the first slice holds.
             */
            const slice0 = 'Bowl 1 of the parents’ is full. Bowl 2 of the parents’ is full.';
            /**
             The two receiving slices.
             */
            const englishes = [
              { sliceIndex: 0, english: slice0, },
              { sliceIndex: 1, english: 'Then Bowl 1 of the family is empty.', },
              { sliceIndex: 2, english: 'Then Bowl 2 of the family is empty.', },
            ];
            /**
             The two gains: a leading mark before the first slice's words, a trailing mark after the second slice's.
             */
            const splices: readonly Splice[] = [
              { slice: 1, start: 5, end: 25, text: '’Bowl 1 of the folks', lead: true, trail: false, },
              { slice: 2, start: 5, end: 25, text: 'Bowl 2 of the parents’', lead: false, trail: true, },
            ];
            /**
             The page.
             */
            const page: Page = {
              settled: splices.map(function groupOf(splice, at,): SettledGroup {
                /**
                 The receiving English of the gain.
                 */
                const { english, } = nonNullishOrThrow(englishes[splice.slice],);
                return withEdit({
                  group: groupIn({
                    places: [
                      { slice: 0, english: slice0, quote: `Bowl ${String(at + 1,)} of the parents’`, },
                      { slice: splice.slice, english, quote: `Bowl ${String(at + 1,)} of the family`, },
                    ],
                  },),
                  edit: {
                    kind: 'splice',
                    placePositions: [1,],
                    sliceIndex: splice.slice,
                    start: splice.start,
                    end: splice.end,
                    text: splice.text,
                  },
                },);
              },),
              englishes,
              splices,
            };
            expect([
              isValid({ page, subset: [0, 1,], },),
              isValid({ page, subset: [0,], },),
              isValid({ page, subset: [1,], },),
              madeOn({ page, },).made,
            ],).toEqual([false, true, true, [0,],],);
          },
        },),
        ...([
          [918, false,],
          [1_991, true,],
          [2_385, true,],
          [3_262, true,],
          [3_855, true,],
          [4_844, true,],
          [5_621, true,],
          [7_322, false,],
        ] as const).map(function seedTest([seed, markup,],) {
          return it({
            name: `PUTS BACK THE LARGEST SET THE PAGE CAN HOLD where the order of the page makes a smaller one: seed ${String(seed,)}`,
            fn: async () => {
              const {
                made,
                best,
                madeIsValid,
              } = readingOf({
                seed,
                markup,
              },);
              expect(madeIsValid,).toBe(true,);
              expect(made.length,).toBe(nonNullishOrThrow(best[0],).length,);
              // A pin for these seeds, not a law: on these pages the pass makes the earliest of the sets of the largest size, but where
              // gains of one slice overlap, the check stage lets the later one stand before the sets are looked at.
              expect(made,).toEqual(best[0],);
            },
          },);
        },),
      ],
    },),
  ],
},);
