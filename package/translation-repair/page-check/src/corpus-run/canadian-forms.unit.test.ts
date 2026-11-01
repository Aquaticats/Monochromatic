/**
 Guards class one hundred thirty-four (hulicaijia19, 2026-09-25): the page is
 Canadian English (class one hundred thirty-two), yet it carried "On 4 May"
 and "29th April" beside "March 13", and "liquorice" where Canadian writes
 "licorice", because the archive's day-first dates and British spellings
 stood on slices no lane rewrote. The page-assembly pass writes every date
 month first and respells a closed list of words, on every slice, outside
 markup, links, code, comments and the sealed English original.

 Cat-themed invention throughout; no corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  canadianizePage,
  canadianizeText,
} from '../../dist/final/node/index.mjs';
import { pairAt, } from '@monochromatic-dev/translation-repair-slice-reading/ts/corpus-run/archive-slice-pair.test-fixture.ts';

/**
 Rewrites one text whose original carries no English, and returns only the text.

 @param text - text to rewrite

 @returns Rewritten text

 @example
 ```ts
 rewritten({ text: 'On 4 May the cat napped.', },); // 'On May 4 the cat napped.'
 ```
 */
function rewritten(
  { text, }: { readonly text: string; },
): string {
  return canadianizeText({
    text,
    source: '她打盹。',
  },).text;
}

await describe({
  name: 'canadianizeText and canadianizePage (class one hundred thirty-four)',
  children: [
    it({
      name: 'WRITES a day-first date month first and leaves the one to three digits after its month as they stand',
      fn: async () => {
        expect([
          rewritten({ text: 'On 4 May 12 cats napped.', },),
          rewritten({ text: 'On 4 May 123 cats napped.', },),
        ],).toEqual([
          'On May 4 12 cats napped.',
          'On May 4 123 cats napped.',
        ],);
      },
    },),
    it({
      name: 'WRITES day-first dates month first, with a comma before a year and after it where the sentence '
        + 'goes on (ledger K1: the Language Portal of Canada sets the year off on both sides)',
      fn: async () => {
        expect([
          rewritten({ text: 'On 4th May, the cat napped.', },),
          rewritten({ text: '29th April was the cat\'s birthday.', },),
          rewritten({ text: 'The cat was born on 13 March 2024 in a box.', },),
          rewritten({ text: 'On 4 May 2024 at 22:55 the cat napped.', },),
          rewritten({ text: 'The cat was born on 13 March 2024.', },),
        ],).toEqual([
          'On May 4, the cat napped.',
          'April 29 was the cat\'s birthday.',
          'The cat was born on March 13, 2024, in a box.',
          'On May 4, 2024, at 22:55 the cat napped.',
          'The cat was born on March 13, 2024.',
        ],);
      },
    },),
    it({
      name: 'LEAVES month-first dates with nothing to change, numbers that are no day, and words that only look '
        + 'like months or start a name (ledger K8)',
      fn: async () => {
        /**
         Texts that must come back unchanged.
         */
        const unchanged = [
          'On March 13 the cat napped.',
          'The cat ate 40 May beetles.',
          'The cat may 4 times a day nap.',
          'The cat napped on 32 March, which is no date.',
          'The cat chased 5 May beetles.',
          'She read 3 April Fools’ jokes to the cat.',
          'The cat met 2 June Carter fans.',
          'The cat was born 4 May2024.',
          'Born 4 May 2024a.',
          'On the 5th, the cat napped.',
        ];
        expect(unchanged.map(function rewrite(text,): string {
          return rewritten({ text, },);
        },),).toEqual(unchanged,);
      },
    },),
    it({
      name: 'WRITES A RANGE ONCE, month first, whichever end carries the month, across a line break too, and '
        + 'dates each end of a range whose ends both carry one (ledger K2)',
      fn: async () => {
        expect([
          rewritten({ text: 'From 1st to 3rd June the cat slept.', },),
          rewritten({ text: 'The cat slept from 1st\nto 3rd June.', },),
          rewritten({ text: 'The cat slept from 1st and\n3rd June.', },),
          rewritten({ text: 'The cat slept from 3 June to 5 July.', },),
          rewritten({ text: 'The cat napped on 5 June and 6 June.', },),
          rewritten({ text: 'The cat napped 2 June–3 July.', },),
        ],).toEqual([
          'From June 1 to 3 the cat slept.',
          'The cat slept from June 1\nto 3.',
          'The cat slept from June 1 and\n3.',
          'The cat slept from June 3 to July 5.',
          'The cat napped on June 5 and June 6.',
          'The cat napped June 2–July 3.',
        ],);
      },
    },),
    it({
      name: 'WRITES A DATE AFTER A WORD THAT ONLY LOOKS LIKE A RANGE, and one a list marker or a dash opens '
        + '(ledger K2)',
      fn: async () => {
        expect([
          rewritten({ text: 'The cat stayed until 4 May.', },),
          rewritten({ text: 'The shelter was closed till 4 May.', },),
          rewritten({ text: 'The vet visit moved to 4 May.', },),
          rewritten({ text: '- 4 May: the cat napped.', },),
          rewritten({ text: 'The cat napped—4 May, to be exact.', },),
          rewritten({ text: 'On\u00A04 May the cat napped.', },),
        ],).toEqual([
          'The cat stayed until May 4.',
          'The shelter was closed till May 4.',
          'The vet visit moved to May 4.',
          '- May 4: the cat napped.',
          'The cat napped—May 4, to be exact.',
          'On\u00A0May 4 the cat napped.',
        ],);
      },
    },),
    it({
      name: 'DROPS THE ARTICLE AND "OF" of a day-first date, and a month-first ordinal\'s suffix (ledger K4, K8)',
      fn: async () => {
        expect([
          rewritten({ text: 'It happened on the 4th May.', },),
          rewritten({ text: 'The 29th April was her birthday.', },),
          rewritten({ text: 'On the 4th of May the cat napped.', },),
          rewritten({ text: 'The cat was found on December 29th.', },),
          rewritten({ text: 'On May 14th, 2023 the cat was taken home.', },),
          rewritten({ text: 'The cat slept from May 11th to 13th.', },),
        ],).toEqual([
          'It happened on May 4.',
          'April 29 was her birthday.',
          'On May 4 the cat napped.',
          'The cat was found on December 29.',
          'On May 14, 2023, the cat was taken home.',
          'The cat slept from May 11 to 13.',
        ],);
      },
    },),
    it({
      name: 'WRITES YEAR-FIRST DATES AND ABBREVIATED MONTHS month first in full (ledger K4), and closes a year '
        + 'the page already wrote month first',
      fn: async () => {
        expect([
          rewritten({ text: 'On 2021 July 20th, the cat napped.', },),
          rewritten({ text: '> @whiskers 2023 Feb 25th:', },),
          rewritten({ text: 'The cat was archived at 2023, 31 Mar.', },),
          rewritten({ text: 'On 4 Sept 2024 the cat napped.', },),
          rewritten({ text: 'On December 21, 2023 the cat left.', },),
        ],).toEqual([
          'On July 20, 2021, the cat napped.',
          '> @whiskers February 25, 2023:',
          'The cat was archived at March 31, 2023.',
          'On September 4, 2024, the cat napped.',
          'On December 21, 2023, the cat left.',
        ],);
      },
    },),
    it({
      name: 'READS EACH SHAPE\'S EDGES: a month and year with no day, a period that ends the sentence or only '
        + 'the abbreviation, a pair, a hyphen after a date, a bare day\'s article, "I" after the month, and a day right after a tag',
      fn: async () => {
        expect([
          rewritten({ text: 'The cats were adopted between Nov 2023 and Feb. 2024.', },),
          rewritten({ text: 'Aug. 8th, 2018 was hot.', },),
          rewritten({ text: 'On 4 Sept. the cat napped.', },),
          rewritten({ text: 'The shelter closed on 31 Mar. The cat left.', },),
          rewritten({ text: 'The cat napped on the 4th or 5th May.', },),
          rewritten({ text: 'The cat napped 2 June-3 July.', },),
          rewritten({ text: 'The cat slept from May 30th to 2nd June.', },),
          rewritten({ text: 'The cat missed the 4 May deadline.', },),
          rewritten({ text: 'On 4 May I fed the cat.', },),
          rewritten({ text: '😺4 May', },),
          rewritten({ text: 'The cat napped on <b>4 May</b>.', },),
        ],).toEqual([
          'The cats were adopted between November 2023 and February 2024.',
          'August 8, 2018, was hot.',
          'On September 4 the cat napped.',
          'The shelter closed on March 31. The cat left.',
          'The cat napped on May 4 or 5.',
          'The cat napped June 2-July 3.',
          'The cat slept from May 30 to June 2.',
          'The cat missed the May 4 deadline.',
          'On May 4 I fed the cat.',
          '😺May 4',
          'The cat napped on <b>May 4</b>.',
        ],);
      },
    },),
    it({
      name: 'LEAVES a comma-joined year whose day-first date goes on, a year-first date with no suffix that goes '
        + 'on, a hyphen inside a longer token, and an "of" after a bare day',
      fn: async () => {
        /**
         Texts whose dates are read as no year-first date, or as no date at all.
         */
        const kept = [
          'In 2021 May 4 was a Tuesday for the cat.',
          'The cat read COVID-19 May updates.',
          'The cat fed 3 of June’s kittens.',
        ];
        expect([
          rewritten({ text: 'In 2020, 4 May was a holiday for the cat.', },),
          ...kept.map(function rewrite(text,): string {
            return rewritten({ text, },);
          },),
        ],).toEqual([
          'In 2020, May 4 was a holiday for the cat.',
          ...kept,
        ],);
      },
    },),
    it({
      name: 'RESPELLS LISTED WORDS in lower case mid-sentence, and leaves a capital mid-sentence or after a title, '
        + 'which names someone',
      fn: async () => {
        expect([
          rewritten({ text: 'The cat licked compound liquorice tablets.', },),
          rewritten({ text: 'Her favorite color was gray; she realised it at the center.', },),
          rewritten({ text: 'The cat visited the Lincoln Center and Mr. Gray.', },),
        ],).toEqual([
          'The cat licked compound licorice tablets.',
          'Her favourite colour was grey; she realized it at the centre.',
          'The cat visited the Lincoln Center and Mr. Gray.',
        ],);
      },
    },),
    it({
      name: 'WRITES the handle word id as ID and leaves an id attribute and longer words (class one hundred sixty-nine)',
      fn: async () => {
        expect([
          rewritten({ text: 'The kitten used this id on every site.', },),
          rewritten({ text: '<Paw id="left" /> The kitten was idle.', },),
        ],).toEqual([
          'The kitten used this ID on every site.',
          '<Paw id="left" /> The kitten was idle.',
        ],);
      },
    },),
    it({
      name: 'READS a word written with a combining accent as its composed spelling is read (ledger B18): a month '
        + 'with a mark after it is no month, a compound, an initial, a heading\'s small word and an identifier '
        + 'keep their marks, and a year or an underscore with a mark on it reads as the composed forms do',
      fn: async () => {
        /**
         Each text written with combining marks.
         */
        const combining = [
          'We napped on 4 May\u{0301}.',
          'On 2 May bug\u{0301}s swarmed.',
          'She met E\u{0301}. Color at noon.',
          '## The Color of a\u{0300} Cat\n\nShe napped.',
          'Set cafe\u{0301}_color now.',
          'On 4 May 2024\u{0301} she napped.',
          'Set color_\u{0301} now.',
        ];
        /**
         Each rewritten.
         */
        const outputs = combining.map(function rewrite(text,): string {
          return rewritten({ text, },);
        },);
        expect(outputs,).toEqual([
          'We napped on 4 May\u{0301}.',
          'On May 2 bug\u{0301}s swarmed.',
          'She met E\u{0301}. Color at noon.',
          '## The Color of a\u{0300} Cat\n\nShe napped.',
          'Set cafe\u{0301}_color now.',
          'On 4 May 2024\u{0301} she napped.',
          'Set colour_\u{0301} now.',
        ],);
        expect(outputs.map(function composed(text,): string {
          return text.normalize('NFC',);
        },),).toEqual(combining.map(function composedFirst(text,): string {
          return rewritten({ text: text.normalize('NFC',), },).normalize('NFC',);
        },),);
      },
    },),
    it({
      name: 'READS a cased letter beyond the first plane whole, as a Latin letter is read (ledger B21): a date or a '
        + 'listed word it touches is part of a longer token, while an emoji before a date still opens it',
      fn: async () => {
        /**
         Dates and a listed word glued to a Deseret capital, which has case.
         */
        const glued = [
          'We napped on 4 May\u{10414}.',
          'We napped \u{10414}4 May.',
          'The cat ate \u{10414}liquorice.',
          'The cat ate liquorice\u{10414}.',
        ];
        /**
         The same texts with a Latin letter in its place.
         */
        const latin = glued.map(function asLatin(text,): string {
          return text.replaceAll('\u{10414}', 'x',);
        },);
        expect(latin.map(function rewrite(text,): string {
          return rewritten({ text, },);
        },),).toEqual(latin,);
        expect(glued.map(function rewrite(text,): string {
          return rewritten({ text, },);
        },),).toEqual(glued,);
        expect(rewritten({ text: 'We napped \u{1F431}4 May.', },),).toBe('We napped \u{1F431}May 4.',);
      },
    },),
    it({
      name: 'REWRITES A TEXT WITH DESERET LETTERS AS IT REWRITES THE SAME TEXT WITH LATIN ONES (ledger B21), at '
        + 'every neighbour the date, spelling and capital readers test: word runs read back, a sentence\'s last '
        + 'word, a heading\'s capitals, a name\'s second word, emphasis underscores, a year\'s edges and the word '
        + 'after it, a day\'s suffix, an abbreviation\'s period, an article, a month-first date, and a path',
      fn: async () => {
        /**
         Texts whose outcome turns on a neighbour of a date or a listed word,
         written with a Deseret capital and small letter, which have case.
         The prose carries no Latin x, which stands in for them.
         */
        const deseret = [
          'The cat saw \u{10414}my mum.',
          'She met \u{10414}J. Color was calm.',
          '## The Color of \u{10414}\u{1043C}\u{1043C}',
          'Center \u{10414}\u{1043C} hosted the cats.',
          'Nap \u{10414}_\u{10414} time. Color was calm.',
          'We napped 2023, 31 Mar.\u{10414} then.',
          'On 4 May 2024 \u{10414}\u{1043C} napped.',
          'We napped on 4 May 2024\u{10414}.',
          'We napped on May 4th\u{10414} then.',
          'We napped on May 3rd-4th\u{10414}.',
          'We napped on 4 Sept.\u{10414} then.',
          'We napped on 2 June \u{10414}\u{1043C} then.',
          'We napped on 31 Mar. \u{1043C}\u{1043C} then.',
          'We napped on \u{10414}the 4th May.',
          'We napped \u{10414}Sept 4 then.',
          'See color/cat.\u{10414}\u{1043C} now.',
          'Set \u{10414}_color now.',
          'Set color_\u{10414} now.',
        ];
        expect(deseret.map(function rewrite(text,): string {
          return rewritten({ text, },);
        },),).toEqual(deseret.map(function throughLatin(text,): string {
          return rewritten({
            text: text
              .replaceAll('\u{10414}', 'X',)
              .replaceAll('\u{1043C}', 'x',),
          },)
            .replaceAll('X', '\u{10414}',)
            .replaceAll('x', '\u{1043C}',);
        },),);
        /**
         The same texts in bold script letters, which the corpus writes
         handles in: cased letters by general category with no case mapping,
         as `quote-neighbours.ts` reads them (class ninety-six).
         */
        const script = deseret.map(function asScript(text,): string {
          return text
            .replaceAll('\u{10414}', '\u{1D4E7}',)
            .replaceAll('\u{1043C}', '\u{1D501}',);
        },);
        expect(script.map(function rewrite(text,): string {
          return rewritten({ text, },);
        },),).toEqual(script.map(function throughLatin(text,): string {
          return rewritten({
            text: text
              .replaceAll('\u{1D4E7}', 'X',)
              .replaceAll('\u{1D501}', 'x',),
          },)
            .replaceAll('X', '\u{1D4E7}',)
            .replaceAll('x', '\u{1D501}',);
        },),);
      },
    },),
    it({
      name: 'LEAVES tag attributes, a link destination, code, a comment and a capitalised title in emphasis '
        + 'untouched, and rewrites the prose between them',
      fn: async () => {
        /**
         Text whose every rewritable word sits where the page's form is not prose.
         */
        const text = [
          '<p style="text-align: center;">(The cat on 4 May)</p>',
          '[a favorite](https://example.com/color/4-May) and `color` and *The Color of Cats*',
          '<!-- 4 May: gray -->',
        ].join('\n',);
        expect(rewritten({ text, },),).toEqual([
          '<p style="text-align: center;">(The cat on May 4)</p>',
          '[a favourite](https://example.com/color/4-May) and `color` and *The Color of Cats*',
          '<!-- 4 May: gray -->',
        ].join('\n',),);
      },
    },),
    it({
      name: 'LEAVES the attributes of a tag wherever the MDX compiler opens one (ledger B18): a name outside ASCII '
        + 'and a name opening on a dollar sign',
      fn: async () => {
        /**
         Texts each holding one such tag.
         */
        const texts = [
          'She drew <猫 title="color" /> today.',
          'She drew <$Paw title="color" /> today.',
        ];
        expect(texts.map(function rewrite(text,): string {
          return rewritten({ text, },);
        },),).toEqual(texts,);
      },
    },),
    it({
      name: 'REWRITES a slice no lane replaced, and leaves a slice inside the sealed English original',
      fn: async () => {
        /**
         Three archive slices: one untouched, one replaced, one sealed.
         */
        const slices = [
          pairAt({ sliceIndex: 0, target: '29th April was the cat\'s birthday.', startOffset: 0, },),
          pairAt({ sliceIndex: 1, target: 'The cat napped.', startOffset: 40, },),
          pairAt({ sliceIndex: 2, target: 'On 4 May the cat wrote this in English.', startOffset: 80, },),
        ];
        /**
         Pass over the page with slice 1 replaced and slice 2 sealed.
         */
        const page = canadianizePage({
          slices,
          replacements: [{ sliceIndex: 1, replacementText: 'On 4 May the cat napped in the liquorice.', },],
          archiveOriginalSpans: [{ startOffset: 80, endOffset: 200, note: 'The cat wrote this in English.', },],
        },);
        expect({
          rows: page.replacements.map(function textOf(row,): string {
            return `${String(row.sliceIndex,)}: ${row.replacementText}`;
          },)
            .toSorted(),
          restored: page.restored.map(function indexOf(row,): number {
            return row.sliceIndex;
          },)
            .toSorted(function ascending(left, right,): number {
              return left - right;
            },),
        },).toEqual({
          rows: [
            '0: April 29 was the cat\'s birthday.',
            '1: On May 4 the cat napped in the licorice.',
          ],
          restored: [0, 1,],
        },);
      },
    },),

    it({
      name: 'LEAVES a month-first date whose year the read refuses, ordinal suffix and all, and REWRITES a '
        + 'year-first date month first, closing its year with a comma only where the sentence runs on',
      fn: async () => {
        // The suffix is what the refusal keeps: read with that year, the date
        // would lose its "rd" and keep the five-digit year beside it.
        expect([
          rewritten({ text: 'May 3rd 20233 by the window, the cat napped.', },),
          rewritten({ text: 'The cat was born on 2026 May 3rd in a box.', },),
          rewritten({ text: 'The cat was born on 2026 May 3rd.', },),
        ],).toEqual([
          'May 3rd 20233 by the window, the cat napped.',
          'The cat was born on May 3, 2026, in a box.',
          'The cat was born on May 3, 2026.',
        ],);
      },
    },),
  ],
},);
