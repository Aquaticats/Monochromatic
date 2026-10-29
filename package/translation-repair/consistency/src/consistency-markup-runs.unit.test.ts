/**
 Tests for the two runs of markup beside a word: the reading run a single mark is read past (closing delimiters of emphasis by
 the parse and a link's closing `](…)`, the trailing side only) and the refusal run a gain is refused beside (wide, both
 sides), which the joints, the marks past a span, the proof and the edge apostrophe of a span all step over.

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
  CLOSING_CURLY_DOUBLE,
  closingRunMarksOf,
  EMPHASIS_CHARACTERS,
  emphasisDelimitersOf,
  HORIZONTAL_SPACE,
  LEADING_REFUSAL_RUN,
  TRAILING_REFUSAL_RUN,
} from '../dist/final/node/index.mjs';

/**
 A text with each mark that ends a reading run shown between white square brackets, so a case reads as the text it is about, the
 closing delimiters read by the parse as the proof reads them.

 @param text - English

 @returns Text with the marks that end a reading run bracketed; the text itself when none ends one

 @example
 ```ts
 const shown = markedRunEnd({ text: 'The *kittens*’ bowl', },); // 'The *kittens*⟦’⟧ bowl'
 ```
 */
function markedRunEnd({ text, }: { readonly text: string; },): string {
  /**
   The offsets of the marks that end a reading run.
   */
  const marks = closingRunMarksOf({
    text,
    closing: emphasisDelimitersOf({ english: text, },).closing,
  },);
  return Array.from(
    { length: text.length, },
    function unitShown(_, at,): string {
      return marks.has(at,) ? `⟦${text.charAt(at,)}⟧` : text.charAt(at,);
    },
  )
    .join('',);
}

/**
 Whether a character is in the trailing set of the refusal run and whether it is in the leading set.

 @param character - character asked about

 @returns Two answers, trailing first

 @example
 ```ts
 const held = setsHolding(')',); // [true, false]
 ```
 */
function setsHolding(character: string,): readonly [boolean, boolean] {
  return [TRAILING_REFUSAL_RUN.has(character,), LEADING_REFUSAL_RUN.has(character,),];
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: 'the characters of the refusal run',
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'HOLDS NO EMPHASIS CHARACTER IN EITHER SET, since the run adds the three emphasis characters itself, whether or not '
            + 'the parse makes one a delimiter',
          fn: async () => {
            expect([
              [...EMPHASIS_CHARACTERS,],
              [...EMPHASIS_CHARACTERS,].map(setsHolding,),
            ],).toEqual([['*', '_', '~',], [[false, false,], [false, false,], [false, false,],],],);
          },
        },),
        it({
          name: 'HOLDS THE CLOSING BRACKETS AND CLOSING QUOTATION MARKS ONLY IN THE TRAILING SET, and the opening ones only in the '
            + 'leading set, the straight double mark and the backtick being in both',
          fn: async () => {
            expect([
              [')', ']', CLOSING_CURLY_DOUBLE, '」', '』',].map(setsHolding,),
              ['(', '[', '“', '「', '『',].map(setsHolding,),
              ['"', '`',].map(setsHolding,),
            ],).toEqual([
              [[true, false,], [true, false,], [true, false,], [true, false,], [true, false,],],
              [[false, true,], [false, true,], [false, true,], [false, true,], [false, true,],],
              [[true, true,], [true, true,],],
            ],);
          },
        },),
        it({
          name: 'HOLDS EVERY CHARACTER JAVASCRIPT READS AS WHITE SPACE BUT THE LINE TERMINATORS IN THE SET OF HORIZONTAL WHITE SPACE, '
            + 'the space the proof\'s parting test reads, and no line break, zero-width character, letter or single mark, so a run '
            + 'steps over white space of every width and ends at a line terminator',
          fn: async () => {
            /**
             The characters of the first plane that JavaScript trims as white space (the characters its `\s` matches, line
             terminators included) and that are no line terminator.
             */
            const horizontal = Array.from(
              { length: 0x1_00_00, },
              function characterOf(_unused, code,): string {
                return String.fromCodePoint(code,);
              },
            )
              .filter(function isHorizontal(character,): boolean {
                return (character.trim() === '') && (!['\n', '\r', '\u{2028}', '\u{2029}',].includes(character,));
              },);
            expect([...HORIZONTAL_SPACE,].toSorted(),).toEqual(horizontal.toSorted(),);
            expect(horizontal.length > 20,).toBe(true,);
            expect([' ', '\t', '\u{00A0}', '\u{2009}', '\u{3000}', '\u{FEFF}', '\n', '\r', '\u{2028}', '\u{2029}', '\u{0085}', '\u{200B}', 'a', '\'', '’',].map(function held(character,): boolean {
              return HORIZONTAL_SPACE.has(character,);
            },),).toEqual([true, true, true, true, true, true, false, false, false, false, false, false, false, false, false,],);
            expect([' ', '\t', '\u{00A0}',].map(setsHolding,),).toEqual([[false, false,], [false, false,], [false, false,],],);
          },
        },),
        it({
          name: 'HOLDS NO LETTER, DIGIT OR SINGLE MARK IN EITHER SET, so a run ends at the word or the mark beside it',
          fn: async () => {
            expect(['a', '5', '\'', '’', '‘', '猫', '{', '}', '«', '»',].map(setsHolding,),).toEqual([
              [false, false,],
              [false, false,],
              [false, false,],
              [false, false,],
              [false, false,],
              [false, false,],
              [false, false,],
              [false, false,],
              [false, false,],
              [false, false,],
            ],);
          },
        },),
      ],
    },),
    describe({
      name: closingRunMarksOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'NAMES THE MARK THAT ENDS A RUN OF CLOSING DELIMITERS OF EMPHASIS BY THE PARSE OR A LINK\'S CLOSING `](…)`, after a '
            + 'Latin letter or a digit, and names one a letter follows as well',
          fn: async () => {
            expect([
              markedRunEnd({ text: 'The *kittens*’ bowl', },),
              markedRunEnd({ text: 'The **kittens**\' bowl', },),
              markedRunEnd({ text: 'The ~~kittens~~\u{02BC} bowl', },),
              markedRunEnd({ text: 'The _kittens_’ bowl', },),
              markedRunEnd({ text: 'The [kittens](https://cats.example/k/)’ bowl', },),
              markedRunEnd({ text: 'The **[kittens](x)**’ bowl', },),
              markedRunEnd({ text: 'The [**kittens**](x)’ bowl', },),
              markedRunEnd({ text: 'The 5 *cats 5*’', },),
              markedRunEnd({ text: 'The *kittens*’s bowl', },),
            ],).toEqual([
              'The *kittens*⟦’⟧ bowl',
              'The **kittens**⟦\'⟧ bowl',
              'The ~~kittens~~⟦\u{02BC}⟧ bowl',
              'The _kittens_⟦’⟧ bowl',
              'The [kittens](https://cats.example/k/)⟦’⟧ bowl',
              'The **[kittens](x)**⟦’⟧ bowl',
              'The [**kittens**](x)⟦’⟧ bowl',
              'The 5 *cats 5*⟦’⟧',
              'The *kittens*⟦’⟧s bowl',
            ],);
          },
        },),
        it({
          name: 'READS NO CLOSING BRACKET, QUOTATION MARK OR INLINE CODE AS PART OF THE RUN, since after them a single mark is most '
            + 'often a closing quotation mark and is read where it stands',
          fn: async () => {
            expect([
              'The "kittens"’ bowl',
              'The “kittens”’ bowl',
              'The (kittens)’ bowl',
              'The [kittens]’ bowl',
              'The `kittens`’ bowl',
              'The 「kittens」’ bowl',
              'He said cats are "ok"\' now.',
              'Come (here)’ now',
            ].map(function shown(text,): string {
              return markedRunEnd({ text, },);
            },),).toEqual([
              'The "kittens"’ bowl',
              'The “kittens”’ bowl',
              'The (kittens)’ bowl',
              'The [kittens]’ bowl',
              'The `kittens`’ bowl',
              'The 「kittens」’ bowl',
              'He said cats are "ok"\' now.',
              'Come (here)’ now',
            ],);
          },
        },),
        it({
          name: 'NAMES NO MARK WHEN NO LATIN LETTER OR DIGIT STANDS BEFORE THE RUN, when the run is empty or broken by a space, when a '
            + 'link\'s destination holds a space or is never closed, or when the mark is the opening curly single mark',
          fn: async () => {
            expect([
              'The kittens’ bowl',
              'The * ’ bowl',
              'The *kittens* ’ bowl',
              'The [kittens](a )’ bowl',
              'The [kittens](a’ bowl',
              'The 猫*’ bowl',
              'The *kittens*‘ bowl',
              '*’ bowl',
              '',
            ].map(function shown(text,): string {
              return markedRunEnd({ text, },);
            },),).toEqual([
              'The kittens’ bowl',
              'The * ’ bowl',
              'The *kittens* ’ bowl',
              'The [kittens](a )’ bowl',
              'The [kittens](a’ bowl',
              'The 猫*’ bowl',
              'The *kittens*‘ bowl',
              '*’ bowl',
              '',
            ],);
          },
        },),
      ],
    },),
  ],
},);
