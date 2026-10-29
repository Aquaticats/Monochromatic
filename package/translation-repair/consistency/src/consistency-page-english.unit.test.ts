/**
 Tests for joining the English of the slices of a page into the English the proof of an apostrophe reads: the slices
 that carry English, in page order, a blank line between each two, the offset where each slice starts, and the front
 matter at the start of the page blanked.

 Fixtures are cat-themed invention; no corpus content appears here.

 @module
 */

import {
  caught,
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { spliceSlices, } from '@monochromatic-dev/translation-repair-page-pass';
import {
  NO_PLACE,
  PAGE_SLICE_SEPARATOR,
  type PageLayout,
  pageOfSlices,
  pageReadingOf,
  type PageSlice,
  positionOnPage,
} from '../dist/final/node/index.mjs';
import { anchorAt, } from '@monochromatic-dev/translation-repair-slice-reading/ts/insertion-anchor-pair.test-fixture.ts';
import { pairsOver, } from '@monochromatic-dev/translation-repair-slice-reading/ts/span-pairs.test-fixture.ts';

/**
 The slices of a page from their English, indexed from zero in the order given.

 @param englishes - English of each slice that carries English, in page order

 @returns Slices

 @example
 ```ts
 const slices = slicesOf({ englishes: ['## One', 'Then nap.',], },); // indexes 0 and 1
 ```
 */
function slicesOf({ englishes, }: { readonly englishes: readonly string[]; },): readonly PageSlice[] {
  return englishes.map(function sliceOf(
    english,
    sliceIndex,
  ): PageSlice {
    return {
      sliceIndex,
      english,
    };
  },);
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: pageOfSlices.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'JOINS THE ENGLISH OF THE SLICES IN ORDER WITH A BLANK LINE BETWEEN EACH TWO, and names where each starts by '
            + 'the slice\'s own index, which a slice that carries no English does not take',
          fn: async () => {
            const { text, offsets, } = pageOfSlices({
              slices: [
                { sliceIndex: 0, english: '## One\n\nFill the bowl.', },
                { sliceIndex: 2, english: 'Then nap.', },
                { sliceIndex: 3, english: '', },
                { sliceIndex: 5, english: 'The end.', },
              ],
            },);
            expect({ separator: PAGE_SLICE_SEPARATOR, text, offsets: [...offsets,], },).toEqual({
              separator: '\n\n',
              text: '## One\n\nFill the bowl.\n\nThen nap.\n\n\n\nThe end.',
              offsets: [[0, 0,], [2, 24,], [3, 35,], [5, 37,],],
            },);
            expect([text.slice(24, 33,), text.slice(37,),],).toEqual(['Then nap.', 'The end.',],);
          },
        },),
        it({
          name: 'READS A PAGE OF NO SLICE, AND A SLICE ALONE, as the empty English and as that slice',
          fn: async () => {
            expect([
              pageOfSlices({ slices: [], },),
              pageOfSlices({ slices: slicesOf({ englishes: ['Fill the bowl.',], },), },),
            ].map(function plain({ text, offsets, },): unknown {
              return { text, offsets: [...offsets,], };
            },),).toEqual([
              { text: '', offsets: [], },
              { text: 'Fill the bowl.', offsets: [[0, 0,],], },
            ],);
          },
        },),
        it({
          name: 'BLANKS THE FRONT MATTER AT THE START OF THE PAGE, every unit a space and every line break kept, so no offset '
            + 'moves and a mark in a title is no partner',
          fn: async () => {
            /**
             The first English, metadata then a body.
             */
            const first = '---\ntitle: \'the cats\'\ndescription: it\'s\n---\n\nFill the bowl.';
            const { text, offsets, } = pageOfSlices({ slices: slicesOf({ englishes: [first, 'Then nap.',], },), },);
            /**
             What the fence, the title, the description and the closing fence become.
             */
            const blanks = [
              ' '.repeat(3,),
              ' '.repeat('title: \'the cats\''.length,),
              ' '.repeat('description: it\'s'.length,),
              ' '.repeat(3,),
            ];
            expect({
              length: text.length,
              head: text.slice(0, first.indexOf('Fill',),),
              rest: text.slice(first.indexOf('Fill',),),
              offsets: [...offsets,],
            },).toEqual({
              length: first.length + 2 + 'Then nap.'.length,
              head: `${blanks.join('\n',)}\n\n`,
              rest: 'Fill the bowl.\n\nThen nap.',
              offsets: [[0, 0,], [1, first.length + 2,],],
            },);
          },
        },),
        it({
          name: 'LEAVES THE FIRST ENGLISH AS IT STANDS WHEN IT OPENS WITH NO FRONT MATTER, holds fences that are not metadata, or '
            + 'holds YAML the grammar refuses, and leaves a later English with fences alone',
          fn: async () => {
            /**
             Text with a fence pair whose YAML is a plain string, not a mapping.
             */
            const plain = '---\njust words\n---\n\nFill the bowl.';
            /**
             Text with a fence pair whose YAML the grammar refuses.
             */
            const refused = '---\ntitle: [unclosed\n---\n\nFill the bowl.';
            /**
             A later English that opens with fences.
             */
            const later = '---\ntitle: it\'s\n---\n\nThen nap.';
            expect([
              pageOfSlices({ slices: slicesOf({ englishes: [plain,], },), },).text,
              pageOfSlices({ slices: slicesOf({ englishes: [refused,], },), },).text,
              pageOfSlices({ slices: slicesOf({ englishes: ['Fill the bowl.', later,], },), },).text,
            ],).toEqual([plain, refused, `Fill the bowl.\n\n${later}`,],);
          },
        },),
        it({
          name: 'PUTS THE TEXT THE PAGE CARRIES BETWEEN THE SLICES WHERE IT STANDS, before the first, between each two and after '
            + 'the last, and names where each slice starts by the length of what stands before it',
          fn: async () => {
            const { text, offsets, } = pageOfSlices({
              slices: [
                { sliceIndex: 0, english: '## One', },
                { sliceIndex: 4, english: 'Then nap.', },
              ],
              between: ['<!-- note -->\n\n', '\n\n<!-- seal -->\n\nShe said ‘fill it.\n\n', '\n',],
            },);
            expect({ text, offsets: [...offsets,], },).toEqual({
              text: '<!-- note -->\n\n## One\n\n<!-- seal -->\n\nShe said ‘fill it.\n\nThen nap.\n',
              offsets: [[0, 15,], [4, 58,],],
            },);
            expect([text.slice(15, 21,), text.slice(58, 67,),],).toEqual(['## One', 'Then nap.',],);
          },
        },),
        it({
          name: 'BLANKS THE FRONT MATTER AT THE START OF THE PAGE WHERE THE TEXT BEFORE THE FIRST SLICE HOLDS IT, the fence in '
            + 'the text between the slices as it is in a slice, so a mark in a title is no partner whichever holds it',
          fn: async () => {
            /**
             The page's metadata, which no slice holds.
             */
            const metadata = '---\ntitle: it\'s\n---\n\n';
            const { text, offsets, } = pageOfSlices({
              slices: [{ sliceIndex: 0, english: 'Fill the bowl.', },],
              between: [metadata, '',],
            },);
            expect({ text, offsets: [...offsets,], },).toEqual({
              text: `${' '.repeat(3,)}\n${' '.repeat('title: it\'s'.length,)}\n${' '.repeat(3,)}\n\nFill the bowl.`,
              offsets: [[0, metadata.length,],],
            },);
          },
        },),
        it({
          name: 'REFUSES TEXT BETWEEN THE SLICES THAT DOES NOT COUNT ONE MORE THAN THE SLICES, naming both counts, since a page '
            + 'built on it would put a slice where no text says it goes',
          fn: async () => {
            const refusal = caught(function pagesWithTooFew(): unknown {
              return pageOfSlices({ slices: slicesOf({ englishes: ['One.', 'Two.',], },), between: ['', '',], },);
            },);
            expect(String(refusal,),).toBe(
              'ConsistencyInvariantError: unreachable: the text around the slices of the page holds 2 pieces for 2 slices, '
                + 'and it must hold one more piece than there are slices: the text before the first, between each two and after '
                + 'the last',
            );
          },
        },),
        it({
          name: 'ASSEMBLES THE PAGE WITH THE CODE THAT PUBLISHES IT when handed the layout, so the edges a span keeps and the text '
            + 'no slice holds are the published page`s, and gives where each slice`s English starts and the text around the slices',
          fn: async () => {
            /**
             A page with a sealed note between its slices and a blank line at the start of the last span.
             */
            const targetText = 'One.\n\n<!-- Fill the bowl. -->\n\nTwo.\n\nThree.';
            /**
             Where the spans of the three slices stand.
             */
            const spans = [
              [0, 4,],
              [targetText.indexOf('Two.',), targetText.indexOf('Two.',) + 4,],
              [targetText.indexOf('Two.',) + 4, targetText.length,],
            ] as const;
            const layout: PageLayout = {
              targetText,
              chunks: pairsOver({ targetText, spans, },),
              silent: [],
            };
            const slices = slicesOf({ englishes: ['One.', 'Two cats.', '\n\n\nThree cats.',], },);
            const { text, offsets, around, } = pageOfSlices({ slices, layout, },);
            expect(text,).toBe(spliceSlices({
              targetText,
              slices: layout.chunks,
              replacements: slices.map(function replacementOf({ sliceIndex, english, },) {
                return { sliceIndex, replacementText: english, };
              },),
            },),);
            expect(text,).toBe('One.\n\n<!-- Fill the bowl. -->\n\nTwo cats.\n\nThree cats.',);
            // The last slice's English opens with one blank line more than its span kept, so it starts one before the
            // 40 where the written span starts, and its words, past three blank lines, stand at 42.
            expect([...offsets.entries(),],).toEqual([[0, 0,], [1, 31,], [2, 39,],],);
            expect(around,).toEqual(['', '\n\n<!-- Fill the bowl. -->\n\n', '', '',],);
          },
        },),
        it({
          name: 'WRITES NOTHING OVER A SPAN OF A SLICE THAT SHIPS NO ENGLISH and gives it no start, so the text on both sides of it '
            + 'joins on the page as it does when published',
          fn: async () => {
            const targetText = 'One.\n\nGone.\n\nThree.';
            const layout: PageLayout = {
              targetText,
              chunks: pairsOver({ targetText, spans: [[0, 4,], [6, 11,], [13, 19,],], },),
              silent: [{ sliceIndex: 1, replacementText: '', },],
            };
            const { text, offsets, around, } = pageOfSlices({
              slices: [{ sliceIndex: 0, english: 'One.', }, { sliceIndex: 2, english: 'Three.', },],
              layout,
            },);
            expect(text,).toBe('One.\n\n\n\nThree.',);
            expect([...offsets.entries(),],).toEqual([[0, 0,], [2, 8,],],);
            expect(around,).toEqual(['', '\n\n\n\n', '',],);
          },
        },),
        it({
          name: 'PUTS THE SEPARATORS THE PUBLISHED PAGE HAS AROUND AN INSERTED ANCHOR and starts its English at the body, not at the '
            + 'blank line before it',
          fn: async () => {
            const targetText = 'One.\n\nThree.';
            const layout: PageLayout = {
              targetText,
              chunks: [
                ...pairsOver({ targetText, spans: [[0, 4,],], },),
                anchorAt({ sliceIndex: 1, offset: 6, },),
                ...pairsOver({ targetText, spans: [[6, 12,],], firstIndex: 2, },),
              ],
              silent: [],
            };
            const { text, offsets, } = pageOfSlices({
              slices: [
                { sliceIndex: 0, english: 'One.', },
                { sliceIndex: 1, english: 'Two.', },
                { sliceIndex: 2, english: 'Three.', },
              ],
              layout,
            },);
            expect(text,).toBe('One.\n\nTwo.\n\nThree.',);
            expect([...offsets.entries(),],).toEqual([[0, 0,], [1, 6,], [2, 12,],],);
          },
        },),
        it({
          name: 'BLANKS THE FRONT MATTER OF A PAGE READ WITH A LAYOUT as it does for one read without, so no offset moves',
          fn: async () => {
            const targetText = 'Old title.\n\nOld body.';
            const layout: PageLayout = {
              targetText,
              chunks: pairsOver({ targetText, spans: [[0, targetText.length,],], },),
              silent: [],
            };
            const english = '---\ntitle: it\'s\n---\n\nFill the bowl.';
            const { text, offsets, } = pageOfSlices({ slices: [{ sliceIndex: 0, english, },], layout, },);
            expect(text.length,).toBe(english.length,);
            expect(text.includes('it\'s',),).toBe(false,);
            expect(text.endsWith('\n\nFill the bowl.',),).toBe(true,);
            expect(offsets.get(0,),).toBe(0,);
          },
        },),
      ],
    },),
    describe({
      name: pageReadingOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'GIVES THE PAGE IT WAS HANDED, and the slice alone as the page where it was handed none',
          fn: async () => {
            /**
             A page that holds the slice at its offset.
             */
            const page = { text: 'Fill the bowl.\n\nThen nap.', offset: 16, };
            expect([
              pageReadingOf({ english: 'Then nap.', page, },),
              pageReadingOf({ english: 'Then nap.', },),
            ],).toEqual([page, { text: 'Then nap.', offset: 0, },],);
          },
        },),
        it({
          name: 'REFUSES A PAGE TOO SHORT TO HOLD THE SLICE AT ITS OFFSET, or holding it at a negative one, naming both ends',
          fn: async () => {
            /**
             What each page makes a reader of the slice throw.
             */
            const refused = [
              { text: 'Fill the bowl.', offset: 10, },
              { text: 'Fill the bowl.\n\nThen nap.', offset: -1, },
            ].map(function refusing(page,): string {
              return String(caught(function readsMisplaced(): unknown {
                return pageReadingOf({ english: 'Then nap.', page, },);
              },),);
            },);
            expect(refused,).toEqual([
              'ConsistencyInvariantError: unreachable: the page English handed to the proof ends at 14 and the slice spans 10 '
                + 'to 19 in it, so the slice does not lie inside its page',
              'ConsistencyInvariantError: unreachable: the page English handed to the proof ends at 25 and the slice spans -1 '
                + 'to 8 in it, so the slice does not lie inside its page',
            ],);
          },
        },),
        it({
          name: 'FINDS THE PAGE TOO SHORT BY WHAT IT HOLDS FOR THE SLICE, not by the reader\'s English: a page that holds a shorter '
            + 'shipped wording than the reader\'s English ends where the shipped wording ends',
          fn: async () => {
            /**
             The reader's English, longer than what the page holds for the slice.
             */
            const english = '> “Then nap.”';
            expect(pageReadingOf({ english, page: { text: 'Fill the bowl.\n\n> Then nap.', offset: 16, shipped: '> Then nap.', }, },),)
              .toEqual({ text: 'Fill the bowl.\n\n> Then nap.', offset: 16, shipped: '> Then nap.', },);
            /**
             What reading the same page without the shipped wording raises.
             */
            const raised = caught(function readsPastTheEnd(): unknown {
              return pageReadingOf({ english, page: { text: 'Fill the bowl.\n\n> Then nap.', offset: 16, }, },);
            },);
            expect(String(raised,),)
              .toBe(
              'ConsistencyInvariantError: unreachable: the page English handed to the proof ends at 27 and the slice spans 16 '
                + 'to 29 in it, so the slice does not lie inside its page',
              );
          },
        },),
      ],
    },),
    describe({
      name: positionOnPage.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'KEEPS THE POSITION WHERE THE PAGE HOLDS THE READER\'S WORDS OR WORDS OF THE SAME LENGTH, which is what typography '
            + 'leaves when it curls a quotation mark',
          fn: async () => {
            expect([
              positionOnPage({ written: 'The dogs\'', shipped: 'The dogs\'', offset: 20, at: 8, },),
              positionOnPage({ written: 'The dogs\'', shipped: 'The dogs’', offset: 20, at: 8, },),
            ],).toEqual([28, 28,],);
          },
        },),
        it({
          name: 'MOVES A POSITION AFTER A CHANGE OF LENGTH BY THE DIFFERENCE, and keeps one before it',
          fn: async () => {
            /**
             The reader's words, and the words the page holds with one letter added in the middle.
             */
            const written = 'The color of the dogs’';
            const shipped = 'The colour of the dogs’';
            expect([
              positionOnPage({ written, shipped, offset: 20, at: 0, },),
              positionOnPage({ written, shipped, offset: 20, at: 7, },),
              positionOnPage({ written, shipped, offset: 20, at: written.length - 1, },),
              positionOnPage({ written, shipped, offset: 20, at: written.length, },),
            ],).toEqual([20, 27, 42, 43,],);
          },
        },),
        it({
          name: 'MOVES A POSITION BACK WHERE A QUOTATION MARK WAS TAKEN OFF, and gives none to the character the change reached',
          fn: async () => {
            /**
             The reader's words with a quoted paragraph, and the words the page holds with its opening quotation mark taken off.
             */
            const written = '> “Then the dogs’ nap.';
            const shipped = '> Then the dogs’ nap.';
            expect([
              positionOnPage({ written, shipped, offset: 0, at: 0, },),
              positionOnPage({ written, shipped, offset: 0, at: 2, },),
              positionOnPage({ written, shipped, offset: 0, at: 3, },),
              positionOnPage({ written, shipped, offset: 0, at: written.indexOf('’',), },),
              positionOnPage({ written, shipped, offset: 0, at: written.length - 1, },),
              positionOnPage({ written, shipped, offset: 0, at: written.length, },),
            ],).toEqual([0, NO_PLACE, 2, 15, 20, 21,],);
          },
        },),
        it({
          name: 'GIVES NO PLACE TO THE CHARACTERS BETWEEN TWO CHANGES OF LENGTH, which the start and the end the words share do '
            + 'not reach: a quoted paragraph with both quotation marks taken off',
          fn: async () => {
            /**
             The reader's words with both quotation marks, and the words the page holds without them.
             */
            const written = '> “Then the dogs’ nap.”';
            const shipped = '> Then the dogs’ nap.';
            expect([
              positionOnPage({ written, shipped, offset: 0, at: 1, },),
              positionOnPage({ written, shipped, offset: 0, at: written.indexOf('’',), },),
            ],).toEqual([1, NO_PLACE,],);
          },
        },),
        it({
          name: 'GIVES NO PLACE TO WORDS THE PAGE REPLACED WHOLE: a slice the guard took back ships the archive\'s words, which '
            + 'share no characters with the reader\'s edit',
          fn: async () => {
            expect([
              positionOnPage({ written: 'The dogs’ bowl[^9] is here.', shipped: 'The cats naps.', offset: 0, at: 8, },),
              positionOnPage({ written: 'The dogs’ bowl.', shipped: 'The cats naps.', offset: 0, at: 0, },),
            ],).toEqual([NO_PLACE, 0,],);
          },
        },),
      ],
    },),
  ],
},);
