/**
 Tests that run hand-built pages through the apply stage (`applyConsistency`) to pin what it makes of splices the planner never
 plans: a gain whose mark reads free on the page (a symbol, a pictograph, another script or a single mark stands beside it) must
 not block a later gain with a reason that is false on the final page, and a gain that ends in one of the eight single marks the
 counter and the record reader know must be proven like any other, never made unproven.

 Fixtures are cat-themed invention; no corpus content appears here.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  applyConsistency,
  type SettledGroup,
} from '../../dist/final/node/index.mjs';
import { levelCapturingLogger, } from '@monochromatic-dev/translation-repair-test-fixture/ts/capturing-logger.test-fixture.ts';
import {
  groupIn,
  withEdit,
} from './consistency-apply-group.test-fixture.ts';
import { factsOf, } from './consistency-round-page.test-fixture.ts';

/**
 The English of the first slice, which holds the chosen place.
 */
const FIRST = 'The dogs’ bowl is full.';

/**
 One splice of a later slice: the slice, the words it replaces and the words it puts in, and whether the case leaves it out.
 */
type HandSplice = {
  /**
   The English of the slice the splice is addressed to.
   */
  readonly english: string;

  /**
   The words the splice replaces.
   */
  readonly words: string;

  /**
   The words it puts in.
   */
  readonly text: string;

  /**
   Whether the case leaves the splice out of the page, which keeps its slice and shifts nothing.
   */
  readonly skip?: boolean;

  /**
   Further splices of the same slice, each the words it replaces and the words it puts in, after this one in the order of the
   groups.
   */
  readonly also?: readonly { readonly words: string; readonly text: string; }[];
};

/**
 What the apply stage made of a page.
 */
type Made = {
  /**
   The outcome of the splice of each group, in the order of the groups.
   */
  readonly outcomes: readonly string[];

  /**
   The English of every slice after the stage, the first slice included.
   */
  readonly slices: readonly string[];

  /**
   The reason each finding gives for leaving a splice out, `pairs`, `unproves` or `floors`, in the order of the findings.
   */
  readonly reasons: readonly string[];
};

/**
 Runs the apply stage over a page whose first slice holds the chosen place and whose later slices each hold one splice.

 @param splices - splices of the later slices, in slice order

 @returns Outcomes, slices and reason of each finding

 @example
 ```ts
 const made = applied({ splices: [{ english: 'A cat naps.', words: 'cat', text: 'dogs’', },], });
 ```
 */
function applied({ splices, }: { readonly splices: readonly HandSplice[]; },): Made {
  /**
   The English of every slice, the first slice first.
   */
  const englishes = [
    { sliceIndex: 0, english: FIRST, },
    ...splices.map(function sliceOf({ english, }, at,): { readonly sliceIndex: number; readonly english: string; } {
      return { sliceIndex: at + 1, english, };
    },),
  ];
  /**
   The settled groups, one for each splice that is made part of the case.
   */
  const settled = splices.flatMap(function groupsOf(splice, at,): readonly SettledGroup[] {
    if (splice.skip === true)
      return [];
    return [{ words: splice.words, text: splice.text, }, ...(splice.also ?? []),].map(function groupOf({ words, text, },): SettledGroup {
      /**
       Where the splice's words start in its slice.
       */
      const start = splice.english.indexOf(words,);
      return withEdit({
        group: groupIn({
          places: [
            { slice: 0, english: FIRST, quote: 'dogs’', },
            { slice: at + 1, english: splice.english, quote: words, },
          ],
        },),
        edit: {
          kind: 'splice',
          placePositions: [1,],
          sliceIndex: at + 1,
          start,
          end: start + words.length,
          text,
        },
      },);
    },);
  },);
  /**
   What the stage logged.
   */
  const lines: string[] = [];
  /**
   What the stage made.
   */
  const result = applyConsistency({
    settled,
    slices: factsOf({ englishes, },),
    declared: [],
    declaredNames: [],
    l: levelCapturingLogger({ lines, },),
  },);
  return {
    outcomes: result.groups.map(function outcomeOf({ occurrences, },): string {
      return occurrences[1]?.outcome ?? 'none';
    },),
    slices: englishes.map(function madeOf({ sliceIndex, english, },): string {
      return result.slices.find(function isSlice(entry,): boolean {
        return entry.sliceIndex === sliceIndex;
      },)
        ?.text
        ?? english;
    },),
    reasons: lines.flatMap(function reasonOf(line,): readonly string[] {
      if (line.includes('would read as a quotation mark there',))
        return ['pairs',];
      if (line.includes('would make the apostrophe that another replacement carries in lose its proof',))
        return ['unproves',];
      return line.includes('the floors refuse the slice with it',) ? ['floors',] : [];
    },),
  };
}

/**
 What stands beside the mark of the earlier gain, and the label of the character, in the pages whose earlier gain reads free.
 */
const BESIDE: readonly (readonly [string, string])[] = [
  ['pictograph', '🐱'],
  ['copyright sign', '©'],
  ['trademark sign', '™'],
  ['Cyrillic word', 'Привет'],
  ['Greek word', 'γάτα'],
  ['arrow', '→'],
  ['straight mark kept outside', '\''],
  ['curly mark kept outside', '’'],
];

/**
 The words of a splice that bring one of the low, reversed and angle single marks to an edge of the place, on both sides.
 */
const EIGHT_MARK_TEXTS: readonly string[] = [
  'dogs\u{201A}',
  'dogs\u{201B}',
  'dogs‹',
  'dogs›',
  '\u{201A}dogs',
  '\u{201B}dogs',
  '‹dogs',
  '›dogs',
];

await describe({
  name: applyConsistency.name,
  concurrency: DEFAULT_CONCURRENCY,
  children: [
    ...BESIDE.map(function freeCase([label, beside,],) {
      return it({
        name: `LEAVES THE EARLIER GAIN OUT BECAUSE ITS OWN MARK READS FREE (${label} beside it) AND STILL MAKES THE LATER GAIN, which `
          + 'is proven alone on the page without the earlier one',
        fn: async () => {
          expect(applied({
            splices: [
              { english: `The kittens${beside} bowl is empty.`, words: 'kittens', text: 'dogs’', },
              { english: 'A cat naps.', words: 'cat', text: 'dogs’', },
            ],
          },),).toEqual({
            outcomes: ['kept-current', 'replaced',],
            slices: [FIRST, `The kittens${beside} bowl is empty.`, 'A dogs’ naps.',],
            reasons: ['pairs',],
          },);
        },
      },);
    },),
    it({
      name: 'MAKES THE LATER GAIN WHEN THE EARLIER GAIN IS REFUSED BY THE FLOORS, a Han character beside its mark, since the '
        + 'refused slice keeps its wording and its mark is not on the page',
      fn: async () => {
        expect(applied({
          splices: [
            { english: 'The kittens猫 bowl is empty.', words: 'kittens', text: 'dogs’', },
            { english: 'A cat naps.', words: 'cat', text: 'dogs’', },
          ],
        },),).toEqual({
          outcomes: ['kept-current', 'replaced',],
          slices: [FIRST, 'The kittens猫 bowl is empty.', 'A dogs’ naps.',],
          reasons: ['floors',],
        },);
      },
    },),
    it({
      name: 'STILL MAKES BOTH GAINS WHEN THE MARK BESIDE THE EARLIER ONE IS A SPACE OR A FULL STOP, which it reads as trailing, and '
        + 'makes the later gain alone when the earlier splice is not part of the page',
      fn: async () => {
        expect([
          applied({
            splices: [
              { english: 'The kittens bowl is empty.', words: 'kittens', text: 'dogs’', },
              { english: 'A cat naps.', words: 'cat', text: 'dogs’', },
            ],
          },),
          applied({
            splices: [
              { english: 'The kittens🐱 bowl is empty.', words: 'kittens', text: 'dogs’', skip: true, },
              { english: 'A cat naps.', words: 'cat', text: 'dogs’', },
            ],
          },),
        ],).toEqual([
          {
            outcomes: ['replaced', 'replaced',],
            slices: [FIRST, 'The dogs’ bowl is empty.', 'A dogs’ naps.',],
            reasons: [],
          },
          {
            outcomes: ['replaced',],
            slices: [FIRST, 'The kittens🐱 bowl is empty.', 'A dogs’ naps.',],
            reasons: [],
          },
        ],);
      },
    },),
    ...EIGHT_MARK_TEXTS.map(function eightMarkCase(text,) {
      return it({
        name: `NEVER MAKES A SPLICE THAT ENDS OR BEGINS IN A LOW, REVERSED OR ANGLE SINGLE MARK (${text}), since a gain is a splice `
          + 'that brings any of the eight single marks to an edge and no page can prove such a mark an apostrophe, so the place '
          + 'keeps its words, left out by the check of the apostrophes or refused by the floors',
        fn: async () => {
          const made = applied({ splices: [{ english: 'A cat naps.', words: 'cat', text, },], },);
          expect({
            slices: made.slices,
            outcome: ['kept-current', 'kept-floor',].includes(made.outcomes[0] ?? '',),
          },).toEqual({ slices: [FIRST, 'A cat naps.',], outcome: true, },);
        },
      },);
    },),
    it({
      name: 'LEAVES A SPLICE OUT WITH THE REASON THAT IT WOULD UNPROVE A MARK THAT STANDS, on a page the planner never plans: two '
        + 'splices whose spans touch, the later one changing the character beside the earlier one\'s mark from a comma to a '
        + 'letter, while a loose opening mark before both is made, so that the first round leaves all three out and the '
        + 'trials then put the earlier one back before the later one is asked',
      fn: async () => {
        expect(applied({
          splices: [
            {
              english: 'A cat naps. The kittens, cats nap.',
              words: 'cat',
              text: '‘cat',
              also: [{ words: 'kittens', text: 'dogs’', }, { words: ', cats', text: 'xcats’', },],
            },
          ],
        },),).toEqual({
          outcomes: ['kept-current', 'replaced', 'kept-current',],
          slices: [FIRST, 'A cat naps. The dogs’, cats nap.',],
          reasons: ['pairs', 'unproves',],
        },);
      },
    },),
    it({
      name: 'MAKES THE SECOND OF TWO GAINS OF ONE SLICE WHEN THE FIRST ONE\'S MARK STANDS BESIDE A KEPT MARK (the minimal page of the '
        + 'connector fuzz, seed 4 with the first two splices kept), so the possessive the page can carry is repaired',
      fn: async () => {
        /**
         The English of the slice that holds both splices.
         */
        const second = 'Then Bowl 1 of the family’Cat 1 of the family is empty. He said \'quoted\' twice. ';
        /**
         The English of the other slices of the page.
         */
        const others = [
          ' `code’s` Then Bowl 2 of the family  He said ʼhi is empty.',
          ' Nested ‘a “b” c’ done. Then Bowl 3 of the kennel  _x_’s bowl is empty. Close here.” ',
          'Then Bowl 4 of the family*Cat 4 of the family is empty. Close here.” ',
        ];
        /**
         The first slice of that page, which holds four chosen places.
         */
        const first = 'Bowl 1 of the parents’ is full. Bowl 2 of the parents’ is full. Bowl 3 of the parents’ is full. '
          + 'Bowl 4 of the parents’ is full.';
        /**
         The two splices of the second slice.
         */
        const words = [
          { quote: 'Bowl 1 of the family', text: 'Bowl 1 of the parentsʼ', },
          { quote: 'Cat 1 of the family', text: 'Cat 1 of the parentsʼ', },
        ];
        /**
         The groups, each with one splice of the second slice.
         */
        const settled = words.map(function groupOf({ quote, text, },): SettledGroup {
          /**
           Where the words start in the second slice.
           */
          const start = second.indexOf(quote,);
          return withEdit({
            group: groupIn({
              places: [
                { slice: 0, english: first, quote: 'Bowl 1 of the parents’', },
                { slice: 1, english: second, quote, },
              ],
            },),
            edit: {
              kind: 'splice',
              placePositions: [1,],
              sliceIndex: 1,
              start,
              end: start + quote.length,
              text,
            },
          },);
        },);
        /**
         The English of every slice, the first slice first.
         */
        const englishes = [first, second, ...others,].map(function sliceOf(english, sliceIndex,) {
          return { sliceIndex, english, };
        },);
        /**
         What the stage made.
         */
        const result = applyConsistency({
          settled,
          slices: factsOf({ englishes, },),
          declared: [],
          declaredNames: [],
          l: levelCapturingLogger({ lines: [], },),
        },);
        expect({
          outcomes: result.groups.map(function outcomeOf({ occurrences, },): string {
            return occurrences[1]?.outcome ?? 'none';
          },),
          second: result.slices.find(function isSecond(entry,): boolean {
            return entry.sliceIndex === 1;
          },)
            ?.text,
        },).toEqual({
          outcomes: ['kept-current', 'replaced',],
          second: 'Then Bowl 1 of the family’Cat 1 of the parentsʼ is empty. He said \'quoted\' twice. ',
        },);
      },
    },),
  ],
},);
