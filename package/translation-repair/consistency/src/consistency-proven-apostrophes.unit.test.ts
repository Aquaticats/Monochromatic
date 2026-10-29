/**
 Tests for the single marks of an English that are proven apostrophes: a mark
 at the end of a word that no earlier quotation-like single mark anywhere in the
 text could have opened a quotation for, a mark at the start of a word that no
 later one anywhere in the text could close a quotation for, and never the
 opening curly single mark. Also that the reading visits a bounded number of
 characters for each character of the English, counted without a clock.

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
  markReadingsOf,
  provenApostrophesOf,
} from '../dist/final/node/index.mjs';

/**
 The offsets of the marks a caller names as trailing past closing markup.
 */
type PastMarkupOffsets = {
  readonly trailing?: readonly number[];
};

/**
 The marks a caller names as standing past markup, as the reading takes them.

 @param past - offsets, none by default

 @returns Set of offsets

 @example
 ```ts
 const named = pastMarkupOf({ past: { trailing: [13,], }, },); // { trailing: Set { 13 } }
 ```
 */
function pastMarkupOf(
  { past: { trailing = [], }, }: { readonly past: PastMarkupOffsets; },
): { readonly trailing: ReadonlySet<number>; } {
  return { trailing: new Set(trailing,), };
}

/**
 A text with each proven apostrophe shown between white square brackets, the
 trailing and the leading kind alike, so a case reads as the text it is about.

 @param text - English

 @param past - marks the caller names as standing past markup, none by default

 @returns English with the proven marks bracketed

 @example
 ```ts
 const shown = provenShown({ text: 'The parents’ bowl', },); // 'The parents⟦’⟧ bowl'
 ```
 */
function provenShown(
  { text, past = {}, }: { readonly text: string; readonly past?: PastMarkupOffsets; },
): string {
  const { trailing, leading, } = provenApostrophesOf({ text, pastMarkup: pastMarkupOf({ past, },), },);
  return Array.from(
    { length: text.length, },
    function unitShown(_, at,): string {
      return (trailing.has(at,) || leading.has(at,)) ? `⟦${text.charAt(at,)}⟧` : text.charAt(at,);
    },
  )
    .join('',);
}

/**
 How every apostrophe-like mark of a text reads, as `offset:kind` pairs in the order of the offsets.

 @param text - English

 @param past - marks the caller names as standing past markup, none by default

 @returns One pair for each mark

 @example
 ```ts
 const shown = readingsShown({ text: 'don’t', },); // ['3:attached']
 ```
 */
function readingsShown(
  { text, past = {}, }: { readonly text: string; readonly past?: PastMarkupOffsets; },
): readonly string[] {
  return [...markReadingsOf({ text, pastMarkup: pastMarkupOf({ past, },), },),].map(
    function shown([at, kind,],): string {
      return `${String(at,)}:${kind}`;
    },
  );
}

/**
 A text that counts the characters the reading takes from it, one for each call
 of `charAt` or `codePointAt`, so the work of a reading is measured without a
 clock. It stands in for the English, of which the reading asks only its
 length and its characters; any other method it asks for is absent and the case
 throws.

 @param text - English the double holds

 @returns Double, which the reading takes as the English, and a reader of
 the count of characters read from it so far

 @example
 ```ts
 const { english, readsSoFar, } = countedEnglish({ text: 'The parents’ bowl', },);
 ```
 */
function countedEnglish({ text, }: { readonly text: string; },): {
  readonly english: string;
  readonly readsSoFar: () => number;
} {
  /**
   Characters read so far, mutated only by the double's own methods.
   */
  const counted = { reads: 0, };
  const double = {
    length: text.length,
    charAt: (position: number,) => {
      counted.reads += 1;
      return text.charAt(position,);
    },
    codePointAt: (position: number,) => {
      counted.reads += 1;
      return text.codePointAt(position,);
    },
  };
  return {
    // The double has the reads the reading makes of a string, and no others.
    english: double as unknown as string,
    readsSoFar: () => counted.reads,
  };
}

/**
 How many characters the reading takes from an English of the given text.

 @param text - English

 @returns Characters read

 @example
 ```ts
 const reads = readsFor({ text: 'The parents’ bowl', },);
 ```
 */
function readsFor({ text, }: { readonly text: string; },): number {
  const { english, readsSoFar, } = countedEnglish({ text, },);
  provenApostrophesOf({ text: english, },);
  return readsSoFar();
}

/**
 The most characters the reading may take for each character of the English.
 */
const LINEAR_READS_PER_CHARACTER = 8;

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: provenApostrophesOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'PROVES A TRAILING MARK WHEN NO EARLIER MARK OF THE ENGLISH IS LEADING OR FREE: a plural possessive in '
            + 'each of the three apostrophe-like spellings, at the end of the text, and after a digit; a letter of another '
            + 'script before the mark proves nothing, since the neighbour test of the consistency pass names Latin '
            + 'letters, ASCII digits and their compatibility forms only',
          fn: async () => {
            expect([
              provenShown({ text: 'The parents’ preference.', },),
              provenShown({ text: 'The parents\' preference.', },),
              provenShown({ text: 'The parents\u{02BC} preference.', },),
              provenShown({ text: 'Fill the cats’', },),
              provenShown({ text: 'The 1990’ cats', },),
              provenShown({ text: 'Fill the Кошек’ bowl', },),
              // U+10428 DESERET SMALL LETTER LONG I, a letter beyond the first plane that no Latin form stands for.
              provenShown({ text: 'Fill the \u{10428}’ bowl', },),
            ],).toEqual([
              'The parents⟦’⟧ preference.',
              'The parents⟦\'⟧ preference.',
              'The parents⟦\u{02BC}⟧ preference.',
              'Fill the cats⟦’⟧',
              'The 1990⟦’⟧ cats',
              'Fill the Кошек’ bowl',
              'Fill the \u{10428}’ bowl',
            ],);
          },
        },),
        it({
          name: 'DOES NOT PROVE A TRAILING MARK WHEN AN EARLIER MARK OF THE ENGLISH IS LEADING: the closing mark of a '
            + 'quotation, whether the opener is straight or curly, and the opener is not proven either',
          fn: async () => {
            expect([
              provenShown({ text: 'She said \'Mio\' twice.', },),
              provenShown({ text: 'She said ‘Mio’ twice.', },),
              provenShown({ text: 'She said ’Mio’ twice.', },),
              provenShown({ text: 'She said ‘the cat’s bowl’ twice.', },),
            ],).toEqual([
              'She said \'Mio\' twice.',
              'She said ‘Mio’ twice.',
              'She said ’Mio’ twice.',
              'She said ‘the cat’s bowl’ twice.',
            ],);
          },
        },),
        it({
          name: 'DOES NOT PROVE A TRAILING MARK WHEN AN EARLIER MARK OF THE ENGLISH IS FREE: a mark with no letter or '
            + 'digit beside it could have opened a quotation',
          fn: async () => {
            expect([
              provenShown({ text: 'She said \' hello the parents’ bowl.', },),
              provenShown({ text: 'Why ’ the parents’ preference?', },),
              provenShown({ text: 'Why (’) the parents’ preference?', },),
            ],).toEqual([
              'She said \' hello the parents’ bowl.',
              'Why ’ the parents’ preference?',
              'Why (’) the parents’ preference?',
            ],);
          },
        },),
        it({
          name: 'LETS A MARK ATTACHED TO A WORD STAND NEITHER FOR NOR AGAINST: an earlier attached mark is no partner, and an '
            + 'attached mark is never proven itself',
          fn: async () => {
            expect([
              provenShown({ text: 'Don’t ask the parents’ preference.', },),
              provenShown({ text: 'The cat’s bowl and the parents’ bowl.', },),
              provenShown({ text: 'The parents’ bowl and the cat’s bowl.', },),
              provenShown({ text: 'Don\'t you\'ll.', },),
            ],).toEqual([
              'Don’t ask the parents⟦’⟧ preference.',
              'The cat’s bowl and the parents⟦’⟧ bowl.',
              'The parents⟦’⟧ bowl and the cat’s bowl.',
              'Don\'t you\'ll.',
            ],);
          },
        },),
        it({
          name: 'PROVES A LEADING MARK WHEN NO LATER MARK OF THE ENGLISH IS TRAILING OR FREE, and does not prove one that '
            + 'a later trailing or free mark could close',
          fn: async () => {
            expect([
              provenShown({ text: 'Guard her ’round the cats.', },),
              provenShown({ text: 'Guard her \'round the cats.', },),
              provenShown({ text: 'Guard her \u{02BC}round the cats.', },),
              provenShown({ text: '’Tis the season.', },),
              provenShown({ text: 'Guard her ’round the cats’ bowl.', },),
              provenShown({ text: 'Guard her ’round the cats ’ bowl.', },),
              provenShown({ text: 'Guard her ’round the cat’s bowl and Mio’s.', },),
            ],).toEqual([
              'Guard her ⟦’⟧round the cats.',
              'Guard her ⟦\'⟧round the cats.',
              'Guard her ⟦\u{02BC}⟧round the cats.',
              '⟦’⟧Tis the season.',
              'Guard her ’round the cats’ bowl.',
              'Guard her ’round the cats ’ bowl.',
              'Guard her ⟦’⟧round the cat’s bowl and Mio’s.',
            ],);
          },
        },),
        it({
          name: 'NEVER PROVES THE OPENING CURLY SINGLE MARK, leading, trailing or free, and counts it as a mark that could '
            + 'open or close a quotation wherever it stands, even glued between two letters',
          fn: async () => {
            expect([
              provenShown({ text: 'Guard her ‘round the cats.', },),
              provenShown({ text: 'Fill the parents‘ bowl.', },),
              provenShown({ text: 'Guard her ‘round the parents’ bowl.', },),
              provenShown({ text: 'Fill the parents’ bowl, then ‘round.', },),
              provenShown({ text: 'Fill the cats‘ bowl and the parents’ bowl.', },),
              provenShown({ text: 'She said‘fill the kittens’ bowl and left.', },),
              provenShown({ text: 'Guard her ’round the cats and say‘hi.', },),
            ],).toEqual([
              'Guard her ‘round the cats.',
              'Fill the parents‘ bowl.',
              'Guard her ‘round the parents’ bowl.',
              'Fill the parents⟦’⟧ bowl, then ‘round.',
              'Fill the cats‘ bowl and the parents’ bowl.',
              'She said‘fill the kittens’ bowl and left.',
              'Guard her ’round the cats and say‘hi.',
            ],);
          },
        },),
        it({
          name: 'COUNTS EVERY OTHER SINGLE QUOTATION CHARACTER AS A PARTNER, wherever it stands: the low and the reversed '
            + 'single quotation marks, both angle quotation marks and a backtick, each before a trailing mark and after a '
            + 'leading one',
          fn: async () => {
            expect([
              provenShown({ text: 'She said ‚fill the kittens’ bowl and left.', },),
              provenShown({ text: 'She said ‛fill the kittens’ bowl and left.', },),
              provenShown({ text: 'She said ‹fill the kittens’ bowl and left.', },),
              provenShown({ text: 'She said ›fill the kittens’ bowl and left.', },),
              provenShown({ text: 'She said `fill the kittens’ bowl and left.', },),
              provenShown({ text: 'Guard her ’round the cats, said ‚Mio‘.', },),
              provenShown({ text: 'Guard her ’round the cats, said ‹Mio›.', },),
              provenShown({ text: 'Guard her ’round the cats, said `Mio`.', },),
              provenShown({ text: 'Guard her ’round the cats, said ›Mio.', },),
            ],).toEqual([
              'She said ‚fill the kittens’ bowl and left.',
              'She said ‛fill the kittens’ bowl and left.',
              'She said ‹fill the kittens’ bowl and left.',
              'She said ›fill the kittens’ bowl and left.',
              'She said `fill the kittens’ bowl and left.',
              'Guard her ’round the cats, said ‚Mio‘.',
              'Guard her ’round the cats, said ‹Mio›.',
              'Guard her ’round the cats, said `Mio`.',
              'Guard her ’round the cats, said ›Mio.',
            ],);
          },
        },),
        it({
          name: 'READS THE WHOLE ENGLISH, NOT THE PARAGRAPH: a mark in another paragraph is a partner, whether a blank line, '
            + 'a line of spaces or tabs, a carriage return pair or a run of blank lines parts the two',
          fn: async () => {
            expect([
              provenShown({ text: '‘Mio is here.\n\nThe parents’ preference.', },),
              provenShown({ text: 'The parents’ preference.\n\n‘Mio is here.', },),
              provenShown({ text: '‘Mio is here.\nThe parents’ preference.', },),
              provenShown({ text: '‘Mio is here.\n \t \nThe parents’ preference.', },),
              provenShown({ text: '‘Mio is here.\r\n\r\nThe parents’ preference.', },),
              provenShown({ text: '‘Mio is here.\n\n\n\nThe parents’ preference.', },),
              provenShown({ text: 'Why ’ not?\n\nGuard her ’round the cats.', },),
              provenShown({ text: 'Guard her ’round the cats.\n\nThe parents’ ’ preference.', },),
              provenShown({ text: 'Then feed the kittens’.\n\n‘Fill the bowls.', },),
            ],).toEqual([
              '‘Mio is here.\n\nThe parents’ preference.',
              'The parents⟦’⟧ preference.\n\n‘Mio is here.',
              '‘Mio is here.\nThe parents’ preference.',
              '‘Mio is here.\n \t \nThe parents’ preference.',
              '‘Mio is here.\r\n\r\nThe parents’ preference.',
              '‘Mio is here.\n\n\n\nThe parents’ preference.',
              'Why ’ not?\n\nGuard her ⟦’⟧round the cats.',
              'Guard her ’round the cats.\n\nThe parents’ ’ preference.',
              'Then feed the kittens⟦’⟧.\n\n‘Fill the bowls.',
            ],);
          },
        },),
        it({
          name: 'KEEPS A QUOTATION THAT OPENS IN ONE PARAGRAPH AND CLOSES IN ANOTHER, straight or curly, inside a block quote or '
            + 'not: neither its opener nor its closer is proven, the reviewer\'s cases',
          fn: async () => {
            expect([
              provenShown({ text: '‘Fill the bowls.\n\nThen feed the kittens’.', },),
              provenShown({ text: 'He said, \'Fill the bowls for the kittens.\n\nThen feed them.\'', },),
              provenShown({ text: '> ‘Fill the bowls.\n>\n> Then feed the kittens’.', },),
              provenShown({ text: '\'Tis for the kittens\n\nand for the cats\' bowl.', },),
              provenShown({ text: '- ‘Fill the bowls.\n- Then feed the kittens’.', },),
            ],).toEqual([
              '‘Fill the bowls.\n\nThen feed the kittens’.',
              'He said, \'Fill the bowls for the kittens.\n\nThen feed them.\'',
              '> ‘Fill the bowls.\n>\n> Then feed the kittens’.',
              '\'Tis for the kittens\n\nand for the cats\' bowl.',
              '- ‘Fill the bowls.\n- Then feed the kittens’.',
            ],);
          },
        },),
        it({
          name: 'READS NO MORE THAN A SMALL MULTIPLE OF THE ENGLISH, however many marks it holds: a paragraph of 4,000 marks '
            + 'of every class reads each character a bounded number of times, and twice the English reads about twice as much',
          fn: async () => {
            /**
             Four marks of four classes (trailing, leading, free, opening), a thousand times, in one paragraph.
             */
            const paragraph = 'x’ ’y ’ ‘z '.repeat(1_000,);
            /**
             The same English twice as long.
             */
            const twice = `${paragraph}${paragraph}`;
            const once = readsFor({ text: paragraph, },);
            const doubled = readsFor({ text: twice, },);
            expect({
              marks: Array.from(paragraph,)
                .filter(function isMark(character,): boolean {
                  return (character === '’') || (character === '‘');
                },)
                .length,
              withinBound: once <= (LINEAR_READS_PER_CHARACTER * paragraph.length),
              growsLinearly: doubled <= (2.1 * once),
            },).toEqual({ marks: 4_000, withinBound: true, growsLinearly: true, },);
          },
        },),
      ],
    },),
    describe({
      name: `${provenApostrophesOf.name} with the neighbour test of the consistency pass`,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'READS A MARK BESIDE A LATIN LETTER, WITH ITS ACCENT, OR AN ASCII DIGIT AS BESIDE A WORD, and a mark beside any '
            + 'other script as beside none: a mark between two Han characters is attached to nothing',
          fn: async () => {
            expect([
              provenShown({ text: 'the cats’ bowl', },),
              provenShown({ text: 'the café’ bowl', },),
              provenShown({ text: 'the 猫’ bowl', },),
              provenShown({ text: 'the αβ’ bowl', },),
              provenShown({ text: 'the ’猫 bowl', },),
              provenShown({ text: 'the ’5 bowl', },),
            ],).toEqual([
              'the cats⟦’⟧ bowl',
              'the café⟦’⟧ bowl',
              'the 猫’ bowl',
              'the αβ’ bowl',
              'the ’猫 bowl',
              'the ⟦’⟧5 bowl',
            ],);
          },
        },),
        it({
          name: 'PROVES A MARK BESIDE A MATHEMATICAL OR FULLWIDTH LETTER, whose NFKC form is Latin, as beside a word, and not one '
            + 'beside a letter of another script or a fraction',
          fn: async () => {
            expect([
              // U+1D4E0 MATHEMATICAL BOLD SCRIPT CAPITAL Q, U+1D4FE, U+1D4EA and U+1D4F7 spell `Quan`.
              provenShown({ text: 'the \u{1D4E0}\u{1D4FE}\u{1D4EA}\u{1D4F7}’s bowl', },),
              // U+1D520 MATHEMATICAL FRAKTUR SMALL C before a trailing mark.
              provenShown({ text: 'Fill the \u{1D520}’ bowl', },),
              // U+FF21 FULLWIDTH LATIN CAPITAL LETTER A before a trailing mark.
              provenShown({ text: 'Fill the \u{FF21}’ bowl', },),
              // U+00BD VULGAR FRACTION ONE HALF, NFKC `1`, U+2044 and `2`.
              provenShown({ text: 'Fill the \u{00BD}’ bowl', },),
            ],).toEqual([
              'the \u{1D4E0}\u{1D4FE}\u{1D4EA}\u{1D4F7}’s bowl',
              'Fill the \u{1D520}⟦’⟧ bowl',
              'Fill the \u{FF21}⟦’⟧ bowl',
              'Fill the \u{00BD}’ bowl',
            ],);
          },
        },),
      ],
    },),
    describe({
      name: `${markReadingsOf.name} with compatibility letters`,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'READS A MARK BESIDE A LETTER OR DIGIT WHOSE NFKC FORM IS LATIN AS ATTACHED, read by whole character: mathematical '
            + 'script letters beyond the first plane and fullwidth Latin letters and digits',
          fn: async () => {
            expect([
              // Four mathematical bold script letters, two UTF-16 units each, then the mark at offset 8.
              readingsShown({ text: '\u{1D4E0}\u{1D4FE}\u{1D4EA}\u{1D4F7}’s', },),
              // Fullwidth `Ａ`, the mark, fullwidth `ｓ`.
              readingsShown({ text: '\u{FF21}’\u{FF53}', },),
              // A fullwidth digit one, a decimal digit whose NFKC form is the ASCII digit `1`, so it counts as a digit does.
              readingsShown({ text: '\u{FF11}’s', },),
              // A mathematical bold capital A before the mark and a fullwidth digit after it.
              readingsShown({ text: '\u{1D400}’\u{FF11}', },),
              // A ligature `ﬁ`, NFKC `fi`, before a mark that ends the word.
              readingsShown({ text: 'o\u{FB01}’ bowl', },),
            ],).toEqual([
              ['8:attached',],
              ['1:attached',],
              ['1:attached',],
              ['2:attached',],
              ['2:trailing',],
            ],);
          },
        },),
        it({
          name: 'READS A MARK BESIDE A CHARACTER WHOSE NFKC FORM IS NOT LATIN, OR THAT IS NO LETTER OR DIGIT, AS BESIDE NO LETTER, so '
            + 'Han, Greek, a micro sign (NFKC Greek mu), a fraction, a lone surrogate, a circled digit, a trade mark sign and a '
            + 'superscript digit leave the mark free, as a mark beside a letter of another script or a symbol is free',
          fn: async () => {
            expect([
              readingsShown({ text: '猫’猫', },),
              readingsShown({ text: 'αβ’γδ', },),
              // A micro sign before `s`: Greek mu in NFKC, so no Latin letter stands before the mark and it opens nothing.
              readingsShown({ text: '\u{00B5}’s', },),
              // A vulgar fraction one half, NFKC `1`, a fraction slash and `2`, before `s`.
              readingsShown({ text: '\u{00BD}’s', },),
              // A lone high surrogate, which no whole character is made of, before `s`.
              readingsShown({ text: '\u{D835}’s', },),
              // A circled digit one (category No, NFKC `1`) before `s`.
              readingsShown({ text: '\u{2460}’s', },),
              // A trade mark sign (NFKC `TM`) before `s`, and a superscript two (category No, NFKC `2`) before `s`.
              readingsShown({ text: 'cat\u{2122}’s', },),
              readingsShown({ text: 'cat\u{00B2}’s', },),
            ],).toEqual([
              ['1:free',],
              ['2:free',],
              ['1:free',],
              ['1:free',],
              ['1:free',],
              ['1:free',],
              ['4:free',],
              ['4:free',],
            ],);
          },
        },),
      ],
    },),
    describe({
      name: `${markReadingsOf.name} by the character beside the mark`,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'READS A MARK AS TRAILING ONLY WHEN WHAT STANDS AFTER IT PARTS IT FROM A WORD: white space, punctuation other than '
            + 'a single mark, or the end of the text, and a mark as leading only when what stands before it parts it likewise',
          fn: async () => {
            expect([
              readingsShown({ text: 'the kittens’ bowl', },),
              readingsShown({ text: 'the kittens’, bowl', },),
              readingsShown({ text: '(the kittens’)', },),
              readingsShown({ text: 'the kittens’', },),
              readingsShown({ text: 'go home ’til dawn', },),
              readingsShown({ text: '(’til dawn)', },),
              readingsShown({ text: '’til dawn', },),
            ],).toEqual([
              ['11:trailing',],
              ['11:trailing',],
              ['12:trailing',],
              ['11:trailing',],
              ['8:leading',],
              ['1:leading',],
              ['0:leading',],
            ],);
          },
        },),
        it({
          name: 'READS A MARK BESIDE A LETTER OR DIGIT OF ANOTHER SCRIPT, A SYMBOL OR A PICTOGRAPH AS FREE, on the side that is not a '
            + 'Latin word as on the side that is: the mark is no apostrophe for the word it touches',
          fn: async () => {
            expect([
              readingsShown({ text: 'the cats’猫 sat', },),
              readingsShown({ text: '猫’rock', },),
              readingsShown({ text: 'the 🐱\'s bowl', },),
              readingsShown({ text: 'the cat™’s bowl', },),
              readingsShown({ text: 'the cats’+ bowl', },),
              readingsShown({ text: 'the cats’² bowl', },),
              readingsShown({ text: 'the cats’🐱 bowl', },),
            ],).toEqual([
              ['8:free',],
              ['1:free',],
              ['6:free',],
              ['8:free',],
              ['8:free',],
              ['8:free',],
              ['8:free',],
            ],);
            expect([
              provenShown({ text: 'the cats’猫 sat', },),
              provenShown({ text: '猫’rock', },),
              provenShown({ text: 'the 🐱\'s bowl', },),
              provenShown({ text: 'the cat™’s bowl', },),
              provenShown({ text: 'the kittens’ bowl', },),
              provenShown({ text: 'go home ’til dawn', },),
            ],).toEqual([
              'the cats’猫 sat',
              '猫’rock',
              'the 🐱\'s bowl',
              'the cat™’s bowl',
              'the kittens⟦’⟧ bowl',
              'go home ⟦’⟧til dawn',
            ],);
          },
        },),
        it({
          name: 'READS A SECOND QUOTATION MARK THAT A PICTOGRAPH OR A SYMBOL STANDS BEFORE AS FREE, so a quotation that ends after '
            + 'one is no longer read as holding an apostrophe that opens a word, and neither of its marks is proven',
          fn: async () => {
            expect([
              provenShown({ text: 'He said \'cats 🐱\'5 now.', },),
              provenShown({ text: 'He said \'cats 🐱\'ss now.', },),
              readingsShown({ text: 'He said \'cats 🐱\'5 now.', },),
            ],).toEqual([
              'He said \'cats 🐱\'5 now.',
              'He said \'cats 🐱\'ss now.',
              ['8:leading', '16:free',],
            ],);
          },
        },),
        it({
          name: 'READS A SINGLE MARK DIRECTLY BESIDE ANOTHER SINGLE MARK AS FREE, both of them, whatever the two spellings and whatever '
            + 'stands beside the pair',
          fn: async () => {
            expect([
              readingsShown({ text: 'He said \'\'cats now.', },),
              readingsShown({ text: 'the kittens’’ bowl', },),
              readingsShown({ text: 'the dogs\u{02BC}’ bowl', },),
              readingsShown({ text: 'the cats’‘dogs bowl', },),
              readingsShown({ text: 'the ’’tis bowl', },),
            ],).toEqual([
              ['8:free', '9:free',],
              ['11:free', '12:free',],
              ['8:free', '9:free',],
              ['8:free',],
              ['4:free', '5:free',],
            ],);
            expect([
              provenShown({ text: 'the kittens’’ bowl', },),
              provenShown({ text: 'the dogs\u{02BC}’ bowl', },),
              provenShown({ text: 'the ’’tis bowl', },),
            ],).toEqual([
              'the kittens’’ bowl',
              'the dogs\u{02BC}’ bowl',
              'the ’’tis bowl',
            ],);
          },
        },),
        it({
          name: 'DOES NOT PROVE A STRAIGHT LEADING MARK OR A LATER TRAILING MARK BY THE OTHER\'S ABSENCE, since the two could be '
            + 'the opening and the closing of one quotation',
          fn: async () => {
            expect([
              provenShown({ text: 'He said \'cats and the dogs’ bowl', },),
              provenShown({ text: 'He said \'cats and the dogs\' bowl', },),
              provenShown({ text: 'He said \'cats and the dogs\u{02BC} bowl', },),
            ],).toEqual([
              'He said \'cats and the dogs’ bowl',
              'He said \'cats and the dogs\' bowl',
              'He said \'cats and the dogs\u{02BC} bowl',
            ],);
          },
        },),
      ],
    },),
    describe({
      name: markReadingsOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'READS EACH APOSTROPHE-LIKE MARK AS ATTACHED, JOINT, TRAILING, LEADING OR FREE, from the letters beside it and the '
            + 'marks the caller names as joined, and reads the opening curly single mark as none of them',
          fn: async () => {
            /**
             The readings of a text as `offset:kind` pairs, in the order of the offsets.
             */
            function readingsOf(
              { text, attached = [], }: { readonly text: string; readonly attached?: readonly number[]; },
            ): readonly string[] {
              return [...markReadingsOf({ text, attached: new Set(attached,), },),].map(
                function shown([at, kind,],): string {
                  return `${String(at,)}:${kind}`;
                },
              );
            }
            expect([
              readingsOf({ text: 'don’t', },),
              readingsOf({ text: 'cats’ bowl', },),
              readingsOf({ text: 'a ’tis', },),
              readingsOf({ text: 'a ’ b', },),
              readingsOf({ text: 'x*’s', attached: [2,], },),
              readingsOf({ text: 'a ‘b', },),
              readingsOf({ text: '猫’猫 αβ’γδ', },),
            ],).toEqual([
              ['3:attached',],
              ['4:trailing',],
              ['2:leading',],
              ['2:free',],
              ['2:joint',],
              [],
              ['1:free', '6:free',],
            ],);
          },
        },),
      ],
    },),
    describe({
      name: `${provenApostrophesOf.name} and ${markReadingsOf.name} past markup`,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'READS A FREE MARK THE CALLER NAMES AS TRAILING PAST MARKUP AS TRAILING, so it is proven when no opener stands '
            + 'before it and no longer stops a later trailing mark from being proven; left unnamed it is free and stops it',
          fn: async () => {
            /**
             The English whose emphasis closes before a plural possessive mark and which holds a later plural possessive.
             */
            const text = 'The *kittens*’ bowl is empty. A dogs’ naps.';
            expect([
              provenShown({ text, },),
              provenShown({ text, past: { trailing: [13,], }, },),
              provenShown({ text: 'He said ‘The *kittens*’ bowl is empty.', past: { trailing: [22,], }, },),
              readingsShown({ text: 'The *kittens*’ bowl', },),
              readingsShown({ text: 'The *kittens*’ bowl', past: { trailing: [13,], }, },),
            ],).toEqual([
              'The *kittens*’ bowl is empty. A dogs’ naps.',
              'The *kittens*⟦’⟧ bowl is empty. A dogs⟦’⟧ naps.',
              'He said ‘The *kittens*’ bowl is empty.',
              ['13:free',],
              ['13:trailing',],
            ],);
          },
        },),
        it({
          name: 'READS A MARK BEFORE OPENING MARKUP WHERE IT STANDS, free, and takes no naming of it from the caller: the reading run '
            + 'has no leading mirror, so the marks past markup are the trailing ones alone',
          fn: async () => {
            /**
             The English whose mark stands before opening emphasis.
             */
            const text = 'He said ’*tis* well, said the cats’ owner.';
            expect([
              provenShown({ text, },),
              readingsShown({ text: 'He said ’*tis* well.', },),
            ],).toEqual([
              'He said ’*tis* well, said the cats’ owner.',
              ['8:free',],
            ],);
          },
        },),
        it({
          name: 'LEAVES A MARK FREE THAT ENDS A READING RUN WHEN A SINGLE MARK, A LETTER OF ANOTHER SCRIPT OR A SYMBOL STANDS RIGHT '
            + 'AFTER IT, since nothing then parts it from a word: the caller names the mark and the reading refuses it',
          fn: async () => {
            expect([
              readingsShown({ text: 'the *kittens*’’ bowl', past: { trailing: [13,], }, },),
              readingsShown({ text: 'the *kittens*’猫 bowl', past: { trailing: [13,], }, },),
              readingsShown({ text: 'the *kittens*’+ bowl', past: { trailing: [13,], }, },),
              readingsShown({ text: 'the *kittens*’. bowl', past: { trailing: [13,], }, },),
            ],).toEqual([
              ['13:free', '14:free',],
              ['13:free',],
              ['13:free',],
              ['13:trailing',],
            ],);
          },
        },),
        it({
          name: 'LEAVES A MARK ALONE THAT ANOTHER RULE READS: a joint stays a joint, a mark a letter stands beside keeps its '
            + 'reading, and a mark after one run and before another is read past the first run only',
          fn: async () => {
            expect([
              [
                ...markReadingsOf({
                  text: 'x*’s',
                  attached: new Set([2,],),
                  pastMarkup: pastMarkupOf({ past: { trailing: [2,], }, },),
                },),
              ],
              readingsShown({ text: 'x*’s', past: { trailing: [2,], }, },),
              readingsShown({ text: 'a ’tis', past: { trailing: [2,], }, },),
              readingsShown({ text: 'cats’ bowl', past: { trailing: [4,], }, },),
              readingsShown({ text: 'the *cat*’*dog* naps', past: { trailing: [9,], }, },),
            ],).toEqual([
              [[2, 'joint',],],
              ['2:leading',],
              ['2:leading',],
              ['4:trailing',],
              ['9:trailing',],
            ],);
          },
        },),
      ],
    },),
  ],
},);
