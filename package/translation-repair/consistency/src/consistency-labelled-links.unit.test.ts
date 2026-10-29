/**
 Tests for a link with a label in the locator of a judge's quote: a link shows a reader its label and hides its brackets,
 destination, title and reference, so a word beside the link, or inside its label, is found as in the text a reader sees (n173).

 Fixtures are cat-themed invention; no corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  type ConsistencyVoice,
  countConsistency,
  locateWords,
} from '../dist/final/node/index.mjs';

/**
 The kinds of answer the locator gives for each pair of texts: one with a link written, and its twin, the text a reader sees of it.

 @param rows - text with the link, twin, and words asked for

 @returns Kind of answer for the text with the link of each row, and for the twin of each row

 @example
 ```ts
 const { kinds, twins, } = twinKindsOf({ rows: [['kit[kit](u)kit', 'kitkitkit', 'kit',],], },); // both ['none']
 ```
 */
function twinKindsOf(
  { rows, }: { readonly rows: readonly (readonly [string, string, string])[]; },
): { readonly kinds: readonly string[]; readonly twins: readonly string[]; } {
  return {
    kinds: rows.map(function linkedKindOf([linked, , words,],): string {
      return locateWords({
        haystack: linked,
        words,
      },).kind;
    },),
    twins: rows.map(function twinKindOf([, twin, words,],): string {
      return locateWords({
        haystack: twin,
        words,
      },).kind;
    },),
  };
}

/**
 The kind of answer for a text on a page whose other slices define one reference label.

 @param haystack - English of the slice

 @param words - words asked for

 @param defined - label a definition line of another slice defines

 @returns Kind of answer

 @example
 ```ts
 const kind = definedOnPage({ haystack: '[kit][r]', words: 'kit', defined: 'r', },); // 'once'
 ```
 */
function definedOnPage(
  {
    haystack,
    words,
    defined,
  }: {
    readonly haystack: string;
    readonly words: string;
    readonly defined: string;
  },
): string {
  /**
   The page: the slice, a blank line and the one definition.
   */
  const text = [haystack, `[${defined}]: u`,].join('\n\n',);
  return locateWords({
    haystack,
    words,
    page: {
      text,
      offset: 0,
    },
  },).kind;
}

await describe({
  name: '',
  children: [
    describe({
      name: locateWords.name,
      children: [
        //region A word beside the label of a link

        it({
          name: 'FINDS NO WORD ENDING OR STARTING AT THE BRACKETS OF A LINK WHOSE LABEL HOLDS LETTERS, since the text on the two sides of '
            + 'the brackets and the label is one word to a reader (n173)',
          fn: async () => {
            /**
             Each text, the text a reader sees of it, and the word asked for.
             */
            const rows: readonly (readonly [string, string, string])[] = [
              ['kit[kit](u&#116;)', 'kitkit', 'kit',],
              ['kit[kit](u)kit', 'kitkitkit', 'kit',],
              ['kit[tens](u)kit', 'kittenskit', 'kit',],
              ['tens[kit](u)tens', 'tenskittens', 'tens',],
              ['[kit](u)kit', 'kitkit', 'kit',],
              ['[kit](u)tens', 'kittens', 'kit',],
              ['[kit](u)tens', 'kittens', 'tens',],
              ['kit[kit ](u)', 'kitkit ', 'kit',],
              ['kit[kit](u "t")kit', 'kitkitkit', 'kit',],
              ['kit[kit](<u v>)kit', 'kitkitkit', 'kit',],
              ['kit[kit][r]kit\n\n[r]: u', 'kitkitkit\n\n[r]: u', 'kit',],
              ['kit[tens][r]kit\n\n[r]: u', 'kittenskit\n\n[r]: u', 'tens',],
              ['kit[kit](u)[kit](v)kit', 'kitkitkitkit', 'kit',],
              ['1[2](u)3', '123', '2',],
            ];
            const { kinds, twins, } = twinKindsOf({ rows, },);
            expect(kinds,).toEqual(twins,);
            expect(twins,).toEqual(rows.map(function noneFor(): string {
              return 'none';
            },),);
          },
        },),
        it({
          name: 'JOINS A WORD ACROSS A LABEL OF MARKS as a hyphen or an apostrophe joins it in the text a reader sees: the brackets and the '
            + 'destination are no neighbour of the mark (n173)',
          fn: async () => {
            /**
             Each text, the text a reader sees of it, and the word asked for.
             */
            const rows: readonly (readonly [string, string, string])[] = [
              ['kit[-](u)kit', 'kit-kit', 'kit',],
              ['kit[\'](u)kit', 'kit\'kit', 'kit',],
              ['kit-[kit](u)', 'kit-kit', 'kit',],
              ['kit\'[kit](u)', 'kit\'kit', 'kit',],
              ['[kit](u)-kit', 'kit-kit', 'kit',],
              ['[kit-](u)kit', 'kit-kit', 'kit',],
              ['[-kit](u)kit', '-kitkit', 'kit',],
              ['kit[-kit](u)', 'kit-kit', 'kit',],
              ['tens[-](u)tens', 'tens-tens', 'tens',],
              ['kit[-](u&#116;)kit', 'kit-kit', 'kit',],
            ];
            const { kinds, twins, } = twinKindsOf({ rows, },);
            expect(kinds,).toEqual(twins,);
            expect(twins,).toEqual(['none', 'none', 'none', 'none', 'none', 'none', 'none', 'none', 'none', 'none',],);
          },
        },),
        it({
          name: 'KEEPS THE PRESENT ANSWER where a space or a line break stands beside a link with a label, and where the label holds the '
            + 'words alone (n173)',
          fn: async () => {
            /**
             Each text, the text a reader sees of it, and the word asked for.
             */
            const rows: readonly (readonly [string, string, string])[] = [
              ['kit [kit](u) kit', 'kit kit kit', 'kit',],
              ['[kit](u)', 'kit', 'kit',],
              ['a [kit](u) b', 'a kit b', 'kit',],
              ['kit[ kit](u)', 'kit kit', 'kit',],
              ['kit[*kit*](u)', 'kit*kit*', 'kit',],
              ['The [cat](u) naps.', 'The cat naps.', 'cat',],
              ['The [the cat](u)\'s bowl.', 'The the cat\'s bowl.', 'cat',],
              ['[kit](u)tens', 'kittens', 'kit',],
              ['[kit](u)tens', 'kittens', 'tens',],
              ['kit[tens][r]kit\n\n[r]: u', 'kittenskit\n\n[r]: u', 'tens',],
              ['1[2](u)3', '123', '2',],
            ];
            const { kinds, twins, } = twinKindsOf({ rows, },);
            expect(kinds,).toEqual(twins,);
            expect(twins,).toEqual(['many', 'once', 'once', 'many', 'many', 'once', 'none', 'none', 'none', 'none', 'none',],);
          },
        },),
        it({
          name: 'COUNTS THE STANDS BESIDE A LINK WITH A LABEL as the text a reader sees does: the words on each side of the brackets and '
            + 'the label are one word, a space or a line break parts them (n173)',
          fn: async () => {
            /**
             Each text, the text a reader sees of it, and the word asked for.
             */
            const rows: readonly (readonly [string, string, string])[] = [
              ['kit [kit](u)kit', 'kit kitkit', 'kit',],
              ['kit[kit](u) kit', 'kitkit kit', 'kit',],
              ['kit[kit](u)\nkit', 'kitkit\nkit', 'kit',],
            ];
            const { kinds, twins, } = twinKindsOf({ rows, },);
            expect(kinds,).toEqual(twins,);
            expect(twins,).toEqual(['once', 'once', 'once',],);
          },
        },),

        //endregion A word beside the label of a link

        //region Emphasis and a reference the page defines

        it({
          name: 'READS THE EMPHASIS MARKS OF A LABEL AS THE PAGE SHOWS THEM: a reference defined in another slice is a link, so the '
            + 'asterisk in its label opens no emphasis that a later asterisk closes (n173)',
          fn: async () => {
            expect([
              definedOnPage({ haystack: '[*a][r] kit*-kit', words: 'kit', defined: 'r', },),
              definedOnPage({ haystack: '[*a][r] kit*-kit', words: 'kit', defined: 'x', },),
              locateWords({ haystack: '[*a][r] kit*-kit', words: 'kit', },).kind,
              locateWords({ haystack: '[*a](u) kit*-kit', words: 'kit', },).kind,
            ],).toEqual(['many', 'none', 'none', 'many',],);
          },
        },),

        //endregion Emphasis and a reference the page defines
      ],
    },),
    describe({
      name: countConsistency.name,
      children: [
        //region A frame's slot inside a span that holds a reference the page defines

        it({
          name: 'LOCATES A FRAME\'S SLOT BY THE REFERENCES THE PAGE DEFINES: the empty reference in the span is a link when another slice '
            + 'defines it, so the slot stands once, and as text where no slice defines it (n173)',
          fn: async () => {
            /**
             The span the three ballots quote, and the slot they give.
             */
            const quote = 'kit[][r]\'kit kit';
            /**
             The ballots, all alike.
             */
            const voices: readonly ConsistencyVoice[] = ['a', 'b', 'c',].map(function voiceBy(seat,): ConsistencyVoice {
              return {
                seat,
                reason: 'The first rendering is the more faithful.',
                quotes: [{ label: 'OCCURRENCE 1', quote, slot: 'kit', },],
                choice: 'OCCURRENCE 1',
                keep: [],
              };
            },);
            /**
             The rendering of the first place on a page that holds some definition line.
             */
            function renderingOnPage({ definition, }: { readonly definition: string; },): unknown {
              return countConsistency({
                voices,
                places: [
                  {
                    label: 'OCCURRENCE 1',
                    english: { kind: 'present', text: quote, },
                    page: { text: `${quote}\n\n${definition}`, offset: 0, },
                  },
                  {
                    label: 'OCCURRENCE 2',
                    english: { kind: 'present', text: 'In the evening, the sunlight on the sill lulled the kitten to sleep.', },
                  },
                ],
                framed: true,
                quorum: 2,
              },).renderings[0];
            }
            expect(renderingOnPage({ definition: '[r]: u', },),).toEqual({
              kind: 'located',
              quote,
              core: quote,
              start: 0,
              slot: 'kit',
              slotStart: 13,
              // The definition makes the slot a stand, so the rendering carries it for a reader that holds no page.
              definitions: '[r]: u',
            },);
            expect(renderingOnPage({ definition: '[x]: u', },),).toEqual({
              kind: 'located',
              quote,
              core: quote,
              start: 0,
            },);
          },
        },),

        //endregion A frame's slot inside a span that holds a reference the page defines
      ],
    },),
  ],
},);
