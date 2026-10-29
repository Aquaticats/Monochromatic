/**
 Tests for the forward reading of a stand: whether, past the closing marks that follow the stand (a closing
 quotation mark, a bracket, a link and its destination, emphasis), an apostrophe that a letter or a digit follows
 comes next, which marks close and which open, and where the reading gives up.

 Fixtures are cat-themed invention; no corpus content appears here.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { normalizePunctuation, } from '@monochromatic-dev/translation-repair-typography';
import {
  closingJointsOf,
  emphasisDelimitersOf,
  markPastRunOf,
} from '../dist/final/node/index.mjs';

/**
 The code points of the horizontal spaces the proof's parting test reads as white space: JavaScript white space without the
 line terminators, which are the tab, the vertical tab, the form feed, the space, the no-break space, the Ogham space mark,
 the spaces from the en quad to the hair space, the narrow no-break space, the medium mathematical space, the
 ideographic space and the byte order mark.
 */
const HORIZONTAL_SPACE_CODES: readonly number[] = [
  0x09,
  0x0B,
  0x0C,
  0x20,
  0xA0,
  0x16_80,
  0x20_00,
  0x20_01,
  0x20_02,
  0x20_03,
  0x20_04,
  0x20_05,
  0x20_06,
  0x20_07,
  0x20_08,
  0x20_09,
  0x20_0A,
  0x20_2F,
  0x20_5F,
  0x30_00,
  0xFE_FF,
];

/**
 The code points that render nothing and are no white space, a sample of Unicode's Default_Ignorable_Code_Point: the soft
 hyphen, the combining grapheme joiner, the Arabic letter mark, the Hangul fillers, the Khmer inherent vowels, the Mongolian
 free variation selectors and vowel separator, the zero-width characters and direction marks, the embedding controls, the word
 joiner and the invisible operators, the halfwidth Hangul filler, the variation selectors of the basic plane, the musical
 formatting characters, the tag characters and the variation selectors supplement (the last three beyond the first plane, two
 UTF-16 units each).
 */
const IGNORABLE_CODES: readonly number[] = [
  0xAD,
  0x03_4F,
  0x06_1C,
  0x11_5F,
  0x11_60,
  0x17_B4,
  0x17_B5,
  0x18_0B,
  0x18_0D,
  0x18_0E,
  0x20_0B,
  0x20_0C,
  0x20_0D,
  0x20_0E,
  0x20_0F,
  0x20_2A,
  0x20_2E,
  0x20_60,
  0x20_64,
  0x20_6F,
  0x31_64,
  0xFE_00,
  0xFE_0F,
  0xFF_A0,
  0x1_D1_73,
  0x1_D1_7A,
  0xE_00_01,
  0xE_00_7F,
  0xE_01_00,
  0xE_01_EF,
];

/**
 The code points that end the run: the NUL character, which a page shows as the replacement character U+FFFD.
 */
const NOT_IGNORABLE_CODES: readonly number[] = [
  0x00,
];

/**
 The code points that show nothing yet are no default-ignorable code point, which the run steps over: the controls from U+0001 to
 U+0008, from U+000E to U+001F and U+007F, and the Braille blank U+2800, which renders as an empty cell, a blank.
 */
const BLANK_CODES: readonly number[] = [
  0x01,
  0x02,
  0x03,
  0x04,
  0x05,
  0x06,
  0x07,
  0x08,
  0x0E,
  0x0F,
  0x10,
  0x11,
  0x12,
  0x13,
  0x14,
  0x15,
  0x16,
  0x17,
  0x18,
  0x19,
  0x1A,
  0x1B,
  0x1C,
  0x1D,
  0x1E,
  0x1F,
  0x7F,
  0x28_00,
];

/**
 The C1 controls the run steps over (n156, W4): every one from U+0080 to U+009F but the next-line control U+0085, which is a line
 break. The page parser keeps each as text and a browser draws it with no width.
 */
const C1_CODES: readonly number[] = [
  0x80,
  0x81,
  0x82,
  0x83,
  0x84,
  0x86,
  0x87,
  0x88,
  0x89,
  0x8A,
  0x8B,
  0x8C,
  0x8D,
  0x8E,
  0x8F,
  0x90,
  0x91,
  0x92,
  0x93,
  0x94,
  0x95,
  0x96,
  0x97,
  0x98,
  0x99,
  0x9A,
  0x9B,
  0x9C,
  0x9D,
  0x9E,
  0x9F,
];

/**
 The code points the run reads as a single line break: the next-line control and the line and paragraph separators, besides
 the line feed and the carriage return.
 */
const SEPARATOR_CODES: readonly number[] = [
  0x85,
  0x20_28,
  0x20_29,
];

/**
 Whether the one stand of a word in a text (a stand is a place where words lie in an English, defined in
 `consistency-fold.ts`) is followed by closing marks and an apostrophe that a letter or a digit follows, the text read as
 the locator reads an English.

 @param text - text before the fold, of characters in the first plane

 @param word - word, which stands once in the text

 @returns Whether the reading finds the apostrophe

 @example
 ```ts
 const inside = followedByApostropheIn({ text: 'Fill the "cat"\'s bowl.', word: 'cat', },); // true
 ```
 */
function followedByApostropheIn(
  {
    text,
    word,
  }: {
    readonly text: string;
    readonly word: string;
  },
): boolean {
  const folded = normalizePunctuation({ text, },);
  return closingJointsOf({
    folded,
    originals: Array.from(text,),
    // The texts of these cases fold unit for unit, so the offsets of the parse are the offsets of the fold.
    closing: emphasisDelimitersOf({ english: text, },).closing,
  },)
    .has(folded.indexOf(word,) + word.length,);
}

/**
 The mark past the run that follows a word of a text, as a text.

 @param text - English

 @param word - word, which stands once in the text

 @returns Mark, empty when none ends the run

 @example
 ```ts
 const mark = afterOf({ text: 'Fill the *kittens*’ bowl.', word: 'kittens', },); // '’'
 ```
 */
function afterOf({ text, word, }: { readonly text: string; readonly word: string; },): string {
  return markPastRunOf({ text, at: text.indexOf(word,) + word.length, side: 'after', },).join('',);
}

/**
 The mark past the run that precedes a word of a text, as a text.

 @param text - English

 @param word - word, which stands once in the text

 @returns Mark, empty when none ends the run

 @example
 ```ts
 const mark = beforeOf({ text: 'He said \'*5 cats* now.', word: '5 cats', },); // '\''
 ```
 */
function beforeOf({ text, word, }: { readonly text: string; readonly word: string; },): string {
  return markPastRunOf({ text, at: text.indexOf(word,), side: 'before', },).join('',);
}

/**
 The mark past the run on one side of a word of a text, as a text.

 @param text - English

 @param word - word, which stands once in the text

 @param side - `after` for the run that follows the word, `before` for the run that precedes it

 @returns Mark, empty when none ends the run

 @example
 ```ts
 const mark = pastOf({ text: 'Fill the *kittens*’ bowl.', word: 'kittens', side: 'after', },); // '’'
 ```
 */
function pastOf(
  {
    text,
    word,
    side,
  }: {
    readonly text: string;
    readonly word: string;
    readonly side: 'after' | 'before';
  },
): string {
  return markPastRunOf({
    text,
    at: (side === 'after') ? (text.indexOf(word,) + word.length) : text.indexOf(word,),
    side,
  },).join('',);
}

/**
 The mark past the run that follows or precedes the word `kittens`, for each code point put between a head and a tail.

 @param codes - code points, one text for each

 @param head - text before the character, which holds the word `kittens` once

 @param tail - text after the character

 @param side - `after` for the run that follows the word, `before` for the run that precedes it

 @returns Mark for each code point, empty where none ends the run

 @example
 ```ts
 const marks = marksAround({ codes: [0x20_0B,], head: 'The kittens', tail: '’ bowl.', side: 'after', },); // ['’']
 ```
 */
function marksAround(
  {
    codes,
    head,
    tail,
    side,
  }: {
    readonly codes: readonly number[];
    readonly head: string;
    readonly tail: string;
    readonly side: 'after' | 'before';
  },
): readonly string[] {
  return codes.map(function markFor(code,): string {
    return pastOf({ text: `${head}${String.fromCodePoint(code,)}${tail}`, word: 'kittens', side, },);
  },);
}

/**
 Every offset of a text that the reading records, in ascending order.

 @param text - text before the fold, of characters in the first plane

 @returns Offsets

 @example
 ```ts
 const offsets = recordedFor({ text: 'a)))\'s', },); // [1, 2, 3, 4]
 ```
 */
function recordedFor({ text, }: { readonly text: string; },): readonly number[] {
  return [
    ...closingJointsOf({
      folded: normalizePunctuation({ text, },),
      originals: Array.from(text,),
      closing: emphasisDelimitersOf({ english: text, },).closing,
    },),
  ].toSorted(function ascending(
    left,
    right,
  ): number {
    return left - right;
  },);
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: closingJointsOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'IS TRUE PAST A RUN OF CLOSING MARKS BEFORE AN APOSTROPHE AND A LETTER OR A DIGIT: a straight or curly closing '
            + 'double quotation mark, a closing round or square bracket, a link with its destination, emphasis, and any '
            + 'mix, whether the closing mark is the stand\'s last character or follows it',
          fn: async () => {
            expect([
              followedByApostropheIn({ text: 'Fill the "cat"\'s bowl.', word: 'cat', },),
              followedByApostropheIn({ text: 'Fill the "cat"\'s bowl.', word: '"cat"', },),
              followedByApostropheIn({ text: 'Fill the “cat”’s bowl.', word: 'cat', },),
              followedByApostropheIn({ text: 'Fill the (grey cat)\'s bowl.', word: 'grey cat', },),
              followedByApostropheIn({ text: 'Fill the [grey cat]\'s bowl.', word: '[grey cat]', },),
              followedByApostropheIn({ text: 'Fill [the cat](x)\'s bowl.', word: 'the cat', },),
              followedByApostropheIn({ text: 'Fill [the cat](x)\'s bowl.', word: '[the cat](x)', },),
              followedByApostropheIn({ text: 'Fill the **"cat"**\'s bowl.', word: 'cat', },),
              followedByApostropheIn({ text: 'Fill the "cat"\'5 bowl.', word: 'cat', },),
              followedByApostropheIn({ text: 'Fill the cat"\'s bowl.', word: 'cat', },),
            ],).toEqual([true, true, true, true, true, true, true, true, true, true,],);
          },
        },),
        it({
          name: 'IS FALSE WHERE NO APOSTROPHE FOLLOWS THE CLOSING MARKS, where none closes, where the apostrophe is a corner '
            + 'bracket the fold wrote as one or is followed by no letter or digit, or where the stand ends in no closing mark '
            + 'and the apostrophe stands right after it',
          fn: async () => {
            expect([
              followedByApostropheIn({ text: 'Fill the "cat" bowl.', word: 'cat', },),
              followedByApostropheIn({ text: 'Fill the cat', word: 'cat', },),
              followedByApostropheIn({ text: 'Say "hi"\' now.', word: 'hi', },),
              followedByApostropheIn({ text: 'Say "hi"\'', word: 'hi', },),
              followedByApostropheIn({ text: 'Say "hi"\'. Then nap.', word: 'hi', },),
              followedByApostropheIn({ text: 'He called it 『cat』\'s bowl.', word: 'cat', },),
              followedByApostropheIn({ text: 'He called it 「cat」\'s bowl.', word: 'cat', },),
              followedByApostropheIn({ text: 'Fill the bowl.\'s cat', word: 'bowl.', },),
              followedByApostropheIn({ text: 'Fill the cat\'s bowl.', word: 'cat', },),
              followedByApostropheIn({ text: 'Fill [the cat](x)(y)\'s bowl.', word: 'the cat', },),
            ],).toEqual([false, false, false, false, false, false, false, false, false, false,],);
          },
        },),
        it({
          name: 'READS AN APOSTROPHE FOLLOWED BY A COMPATIBILITY LETTER OR DIGIT AS BESIDE A WORD, by the neighbour test of the '
            + 'consistency pass: a mathematical script letter beyond the first plane (`𝓠𝓾𝓪𝓷’s`), a fullwidth letter, a ligature and a '
            + 'fullwidth digit; and a letter whose NFKC form is Greek or a fraction, a circled digit and a superscript digit as '
            + 'beside none',
          fn: async () => {
            expect([
              // U+1D4E0 MATHEMATICAL BOLD SCRIPT CAPITAL Q, NFKC `Q`.
              followedByApostropheIn({ text: 'Fill the "cat"\'\u{1D4E0}uan bowl.', word: 'cat', },),
              // U+FF21 FULLWIDTH LATIN CAPITAL LETTER A, NFKC `A`.
              followedByApostropheIn({ text: 'Fill the "cat"\'\u{FF21}s bowl.', word: 'cat', },),
              // U+FB01 LATIN SMALL LIGATURE FI, NFKC `fi`.
              followedByApostropheIn({ text: 'Fill the "cat"\'\u{FB01}sh bowl.', word: 'cat', },),
              // U+FF11 FULLWIDTH DIGIT ONE, a decimal digit, NFKC `1`.
              followedByApostropheIn({ text: 'Fill the "cat"\'\u{FF11} bowl.', word: 'cat', },),
              // U+00B5 MICRO SIGN, NFKC U+03BC GREEK SMALL LETTER MU.
              followedByApostropheIn({ text: 'Fill the "cat"\'\u{00B5}s bowl.', word: 'cat', },),
              // U+00BD VULGAR FRACTION ONE HALF, NFKC `1`, U+2044 FRACTION SLASH and `2`.
              followedByApostropheIn({ text: 'Fill the "cat"\'\u{00BD} bowl.', word: 'cat', },),
              // U+2460 CIRCLED DIGIT ONE (category No, NFKC `1`) and U+00B2 SUPERSCRIPT TWO: no letter and no decimal digit.
              followedByApostropheIn({ text: 'Fill the "cat"\'\u{2460} bowl.', word: 'cat', },),
              followedByApostropheIn({ text: 'Fill the "cat"\'\u{00B2} bowl.', word: 'cat', },),
            ],).toEqual([true, true, true, true, false, false, false, false,],);
          },
        },),
        it({
          name: 'READS AN EMPHASIS MARK OR A STRAIGHT QUOTATION MARK AS CLOSING ONLY WHEN A CHARACTER THAT IS NO WHITE SPACE '
            + 'STANDS BEFORE IT, and a curly opening quotation mark as closing nothing, so a stand that ends in an opening mark '
            + 'before an apostrophe is not inside a word',
          fn: async () => {
            expect([
              followedByApostropheIn({ text: 'Say *\'Tis the season*', word: 'Say *', },),
              followedByApostropheIn({ text: 'He said "\'Tis" twice.', word: 'He said "', },),
              followedByApostropheIn({ text: 'He said “\'Tis” twice.', word: 'He said "', },),
              followedByApostropheIn({ text: '"\'Tis" is old.', word: '"', },),
              followedByApostropheIn({ text: 'Say *the cat*\'s bowl.', word: 'cat', },),
              followedByApostropheIn({ text: 'Say “hi”\'s bowl.', word: 'hi', },),
            ],).toEqual([false, false, false, false, true, true,],);
          },
        },),
        it({
          name: 'RECORDS EVERY OFFSET FROM WHICH A RUN OF CLOSING MARKS REACHES THE APOSTROPHE, in one reading of the whole text, '
            + 'and none for an apostrophe that no closing mark precedes',
          fn: async () => {
            expect([
              recordedFor({ text: 'a)))\'s', },),
              recordedFor({ text: 'a *b*\'s c', },),
              recordedFor({ text: '\'s', },),
              recordedFor({ text: 'a \'s', },),
              recordedFor({ text: '[a](b)\'s', },),
              recordedFor({ text: '[a](b c)\'s', },),
            ],).toEqual([[1, 2, 3, 4,], [4, 5,], [], [], [2, 5, 6,], [7, 8,],],);
          },
        },),
        it({
          name: 'FOLLOWS A RUN OF CLOSING MARKS OF ANY LENGTH AND A LINK DESTINATION OF ANY LENGTH, and gives up only at a '
            + 'destination with white space in it, one never closed, or one at the edge of the text',
          fn: async () => {
            expect([
              followedByApostropheIn({ text: `Fill the cat${')'.repeat(9,)}'s bowl.`, word: 'cat', },),
              followedByApostropheIn({ text: `Fill the cat${')'.repeat(10_000,)}'s bowl.`, word: 'cat', },),
              followedByApostropheIn({ text: `Fill [the cat](${'a'.repeat(512,)})'s bowl.`, word: 'the cat', },),
              followedByApostropheIn({ text: `Fill [the cat](${'a'.repeat(100_000,)})'s bowl.`, word: 'the cat', },),
              followedByApostropheIn({ text: `Fill the cat${')'.repeat(10_000,)}' bowl.`, word: 'cat', },),
              followedByApostropheIn({ text: 'Fill [the cat](x y)\'s bowl.', word: 'the cat', },),
              followedByApostropheIn({ text: 'Fill [the cat](unclosed', word: 'the cat', },),
              followedByApostropheIn({ text: 'Fill [the cat](', word: 'the cat', },),
            ],).toEqual([true, true, true, true, false, false, false, false,],);
          },
        },),
      ],
    },),
    describe({
      name: `${closingJointsOf.name} by the parse`,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'READS AN EMPHASIS MARK AS CLOSING ONLY WHERE THE PARSE MAKES IT THE CLOSING DELIMITER of an emphasis, strong or '
            + 'strikethrough stretch, so a lone asterisk or underscore is a character, and a stretch still joins',
          fn: async () => {
            expect([
              followedByApostropheIn({ text: 'The cat*\'s bowl.', word: 'cat', },),
              followedByApostropheIn({ text: 'The cat_\'s bowl.', word: 'cat', },),
              followedByApostropheIn({ text: 'The cat**\'s bowl.', word: 'cat', },),
              followedByApostropheIn({ text: 'The *cat*\'s bowl.', word: 'cat', },),
              followedByApostropheIn({ text: 'The _cat_\'s bowl.', word: 'cat', },),
              followedByApostropheIn({ text: 'The **cat**\'s bowl.', word: 'cat', },),
              followedByApostropheIn({ text: 'The ~~cat~~\'s bowl.', word: 'cat', },),
              followedByApostropheIn({ text: 'The ~cat~\'s bowl.', word: 'cat', },),
              followedByApostropheIn({ text: 'The ***cat***\'s bowl.', word: 'cat', },),
            ],).toEqual([false, false, false, true, true, true, true, true, true,],);
          },
        },),
        it({
          name: 'READS A STRAIGHT QUOTATION MARK AS CLOSING WHEN A CHARACTER THAT IS NEITHER WHITE SPACE NOR AN OPENING MARK '
            + 'STANDS BEFORE IT, so a quotation mark right after an opening bracket opens',
          fn: async () => {
            expect([
              followedByApostropheIn({ text: 'Say ("\'Tis") now.', word: 'Say (', },),
              followedByApostropheIn({ text: 'Say [“\'Tis”] now.', word: 'Say [', },),
              followedByApostropheIn({ text: 'Say "cat"\'s now.', word: 'cat', },),
            ],).toEqual([false, false, true,],);
          },
        },),
      ],
    },),
    describe({
      name: markPastRunOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'GIVES THE SINGLE MARK PAST A RUN OF MARKUP OR CLOSING CHARACTERS AFTER A SPAN, over emphasis, a straight or '
            + 'curly closing double mark, a bracket and a link with its destination, and none for a mark directly beside',
          fn: async () => {
            expect([
              afterOf({ text: 'Fill the *kittens*’ bowl.', word: 'kittens', },),
              afterOf({ text: 'Fill the **kittens**\' bowl.', word: 'kittens', },),
              afterOf({ text: 'Fill the "kittens"’ bowl.', word: 'kittens', },),
              afterOf({ text: 'Fill the “kittens”’ bowl.', word: 'kittens', },),
              afterOf({ text: 'Fill the (kittens)’ bowl.', word: 'kittens', },),
              afterOf({ text: 'Fill the [kittens](https://cats.example/k)’ bowl.', word: 'kittens', },),
              afterOf({ text: 'Fill the ~~kittens~~’ bowl.', word: 'kittens', },),
              afterOf({ text: 'Fill the kittens’ bowl.', word: 'kittens', },),
              afterOf({ text: 'Fill the *kittens* bowl’.', word: 'kittens', },),
              afterOf({ text: 'Fill the [kittens](a b)’ bowl.', word: 'kittens', },),
              afterOf({ text: 'Fill the `kittens`’ bowl.', word: 'kittens', },),
            ],).toEqual(['’', '\'', '’', '’', '’', '’', '’', '', '', '', '’',],);
          },
        },),
        it({
          name: 'GIVES THE SINGLE MARK PAST A RUN OF MARKUP OR OPENING CHARACTERS BEFORE A SPAN, and none for a mark directly beside',
          fn: async () => {
            expect([
              beforeOf({ text: 'He said \'*5 cats* now.', word: '5 cats', },),
              beforeOf({ text: 'He said ’(_cats_ now.', word: 'cats', },),
              beforeOf({ text: 'He said ‘"cats now.', word: 'cats', },),
              beforeOf({ text: 'He said ’cats now.', word: 'cats', },),
              beforeOf({ text: 'He said *cats now.', word: 'cats', },),
              beforeOf({ text: 'He said \'`cats` now.', word: 'cats', },),
            ],).toEqual(['\'', '’', '‘', '', '', '\'',],);
          },
        },),
        it({
          name: 'STEPS OVER EVERY `*`, `_` AND `~` WHETHER OR NOT THE PARSE MAKES IT A DELIMITER, on both sides of a span: the '
            + 'refusal run stays wide (ruling on the J1 shapes), so a literal emphasis character is still in the run and the mark '
            + 'beyond it is a mark past a run; how a mark reads is the reading run\'s, and that one asks the parse',
          fn: async () => {
            expect([
              pastOf({ text: 'He said \'*5 cats now.', word: '5 cats', side: 'before', },),
              pastOf({ text: 'He said \'_cats_s now.', word: 'cats', side: 'before', },),
              pastOf({ text: 'Fill the kittens*’ bowl and *', word: 'kittens', side: 'after', },),
              pastOf({ text: 'He said 猫_cats_’ now.', word: 'cats', side: 'after', },),
              pastOf({ text: 'Fill the kittens~’ bowl.', word: 'kittens', side: 'after', },),
              pastOf({ text: 'He said \'*5 cats* now.', word: '5 cats', side: 'before', },),
              pastOf({ text: 'Fill the *kittens*’ bowl.', word: 'kittens', side: 'after', },),
            ],).toEqual(['\'', '\'', '’', '’', '’', '\'', '’',],);
          },
        },),
        it({
          name: 'STEPS OVER EVERYTHING THE PROSE VIEW DOES NOT READ AS WORDS AFTER A SPAN (a footnote reference, a tag, a reference '
            + 'link, a link\'s destination with its title, a line break element), an HTML comment, a character reference and '
            + 'horizontal white space, so the kept mark beyond them is a mark past a run',
          fn: async () => {
            expect([
              afterOf({ text: 'The kittens[^1]’ bowl.\n\n[^1]: A note.', word: 'kittens', },),
              afterOf({ text: 'The *kittens*[^1]’ bowl.\n\n[^1]: A note.', word: 'kittens', },),
              afterOf({ text: 'The kittens[^1]\' bowl.\n\n[^1]: A note.', word: 'kittens', },),
              afterOf({ text: 'The <b>kittens</b>’ bowl.', word: 'kittens', },),
              afterOf({ text: 'The <span class="x">kittens</span>’ bowl.', word: 'kittens', },),
              afterOf({ text: 'The [kittens][1]’ bowl.\n\n[1]: https://cats.example/k', word: 'kittens', },),
              afterOf({ text: 'The [kittens](https://cats.example/k "the cats")’ bowl.', word: 'kittens', },),
              afterOf({ text: 'The kittens<br/>’ bowl.', word: 'kittens', },),
              afterOf({ text: 'The kittens<!-- c -->’ bowl.', word: 'kittens', },),
              afterOf({ text: 'The kittens ’ bowl.', word: 'kittens', },),
              afterOf({ text: 'The kittens\u{00A0}’ bowl.', word: 'kittens', },),
              afterOf({ text: 'The kittens&nbsp;’ bowl.', word: 'kittens', },),
              afterOf({ text: 'The kittens&#160;’ bowl.', word: 'kittens', },),
            ],).toEqual(['’', '’', '\'', '’', '’', '’', '’', '’', '’', '’', '’', '’', '’',],);
          },
        },),
        it({
          name: 'STEPS OVER THE SAME CHARACTERS BEFORE A SPAN, a tag, a footnote reference, a comment, a character reference and '
            + 'white space, so the kept mark beyond them is a mark past a run',
          fn: async () => {
            expect([
              beforeOf({ text: 'He said ’<b>kittens</b> now.', word: 'kittens', },),
              beforeOf({ text: 'He said ’<span class="x">kittens</span> now.', word: 'kittens', },),
              beforeOf({ text: 'He said ’[^1]kittens now.\n\n[^1]: A note.', word: 'kittens', },),
              beforeOf({ text: 'He said ’<!-- c -->kittens now.', word: 'kittens', },),
              beforeOf({ text: 'He said ’ kittens now.', word: 'kittens', },),
              beforeOf({ text: 'He said ’&nbsp;kittens now.', word: 'kittens', },),
              beforeOf({ text: 'He said ’\u{00A0}kittens now.', word: 'kittens', },),
            ],).toEqual(['’', '’', '’', '’', '’', '’', '’',],);
          },
        },),
        it({
          name: 'STOPS THE RUN AT A LETTER, A COMMA, A BLANK LINE, A BACKSLASH BEFORE A LETTER AND THE NUL CHARACTER, which a page shows '
            + 'as the replacement character, and at the mark that ends it, so a mark beyond them is not a mark past a run',
          fn: async () => {
            expect([
              pastOf({ text: 'The kittens x’ bowl.', word: 'kittens', side: 'after', },),
              pastOf({ text: 'The kittens, ’ bowl.', word: 'kittens', side: 'after', },),
              pastOf({ text: 'The kittens\n\n’ bowl.', word: 'kittens', side: 'after', },),
              pastOf({ text: 'The kittens\u{0000}’ bowl.', word: 'kittens', side: 'after', },),
              pastOf({ text: 'The kittens[^1] bowl’', word: 'kittens', side: 'after', },),
              pastOf({ text: 'The kittens ’’ bowl.', word: 'kittens', side: 'after', },),
              pastOf({ text: String.raw`The kittens\x’ bowl.`, word: 'kittens', side: 'after', },),
              pastOf({ text: 'He said ’ x kittens now.', word: 'kittens', side: 'before', },),
              pastOf({ text: 'He said ’, kittens now.', word: 'kittens', side: 'before', },),
              pastOf({ text: 'He said ’\n\nkittens now.', word: 'kittens', side: 'before', },),
            ],).toEqual(['', '', '', '', '', '’', '', '', '', '',],);
          },
        },),
        it({
          name: 'READS A BACKSLASH ESCAPE AS THE ESCAPED CHARACTER, so an escaped emphasis character, bracket or backslash is '
            + 'stepped over like the character is, and an escaped single mark is the mark that ends the run',
          fn: async () => {
            expect([
              afterOf({ text: String.raw`The kittens\*’ bowl.`, word: 'kittens', },),
              afterOf({ text: String.raw`The kittens\_’ bowl.`, word: 'kittens', },),
              afterOf({ text: String.raw`The kittens\~’ bowl.`, word: 'kittens', },),
              afterOf({ text: String.raw`The kittens\]’ bowl.`, word: 'kittens', },),
              afterOf({ text: String.raw`The kittens\)’ bowl.`, word: 'kittens', },),
              afterOf({ text: String.raw`The kittens\\’ bowl.`, word: 'kittens', },),
              afterOf({ text: String.raw`The \*kittens\*’ bowl.`, word: 'kittens', },),
              afterOf({ text: String.raw`The kittens\' bowl.`, word: 'kittens', },),
              afterOf({ text: String.raw`The kittens\*\' bowl.`, word: 'kittens', },),
              afterOf({ text: String.raw`The kittens\.’ bowl.`, word: 'kittens', },),
              beforeOf({ text: String.raw`He said ’\*5 cats now.`, word: '5 cats', },),
              beforeOf({ text: String.raw`He said ’\(cats now.`, word: 'cats', },),
              beforeOf({ text: String.raw`He said ’\\*cats now.`, word: 'cats', },),
              beforeOf({ text: String.raw`He said \'*cats now.`, word: 'cats', },),
              beforeOf({ text: String.raw`He said ’\.cats now.`, word: 'cats', },),
            ],).toEqual(['’', '’', '’', '’', '’', '’', '’', '\'', '\'', '', '’', '’', '’', '\'', '',],);
          },
        },),
        it({
          name: 'STEPS OVER EVERY HORIZONTAL SPACE THE PROOF READS AS WHITE SPACE (JavaScript white space but the line terminators), '
            + 'on both sides of a span',
          fn: async () => {
            /**
             The mark the refusal run names past the character of each code point, after a word.
             */
            function afterEach(codes: readonly number[],): readonly string[] {
              return codes.map(function afterCode(code,): string {
                return afterOf({ text: `The kittens${String.fromCodePoint(code,)}’ bowl.`, word: 'kittens', },);
              },);
            }
            /**
             The mark the refusal run names past the character of each code point, before a word.
             */
            function beforeEach(codes: readonly number[],): readonly string[] {
              return codes.map(function beforeCode(code,): string {
                return beforeOf({ text: `He said ’${String.fromCodePoint(code,)}kittens now.`, word: 'kittens', },);
              },);
            }
            expect([
              afterEach(HORIZONTAL_SPACE_CODES,),
              beforeEach(HORIZONTAL_SPACE_CODES,),
            ],).toEqual([
              HORIZONTAL_SPACE_CODES.map(function mark(): string {
                return '’';
              },),
              HORIZONTAL_SPACE_CODES.map(function mark(): string {
                return '’';
              },),
            ],);
          },
        },),
        it({
          name: 'STEPS OVER EVERY DEFAULT-IGNORABLE CODE POINT (zero-width and format characters, the soft hyphen, the combining '
            + 'grapheme joiner, the Mongolian vowel separator, variation selectors, tag characters), alone or among spaces, on '
            + 'both sides of a span, a pair of UTF-16 units for one beyond the first plane being one character',
          fn: async () => {
            /**
             What each list of marks is when every character is stepped over.
             */
            const every = IGNORABLE_CODES.map(function mark(): string {
              return '’';
            },);
            expect({
              afterAlone: marksAround({ codes: IGNORABLE_CODES, head: 'The kittens', tail: '’ bowl.', side: 'after', },),
              afterSpaced: marksAround({ codes: IGNORABLE_CODES, head: 'The kittens ', tail: ' ’ bowl.', side: 'after', },),
              beforeAlone: marksAround({ codes: IGNORABLE_CODES, head: 'He said ’', tail: 'kittens now.', side: 'before', },),
              beforeSpaced: marksAround({
                codes: IGNORABLE_CODES,
                head: 'He said ’ ',
                tail: ' kittens now.',
                side: 'before',
              },),
            },).toEqual({
              afterAlone: every,
              afterSpaced: every,
              beforeAlone: every,
              beforeSpaced: every,
            },);
          },
        },),
        it({
          name: 'ENDS AT THE NUL CHARACTER, which a page shows as the replacement character, on both sides of a span',
          fn: async () => {
            expect([
              marksAround({ codes: NOT_IGNORABLE_CODES, head: 'The kittens', tail: '’ bowl.', side: 'after', },),
              marksAround({ codes: NOT_IGNORABLE_CODES, head: 'He said ’', tail: 'kittens now.', side: 'before', },),
            ],).toEqual([[''], [''],],);
          },
        },),
        it({
          name: 'STEPS OVER A CONTROL THAT SHOWS NOTHING (U+0001 to U+0008, U+000E to U+001F, U+007F) AND THE BRAILLE BLANK U+2800, '
            + 'alone or among spaces, on both sides of a span',
          fn: async () => {
            /**
             What each list of marks is when every character is stepped over.
             */
            const every = BLANK_CODES.map(function mark(): string {
              return '’';
            },);
            expect({
              afterAlone: marksAround({ codes: BLANK_CODES, head: 'The kittens', tail: '’ bowl.', side: 'after', },),
              afterSpaced: marksAround({ codes: BLANK_CODES, head: 'The kittens ', tail: ' ’ bowl.', side: 'after', },),
              beforeAlone: marksAround({ codes: BLANK_CODES, head: 'He said ’', tail: 'kittens now.', side: 'before', },),
              beforeSpaced: marksAround({ codes: BLANK_CODES, head: 'He said ’ ', tail: ' kittens now.', side: 'before', },),
            },).toEqual({
              afterAlone: every,
              afterSpaced: every,
              beforeAlone: every,
              beforeSpaced: every,
            },);
          },
        },),
        it({
          name: 'STEPS OVER EVERY C1 CONTROL BUT THE NEXT-LINE CONTROL (U+0080 to U+0084, U+0086 to U+009F), alone or among spaces, on '
            + 'both sides of a span (n156, W4)',
          fn: async () => {
            /**
             What each list of marks is when every character is stepped over.
             */
            const every = C1_CODES.map(function mark(): string {
              return '’';
            },);
            expect({
              afterAlone: marksAround({ codes: C1_CODES, head: 'The kittens', tail: '’ bowl.', side: 'after', },),
              afterSpaced: marksAround({ codes: C1_CODES, head: 'The kittens ', tail: ' ’ bowl.', side: 'after', },),
              beforeAlone: marksAround({ codes: C1_CODES, head: 'He said ’', tail: 'kittens now.', side: 'before', },),
              beforeSpaced: marksAround({ codes: C1_CODES, head: 'He said ’ ', tail: ' kittens now.', side: 'before', },),
            },).toEqual({
              afterAlone: every,
              afterSpaced: every,
              beforeAlone: every,
              beforeSpaced: every,
            },);
          },
        },),
        it({
          name: 'STEPS OVER THE NEXT-LINE CONTROL AND THE LINE AND PARAGRAPH SEPARATORS AS A SINGLE LINE BREAK, with or without a '
            + 'space beside it, on both sides of a span, and steps over two of them or one beside a line feed as well, since the page '
            + 'grammar keeps one paragraph there',
          fn: async () => {
            expect([
              marksAround({ codes: SEPARATOR_CODES, head: 'The kittens', tail: '’ bowl.', side: 'after', },),
              marksAround({ codes: SEPARATOR_CODES, head: 'The kittens ', tail: ' ’ bowl.', side: 'after', },),
              marksAround({ codes: SEPARATOR_CODES, head: 'He said ’', tail: 'kittens now.', side: 'before', },),
              SEPARATOR_CODES.map(function afterBlank(code,): string {
                return afterOf({
                  text: `The kittens${String.fromCodePoint(code,)}${String.fromCodePoint(code,)}’ bowl.`,
                  word: 'kittens',
                },);
              },),
              SEPARATOR_CODES.map(function afterBesideFeed(code,): string {
                return afterOf({ text: `The kittens\n${String.fromCodePoint(code,)}’ bowl.`, word: 'kittens', },);
              },),
            ],).toEqual([
              ['’', '’', '’',],
              ['’', '’', '’',],
              ['’', '’', '’',],
              ['’', '’', '’',],
              ['’', '’', '’',],
            ],);
          },
        },),
        it({
          name: 'CONTINUES ACROSS A SINGLE LINE BREAK IN A PARAGRAPH, a soft break, two spaces and a break, a backslash and a break '
            + 'and a carriage return with a line feed, on both sides of a span, and ends at a blank line',
          fn: async () => {
            expect([
              afterOf({ text: 'The kittens\n’ bowl.', word: 'kittens', },),
              afterOf({ text: 'The kittens  \n’ bowl.', word: 'kittens', },),
              afterOf({ text: 'The kittens\\\n’ bowl.', word: 'kittens', },),
              afterOf({ text: 'The kittens\n ’ bowl.', word: 'kittens', },),
              afterOf({ text: 'The kittens\r\n’ bowl.', word: 'kittens', },),
              afterOf({ text: 'The kittens\r’ bowl.', word: 'kittens', },),
              afterOf({ text: 'The *kittens*\n’ bowl.', word: 'kittens', },),
              afterOf({ text: 'The kittens\n\n’ bowl.', word: 'kittens', },),
              afterOf({ text: 'The kittens\n \n’ bowl.', word: 'kittens', },),
              afterOf({ text: 'The kittens\r\n\r\n’ bowl.', word: 'kittens', },),
              afterOf({ text: 'The kittens\n- ’ bowl.', word: 'kittens', },),
              beforeOf({ text: 'He said ’\nkittens now.', word: 'kittens', },),
              beforeOf({ text: 'He said ’  \nkittens now.', word: 'kittens', },),
              beforeOf({ text: 'He said ’\r\nkittens now.', word: 'kittens', },),
              beforeOf({ text: 'He said ’\n\nkittens now.', word: 'kittens', },),
              beforeOf({ text: 'He said ’\n \nkittens now.', word: 'kittens', },),
            ],).toEqual(['’', '’', '’', '’', '’', '’', '’', '', '', '', '', '’', '’', '’', '', '',],);
          },
        },),
        it({
          name: 'CONTINUES ACROSS A LINE OF ONLY ONE INVISIBLE OR NON-ASCII SPACE CHARACTER (n156, W9), which CommonMark keeps inside the '
            + 'paragraph, on both sides of a span, and still ends at a line of a space or a tab',
          fn: async () => {
            /**
             The characters, built from code points so no literal separator stands in this file.
             */
            const lines = [0x30_00, 0x20_0B, 0xFE_FF, 0xA0, 0x20_60, 0x18_0E, 0x20_2F, 0x20_07, 0x20_0C,].map(
              function lineOf(code,): string {
                return String.fromCodePoint(code,);
              },
            );
            expect({
              after: lines.map(function afterLine(line,): string {
                return afterOf({ text: `The kittens\n${line}\n’ bowl.`, word: 'kittens', },);
              },),
              before: lines.map(function beforeLine(line,): string {
                return beforeOf({ text: `He said ’\n${line}\nkittens now.`, word: 'kittens', },);
              },),
              spaceAndTab: [
                afterOf({ text: 'The kittens\n \n’ bowl.', word: 'kittens', },),
                afterOf({ text: 'The kittens\n\t\n’ bowl.', word: 'kittens', },),
              ],
            },).toEqual({
              after: lines.map(function mark(): string {
                return '’';
              },),
              before: lines.map(function mark(): string {
                return '’';
              },),
              spaceAndTab: ['', '',],
            },);
          },
        },),
        it({
          name: 'ENDS AT A LINE BREAK THAT LEAVES THE PARAGRAPH, a list item, a heading or a quotation that starts on the next line, '
            + 'whose marker the prose does not read as words, and continues across one inside a list item and across every break of '
            + 'an English the grammar refuses, a blank line included, since it has no paragraphs to tell',
          fn: async () => {
            expect([
              afterOf({ text: 'The kittens\n- ’ bowl.', word: 'kittens', },),
              afterOf({ text: 'The kittens\n1. ’ bowl.', word: 'kittens', },),
              afterOf({ text: 'The kittens\n# ’ bowl.', word: 'kittens', },),
              afterOf({ text: 'The kittens\n> ’ bowl.', word: 'kittens', },),
              afterOf({ text: 'The kittens\n\n- ’ bowl.', word: 'kittens', },),
              afterOf({ text: '- The kittens\n  ’ bowl.', word: 'kittens', },),
              afterOf({ text: 'The kittens\n’ bowl {', word: 'kittens', },),
              afterOf({ text: 'The kittens\n\n’ bowl {', word: 'kittens', },),
              beforeOf({ text: 'He said ’\n- kittens now.', word: 'kittens', },),
            ],).toEqual(['', '', '', '', '', '’', '’', '’', '',],);
          },
        },),
        it({
          name: 'STEPS OVER THE QUOTATION MARKERS AT THE START OF THE NEXT LINE OF A PARAGRAPH, the `>` with its space or without, '
            + 'one for each quotation the paragraph is in, in a list item as around one, and over the indentation of a '
            + 'list item and of a lazy line, on both sides of a span',
          fn: async () => {
            expect([
              afterOf({ text: '> The kittens\n> ’ bowl.', word: 'kittens', },),
              afterOf({ text: '> The kittens\n>’ bowl.', word: 'kittens', },),
              afterOf({ text: '> The kittens\r\n> ’ bowl.', word: 'kittens', },),
              afterOf({ text: '> > The kittens\n> > ’ bowl.', word: 'kittens', },),
              afterOf({ text: '>> The kittens\n>> ’ bowl.', word: 'kittens', },),
              afterOf({ text: '> > The kittens\n> ’ bowl.', word: 'kittens', },),
              afterOf({ text: '> - The kittens\n>   ’ bowl.', word: 'kittens', },),
              afterOf({ text: '- > The kittens\n  > ’ bowl.', word: 'kittens', },),
              afterOf({ text: '- The kittens\n  ’ bowl.', word: 'kittens', },),
              afterOf({ text: '> The kittens\n’ bowl.', word: 'kittens', },),
              afterOf({ text: '> The kittens\n   ’ bowl.', word: 'kittens', },),
              afterOf({ text: '> The *kittens*\n> ’ bowl.', word: 'kittens', },),
              beforeOf({ text: '> He said ’\n> kittens now.', word: 'kittens', },),
              beforeOf({ text: '> He said ’\n>kittens now.', word: 'kittens', },),
              beforeOf({ text: '> > He said ’\n> > kittens now.', word: 'kittens', },),
              beforeOf({ text: '>> He said ’\n>> kittens now.', word: 'kittens', },),
              beforeOf({ text: '> He said ’\r\n> kittens now.', word: 'kittens', },),
              beforeOf({ text: '> - He said ’\n>   kittens now.', word: 'kittens', },),
            ],).toEqual([
              '’', '’', '’', '’', '’', '’', '’', '’', '’', '’', '’', '’', '’', '’', '’', '’', '’', '’',
            ],);
          },
        },),
        it({
          name: 'ENDS AT A `>` THAT IS NO QUOTATION MARKER OF THE PARAGRAPH, in the middle of a line, at the start of a line outside '
            + 'every quotation, after a blank `>` line, past more markers than the paragraph has quotations, and escaped',
          fn: async () => {
            expect([
              afterOf({ text: 'The kittens > ’ bowl.', word: 'kittens', },),
              afterOf({ text: '> The kittens > ’ bowl.', word: 'kittens', },),
              afterOf({ text: 'The kittens\n> ’ bowl.', word: 'kittens', },),
              afterOf({ text: 'The kittens\n    > ’ bowl.', word: 'kittens', },),
              afterOf({ text: '> The kittens\n>\n> ’ bowl.', word: 'kittens', },),
              afterOf({ text: '> The kittens\n> > ’ bowl.', word: 'kittens', },),
              afterOf({ text: 'The kittens\n\\> ’ bowl.', word: 'kittens', },),
              afterOf({ text: '> The kittens\n\n> ’ bowl.', word: 'kittens', },),
              beforeOf({ text: 'He said ’ > kittens now.', word: 'kittens', },),
              beforeOf({ text: 'He said ’\n> kittens now.', word: 'kittens', },),
              beforeOf({ text: '> He said ’\n>\n> kittens now.', word: 'kittens', },),
              beforeOf({ text: '> He said ’\n> > kittens now.', word: 'kittens', },),
            ],).toEqual(['', '', '', '', '', '', '', '', '', '', '', '',],);
          },
        },),
        it({
          name: 'CONTINUES ACROSS A SINGLE LINE BREAK INSIDE THE TEXT OF A SETEXT HEADING, which spans lines as a paragraph does, '
            + 'and ends at the break before the underline and at the break after a heading of one line',
          fn: async () => {
            expect([
              afterOf({ text: 'The kittens\n’ x\n===', word: 'kittens', },),
              afterOf({ text: 'The kittens\n’ x\n---', word: 'kittens', },),
              afterOf({ text: 'The kittens  \n’ x\n===', word: 'kittens', },),
              afterOf({ text: 'The kittens\\\n’ x\n===', word: 'kittens', },),
              afterOf({ text: '> The kittens\n> ’ x\n> ===', word: 'kittens', },),
              beforeOf({ text: 'He ate ’\nthe kitten now\n===', word: 'the kitten', },),
              beforeOf({ text: 'He ate ’\nthe kitten now\n---', word: 'the kitten', },),
              afterOf({ text: '# The kittens\n’ x', word: 'kittens', },),
              afterOf({ text: 'The kittens\n# ’ x', word: 'kittens', },),
              afterOf({ text: 'The kittens\n===\n’ x', word: 'kittens', },),
            ],).toEqual(['’', '’', '’', '’', '’', '’', '’', '', '', '',],);
          },
        },),
      ],
    },),
  ],
},);
