/**
 Tests for the closing joints of an English with links in it (`consistency-closing-links.ts`, n169): the offsets at which a run of
 closing marks ends in an apostrophe that a letter or a digit follows, read over the text a reader sees where the parse holds a link
 that shows nothing or only closing marks, and the reading of the text alone where no link is given.

 Fixtures are cat-themed invention.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  closingJointsOf,
  emphasisDelimitersOf,
  foldedLinksOf,
  foldWithOffsets,
  locateWords,
  NO_FOLDED_LINKS,
} from '../dist/final/node/index.mjs';

/**
 Whether the closing joints of an English, read with the links the fold holds, include an offset.

 @param english - English, which folds unit for unit

 @param at - offset asked about

 @returns Whether a stand ending at the offset lies inside a word made with an apostrophe after closing marks

 @example
 ```ts
 const inside = jointAt({ english: 'kit[\\]](u)\'kit', at: 3, },); // true
 ```
 */
function jointAt(
  {
    english,
    at,
  }: {
    readonly english: string;
    readonly at: number;
  },
): boolean {
  /**
   The English in the fold.
   */
  const folded = foldWithOffsets({ text: english, },);
  return closingJointsOf({
    folded: folded.folded,
    originals: folded.originals,
    closing: emphasisDelimitersOf({ english, },).closing,
    links: foldedLinksOf({
      english,
      folded,
    },),
  },)
    .has(at,);
}

await describe({
  name: closingJointsOf.name,
  concurrency: 1,
  children: [
    it({
      name: 'CROSSES A LINK THAT SHOWS NOTHING OR ONLY CLOSING MARKS, so the joint at the offset where the link starts is the joint of '
        + 'the text a reader sees there, whatever the link writes in its destination, and the run of links is one pass (n169)',
      fn: async () => {
        /**
         Each English with a link, the offset a stand of `kit` ends at, and the text a reader sees of it.
         */
        const rows: readonly (readonly [string, number, string])[] = [
          [String.raw`kit[\]](u)'kit`, 3, 'kit]\'kit',],
          ['kit[”](u)\'kit', 3, 'kit”\'kit',],
          ['kit[))](u)\'kit', 3, 'kit))\'kit',],
          ['kit[](u)]\'kit', 3, 'kit]\'kit',],
          ['kit[](u)[](v)]\'kit', 3, 'kit]\'kit',],
          [String.raw`kit[](u)[\]](v)'kit`, 3, 'kit]\'kit',],
          [String.raw`kit[\]](u)[](v)'kit`, 3, 'kit]\'kit',],
          ['"kit"[](u)\'s', 4, '"kit"\'s',],
          ['**kit**[](u)\'s', 7, '**kit**\'s',],
          ['(kit)[](u)\'s', 4, '(kit)\'s',],
          ['kit[](u)\'kit', 3, 'kit\'kit',],
          ['kit[-](u)\'kit', 3, 'kit-\'kit',],
          ['kit[x](u)\'kit', 3, 'kitx\'kit',],
          ['kit[()](u)\'kit', 3, 'kit()\'kit',],
          ['kit[](u)', 3, 'kit',],
        ];
        expect(rows.map(function linkedJointOf([english, at,],): boolean {
          return jointAt({
            english,
            at,
          },);
        },),).toEqual(rows.map(function twinJointOf([, at, twin,],): boolean {
          return jointAt({
            english: twin,
            at,
          },);
        },),);
        expect(rows.map(function twinJointOf([, at, twin,],): boolean {
          return jointAt({
            english: twin,
            at,
          },);
        },),).toEqual([true, true, true, true, true, true, true, true, true, true, false, false, false, false, false,],);
        expect(jointAt({ english: `kit${'[](u)'.repeat(2_000,)}]'kit`, at: 3, },),).toBe(true,);
        expect(jointAt({ english: `kit${'[](u)'.repeat(2_000,)}'kit`, at: 3, },),).toBe(false,);
      },
    },),

    it({
      name: 'ALSO NAMES THE OFFSET AFTER A LINK THAT SHOWS NOTHING when a closing mark stands before it, since a stand that ends in the '
        + 'closing mark ends at the link as it ends at the apostrophe in the text a reader sees (n169)',
      fn: async () => {
        expect([
          jointAt({ english: '"kit"[](u)\'s', at: 5, },),
          jointAt({ english: '"kit"\'s', at: 5, },),
          jointAt({ english: '"kit"[](u)[](v)\'s', at: 5, },),
          jointAt({ english: '"kit"[](u)[](v)\'s', at: 10, },),
        ],).toEqual([true, true, true, true,],);
      },
    },),

    it({
      name: 'COUNTS A LINK WITH A LABEL AS A CLOSING BRACKET ONLY WHEN ITS LABEL ENDS IN A CLOSING MARK: the destination of '
        + '`[kit!](u)`, `[Kit.](u)` and `[kit ](u)` reads as no run of closing marks before the apostrophe, as the text a reader '
        + 'sees (`kit!\'s`) reads, and `[kit)](u)\'s` keeps the joints of `kit)\'s`',
      fn: async () => {
        /**
         The offsets at which a stand may end to lie inside a word, for each English.
         */
        const jointsOf = ['[kit!](u)\'s bowl', '[Kit.](u)\'s bowl', '[kit ](u)\'s bowl', '[kit)](u)\'s bowl',].map(
          function jointsOfEnglish(english,): readonly number[] {
            /**
             The English in the fold.
             */
            const folded = foldWithOffsets({ text: english, },);
            return [
              ...closingJointsOf({
                folded: folded.folded,
                originals: folded.originals,
                closing: emphasisDelimitersOf({ english, },).closing,
                links: foldedLinksOf({
                  english,
                  folded,
                },),
              },),
            ].toSorted(function ascending(
              left,
              right,
            ): number {
              return left - right;
            },);
          },
        );
        expect(jointsOf,).toEqual([[], [], [], [4, 5, 8, 9,],],);
      },
    },),

    it({
      name: 'CUTS A STAND THAT BEGINS AFTER THE APOSTROPHE OF A LINK WITH A LABEL ALIKE IN PLAIN TEXT AND IN THE LINK: `s bowl` '
        + 'is no part of a longer word after `[kit!](u)\'`, `[Kit.](u)\'` or `[kit ](u)\'`, as after `kit!\'`, and is cut '
        + 'after a label that ends in a closing mark, as after `kit)\'`',
      fn: async () => {
        /**
         Each linked English and the English of the same text without the link.
         */
        const rows: readonly (readonly [string, string])[] = [
          ['[kit!](u)\'s bowl', 'kit!\'s bowl',],
          ['[Kit.](u)\'s bowl', 'Kit.\'s bowl',],
          ['[kit ](u)\'s bowl', 'kit \'s bowl',],
          ['[kit)](u)\'s bowl', 'kit)\'s bowl',],
        ];
        expect(rows.map(function linkedKind([linked,],): string {
          return locateWords({ haystack: linked, words: 's bowl', },).kind;
        },),).toEqual(rows.map(function plainKind([, plain,],): string {
          return locateWords({ haystack: plain, words: 's bowl', },).kind;
        },),);
        expect(rows.map(function plainKind([, plain,],): string {
          return locateWords({ haystack: plain, words: 's bowl', },).kind;
        },),).toEqual(['once', 'once', 'once', 'none',],);
      },
    },),

    it({
      name: 'READS THE TEXT ALONE where no link is given or the parse holds none that shows nothing or only closing marks',
      fn: async () => {
        /**
         The English, which holds a link that shows a word.
         */
        const english = 'Fill [the cat](x)\'s bowl.';
        /**
         The English in the fold.
         */
        const folded = foldWithOffsets({ text: english, },);
        /**
         The joints with the links the parse holds, and with none given.
         */
        const [withLinks, withNone,] = [
          foldedLinksOf({
            english,
            folded,
          },),
          NO_FOLDED_LINKS,
        ].map(function jointsWith(links,): ReadonlySet<number> {
          return closingJointsOf({
            folded: folded.folded,
            originals: folded.originals,
            closing: new Set(),
            links,
          },);
        },);
        expect([...(withLinks ?? []),],).toEqual([...(withNone ?? []),],);
        expect(withNone?.has(13,),).toBe(true,);
        expect(closingJointsOf({ folded: english, originals: Array.from(english,), closing: new Set(), },).has(13,),).toBe(true,);
      },
    },),
  ],
},);
