/**
 Tests the Latin letter tests the package's prose scanners share (audit area
 six): each block at its edges, the characters just outside them, the two
 Latin-1 signs, and the Greek and Cyrillic letters a kaomoji carries; and
 the word scan built on them, over accented words opening, inside and
 decomposed. The package reads a word's neighbour with `continuesLatinWord`,
 which reads a character as written, and the consistency pass alone reads it
 with its own neighbour test, which also admits a whole character whose NFKC form
 is Latin letters, accents or ASCII digits.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  continuesLatinWord,
  continuesLatinWordOrCompatibilityForm,
  foldLatinWord,
  foldedLatinWords,
  isCombiningMark,
  isLatinCapital,
  isLatinLetter,
  isLatinLetterOrMark,
  isLatinWordCharacter,
  isSpacedScriptWordCharacter,
  latinWordSpans,
  lowerCaseLatinWords,
} from '../dist/final/node/index.mjs';

/**
 Characters each test is read against, named by code point where the glyph
 alone would not say which one it is.
 */
const PROBES: readonly string[] = [
  'a',
  'Z',
  '7',
  '_',
  // U+00BF INVERTED QUESTION MARK, just before the Latin-1 letters.
  '\u{00BF}',
  // U+00C0 LATIN CAPITAL LETTER A WITH GRAVE, the first Latin-1 letter.
  '\u{00C0}',
  // U+00D7 MULTIPLICATION SIGN, inside the Latin-1 letters.
  '\u{00D7}',
  // U+00F7 DIVISION SIGN, inside the Latin-1 letters.
  '\u{00F7}',
  'é',
  'ǎ',
  // U+024F LATIN SMALL LETTER Y WITH STROKE, the last of Latin Extended-B.
  '\u{024F}',
  // U+0250 LATIN SMALL LETTER TURNED A, the first IPA extension.
  '\u{0250}',
  // U+0300 COMBINING GRAVE ACCENT, the first combining mark.
  '\u{0300}',
  // U+036F COMBINING LATIN SMALL LETTER X, the last combining mark.
  '\u{036F}',
  // U+0370 GREEK CAPITAL LETTER HETA, just past the combining marks.
  '\u{0370}',
  'ω',
  'д',
  // U+1DFF COMBINING RIGHT ARROWHEAD AND DOWN ARROWHEAD BELOW, just before Latin Extended Additional.
  '\u{1DFF}',
  // U+1E00 LATIN CAPITAL LETTER A WITH RING BELOW, the first of Latin Extended Additional.
  '\u{1E00}',
  // U+1EFF LATIN SMALL LETTER Y WITH LOOP, the last of Latin Extended Additional.
  '\u{1EFF}',
  // U+1F00 GREEK SMALL LETTER ALPHA WITH PSILI, just past it.
  '\u{1F00}',
  '，',
  '猫',
  '',
];

/**
 Whole characters that are none of a Latin word's characters as written but whose NFKC form is made of Latin
 letters, accents or ASCII digits alone, each beside its code point.
 */
const COMPATIBILITY_CHARACTERS: readonly string[] = [
  // U+1D4E0 MATHEMATICAL BOLD SCRIPT CAPITAL Q, NFKC `Q`.
  '\u{1D4E0}',
  // U+1D4FE MATHEMATICAL BOLD SCRIPT SMALL U, NFKC `u`.
  '\u{1D4FE}',
  // U+1D400 MATHEMATICAL BOLD CAPITAL A, NFKC `A`.
  '\u{1D400}',
  // U+1D7CF MATHEMATICAL BOLD DIGIT ONE, NFKC `1`.
  '\u{1D7CF}',
  // U+FF21 FULLWIDTH LATIN CAPITAL LETTER A, NFKC `A`.
  '\u{FF21}',
  // U+FF53 FULLWIDTH LATIN SMALL LETTER S, NFKC `s`.
  '\u{FF53}',
  // U+FF11 FULLWIDTH DIGIT ONE, NFKC `1`.
  '\u{FF11}',
  // U+00AA FEMININE ORDINAL INDICATOR, a letter (category Lo), NFKC `a`.
  '\u{00AA}',
  // U+FB01 LATIN SMALL LIGATURE FI, a letter (category Ll), NFKC `fi`: every character of the form is a Latin letter.
  '\u{FB01}',
  // U+212A KELVIN SIGN, a letter (category Lu), NFKC `K`.
  '\u{212A}',
];

/**
 Whole characters whose NFKC form is made of Latin letters or ASCII digits alone but that are a sign and no letter or
 digit (a symbol or a number that is not a decimal digit), each beside its code point: the neighbour test refuses them,
 so a mark beside one is read as beside none.
 */
const SIGN_CHARACTERS: readonly string[] = [
  // U+2460 CIRCLED DIGIT ONE, category No, NFKC `1`.
  '\u{2460}',
  // U+00B2 SUPERSCRIPT TWO, category No, NFKC `2`.
  '\u{00B2}',
  // U+216B ROMAN NUMERAL TWELVE, category Nl, NFKC `XII`.
  '\u{216B}',
  // U+2122 TRADE MARK SIGN, category So, NFKC `TM`.
  '\u{2122}',
];

/**
 Characters no neighbour test of a word admits: a form that is not Latin, one with a sign in it, a lone surrogate,
 two characters and nothing.
 */
const UNLIKE_CHARACTERS: readonly string[] = [
  // U+00B5 MICRO SIGN, NFKC U+03BC GREEK SMALL LETTER MU.
  '\u{00B5}',
  // U+1D6C2 MATHEMATICAL BOLD SMALL ALPHA, NFKC U+03B1 GREEK SMALL LETTER ALPHA.
  '\u{1D6C2}',
  // U+00BD VULGAR FRACTION ONE HALF, NFKC `1`, U+2044 FRACTION SLASH and `2`: one character of the form is a sign.
  '\u{00BD}',
  // U+2103 DEGREE CELSIUS, NFKC U+00B0 DEGREE SIGN and `C`.
  '\u{2103}',
  // U+FF07 FULLWIDTH APOSTROPHE, NFKC `'`.
  '\u{FF07}',
  // U+FF0C FULLWIDTH COMMA, NFKC `,`.
  '\u{FF0C}',
  // U+D835 a lone high surrogate, the first unit of a mathematical letter read by UTF-16 unit.
  '\u{D835}',
  // U+DCE0 a lone low surrogate.
  '\u{DCE0}',
  // Two fullwidth capital letters, which are two characters and not one.
  '\u{FF21}\u{FF22}',
  '猫',
  'ω',
  '_',
  '',
];

/**
 Probes one test admits, in probe order.

 @param admits - test under read

 @returns Probes the test admits

 @example
 ```ts
 admitted(isLatinLetter);
 ```
 */
function admitted(
  admits: (argument: { readonly character: string; },) => boolean,
): readonly string[] {
  return PROBES.filter(function admittedProbe(character,): boolean {
    return admits({ character, },);
  },);
}

/**
 What a test says of each character, in order.

 @param admits - test under read
 @param characters - characters put to it

 @returns One answer for each character

 @example
 ```ts
 answersOf(continuesLatinWord, ['a', '猫',]); // [true, false]
 ```
 */
function answersOf(
  admits: (argument: { readonly character: string; },) => boolean,
  characters: readonly string[],
): readonly boolean[] {
  return characters.map(function answer(character,): boolean {
    return admits({ character, },);
  },);
}

/**
 The same answer for each of a list of characters.

 @param answer - answer
 @param characters - characters

 @returns One answer for each character

 @example
 ```ts
 allAnswered(true, ['a', 'b',]); // [true, true]
 ```
 */
function allAnswered(answer: boolean, characters: readonly string[],): readonly boolean[] {
  return characters.map(function same(): boolean {
    return answer;
  },);
}

await describe({
  name: 'latin letters',
  children: [
    it({
      name: 'ADMITS ASCII, Latin-1, Extended-A, -B and Additional letters, and not the signs, digits or other scripts',
      fn: async () => {
        expect(admitted(isLatinLetter,),).toEqual([
          'a',
          'Z',
          '\u{00C0}',
          'é',
          'ǎ',
          '\u{024F}',
          '\u{1E00}',
          '\u{1EFF}',
        ],);
      },
    },),
    it({
      name: 'ADMITS ASCII digits besides the letters as word characters, and nothing else',
      fn: async () => {
        expect(admitted(isLatinWordCharacter,),).toEqual([
          'a',
          'Z',
          '7',
          '\u{00C0}',
          'é',
          'ǎ',
          '\u{024F}',
          '\u{1E00}',
          '\u{1EFF}',
        ],);
      },
    },),
    it({
      name: 'ADMITS the combining diacritical marks block at both edges and nothing beside it',
      fn: async () => {
        expect(admitted(isCombiningMark,),).toEqual(['\u{0300}', '\u{036F}',],);
      },
    },),
    it({
      name: 'CONTINUES a letter run with a letter or a mark, and a word also with a digit',
      fn: async () => {
        expect(admitted(isLatinLetterOrMark,),).toEqual([
          'a',
          'Z',
          '\u{00C0}',
          'é',
          'ǎ',
          '\u{024F}',
          '\u{0300}',
          '\u{036F}',
          '\u{1E00}',
          '\u{1EFF}',
        ],);
        expect(admitted(continuesLatinWord,),).toEqual([
          'a',
          'Z',
          '7',
          '\u{00C0}',
          'é',
          'ǎ',
          '\u{024F}',
          '\u{0300}',
          '\u{036F}',
          '\u{1E00}',
          '\u{1EFF}',
        ],);
      },
    },),
    it({
      name: 'REFUSES, IN THE WORD TEST OF THE PACKAGE, EVERY COMPATIBILITY CHARACTER: a mathematical script letter, a fullwidth '
        + 'letter, a ligature, a circled digit, a trade mark sign and a superscript digit are no letter of a word as written',
      fn: async () => {
        expect({
          mathematicalScriptQ: continuesLatinWord({ character: '\u{1D4E0}', },),
          tradeMark: continuesLatinWord({ character: '\u{2122}', },),
          superscriptTwo: continuesLatinWord({ character: '\u{00B2}', },),
          ligatureFi: continuesLatinWord({ character: '\u{FB01}', },),
        },).toEqual({
          mathematicalScriptQ: false,
          tradeMark: false,
          superscriptTwo: false,
          ligatureFi: false,
        },);
        expect(answersOf(
          continuesLatinWord,
          COMPATIBILITY_CHARACTERS,
        ),).toEqual(allAnswered(
          false,
          COMPATIBILITY_CHARACTERS,
        ),);
        expect(answersOf(
          continuesLatinWord,
          UNLIKE_CHARACTERS,
        ),).toEqual(allAnswered(
          false,
          UNLIKE_CHARACTERS,
        ),);
        expect(answersOf(
          continuesLatinWord,
          SIGN_CHARACTERS,
        ),).toEqual(allAnswered(
          false,
          SIGN_CHARACTERS,
        ),);
      },
    },),
    it({
      name: 'ADMITS, IN THE NEIGHBOUR TEST OF THE CONSISTENCY PASS, A WHOLE LETTER OR DECIMAL DIGIT WHOSE NFKC FORM IS LATIN LETTERS, '
        + 'ACCENTS OR ASCII DIGITS ONLY, mathematical alphanumerics, fullwidth Latin and ligatures among them, and refuses a form '
        + 'with a sign or another script in it, and a symbol or number that is no letter or digit (a trade mark sign, a '
        + 'superscript or circled digit, a Roman numeral)',
      fn: async () => {
        expect({
          mathematicalScriptQ: continuesLatinWordOrCompatibilityForm({ character: '\u{1D4E0}', },),
          tradeMark: continuesLatinWordOrCompatibilityForm({ character: '\u{2122}', },),
          superscriptTwo: continuesLatinWordOrCompatibilityForm({ character: '\u{00B2}', },),
          ligatureFi: continuesLatinWordOrCompatibilityForm({ character: '\u{FB01}', },),
        },).toEqual({
          mathematicalScriptQ: true,
          tradeMark: false,
          superscriptTwo: false,
          ligatureFi: true,
        },);
        expect(answersOf(
          continuesLatinWordOrCompatibilityForm,
          SIGN_CHARACTERS,
        ),).toEqual(allAnswered(
          false,
          SIGN_CHARACTERS,
        ),);
        expect(answersOf(
          continuesLatinWordOrCompatibilityForm,
          COMPATIBILITY_CHARACTERS,
        ),).toEqual(allAnswered(
          true,
          COMPATIBILITY_CHARACTERS,
        ),);
        expect(answersOf(
          continuesLatinWordOrCompatibilityForm,
          UNLIKE_CHARACTERS,
        ),).toEqual(allAnswered(
          false,
          UNLIKE_CHARACTERS,
        ),);
      },
    },),
    it({
      name: 'ADMITS, IN THE NEIGHBOUR TEST OF THE CONSISTENCY PASS, EVERY CHARACTER THE WORD TEST OF THE PACKAGE ADMITS, so it only '
        + 'widens that test',
      fn: async () => {
        expect(admitted(continuesLatinWordOrCompatibilityForm,),).toEqual(admitted(continuesLatinWord,),);
      },
    },),
    it({
      name: 'KEEPS THE LETTER TEST AND THE WORD CHARACTER TEST TO LATIN LETTERS AND ASCII DIGITS ONLY, as the word test of the '
        + 'package is, since only the neighbour test of the consistency pass reads a compatibility character through its NFKC form',
      fn: async () => {
        expect([
          isLatinLetter({ character: '\u{FF21}', },),
          isLatinWordCharacter({ character: '\u{FF11}', },),
          isLatinLetterOrMark({ character: '\u{FB01}', },),
          isLatinCapital({ character: '\u{1D400}', },),
        ],).toEqual([false, false, false, false,],);
      },
    },),
    it({
      name: 'ADMITS, IN THE SHOWN-WORD TEST (n159, X9), A LETTER, MARK OR DECIMAL DIGIT OF ANY SCRIPT THAT SEPARATES ITS WORDS WITH SPACES, '
        + 'and not the signs, the unspaced scripts Han, Hiragana and Katakana, the modifier letter apostrophe, or an empty string',
      fn: async () => {
        expect([
          'a',
          'П',
          'ω',
          '5',
          '\u{0663}',
          '\u{0301}',
          '猫',
          'あ',
          'カ',
          '\u{02BC}',
          '_',
          '-',
          '',
        ].map(function admits(character,): boolean {
          return isSpacedScriptWordCharacter({ character, },);
        },),).toEqual([true, true, true, true, true, true, false, false, false, false, false, false, false,],);
      },
    },),
    it({
      name: 'SCANS words that open on, carry or end in an accent, composed or combining, as one word each, where they start',
      fn: async () => {
        // An ASCII scan read `Heřmánek` as `He`, `m` and `nek`, and `Åhe` as
        // `he` from its second letter.
        expect(latinWordSpans({ text: 'Heřmánek met Åhe at the Cafe\u{0301}.', },),).toEqual([
          { word: 'Heřmánek', start: 0, },
          { word: 'met', start: 9, },
          { word: 'Åhe', start: 13, },
          { word: 'at', start: 17, },
          { word: 'the', start: 20, },
          { word: 'Cafe\u{0301}', start: 24, },
        ],);
      },
    },),
    it({
      name: 'SPLITS at digits, apostrophes, spaces, other scripts, and a combining mark with no letter before it',
      fn: async () => {
        expect(lowerCaseLatinWords({ text: 'Cat9lives don\'t 猫猫 ω\u{0301}x (・ω・) Z', },),).toEqual([
          'cat',
          'lives',
          'don',
          't',
          'x',
          'z',
        ],);
      },
    },),
    it({
      name: 'ADMITS as capitals the upper-case Latin letters among the probes, accented or not',
      fn: async () => {
        expect(admitted(isLatinCapital,),).toEqual([
          'Z',
          '\u{00C0}',
          '\u{1E00}',
        ],);
      },
    },),
    it({
      name: 'FOLDS a word written with its accent, a separate accent or none to one lower-case form',
      fn: async () => {
        expect([
          'Kätzchen',
          'Ka\u{0308}tzchen',
          'KATZCHEN',
          'Château',
        ].map(function folded(word,): string {
          return foldLatinWord({ word, },);
        },),).toEqual([
          'katzchen',
          'katzchen',
          'katzchen',
          'chateau',
        ],);
        expect(foldedLatinWords({ text: 'Château, Cha\u{0302}teau and Éa9', },),).toEqual([
          'chateau',
          'chateau',
          'and',
          'ea',
        ],);
      },
    },),
  ],
},);
