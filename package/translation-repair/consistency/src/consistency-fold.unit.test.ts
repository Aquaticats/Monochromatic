/**
 Tests for reading a judge's quote in a place's own English: what the fold
 treats as one text, where a quote stands as the English holds it, and the
 cases it must not merge.

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
  foldedWordsOf,
  locateWords,
  marksHeldBesideWords,
  standsOf,
} from '../dist/final/node/index.mjs';
import { pageMarksOf, } from '@monochromatic-dev/translation-repair-test-fixture/ts/consistency-marks-reading.test-fixture.ts';
import { longerRuns, } from '@monochromatic-dev/translation-repair-test-fixture/ts/runs-of-pieces.test-fixture.ts';

/**
 The stretch of an English some words name, as the English holds it.

 @param haystack - English

 @param words - quote that names the span

 @returns Stretch between the offsets the locator gives

 @throws Error when the words do not stand exactly once

 @example
 ```ts
 const span = spanIn({ haystack: 'The cat naps.', words: 'cat.', },); // 'cat'
 ```
 */
function spanIn(
  {
    haystack,
    words,
  }: {
    readonly haystack: string;
    readonly words: string;
  },
): string {
  const found = locateWords({ haystack, words, },);
  if (found.kind !== 'once')
    throw new Error(`the words stand ${found.kind} in the English, and the case needs them once`,);
  return haystack.slice(found.start, found.end,);
}

/**
 The kind of answer the locator gives for some words in an English.

 @param haystack - English

 @param words - quote that names the span

 @returns `none`, `once` or `many`

 @example
 ```ts
 const kind = kindIn({ haystack: 'The cat naps.', words: 'cat', },); // 'once'
 ```
 */
function kindIn(
  {
    haystack,
    words,
  }: {
    readonly haystack: string;
    readonly words: string;
  },
): string {
  return locateWords({ haystack, words, },).kind;
}

/**
 The kinds of answer the locator gives for each pair of texts: one with markup written, and its twin, the text a reader sees of it.

 @param rows - text with the markup, twin, and words asked for

 @returns Kind of answer for the text with the markup of each row, and for the twin of each row

 @example
 ```ts
 const { kinds, twins, } = twinKindsOf({ rows: [['kit[](u)\'kit', 'kit\'kit', 'kit',],], },); // both ['none']
 ```
 */
function twinKindsOf(
  { rows, }: { readonly rows: readonly (readonly [string, string, string])[]; },
): { readonly kinds: readonly string[]; readonly twins: readonly string[]; } {
  return {
    kinds: rows.map(function linkedKindOf([linked, , words,],): string {
      return kindIn({ haystack: linked, words, },);
    },),
    twins: rows.map(function twinKindOf([, twin, words,],): string {
      return kindIn({ haystack: twin, words, },);
    },),
  };
}

/**
 An English with the one span some words name in it replaced, as the splice
 replaces it: by the offsets the locator gives and nothing else.

 @param haystack - English

 @param words - quote that names the span

 @param text - what the span becomes

 @returns English after the replacement

 @throws Error when the words do not stand exactly once

 @example
 ```ts
 const after = replacedIn({ haystack: 'The cat naps.', words: 'cat', text: 'kitten', },); // 'The kitten naps.'
 ```
 */
function replacedIn(
  {
    haystack,
    words,
    text,
  }: {
    readonly haystack: string;
    readonly words: string;
    readonly text: string;
  },
): string {
  const found = locateWords({ haystack, words, },);
  if (found.kind !== 'once')
    throw new Error(`the words stand ${found.kind} in the English, and the case needs them once`,);
  return `${haystack.slice(0, found.start,)}${text}${haystack.slice(found.end,)}`;
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: foldedWordsOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'TREATS A RUN OF WHITESPACE AS ONE SPACE, a line break and a tab included, and keeps a blank line apart '
            + 'from a space as two line breaks',
          fn: async () => {
            expect([
              foldedWordsOf({ text: 'The kitten   naps', },),
              foldedWordsOf({ text: 'The kitten\nnaps', },),
              foldedWordsOf({ text: 'The kitten\t naps', },),
              foldedWordsOf({ text: 'The kitten\n\nnaps', },),
              foldedWordsOf({ text: 'The kitten \n \n naps', },),
            ],).toEqual([
              'The kitten naps',
              'The kitten naps',
              'The kitten naps',
              'The kitten\n\nnaps',
              'The kitten\n\nnaps',
            ],);
          },
        },),
        it({
          name: 'READS ITS OWN RESULT AGAIN AS THE SAME WORDS, over every run of up to four pieces taken from words, '
            + 'spaces, line breaks, sentence punctuation, quotation marks and brackets',
          fn: async () => {
            const pieces = [
              'cat',
              ' ',
              '\n',
              '\n\n',
              '.',
              '"',
              '“',
              '”',
              '\'',
              '\u{02BC}',
              '(',
              ')',
              '「',
              '」',
              '…',
            ];
            const ones = longerRuns({ runs: [''], pieces, },);
            const twos = longerRuns({ runs: ones, pieces, },);
            const threes = longerRuns({ runs: twos, pieces, },);
            const everyRun = [...ones, ...twos, ...threes, ...longerRuns({ runs: threes, pieces, },),];
            const changed = everyRun.filter(function readsDifferently(text,): boolean {
              const once = foldedWordsOf({ text, },);
              return foldedWordsOf({ text: once, },) !== once;
            },);
            expect(changed,).toEqual([],);
          },
        },),
        it({
          name: 'READS A SOFT WRAP AS A SPACE AND A BLANK LINE AS A BLANK LINE, in the first reading and in the second',
          fn: async () => {
            const soft = foldedWordsOf({ text: 'One.\nTwo', },);
            const blank = foldedWordsOf({ text: 'One.\n\nTwo', },);
            expect([
              soft,
              foldedWordsOf({ text: soft, },),
              blank,
              foldedWordsOf({ text: blank, },),
            ],).toEqual([
              'One. Two',
              'One. Two',
              'One.\n\nTwo',
              'One.\n\nTwo',
            ],);
          },
        },),
        it({
          name: 'TAKES THE QUOTATION MARKS AND BRACKETS THAT ENCLOSE A QUOTE OFF ITS EDGES, straight or curly, single or '
            + 'double, corner brackets included, with the punctuation that closes it',
          fn: async () => {
            const kind = 'So she was kind';
            expect([
              '“So she was kind”.',
              '"So she was kind"',
              '‘So she was kind’',
              '「So she was kind」',
              '『So she was kind』',
              '(So she was kind)',
              '[So she was kind]',
              '("So she was kind").',
              '“So she was kind',
              'So she was kind”',
            ].map(function wordsOf(text,): string {
              return foldedWordsOf({ text, },);
            },),).toEqual([kind, kind, kind, kind, kind, kind, kind, kind, kind, kind,],);
          },
        },),
        it({
          name: 'KEEPS A QUOTATION MARK OR A BRACKET AT AN EDGE THAT PAIRS WITH ONE INSIDE THE QUOTE, since the words need it',
          fn: async () => {
            expect([
              '“So she was kind,” she said.',
              'So she was (kind)',
              '(a) and (b)',
              '"Cat" naps',
              'wasn’t',
            ].map(function wordsOf(text,): string {
              return foldedWordsOf({ text, },);
            },),).toEqual([
              '"So she was kind," she said',
              'So she was (kind)',
              '(a) and (b)',
              '"Cat" naps',
              'wasn\'t',
            ],);
          },
        },),
        it({
          name: 'TAKES A QUOTATION MARK OFF AN EDGE WHEN IT PAIRS WITH NOTHING IN THE QUOTE',
          fn: async () => {
            expect([
              'Cat naps."',
              ')Cat naps(',
            ].map(function wordsOf(text,): string {
              return foldedWordsOf({ text, },);
            },),).toEqual([
              'Cat naps',
              'Cat naps',
            ],);
          },
        },),
        it({
          name: 'CARRIES NO SINGLE MARK AT AN EDGE THAT PAIRS WITH NOTHING IN THE QUOTE, straight or curly, an apostrophe '
            + 'or half a quotation, and takes the pair off that encloses a quote holding a plural',
          fn: async () => {
            expect([
              '’Tis the season',
              'we were goin’',
              'the girls’',
              'the girls\'',
              '‘The cat naps',
              'Cats nap’',
              '‘the cats’ bowl’',
              '\'cats\' bowl\'',
            ].map(function wordsOf(text,): string {
              return foldedWordsOf({ text, },);
            },),).toEqual([
              'Tis the season',
              'we were goin',
              'the girls',
              'the girls',
              'The cat naps',
              'Cats nap',
              'the cats\' bowl',
              'cats\' bowl',
            ],);
          },
        },),
        it({
          name: 'NAMES THE SAME WORDS FOR TWO JUDGES WHO COPIED A PASSAGE WITH AND WITHOUT THE MARK BESIDE IT',
          fn: async () => {
            expect([
              [foldedWordsOf({ text: '‘The cat naps', },), foldedWordsOf({ text: 'The cat naps', },),],
              [foldedWordsOf({ text: 'her parents’', },), foldedWordsOf({ text: 'her parents', },),],
              [foldedWordsOf({ text: '’Tis the season', },), foldedWordsOf({ text: 'Tis the season', },),],
              [foldedWordsOf({ text: 'her parents\u{02BC}', },), foldedWordsOf({ text: 'her parents', },),],
              [foldedWordsOf({ text: '‘the cats’ bowl', },), foldedWordsOf({ text: 'the cats’ bowl', },),],
              [foldedWordsOf({ text: '\'the cats\' bowl', },), foldedWordsOf({ text: 'the cats\' bowl', },),],
            ],).toEqual([
              ['The cat naps', 'The cat naps',],
              ['her parents', 'her parents',],
              ['Tis the season', 'Tis the season',],
              ['her parents', 'her parents',],
              ['the cats\' bowl', 'the cats\' bowl',],
              ['the cats\' bowl', 'the cats\' bowl',],
            ],);
          },
        },),
        it({
          name: 'TREATS STRAIGHT AND CURLY QUOTES AND APOSTROPHES AS EQUAL, by the evidence fold',
          fn: async () => {
            expect([
              foldedWordsOf({ text: 'The kitten wasn’t “asleep”', },),
              foldedWordsOf({ text: 'The kitten wasn\'t "asleep"', },),
            ],).toEqual([
              'The kitten wasn\'t "asleep"',
              'The kitten wasn\'t "asleep"',
            ],);
          },
        },),
        it({
          name: 'IGNORES SENTENCE PUNCTUATION, SPACE AND INVISIBLE CHARACTERS AT THE TWO EDGES of a quote, and nowhere else',
          fn: async () => {
            const invisible = String.fromCodePoint(0x20_0B,);
            expect([
              foldedWordsOf({ text: 'The kitten naps.', },),
              foldedWordsOf({ text: '  The kitten naps?!  ', },),
              foldedWordsOf({ text: `${invisible}The kitten naps${invisible}`, },),
              foldedWordsOf({ text: 'The kitten, then naps.', },),
              foldedWordsOf({ text: '...', },),
              foldedWordsOf({ text: '', },),
            ],).toEqual([
              'The kitten naps',
              'The kitten naps',
              'The kitten naps',
              'The kitten, then naps',
              '',
              '',
            ],);
          },
        },),
        it({
          name: 'DOES NOT MERGE WORDS THAT DIFFER, a letter case or a letter apart',
          fn: async () => {
            expect(new Set([
              foldedWordsOf({ text: 'The kitten naps', },),
              foldedWordsOf({ text: 'the kitten naps', },),
              foldedWordsOf({ text: 'The kittens naps', },),
            ],).size,).toBe(3,);
          },
        },),
      ],
    },),
    describe({
      name: locateWords.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'LOCATES A QUOTE THAT STANDS ONCE with the offsets of the English as it was given, a line break the quote '
            + 'writes as a space included',
          fn: async () => {
            const english = 'It purred.\nThe kitten wasn’t asleep,\nwas it?\nIt blinked.';
            const found = locateWords({ haystack: english, words: 'The kitten wasn\'t asleep, was it?', },);
            expect(found,).toEqual({ kind: 'once', start: 11, end: 43, },);
            expect(english.slice(11, 43,),).toBe('The kitten wasn’t asleep,\nwas it',);
          },
        },),
        it({
          name: 'LOCATES A QUOTE ACROSS A RUN OF SPACES IN THE ENGLISH, the end offset past the whole run it stands for',
          fn: async () => {
            const english = 'The kitten   naps  in the sun.';
            const found = locateWords({ haystack: english, words: 'The kitten naps in', },);
            expect(found,).toEqual({ kind: 'once', start: 0, end: 21, },);
            expect(english.slice(0, 21,),).toBe('The kitten   naps  in',);
          },
        },),
        it({
          name: 'FINDS NOTHING for a quote that stands nowhere, a quote that shows a reader nothing, and an empty English',
          fn: async () => {
            expect([
              locateWords({ haystack: 'The kitten naps.', words: 'The kitten dozes', },),
              locateWords({ haystack: 'The kitten naps.', words: '...', },),
              locateWords({ haystack: '', words: 'The kitten', },),
            ],).toEqual([{ kind: 'none', }, { kind: 'none', }, { kind: 'none', },],);
          },
        },),
        it({
          name: 'FINDS MANY for a quote that stands more than once, overlapping stands included',
          fn: async () => {
            expect([
              locateWords({ haystack: 'The cat naps. The cat naps.', words: 'The cat naps', },),
              locateWords({ haystack: '猫猫猫', words: '猫猫', },),
            ],).toEqual([{ kind: 'many', }, { kind: 'many', },],);
          },
        },),
        it({
          name: 'FINDS NONE for the half of a word that inline code padded by one space on each side shows without the padding (n166, Z6)',
          fn: async () => {
            expect([
              locateWords({ haystack: 'The ` kit `tens ate.', words: 'tens', },),
              locateWords({ haystack: 'The kit` tens ` ate.', words: 'kit', },),
            ],).toEqual([{ kind: 'none', }, { kind: 'none', },],);
            expect([
              spanIn({ haystack: 'The `kit `tens ate.', words: 'tens', },),
              spanIn({ haystack: 'The `  kit  `tens ate.', words: 'tens', },),
            ],).toEqual(['tens', 'tens',],);
          },
        },),
        it({
          name: 'FINDS NONE for the half of a word that inline code padded by line endings shows without them, since CommonMark turns each '
            + 'line ending into a space before it strips one space from each side, and finds the word once where only one side holds a '
            + 'line ending or the content is line endings alone',
          fn: async () => {
            expect([
              locateWords({ haystack: 'The kit`\ntens\n` ate.', words: 'kit', },).kind,
              locateWords({ haystack: 'The kit`\r\ntens\r\n` ate.', words: 'kit', },).kind,
              locateWords({ haystack: 'The kit`\ntens ` ate.', words: 'kit', },).kind,
              locateWords({ haystack: 'The kit`\ntens` ate.', words: 'kit', },).kind,
              locateWords({ haystack: 'The kit`\n` ate.', words: 'kit', },).kind,
            ],).toEqual(['none', 'none', 'none', 'once', 'once',],);
          },
        },),
        it({
          name: 'READS INLINE CODE AS PADDED BY SPACES ALONE when the caller names the earlier reading, as a recount of an artifact written '
            + 'before line endings padded code reads it: the word beside code padded by line endings is a stand there, and the readings '
            + 'agree where the code is padded by spaces or by no line ending at all',
          fn: async () => {
            /**
             The kinds the locator gives for the cases of the line ending reading, under each reading of the padding.
             */
            function kindsUnder({ codePadding, }: { readonly codePadding?: 'spaces' | 'spaces-and-line-endings'; },): readonly string[] {
              return [
                'The kit`\ntens\n` ate.',
                'The kit`\r\ntens\r\n` ate.',
                'The kit`\ntens ` ate.',
                'The kit`\ntens` ate.',
                'The kit`\n` ate.',
                'The kit` tens ` ate.',
              ].map(function kindOf(haystack,): string {
                return locateWords({ haystack, words: 'kit', ...((codePadding === undefined) ? {} : { codePadding, }), },).kind;
              },);
            }
            expect({
              earlier: kindsUnder({ codePadding: 'spaces', },),
              later: kindsUnder({ codePadding: 'spaces-and-line-endings', },),
              byDefault: kindsUnder({},),
            },).toEqual({
              earlier: ['once', 'once', 'once', 'once', 'once', 'none',],
              later: ['none', 'none', 'none', 'once', 'once', 'none',],
              byDefault: ['none', 'none', 'none', 'once', 'once', 'none',],
            },);
          },
        },),
        it({
          name: 'FINDS THE WORD ONCE beside the tags of a raw element, since the parser reads them to their closing tag and they end a word '
            + '(n166, Z7)',
          fn: async () => {
            expect([
              spanIn({ haystack: 'cats<pre>dogs</pre>', words: 'cats', },),
              spanIn({ haystack: 'cats<textarea></textarea>dogs', words: 'cats', },),
              spanIn({ haystack: 'cats<script></script>dogs', words: 'dogs', },),
              spanIn({ haystack: 'cats<style></style>dogs', words: 'dogs', },),
            ],).toEqual(['cats', 'cats', 'dogs', 'dogs',],);
          },
        },),
        it({
          name: 'FINDS THE WORD ONCE after an apostrophe that follows a link whose label holds no word, since no word run reaches the '
            + 'apostrophe (n166, Z9)',
          fn: async () => {
            expect([
              spanIn({ haystack: ' [](u&#116;)\'kit', words: 'kit', },),
              spanIn({ haystack: ' [](u "&#116;")\'kit', words: 'kit', },),
              spanIn({ haystack: '[](u)\'kit', words: 'kit', },),
              spanIn({ haystack: '[ ](u&#116;)\'kit', words: 'kit', },),
              spanIn({ haystack: '[-](u&#116;)\'kit', words: 'kit', },),
              spanIn({ haystack: '-[_](u "&#116;")\'kit', words: 'kit', },),
              spanIn({ haystack: '![](u)\'kit', words: 'kit', },),
            // The apostrophe is the one that opens a word, as in the same text with no link there (the proof reads the twin of the
            // English that a reader sees, as the cut of a word does), so the proven leading apostrophe is in the span: `'kit`.
            ],).toEqual(['\'kit', '\'kit', '\'kit', '\'kit', '\'kit', '\'kit', '\'kit',],);
            expect([
              spanIn({ haystack: ' \'kit', words: 'kit', },),
              spanIn({ haystack: '\'kit', words: 'kit', },),
              spanIn({ haystack: '-\'kit', words: 'kit', },),
            ],).toEqual(['\'kit', '\'kit', '\'kit',],);
          },
        },),
        it({
          name: 'FINDS NONE for the word after an apostrophe that a word run reaches through a link, a bracket or a letter beside an empty '
            + 'link, as before (n166, Z9 controls)',
          fn: async () => {
            expect([
              locateWords({ haystack: ' [x](u&#116;)\'kit', words: 'kit', },),
              locateWords({ haystack: '[the cat](u)\'s bowl', words: 's bowl', },),
              locateWords({ haystack: '(grey cat)\'s bowl', words: 's bowl', },),
              locateWords({ haystack: 'a[](u)\'kit', words: 'kit', },),
            ],).toEqual([{ kind: 'none', }, { kind: 'none', }, { kind: 'none', }, { kind: 'none', },],);
          },
        },),
        it({
          name: 'ANSWERS FOR A WORD AFTER AN EMPTY LINK AND AN APOSTROPHE as for the same text with no link there, whatever the link '
            + 'writes in its destination, its title or its label, and a run of empty links as one (n168)',
          fn: async () => {
            /**
             Each text with the empty link written after a word, the text a reader sees of it (no link), and the word asked for: the
             word before the link and the word after the apostrophe are one, so the stand after the apostrophe is cut.
             */
            const rows: readonly (readonly [string, string, string])[] = [
              ['kit[](u)\'kit', 'kit\'kit', 'kit',],
              ['kit[](u&#116;)\'kit', 'kit\'kit', 'kit',],
              ['kit[](u "t")\'kit', 'kit\'kit', 'kit',],
              ['kit[]()\'kit', 'kit\'kit', 'kit',],
              ['kit[](<>)\'kit', 'kit\'kit', 'kit',],
              ['kit[][r]\'kit\n\n[r]: u', 'kit\'kit\n\n[r]: u', 'kit',],
              ['kit[](u)[](v)\'kit', 'kit\'kit', 'kit',],
              ['kit[](u)\'tens', 'kit\'tens', 'tens',],
              ['kit[](u(a))\'kit', 'kit\'kit', 'kit',],
              [String.raw`kit[](u\(a)'kit`, 'kit\'kit', 'kit',],
              [String.raw`kit[](u\\)'kit`, 'kit\'kit', 'kit',],
              ['kit[](<u(a>)\'kit', 'kit\'kit', 'kit',],
              ['kit[](u "t(a)")\'kit', 'kit\'kit', 'kit',],
              [String.raw`kit[](u "t\(a")'kit`, 'kit\'kit', 'kit',],
              ['kit[](u \'t(a)\')\'kit', 'kit\'kit', 'kit',],
              ['(grey cat)[](u)\'kit', '(grey cat)\'kit', 'kit',],
              ['[the cat](x)[](u)\'kit', '[the cat](x)\'kit', 'kit',],
            ];
            const { kinds, twins, } = twinKindsOf({ rows, },);
            expect(kinds,).toEqual(twins,);
            expect(kinds,).toEqual(rows.map(function none(): string {
              return 'none';
            },),);
          },
        },),
        it({
          name: 'ANSWERS FOR A WORD AFTER AN EMPTY LINK AND AN APOSTROPHE, after a space, as for the same text with no link there: the '
            + 'apostrophe starts the word, whatever the link writes, and an image with no alt text and a label with a bracket of its own '
            + 'are read like the rest (n168)',
          fn: async () => {
            /**
             Each text with the link written after a space, the text a reader sees of it, and the word asked for.
             */
            const rows: readonly (readonly [string, string, string])[] = [
              ['a [](u(a))\'kit', 'a \'kit', 'kit',],
              ['a [](u "t(a)")\'kit', 'a \'kit', 'kit',],
              ['a [](<u(a>)\'kit', 'a \'kit', 'kit',],
              [String.raw`a [](u\(a)'kit`, 'a \'kit', 'kit',],
              ['a [](u)[](v)\'kit', 'a \'kit', 'kit',],
              ['a [](u) [](v)\'kit', 'a  \'kit', 'kit',],
              ['a ![](u)\'kit', 'a \'kit', 'kit',],
              ['a ![][r]\'kit\n\n[r]: u', 'a \'kit\n\n[r]: u', 'kit',],
              ['a [](u)![](v)[]()\'kit', 'a \'kit', 'kit',],
              ['a [](u\n)\'kit', 'a \'kit', 'kit',],
              ['a [ ](u)\'kit', 'a  \'kit', 'kit',],
              ['a [-](u)\'kit', 'a -\'kit', 'kit',],
              [String.raw`a [\[](u)'kit`, 'a [\'kit', 'kit',],
            ];
            const { kinds, twins, } = twinKindsOf({ rows, },);
            expect(kinds,).toEqual(twins,);
            expect(kinds,).toEqual(rows.map(function once(): string {
              return 'once';
            },),);
          },
        },),
        it({
          name: 'ANSWERS FOR A WORD AFTER A LINK THAT SHOWS A WORD OR A CLOSING MARK, a bracket that is no link and a link of marks as for the '
            + 'text a reader sees there, since the apostrophe follows what is shown (n168 controls)',
          fn: async () => {
            /**
             Each text, the same text read as a reader sees it (a bracket for a link), and the word asked for.
             */
            const rows: readonly (readonly [string, string, string])[] = [
              ['kit[x](u)\'kit', 'kitx\'kit', 'kit',],
              ['[the cat](u(a))\'s bowl', 'the cat\'s bowl', 's bowl',],
              ['kit[](u(a)\'kit', 'kit(a)\'kit', 'kit',],
              [String.raw`kit[](u\)'kit`, 'kit(a)\'kit', 'kit',],
              ['kit[](u\nv)\'kit', 'kit(a)\'kit', 'kit',],
              ['kit[](<u)\'kit', 'kit(a)\'kit', 'kit',],
              ['kit[][r]\'kit', 'kit(a)\'kit', 'kit',],
              [String.raw`a [\]](u)'kit`, 'a ]\'kit', 'kit',],
              ['a [()](u)\'kit', 'a ()\'kit', 'kit',],
              ['a [”](u)\'kit', 'a ”\'kit', 'kit',],
              ['a [`-`)](u)\'kit', 'a -)\'kit', 'kit',],
              ['a [*-* )](u)\'kit', 'a - )\'kit', 'kit',],
              ['a [`)`-](u)\'kit', 'a )-\'kit', 'kit',],
              ['a [*)*-](u)\'kit', 'a )-\'kit', 'kit',],
              ['kit[ ](u)\'kit', 'kit \'kit', 'kit',],
              ['kit[-](u)\'kit', 'kit-\'kit', 'kit',],
              ['kit[_](u)\'kit', 'kit_\'kit', 'kit',],
              [String.raw`kit[\[](u)'kit`, 'kit[\'kit', 'kit',],
            ];
            const { kinds, twins, } = twinKindsOf({ rows, },);
            expect(kinds,).toEqual(twins,);
          },
        },),
        it({
          name: 'ANSWERS FOR A WORD BEFORE A LINK WHOSE LABEL IS A STRAIGHT QUOTATION MARK AND AN APOSTROPHE as for the text a reader sees: '
            + 'the mark closes after the word, so the apostrophe after it makes the word part of a longer one',
          fn: async () => {
            /**
             Each text, the text a reader sees of it, and the word asked for.
             */
            const rows: readonly (readonly [string, string, string])[] = [
              ['kit["](u)\'s', 'kit"\'s', 'kit',],
            ];
            const { kinds, twins, } = twinKindsOf({ rows, },);
            expect(kinds,).toEqual(twins,);
            expect(twins,).toEqual(['none',],);
          },
        },),
        it({
          name: 'ANSWERS FOR A WORD AFTER A LINK WHOSE LABEL ENDS IN WHITE SPACE AND AN APOSTROPHE as for the text a reader sees: the '
            + 'apostrophe has white space before it, so it joins no word and the word after it is a stand',
          fn: async () => {
            /**
             Each text, the text a reader sees of it, and the word asked for.
             */
            const rows: readonly (readonly [string, string, string])[] = [
              ['[cat ](u)\'kit', 'cat \'kit', 'kit',],
            ];
            const { kinds, twins, } = twinKindsOf({ rows, },);
            expect(kinds,).toEqual(twins,);
            expect(twins,).toEqual(['once',],);
          },
        },),
        it({
          name: 'ANSWERS FOR A WORD BEFORE AN EMPTY LINK AND AN APOSTROPHE as for the text a reader sees: the forward reading steps over the '
            + 'link, an image with no alt text shows nothing, a destination with a line break is part of the link, and a label of closing '
            + 'marks closes as those marks do (n169)',
          fn: async () => {
            /**
             Each text, the text a reader sees of it, and the word asked for.
             */
            const rows: readonly (readonly [string, string, string])[] = [
              ['kit![](u)\'kit', 'kit\'kit', 'kit',],
              ['kit[](u\n)\'kit', 'kit\'kit', 'kit',],
              [String.raw`kit[\]](u)'kit`, 'kit]\'kit', 'kit',],
              ['kit[”](u)\'kit', 'kit”\'kit', 'kit',],
              ['kit[](u)\'kit kit', 'kit\'kit kit', 'kit',],
            ];
            const { kinds, twins, } = twinKindsOf({ rows, },);
            expect(kinds,).toEqual(twins,);
            expect(twins,).toEqual(['none', 'none', 'none', 'none', 'once',],);
          },
        },),
        it({
          name: 'ANSWERS FOR RUNS OF EMPTY LINKS, an empty link at the start or the end of the text, and an empty link beside or inside '
            + 'emphasis as for the text a reader sees, whichever joint the neighbours make: an apostrophe, a hyphen, a full stop between '
            + 'digits, or closing marks before an apostrophe (n169)',
          fn: async () => {
            /**
             Each text, the text a reader sees of it, and the word asked for.
             */
            const rows: readonly (readonly [string, string, string])[] = [
              ['kit[](u)[](v)\'kit kit', 'kit\'kit kit', 'kit',],
              ['kit[](u)![](v)[]()\'kit kit', 'kit\'kit kit', 'kit',],
              ['kit![][r]\'kit kit\n\n[r]: u', 'kit\'kit kit\n\n[r]: u', 'kit',],
              ['kit[](u)-kit', 'kit-kit', 'kit',],
              ['kit-[](u)kit', 'kit-kit', 'kit',],
              ['kit[](u)[](v)-[](w)kit', 'kit-kit', 'kit',],
              ['kit\'[](u)kit', 'kit\'kit', 'kit',],
              ['kit[](u)\'[](v)kit', 'kit\'kit', 'kit',],
              ['1[](u).5', '1.5', '1',],
              ['[](u)kit', 'kit', 'kit',],
              ['kit[](u)', 'kit', 'kit',],
              ['[](u)kit[](v)', 'kit', 'kit',],
              ['*kit*[](u)\'kit', '*kit*\'kit', 'kit',],
              ['*kit[](u)*\'kit', '*kit*\'kit', 'kit',],
              ['*[](u)kit*\'kit', '*kit*\'kit', 'kit',],
              ['**kit**[](u)\'s', '**kit**\'s', 'kit',],
              ['"kit"[](u)\'s', '"kit"\'s', 'kit',],
              ['(kit)[](u)\'s', '(kit)\'s', 'kit',],
              ['[kit](x)[](u)\'s', '[kit](x)\'s', 'kit',],
              ['kit[](u)]\'kit', 'kit]\'kit', 'kit',],
              [String.raw`kit[](u)[\]](v)'kit`, 'kit]\'kit', 'kit',],
              [String.raw`kit[\]](u)[](v)'kit`, 'kit]\'kit', 'kit',],
              [String.raw`kit[\]](u)[\]](v)'kit`, 'kit]]\'kit', 'kit',],
              ['a [](u)kit', 'a kit', 'kit',],
              ['kit[](u)(a)\'kit', 'kit(a)\'kit', 'kit',],
            ];
            const { kinds, twins, } = twinKindsOf({ rows, },);
            expect(kinds,).toEqual(twins,);
            expect(twins,).toEqual([
              'once',
              'once',
              'once',
              'none',
              'none',
              'none',
              'none',
              'none',
              'none',
              'once',
              'once',
              'once',
              'none',
              'none',
              'none',
              'none',
              'none',
              'none',
              'none',
              'none',
              'none',
              'none',
              'none',
              'once',
              'once',
            ],);
          },
        },),
        it({
          name: 'KEEPS THE PRESENT ANSWER for an image with an alt of only spaces or a word, a link of marks and spaces, a reference with '
            + 'no definition, and a blank line, each as for the text a reader sees (n169 controls)',
          fn: async () => {
            /**
             Each text, the text a reader sees of it, and the word asked for.
             */
            const rows: readonly (readonly [string, string, string])[] = [
              ['kit![ ](u)\'kit', 'kit \'kit', 'kit',],
              ['kit![alt](u)\'kit', 'kit alt\'kit', 'kit',],
              ['kit![][r]\'kit', 'kit(a)\'kit', 'kit',],
              ['kit[ ](u)\'kit', 'kit \'kit', 'kit',],
              ['kit[-](u)\'kit', 'kit-\'kit', 'kit',],
              ['kit[](u)\n\nkit', 'kit\n\nkit', 'kit',],
            ];
            const { kinds, twins, } = twinKindsOf({ rows, },);
            expect(kinds,).toEqual(twins,);
            expect(twins,).toEqual(['many', 'once', 'once', 'many', 'many', 'many',],);
          },
        },),
        it({
          name: 'FINDS NO WORD ENDING OR STARTING AT AN EMPTY LINK THAT LETTERS STAND BESIDE ON BOTH SIDES, since the text on the two sides '
            + 'of the link is one word to a reader, as in the text with no link there (n169)',
          fn: async () => {
            /**
             Each text, the text a reader sees of it, and the word asked for.
             */
            const rows: readonly (readonly [string, string, string])[] = [
              ['kit[](u)kit', 'kitkit', 'kit',],
              ['kit[](u)tens', 'kittens', 'kit',],
              ['kit[](u)tens', 'kittens', 'tens',],
              ['kit[](u)![](v)[]()kit', 'kitkit', 'kit',],
              ['kit![][r]kit\n\n[r]: u', 'kitkit\n\n[r]: u', 'kit',],
              ['1[](u)2', '12', '1',],
              ['kit[](u) kit', 'kit kit', 'kit',],
              ['kit [](u)kit', 'kit kit', 'kit',],
              ['kit[](u)\nkit', 'kit\nkit', 'kit',],
              ['kit[ ](u)kit', 'kit kit', 'kit',],
              ['kit*[](u)kit*', 'kit*kit*', 'kit',],
            ];
            const { kinds, twins, } = twinKindsOf({ rows, },);
            expect(kinds,).toEqual(twins,);
            expect(twins,).toEqual(['none', 'none', 'none', 'none', 'none', 'none', 'many', 'many', 'many', 'many', 'many',],);
          },
        },),
        it({
          name: 'READS A REFERENCE WITH AN EMPTY LABEL AS A LINK where its definition stands in another slice of the page, as the page '
            + 'shows it, and as text where the page holds no definition of that label (n169)',
          fn: async () => {
            /**
             The kind of answer for a text on a page whose other slices hold some definition lines.
             */
            function onPage(
              {
                haystack,
                words,
                definition,
              }: {
                readonly haystack: string;
                readonly words: string;
                readonly definition: string;
              },
            ): string {
              return locateWords({
                haystack,
                words,
                page: {
                  text: `${haystack}\n\n${definition}`,
                  offset: 0,
                },
              },).kind;
            }
            expect([
              onPage({ haystack: 'a [][r]\'kit', words: 'kit', definition: '[r]: u', },),
              onPage({ haystack: 'kit[][r]\'kit kit', words: 'kit', definition: '[r]: u', },),
              onPage({ haystack: 'a ![][r]\'kit', words: 'kit', definition: '[r]: u', },),
              onPage({ haystack: 'a [][r]\'kit', words: 'kit', definition: '[x]: u', },),
              onPage({ haystack: 'kit[][r]\'kit kit', words: 'kit', definition: '[x]: u', },),
              kindIn({ haystack: 'a [][r]\'kit', words: 'kit', },),
              kindIn({ haystack: 'kit[][r]\'kit kit', words: 'kit', },),
            ],).toEqual(['once', 'once', 'once', 'none', 'many', 'none', 'many',],);
            expect([
              kindIn({ haystack: 'a \'kit', words: 'kit', },),
              kindIn({ haystack: 'kit\'kit kit', words: 'kit', },),
            ],).toEqual(['once', 'once',],);
          },
        },),
        it({
          name: 'CUTS THE FINAL EDGE BY THE REFERENCES THE PAGE DEFINES, so a word that a reference shows a letter before names no span '
            + 'when its definition stands in the page or is handed in, and does where no definition of that label stands',
          fn: async () => {
            const rows = ['a [x][r]*kit*', 'a[][r]*kit*',];
            expect(rows.map(function onPageDefining(haystack,): string {
              return locateWords({ haystack, words: 'kit', page: { text: `${haystack}\n\n[r]: u`, offset: 0, }, },).kind;
            },),).toEqual(['none', 'none',],);
            expect(rows.map(function handedDefinitions(haystack,): string {
              return locateWords({ haystack, words: 'kit', definitions: '[r]: u', },).kind;
            },),).toEqual(['none', 'none',],);
            expect(rows.map(function onPageDefiningAnother(haystack,): string {
              return locateWords({ haystack, words: 'kit', page: { text: `${haystack}\n\n[q]: u`, offset: 0, }, },).kind;
            },),).toEqual(['once', 'once',],);
            expect(rows.map(function onNoPage(haystack,): string {
              return locateWords({ haystack, words: 'kit', },).kind;
            },),).toEqual(['once', 'once',],);
          },
        },),
        it({
          name: 'KEEPS THE APOSTROPHE A JOINT after a link that holds a letter, and after an empty link whose own neighbour joins it '
            + 'to the word before (n168 controls)',
          fn: async () => {
            expect([
              locateWords({ haystack: 'a[x](u)\'kit', words: 'kit', },).kind,
              locateWords({ haystack: '[the cat](u)\'s bowl', words: 's bowl', },).kind,
              locateWords({ haystack: 'a[](u(a))\'kit', words: 'kit', },).kind,
              locateWords({ haystack: '[a](x)[](u(a))\'kit', words: 'kit', },).kind,
            ],).toEqual(['none', 'none', 'none', 'none',],);
          },
        },),
        it({
          name: 'KEEPS A BLANK LINE APART FROM A SPACE, so a quote across a paragraph boundary is found only when it holds one',
          fn: async () => {
            const english = 'The kitten naps.\n\nThe dog barks.';
            expect([
              locateWords({ haystack: english, words: 'kitten naps.\n\nThe dog', },),
              locateWords({ haystack: english, words: 'kitten naps. The dog', },),
            ],).toEqual([{ kind: 'once', start: 4, end: 25, }, { kind: 'none', },],);
          },
        },),
        it({
          name: 'LOCATES A QUOTE ACROSS A BLANK LINE WHETHER ITS CALLER FOLDED IT FIRST OR NOT, the offsets those of the '
            + 'English as it was given',
          fn: async () => {
            const english = 'One.\n\nTwo.';
            const raw = 'One.\n\nTwo';
            expect([
              locateWords({ haystack: english, words: raw, },),
              locateWords({ haystack: english, words: foldedWordsOf({ text: raw, },), },),
            ],).toEqual([{ kind: 'once', start: 0, end: 9, }, { kind: 'once', start: 0, end: 9, },],);
          },
        },),
        it({
          name: 'FINDS A WORD NOWHERE WHEN IT STANDS ONLY INSIDE A LONGER WORD, at its start, at its end or in the middle, '
            + 'for letters and for digits',
          fn: async () => {
            expect([
              locateWords({ haystack: 'The kitten also napped.', words: 'so', },),
              locateWords({ haystack: 'So she was a smart girl', words: 'art', },),
              locateWords({ haystack: 'The cats nap.', words: 'cat', },),
              locateWords({ haystack: 'The kitten has 123 toys.', words: '12', },),
              locateWords({ haystack: 'The kitten has 123 toys.', words: '23', },),
            ],).toEqual([
              { kind: 'none', },
              { kind: 'none', },
              { kind: 'none', },
              { kind: 'none', },
              { kind: 'none', },
            ],);
          },
        },),
        it({
          name: 'FINDS A WORD ONCE WHEN ONE OF TWO STANDS IS INSIDE A LONGER WORD, and a word standing apart from its '
            + 'neighbours beside punctuation',
          fn: async () => {
            expect([
              locateWords({ haystack: 'also so', words: 'so', },),
              locateWords({ haystack: 'The kitten has 123 toys.', words: '123', },),
              locateWords({ haystack: 'The kitten, so it napped.', words: 'so', },),
            ],).toEqual([
              { kind: 'once', start: 5, end: 7, },
              { kind: 'once', start: 15, end: 18, },
              { kind: 'once', start: 12, end: 14, },
            ],);
          },
        },),
        it({
          name: 'NEEDS NO BOUNDARY BESIDE HAN TEXT, a Han edge standing inside a run of Han and a Latin edge beside Han',
          fn: async () => {
            expect([
              locateWords({ haystack: '小猫在窗台上打盹', words: '猫', },),
              locateWords({ haystack: '小猫cat咪', words: 'cat', },),
            ],).toEqual([
              { kind: 'once', start: 1, end: 2, },
              { kind: 'once', start: 2, end: 5, },
            ],);
          },
        },),
        it({
          name: 'LOCATES A QUOTE WITHOUT THE MARKS THAT ENCLOSE IT WHERE THE ENGLISH HOLDS THEM, and with the marks that '
            + 'pair inside it',
          fn: async () => {
            expect([
              locateWords({ haystack: 'She said, “So she was kind.”', words: 'So she was kind', },),
              locateWords({ haystack: 'She said, “So she was kind”.', words: '“So she was kind”', },),
              locateWords({ haystack: '“So she was kind,” she said.', words: '"So she was kind," she said', },),
              locateWords({ haystack: 'The "Cat" naps.', words: '"Cat" naps', },),
            ],).toEqual([
              { kind: 'once', start: 11, end: 26, },
              { kind: 'once', start: 11, end: 26, },
              { kind: 'once', start: 0, end: 27, },
              { kind: 'once', start: 4, end: 14, },
            ],);
          },
        },),
        it({
          name: 'TAKES INTO THE SPAN THE SINGLE MARK THE ENGLISH HOLDS BESIDE THE WORDS WHEN IT IS A PROVEN APOSTROPHE, whether '
            + 'the quote copied it or not: the apostrophe of a dropped opening, of a dropped final letter, of a plural '
            + 'and of a year',
          fn: async () => {
            expect([
              locateWords({ haystack: 'Tis the season to be jolly.', words: '\'Tis the season', },),
              locateWords({ haystack: '\'Tis the season to be jolly.', words: '\'Tis the season', },),
              locateWords({ haystack: '’Tis the season to be jolly.', words: 'Tis the season', },),
              locateWords({ haystack: 'We were goin’ home.', words: 'We were goin\'', },),
              locateWords({ haystack: 'Amid her parents’ scolding.', words: 'her parents', },),
              locateWords({ haystack: 'Amid her parents’ scolding.', words: 'her parents’', },),
              locateWords({ haystack: 'Cats of the ’90s nap.', words: '90s', },),
              locateWords({ haystack: 'the cats\u{02BC} bowl is full', words: 'the cats', },),
            ],).toEqual([
              { kind: 'once', start: 0, end: 14, },
              { kind: 'once', start: 0, end: 15, },
              { kind: 'once', start: 0, end: 15, },
              { kind: 'once', start: 0, end: 13, },
              { kind: 'once', start: 5, end: 17, },
              { kind: 'once', start: 5, end: 17, },
              { kind: 'once', start: 12, end: 16, },
              { kind: 'once', start: 0, end: 9, },
            ],);
          },
        },),
        it({
          name: 'KEEPS AN APOSTROPHE INSIDE THE WORDS IN THE SPAN, since the words on both sides of it are the quote\'s',
          fn: async () => {
            const enclosed = 'He saw ‘the cats’ bowl’ there.';
            const found = [
              locateWords({ haystack: 'Fill the cats’ bowl.', words: 'the cats\' bowl', },),
              locateWords({ haystack: enclosed, words: '‘the cats’ bowl’', },),
              locateWords({ haystack: enclosed, words: 'the cats’ bowl', },),
            ];
            expect(found,).toEqual([
              { kind: 'once', start: 5, end: 19, },
              { kind: 'once', start: 8, end: 22, },
              { kind: 'once', start: 8, end: 22, },
            ],);
          },
        },),
        it({
          name: 'LEAVES AN OPENING MARK OUTSIDE THE SPAN WHEN A JUDGE COPIED IT WITH A PLURAL POSSESSIVE LATER IN THE '
            + 'QUOTE, curly or straight, so the replacement keeps the page\'s quotation marks',
          fn: async () => {
            expect([
              locateWords({ haystack: 'He said ‘the cats’ bowl is full.’', words: '‘the cats’ bowl', },),
              locateWords({ haystack: 'He said \'the cats\' bowl is full.\'', words: '\'the cats\' bowl', },),
              locateWords({ haystack: 'He said ‘the cats’ bowl is full.’', words: '‘the cats’ bowl is full', },),
            ],).toEqual([
              { kind: 'once', start: 9, end: 23, },
              { kind: 'once', start: 9, end: 23, },
              { kind: 'once', start: 9, end: 31, },
            ],);
            expect(replacedIn({
              haystack: 'He said ‘the cats’ bowl is full.’',
              words: '‘the cats’ bowl',
              text: 'The kittens’ bowls',
            },),).toBe('He said ‘The kittens’ bowls is full.’',);
          },
        },),
        it({
          name: 'TAKES THE MODIFIER LETTER APOSTROPHE INTO THE SPAN WHEN IT IS PROVEN, at either edge of the quote, as it does '
            + 'the straight and the curly marks',
          fn: async () => {
            expect([
              locateWords({ haystack: 'Fill the cats\u{02BC} bowl.', words: 'the cats\u{02BC}', },),
              locateWords({ haystack: 'Fill the cats\u{02BC} bowl.', words: 'the cats', },),
              locateWords({ haystack: 'Say \u{02BC}Tis the season.', words: '\u{02BC}Tis the season', },),
              locateWords({ haystack: 'Say \u{02BC}Tis the season.', words: 'Tis the season', },),
            ],).toEqual([
              { kind: 'once', start: 5, end: 14, },
              { kind: 'once', start: 5, end: 14, },
              { kind: 'once', start: 4, end: 19, },
              { kind: 'once', start: 4, end: 19, },
            ],);
          },
        },),
        it({
          name: 'CHOOSES BETWEEN STANDS OF THE SAME WORDS BY THE MARK THE QUOTE HELD BESIDE THEM: the stand with a single mark '
            + 'on that side of it is the place, straight or curly, before the words or after them',
          fn: async () => {
            expect([
              locateWords({ haystack: 'Fill the cats\' bowl. Then the cats nap.', words: 'the cats\'', },),
              locateWords({ haystack: 'He was goin\' home, and goin there.', words: 'goin\'', },),
              locateWords({ haystack: '\'Tis the season. Tis the season.', words: '\'Tis', },),
              locateWords({ haystack: 'The \'90s cats and the 90s cats.', words: '\'90s cats', },),
              locateWords({ haystack: 'She said \'em all and em none.', words: '\'em', },),
              locateWords({ haystack: 'Fill the cats’ bowl. Then the cats nap.', words: 'the cats’', },),
              locateWords({ haystack: 'He was goin’ home, and goin there.', words: 'goin’', },),
              locateWords({ haystack: '’Tis the season. Tis the season.', words: '’Tis', },),
              locateWords({ haystack: 'The ’90s cats and the 90s cats.', words: '’90s cats', },),
              locateWords({ haystack: 'She said ’em all and em none.', words: '’em', },),
            ],).toEqual([
              { kind: 'once', start: 5, end: 14, },
              { kind: 'once', start: 7, end: 12, },
              { kind: 'once', start: 0, end: 4, },
              { kind: 'once', start: 4, end: 13, },
              { kind: 'once', start: 9, end: 12, },
              { kind: 'once', start: 5, end: 14, },
              { kind: 'once', start: 7, end: 12, },
              { kind: 'once', start: 0, end: 4, },
              { kind: 'once', start: 4, end: 13, },
              { kind: 'once', start: 9, end: 12, },
            ],);
          },
        },),
        it({
          name: 'CHOOSES BY THE MARK ON EACH SIDE THE QUOTE HELD, both sides at once, and by a mark of another kind than the '
            + 'quote\'s',
          fn: async () => {
            expect([
              locateWords({
                haystack: 'He saw \'the cats\' there. Then the cats nap. Then the cats\' nap.',
                words: '\'the cats\'',
              },),
              locateWords({
                haystack: 'He saw ‘the cats’ there. Then ‘the cats nap. Then the cats’ nap.',
                words: '‘the cats’',
              },),
              locateWords({ haystack: 'Fill the cats\u{02BC} bowl. Then the cats nap.', words: 'the cats\'', },),
            ],).toEqual([
              { kind: 'once', start: 8, end: 16, },
              { kind: 'once', start: 8, end: 16, },
              { kind: 'once', start: 5, end: 14, },
            ],);
          },
        },),
        it({
          name: 'STAYS AMBIGUOUS WHERE THE MARK DECIDES NOTHING: a quote with no mark beside its words, a mark that '
            + 'matches two stands, and a mark that matches none',
          fn: async () => {
            expect([
              locateWords({ haystack: 'Fill the cats\' bowl. Then the cats nap.', words: 'the cats', },),
              locateWords({ haystack: 'Fill the cats\' bowl. Then the cats\' nap.', words: 'the cats\'', },),
              locateWords({ haystack: 'Fill the cats bowl. Then the cats nap.', words: 'the cats\'', },),
              locateWords({ haystack: 'Fill the cats\' bowl. Then the cats nap.', words: 'the cats.', },),
              locateWords({ haystack: 'Fill the cats\' bowl. Then the cats nap.', words: 'the cats \'', },),
              locateWords({ haystack: 'Fill the cats\' bowl. Then the cats nap.', words: '"the cats"', },),
            ],).toEqual([
              { kind: 'many', },
              { kind: 'many', },
              { kind: 'many', },
              { kind: 'many', },
              { kind: 'many', },
              { kind: 'many', },
            ],);
          },
        },),
        it({
          name: 'CHOOSES BETWEEN STANDS BY THE MARK EVIDENCE IT IS GIVEN, not by the marks of the words it is handed: folded '
            + 'words that hold no mark choose the stand with a mark on a side the evidence holds for',
          fn: async () => {
            expect([
              locateWords({
                haystack: 'Fill the cats\' bowl. Then the cats nap.',
                words: 'the cats',
                marks: { before: false, after: true, },
              },),
              locateWords({
                haystack: 'Then the cats nap. Fill the cats\' bowl.',
                words: 'the cats',
                marks: { before: false, after: true, },
              },),
              locateWords({
                haystack: '\'Tis the season. Tis the season.',
                words: 'Tis',
                marks: { before: true, after: false, },
              },),
              locateWords({
                haystack: 'Fill ‘the cats’ bowl. Then the cats nap. Then the cats’ nap.',
                words: 'the cats',
                marks: { before: true, after: true, },
              },),
            ],).toEqual([
              { kind: 'once', start: 5, end: 14, },
              { kind: 'once', start: 24, end: 33, },
              { kind: 'once', start: 0, end: 4, },
              { kind: 'once', start: 6, end: 14, },
            ],);
          },
        },),
        it({
          name: 'LETS THE MARK EVIDENCE OVERRIDE THE QUOTE\'S OWN READING, and keeps the quote\'s own reading when no evidence is '
            + 'given: evidence that holds for no side leaves a quote that held a mark ambiguous',
          fn: async () => {
            expect([
              locateWords({
                haystack: 'Fill the cats\' bowl. Then the cats nap.',
                words: 'the cats\'',
                marks: { before: false, after: false, },
              },),
              locateWords({
                haystack: 'Fill the cats\' bowl. Then the cats nap.',
                words: 'the cats\'',
                marks: { before: true, after: false, },
              },),
              locateWords({ haystack: 'Fill the cats\' bowl. Then the cats nap.', words: 'the cats\'', },),
              locateWords({ haystack: 'Fill the cats\' bowl. Then the cats nap.', words: 'the cats', },),
            ],).toEqual([
              { kind: 'many', },
              { kind: 'many', },
              { kind: 'once', start: 5, end: 14, },
              { kind: 'many', },
            ],);
          },
        },),
        it({
          name: 'USES THE MARK EVIDENCE ONLY WHERE THE WORDS STAND MORE THAN ONCE, a stand that is alone being the place '
            + 'whatever stands beside it, and says none when the evidence matches two stands or none',
          fn: async () => {
            expect([
              locateWords({
                haystack: 'Fill the cats bowl.',
                words: 'the cats',
                marks: { before: true, after: true, },
              },),
              locateWords({
                haystack: 'Fill the cats\' bowl. Then the cats\' nap.',
                words: 'the cats',
                marks: { before: false, after: true, },
              },),
              locateWords({
                haystack: 'Fill the cats bowl. Then the cats nap.',
                words: 'the cats',
                marks: { before: false, after: true, },
              },),
            ],).toEqual([
              { kind: 'once', start: 5, end: 13, },
              { kind: 'many', },
              { kind: 'many', },
            ],);
          },
        },),
        it({
          name: 'TAKES THE MARK THAT CHOSE THE STAND INTO THE SPAN ONLY WHEN IT IS A PROVEN APOSTROPHE, so replacing the span '
            + 'takes it off the page with the words, and keeps a mark that could pair with another',
          fn: async () => {
            const english = 'Fill the cats\' bowl. Then the cats nap.';
            expect(replacedIn({ haystack: english, words: 'the cats\'', text: 'the kittens', },),).toBe(
              'Fill the kittens bowl. Then the cats nap.',
            );
            expect(spanIn({ haystack: english, words: 'the cats\'', },),).toBe('the cats\'',);
            const enclosed = '‘Fill the cats’ bowl. Then the cats nap.';
            expect(replacedIn({ haystack: enclosed, words: 'the cats’', text: 'the kittens', },),).toBe(
              '‘Fill the kittens’ bowl. Then the cats nap.',
            );
            expect(spanIn({ haystack: enclosed, words: 'the cats’', },),).toBe('the cats',);
          },
        },),
        it({
          name: 'TAKES A PROVEN APOSTROPHE INTO THE SPAN OF TWO PLACES WHOSE ENGLISH DIFFERS ONLY BY A MARK BESIDE THEM, and '
            + 'leaves a quotation mark out: a plural possessive, a year, a quotation',
          fn: async () => {
            expect([
              spanIn({ haystack: 'Amid her parents the kitten hid.', words: 'her parents', },),
              spanIn({ haystack: 'Amid her parents’ scolding the kitten hid.', words: 'her parents', },),
              spanIn({ haystack: 'Cats of the 90s doze.', words: '90s', },),
              spanIn({ haystack: 'Cats of the ’90s nap.', words: '90s', },),
              spanIn({ haystack: 'She said, ‘The cat naps,’ and left.', words: 'The cat naps', },),
              spanIn({ haystack: 'The cat naps in the sun.', words: 'The cat naps', },),
            ],).toEqual(['her parents', 'her parents’', '90s', '’90s', 'The cat naps', 'The cat naps',],);
          },
        },),
        it({
          name: 'LEAVES EVERY QUOTATION MARK ON THE PAGE WHEN THE SPAN IS REPLACED, opening or closing, curly or straight, '
            + 'whatever stands later on the line: a dropped letter, a digit, a bracket, a dash, emphasis; the straight mark '
            + 'inside a code span is no quotation mark of the prose, so the apostrophe after it goes out with its words',
          fn: async () => {
            expect([
              replacedIn({ haystack: 'She said, ‘The cat naps,’ and left.', words: '‘The cat naps', text: 'The kitten dozes', },),
              replacedIn({ haystack: '‘Cats nap’, she said.', words: 'Cats nap’', text: 'Kittens doze', },),
              replacedIn({
                haystack: 'She said, \'The cat naps. It is warm.\'',
                words: '\'The cat naps.',
                text: 'The kitten dozes',
              },),
              replacedIn({ haystack: 'He said ‘Cats’ about the ol’ cat.', words: 'Cats', text: 'Kittens', },),
              replacedIn({ haystack: 'He said ‘Cats’ and purred ’n’ purred.', words: 'Cats', text: 'Kittens', },),
              replacedIn({ haystack: 'He said \'cats\' and then the dogs\' bowls were full.\'', words: 'cats', text: 'kittens', },),
              replacedIn({ haystack: 'He said, \'Round the basket,\' and left.', words: 'Round the basket', text: 'By the bowl', },),
              replacedIn({ haystack: 'He said \'Round the basket\' and left.', words: 'Round the basket', text: 'By the bowl', },),
              replacedIn({ haystack: 'He said, \'42 cats nap,\' and left.', words: '42 cats nap', text: '42 kittens doze', },),
              replacedIn({
                haystack: 'He said \'(quietly) the cats nap\' and left.',
                words: '(quietly) the cats nap',
                text: 'the kittens doze',
              },),
              replacedIn({ haystack: 'He said \'\u{2014}the cats nap\' and left.', words: 'the cats nap', text: 'the kittens doze', },),
              replacedIn({ haystack: 'He said \'*Cats* are nice\' and left.', words: '*Cats* are nice', text: 'Kittens are kind', },),
              replacedIn({ haystack: 'Run `\'x` and fill the cats’ bowl.', words: 'the cats', text: 'the kittens', },),
              replacedIn({ haystack: 'He said \'hello. The cats\' bowl is full.', words: 'The cats', text: 'The kittens', },),
            ],).toEqual([
              'She said, ‘The kitten dozes,’ and left.',
              '‘Kittens doze’, she said.',
              'She said, \'The kitten dozes. It is warm.\'',
              'He said ‘Kittens’ about the ol’ cat.',
              'He said ‘Kittens’ and purred ’n’ purred.',
              'He said \'kittens\' and then the dogs\' bowls were full.\'',
              'He said, \'By the bowl,\' and left.',
              'He said \'By the bowl\' and left.',
              'He said, \'42 kittens doze,\' and left.',
              'He said \'the kittens doze\' and left.',
              'He said \'\u{2014}the kittens doze\' and left.',
              'He said \'Kittens are kind\' and left.',
              'Run `\'x` and fill the kittens bowl.',
              'He said \'hello. The kittens\' bowl is full.',
            ],);
          },
        },),
        it({
          name: 'NAMES EXACTLY THE WORDS AND THE PROVEN APOSTROPHE BESIDE THEM, over every English built of some words with one '
            + 'of each mark on either side of them and one of each line ending after them: no other mark beside the words is '
            + 'ever inside the span, and a mark an independent reading proves an apostrophe always is',
          fn: async () => {
            const cores = ['the cats', 'we love goin', 'Round the basket', '42 cats nap', 'Cats', 'her parents',];
            const befores = ['', '‘', '’', '\'', '“', '(',];
            const afters = ['', '’', '\'', '”', ')', '’s',];
            const lefts = ['', 'He said ', 'Then: ',];
            const rights = ['.', ' bowl.', ' about the ol’ cat.', ' and purred ’n’ purred.', ', and the dogs’ bowls too.’',];
            /**
             Every English of the build whose span is not exactly its words, with what the span was.
             */
            const wrong = cores.flatMap(function ofCore(core,): readonly string[] {
              return befores.flatMap(function ofBefore(before,): readonly string[] {
                return afters.flatMap(function ofAfter(after,): readonly string[] {
                  return lefts.flatMap(function ofLeft(left,): readonly string[] {
                    return rights.flatMap(function ofRight(right,): readonly string[] {
                      const english = `${left}${before}${core}${after}${right}`;
                      const found = locateWords({ haystack: english, words: core, },);
                      const start = left.length + before.length;
                      // The independent reading of the fixture says which marks beside the words are proven apostrophes.
                      const proven = pageMarksOf({ text: english, },).filter(function isProven(mark,): boolean {
                        return mark.proven;
                      },);
                      const provenBefore = proven.some(function standsBefore({ at, },): boolean {
                        return at === (start - 1);
                      },);
                      const provenAfter = proven.some(function standsAfter({ at, },): boolean {
                        return at === (start + core.length);
                      },);
                      const named = (found.kind === 'once') && (found.start === (start - (provenBefore ? 1 : 0)))
                        && (found.end === (start + core.length + (provenAfter ? 1 : 0)));
                      // A WORD THAT GOES ON PAST THE WORDS IS NO STAND OF THEM: `Cats’s` holds no `Cats`.
                      const joined = after === '’s';
                      if (joined)
                        return (found.kind === 'none') ? [] : [`${english}: ${JSON.stringify(found,)}`,];
                      return named ? [] : [`${english}: ${JSON.stringify(found,)}`,];
                    },);
                  },);
                },);
              },);
            },);
            expect(wrong,).toEqual([],);
          },
        },),
        it({
          name: 'FINDS A WORD NOWHERE WHEN ITS EDGE IS A JOINT OF A LONGER WORD: a hyphen or an apostrophe between two letters, '
            + 'a full stop or a comma between two digits, at the start or the end of the match',
          fn: async () => {
            expect([
              locateWords({ haystack: 'The so-called cat', words: 'so', },),
              locateWords({ haystack: 'The so-called cat', words: 'called', },),
              locateWords({ haystack: 'The well-known cat', words: 'known', },),
              locateWords({ haystack: 'She don\'t nap', words: 't', },),
              locateWords({ haystack: 'The dog\'s nap', words: 'dog', },),
              locateWords({ haystack: 'The dog’s nap', words: 'dog', },),
              locateWords({ haystack: 'It cost 0.5 dollars', words: '5 dollars', },),
              locateWords({ haystack: 'It cost 0.5 dollars', words: '.5 dollars', },),
              locateWords({ haystack: 'It cost 0.5 dollars', words: 'It cost 0', },),
              locateWords({ haystack: 'It cost 1,500 dollars', words: '500 dollars', },),
              locateWords({ haystack: 'It cost 1,500 dollars', words: 'It cost 1', },),
            ],).toEqual([
              { kind: 'none', },
              { kind: 'none', },
              { kind: 'none', },
              { kind: 'none', },
              { kind: 'none', },
              { kind: 'none', },
              { kind: 'none', },
              { kind: 'none', },
              { kind: 'none', },
              { kind: 'none', },
              { kind: 'none', },
            ],);
          },
        },),
        it({
          name: 'READS EVERY HYPHEN AND APOSTROPHE AS A JOINT: the Unicode hyphen, the non-breaking hyphen, the soft hyphen '
            + 'and the modifier letter apostrophe, and a joint behind emphasis marks',
          fn: async () => {
            expect([
              locateWords({ haystack: 'The so\u{2010}called cat', words: 'so', },),
              locateWords({ haystack: 'The so\u{2011}called cat', words: 'so', },),
              locateWords({ haystack: 'The so\u{00AD}called cat', words: 'so', },),
              locateWords({ haystack: 'The so\u{2011}called cat', words: 'called', },),
              locateWords({ haystack: 'The dog\u{02BC}s nap', words: 'dog', },),
              locateWords({ haystack: 'A **so**-called nap', words: 'so', },),
              locateWords({ haystack: 'A *cat*-like nap', words: 'cat', },),
              locateWords({ haystack: 'A so-**called** nap', words: 'called', },),
              locateWords({ haystack: 'She liked *Catsong*’s tune', words: 'Catsong', },),
            ],).toEqual([
              { kind: 'none', },
              { kind: 'none', },
              { kind: 'none', },
              { kind: 'none', },
              { kind: 'none', },
              { kind: 'none', },
              { kind: 'none', },
              { kind: 'none', },
              { kind: 'none', },
            ],);
          },
        },),
        it({
          name: 'READS NO JOINT IN A CORNER BRACKET, A DASH BETWEEN CLAUSES OR A SLASH: the word beside each stands alone',
          fn: async () => {
            expect([
              locateWords({ haystack: 'He called it『cat』and left', words: 'cat', },),
              locateWords({ haystack: 'He called it『cat』and left', words: '『cat』', },),
              locateWords({ haystack: 'He called it「cat」and left', words: 'cat', },),
              locateWords({ haystack: 'The cat\u{2013}dog truce', words: 'cat', },),
              locateWords({ haystack: 'The cat/dog truce', words: 'cat', },),
              locateWords({ haystack: 'A **so** called nap', words: 'so', },),
            ],).toEqual([
              { kind: 'once', start: 13, end: 16, },
              { kind: 'once', start: 13, end: 16, },
              { kind: 'once', start: 13, end: 16, },
              { kind: 'once', start: 4, end: 7, },
              { kind: 'once', start: 4, end: 7, },
              { kind: 'once', start: 4, end: 6, },
            ],);
          },
        },),
        it({
          name: 'LOCATES A WORD WHOSE EDGE IS NO JOINT: the whole compound, a mark beside a space, a plural beside the '
            + 'apostrophe that closes it, and a neighbour that is Han, an emoji or another mark, where a Greek letter cuts',
          fn: async () => {
            expect([
              locateWords({ haystack: 'The so-called cat', words: 'so-called', },),
              locateWords({ haystack: 'The so - called cat', words: 'so', },),
              locateWords({ haystack: 'The cats\' bowl', words: 'cats', },),
              locateWords({ haystack: 'It cost 0.5 dollars', words: '0.5 dollars', },),
              locateWords({ haystack: 'It cost 0. 5 dollars', words: '5 dollars', },),
              locateWords({ haystack: 'catω naps', words: 'cat', },),
              locateWords({ haystack: 'cat猫 naps', words: 'cat', },),
              locateWords({ haystack: 'cat-猫 naps', words: 'cat', },),
              locateWords({ haystack: `${String.fromCodePoint(0x1_F6_00,)}cat naps`, words: 'cat', },),
              locateWords({ haystack: 'The cat-- naps', words: 'cat', },),
            ],).toEqual([
              { kind: 'once', start: 4, end: 13, },
              { kind: 'once', start: 4, end: 6, },
              { kind: 'once', start: 4, end: 9, },
              { kind: 'once', start: 8, end: 19, },
              { kind: 'once', start: 11, end: 20, },
              // CHANGED IN n159 (X9): a letter of any script that sets its words apart by spaces is a letter of a word, so
              // the Greek `ω` after `cat` makes `cat` half a word, which the span does not name.
              { kind: 'none', },
              { kind: 'once', start: 0, end: 3, },
              { kind: 'once', start: 0, end: 3, },
              { kind: 'once', start: 2, end: 5, },
              { kind: 'once', start: 4, end: 7, },
            ],);
          },
        },),
        it({
          name: 'FINDS A WORD NOWHERE WHEN IT STANDS INSIDE A WEB OR MAIL ADDRESS: in a link, an autolink, a domain, a '
            + 'mail address, at the start of the English and at its end',
          fn: async () => {
            expect([
              locateWords({ haystack: 'Visit https://cats.example.org/page now.', words: 'cats', },),
              locateWords({ haystack: 'Visit https://cats.example.org/page now.', words: 'example', },),
              locateWords({ haystack: 'Visit https://cats.example.org/page now.', words: 'page', },),
              locateWords({ haystack: 'Visit <https://cats.example.org/page> now.', words: 'cats', },),
              locateWords({ haystack: 'Mail cat@example.com now.', words: 'example', },),
              locateWords({ haystack: 'Mail cat@example.com now.', words: 'cat', },),
              locateWords({ haystack: 'See www.cats.example.org now.', words: 'cats', },),
              locateWords({ haystack: 'https://cats.example.org/page is here.', words: 'cats', },),
              locateWords({ haystack: 'Go to http://cats.example.org', words: 'cats', },),
              locateWords({ haystack: 'Go to http://cats.example.org', words: 'example', },),
            ],).toEqual([
              { kind: 'none', },
              { kind: 'none', },
              { kind: 'none', },
              { kind: 'none', },
              { kind: 'none', },
              { kind: 'none', },
              { kind: 'none', },
              { kind: 'none', },
              { kind: 'none', },
              { kind: 'none', },
            ],);
          },
        },),
        it({
          name: 'LOCATES A WORD BESIDE AN ADDRESS BUT NOT IN IT: the text of a link whose address repeats it, the words '
            + 'before an address that holds them, and a run that only looks like one',
          fn: async () => {
            expect([
              locateWords({ haystack: 'Visit [the cats](https://cats.example.org/page) now.', words: 'cats', },),
              locateWords({ haystack: 'Visit the cats at https://cats.example.org/page now.', words: 'cats', },),
              locateWords({ haystack: '[mail the cats](mailto:cats@home.org)', words: 'cats', },),
              locateWords({ haystack: 'Send e-mail to the cats.', words: 'cats', },),
              locateWords({ haystack: 'Write cats@home today.', words: 'cats', },),
              locateWords({ haystack: 'Write to the cats at home.org today.', words: 'home', },),
            ],).toEqual([
              { kind: 'once', start: 11, end: 15, },
              { kind: 'once', start: 10, end: 14, },
              { kind: 'once', start: 10, end: 14, },
              { kind: 'once', start: 19, end: 23, },
              { kind: 'once', start: 6, end: 10, },
              { kind: 'once', start: 21, end: 25, },
            ],);
          },
        },),
        it({
          name: 'TAKES A PROVEN APOSTROPHE BESIDE THE WORDS INTO THE SPAN, whether or not the quote copied it, at the end of '
            + 'the words and at the start of them, in each spelling of the mark',
          fn: async () => {
            expect([
              locateWords({ haystack: 'The parents’ preference.', words: 'The parents', },),
              locateWords({ haystack: 'The parents’ preference.', words: 'The parents’', },),
              locateWords({ haystack: 'The parents\' preference.', words: 'The parents', },),
              locateWords({ haystack: 'The parents\u{02BC} preference.', words: 'The parents', },),
              locateWords({ haystack: 'Guard her ’round the cats.', words: 'round the', },),
              locateWords({ haystack: 'Guard her ’round the cats.', words: '’round the', },),
              locateWords({ haystack: 'Fill the parents’', words: 'the parents', },),
            ],).toEqual([
              { kind: 'once', start: 0, end: 12, },
              { kind: 'once', start: 0, end: 12, },
              { kind: 'once', start: 0, end: 12, },
              { kind: 'once', start: 0, end: 12, },
              { kind: 'once', start: 10, end: 20, },
              { kind: 'once', start: 10, end: 20, },
              { kind: 'once', start: 5, end: 17, },
            ],);
          },
        },),
        it({
          name: 'KEEPS A MARK THAT COULD PAIR WITH ANOTHER OUTSIDE THE SPAN: the closing mark of a quotation of one word, the '
            + 'opening curly mark, a mark after an opener in the same paragraph or in an earlier one, and a mark whose '
            + 'partner is in the same paragraph after a line break only',
          fn: async () => {
            expect([
              locateWords({ haystack: '‘Mio’ is here.', words: 'Mio', },),
              locateWords({ haystack: 'The parents‘ preference.', words: 'The parents', },),
              locateWords({ haystack: 'She said ‘look at the parents’ bowl’ twice.', words: 'the parents', },),
              locateWords({ haystack: '‘Mio is here.\nThe parents’ preference.', words: 'The parents', },),
              locateWords({ haystack: 'Guard her ’round the cats’ bowl.', words: 'round the', },),
              locateWords({ haystack: '‘Mio is here.\n\nThe parents’ preference.', words: 'The parents', },),
            ],).toEqual([
              { kind: 'once', start: 1, end: 4, },
              { kind: 'once', start: 0, end: 11, },
              { kind: 'once', start: 18, end: 29, },
              { kind: 'once', start: 14, end: 25, },
              { kind: 'once', start: 11, end: 20, },
              { kind: 'once', start: 15, end: 26, },
            ],);
          },
        },),
        it({
          name: 'FINDS NO STAND INSIDE A WORD THAT AN APOSTROPHE AFTER MARKUP JOINS: emphasis that ends in the word and a '
            + 'link before the apostrophe, and still locates the same words with a space after the emphasis',
          fn: async () => {
            expect([
              locateWords({ haystack: 'Fill the **cat**\'s bowl.', words: 'the **cat**', },),
              locateWords({ haystack: 'Fill [the cat](x)’s bowl.', words: '[the cat](x)', },),
              locateWords({ haystack: 'Fill the **cat** bowl.', words: 'the **cat**', },),
            ],).toEqual([
              { kind: 'none', },
              { kind: 'none', },
              { kind: 'once', start: 5, end: 16, },
            ],);
          },
        },),
        it({
          name: 'LEAVES THE SPAN AT THE WORDS WHEN THE CALLER ASKS FOR THE APOSTROPHES OUTSIDE IT, as the slot of a frame is '
            + 'found inside a span that already holds its apostrophe',
          fn: async () => {
            expect([
              locateWords({ haystack: 'the parents’ bowl', words: 'parents', },),
              locateWords({ haystack: 'the parents’ bowl', words: 'parents', apostrophes: 'outside', },),
              locateWords({ haystack: 'the ’round cats', words: 'round', apostrophes: 'outside', },),
            ],).toEqual([
              { kind: 'once', start: 4, end: 12, },
              { kind: 'once', start: 4, end: 11, },
              { kind: 'once', start: 5, end: 10, },
            ],);
          },
        },),
        it({
          name: 'PROVES AN APOSTROPHE OVER THE PAGE THE CALLER NAMES: a closing mark whose opener stands in an earlier slice is '
            + 'no apostrophe of the words, the same slice with no opener beside it gives the mark to the span, and a caller '
            + 'that names no page gets the slice as the page',
          fn: async () => {
            /**
             The slice the words stand in.
             */
            const haystack = 'Then feed the kittens’.';
            /**
             The page with an opener in an earlier slice, and the same page without it.
             */
            const opened = `‘Fill the bowls.\n\n${haystack}`;
            const plain = `Fill the bowls.\n\n${haystack}`;
            expect([
              locateWords({ haystack, words: 'the kittens', page: { text: opened, offset: 18, }, },),
              locateWords({ haystack, words: 'the kittens', page: { text: plain, offset: 17, }, },),
              locateWords({ haystack, words: 'the kittens', },),
            ],).toEqual([
              { kind: 'once', start: 10, end: 21, },
              { kind: 'once', start: 10, end: 22, },
              { kind: 'once', start: 10, end: 22, },
            ],);
          },
        },),
        it({
          name: 'CHOOSES BETWEEN STANDS BY THE MARK EVIDENCE AS BEFORE, a proven apostrophe being a single mark beside the words '
            + 'like any other: none leaves two stands ambiguous, a mark after the words chooses the stand that holds one and '
            + 'the span is widened over it, and a mark beside both stands leaves them ambiguous',
          fn: async () => {
            const one = 'Fill the parents’ bowl. Then the parents nap.';
            const both = 'Fill the parents’ bowl. Then the parents’ nap.';
            expect([
              locateWords({ haystack: one, words: 'the parents', },),
              locateWords({ haystack: one, words: 'the parents', marks: { before: false, after: true, }, },),
              locateWords({ haystack: one, words: 'the parents’', },),
              locateWords({ haystack: both, words: 'the parents’', },),
              locateWords({ haystack: one, words: 'the parents', marks: { before: true, after: false, }, },),
            ],).toEqual([
              { kind: 'many', },
              { kind: 'once', start: 5, end: 17, },
              { kind: 'once', start: 5, end: 17, },
              { kind: 'many', },
              { kind: 'many', },
            ],);
          },
        },),
        it({
          name: 'LOCATES A QUOTE THAT HOLDS A CHARACTER BEYOND THE FIRST PLANE, offsets counted in the units the English is cut in',
          fn: async () => {
            const cat = String.fromCodePoint(0x1_F6_3A,);
            const english = `${cat} The kitten naps ${cat}.`;
            const found = locateWords({ haystack: english, words: `The kitten naps ${cat}`, },);
            expect(found,).toEqual({ kind: 'once', start: 3, end: english.length - 1, },);
          },
        },),
        it({
          name: 'TAKES A SPAN WHOLE WHEN THE QUOTE BEGINS OR ENDS INSIDE A REFERENCE OR AN ESCAPE, so a replacement never writes half of '
            + 'one: the unit is taken whole, then kept or cut at the edge as the character it shows would be (a letter or a sign stays, '
            + 'a stop, a space or a single mark goes with all its characters)',
          fn: async () => {
            expect([
              locateWords({ haystack: 'See dogs&amp; cats now.', words: 'amp; cats', },),
              locateWords({ haystack: 'See dogs&#39; cats now.', words: '39; cats', },),
              locateWords({ haystack: String.raw`He said \*kittens* now.`, words: '*kittens*', },),
              locateWords({ haystack: 'He said dogs&amp; cats now.', words: 'dogs&amp', },),
              locateWords({ haystack: String.raw`He said the cats\) was here.`, words: 'the cats\\', },),
            ],).toEqual([
              // The whole `&amp;` stands beside the `s` of `dogs`, so a span taken to its edge cuts the word `dogs&amp;` (n156, W2).
              { kind: 'none', },
              { kind: 'once', start: 14, end: 18, },
              { kind: 'once', start: 8, end: 18, },
              { kind: 'once', start: 8, end: 17, },
              { kind: 'once', start: 8, end: 16, },
            ],);
          },
        },),
        it({
          name: 'LOCATES THE WORDS A QUOTE NAMES WITHOUT THE ESCAPED OR REFERRED STOP, bracket or quotation mark at its end, and ends the '
            + 'span before the unit',
          fn: async () => {
            expect([
              locateWords({ haystack: String.raw`He said (the cats\) was here.`, words: String.raw`the cats\)`, },),
              locateWords({ haystack: String.raw`See kittens\. Then more.`, words: String.raw`kittens\.`, },),
              locateWords({ haystack: 'See Q&A; then more.', words: 'Q&A', },),
              locateWords({ haystack: 'See Q&A; then more.', words: 'Q&A;', },),
              locateWords({ haystack: 'The dogs &amp; bowl.', words: 'dogs &amp;', },),
            ],).toEqual([
              { kind: 'once', start: 9, end: 17, },
              { kind: 'once', start: 4, end: 11, },
              { kind: 'once', start: 4, end: 7, },
              { kind: 'once', start: 4, end: 7, },
              { kind: 'once', start: 4, end: 14, },
            ],);
          },
        },),
      ],
    },),
    describe({
      name: standsOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'COUNTS EVERY STAND OF THE WORDS AT WORD BOUNDARIES, overlaps included, in the fold locateWords reads',
          fn: async () => {
            expect([
              standsOf({ haystack: 'The cat naps. The cat naps.', words: 'The cat naps', },),
              standsOf({ haystack: 'The cat naps.\n\nThe  cat\nnaps.', words: 'The cat naps', },),
              standsOf({ haystack: 'aa aa aa', words: 'aa aa', },),
              standsOf({ haystack: '猫猫猫', words: '猫猫', },),
              standsOf({ haystack: 'She said, “The cat naps.”', words: '"The cat naps"', },),
            ],).toEqual([2, 2, 2, 2, 1,],);
          },
        },),
        it({
          name: 'COUNTS NO STAND INSIDE A LONGER WORD AND NONE FOR WORDS THAT SHOW A READER NOTHING',
          fn: async () => {
            expect([
              standsOf({ haystack: 'She was also kind.', words: 'so', },),
              standsOf({ haystack: 'A smart cat.', words: 'art', },),
              standsOf({ haystack: 'The cat naps.', words: '  ', },),
              standsOf({ haystack: 'The cat naps.', words: '', },),
              standsOf({ haystack: '', words: 'cat', },),
            ],).toEqual([0, 0, 0, 0, 0,],);
          },
        },),
        it({
          name: 'COUNTS NO STAND INSIDE A WEB OR MAIL ADDRESS, as locateWords finds none there, and counts the stand beside it',
          fn: async () => {
            expect([
              standsOf({ haystack: 'Visit the cats at https://cats.example.org/page now.', words: 'cats', },),
              standsOf({ haystack: 'Visit [the cats](https://cats.example.org/page) now.', words: 'cats', },),
              standsOf({ haystack: 'Mail cat@example.com now.', words: 'cat', },),
              standsOf({ haystack: 'Write cats@home today.', words: 'cats', },),
            ],).toEqual([1, 1, 0, 1,],);
          },
        },),
        it({
          name: 'COUNTS NO STAND AT A JOINT OF A LONGER WORD, as locateWords finds none there, and counts the stands beside it',
          fn: async () => {
            expect([
              standsOf({ haystack: 'The so-called cat is so kind.', words: 'so', },),
              standsOf({ haystack: 'She don\'t nap and the dog\'s nap.', words: 'nap', },),
              standsOf({ haystack: 'She don\'t nap and the dog\'s nap.', words: 'dog', },),
              standsOf({ haystack: 'It cost 0.5 dollars and 5 dollars.', words: '5 dollars', },),
            ],).toEqual([1, 2, 0, 1,],);
          },
        },),
      ],
    },),
    describe({
      name: marksHeldBesideWords.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'SAYS ON WHICH SIDE A QUOTE HELD A SINGLE MARK DIRECTLY BESIDE ITS WORDS, straight or curly, and on neither '
            + 'when a space or a line break stands between, a double mark stands there, or no mark does',
          fn: async () => {
            expect([
              marksHeldBesideWords({ text: 'the cats\'', },),
              marksHeldBesideWords({ text: '\'Tis', },),
              marksHeldBesideWords({ text: '’Tis', },),
              marksHeldBesideWords({ text: '‘the cats’', },),
              marksHeldBesideWords({ text: 'the cats\u{02BC}', },),
              marksHeldBesideWords({ text: 'the cats\'.', },),
              marksHeldBesideWords({ text: 'the cats', },),
              marksHeldBesideWords({ text: 'the cats \'', },),
              marksHeldBesideWords({ text: '\'\nthe cats', },),
              marksHeldBesideWords({ text: '"the cats"', },),
            ],).toEqual([
              { before: false, after: true, },
              { before: true, after: false, },
              { before: true, after: false, },
              { before: true, after: true, },
              { before: false, after: true, },
              { before: false, after: true, },
              { before: false, after: false, },
              { before: false, after: false, },
              { before: false, after: false, },
              { before: false, after: false, },
            ],);
          },
        },),
      ],
    },),
  ],
},);
