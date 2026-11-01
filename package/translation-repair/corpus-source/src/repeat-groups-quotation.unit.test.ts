/**
 Tests for the reader of which stretches of a wording stream touch quoted
 speech: the count of quoted characters before each place, and the question
 whether a stretch holds one. Fixtures are cat-themed invention only.

 @module
 */

import {
  caught,
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { parseDocument, } from '@monochromatic-dev/translation-repair-shared-contract';
import {
  quotedCountsOf,
  touchesQuotation,
  wordingStreamOf,
} from '../dist/final/node/index.mjs';

/**
 Counts the quoted characters before each place of a page's wording stream.

 @param text - full page text

 @returns One count per stream index and one for the end

 @example
 ```ts
 const counts = countsOf({ text: '「猫」咪\n', },);
 ```
 */
function countsOf({ text, }: { readonly text: string; },) {
  return quotedCountsOf({
    wording: wordingStreamOf({
      text,
      document: parseDocument({ text, },),
    },),
  },);
}

/**
 Message the refusal of an empty stretch or one starting before the stream carries.
 */
const EMPTY_OR_BEFORE_REFUSAL = 'Error: unreachable: a repeat or clause stretch is empty or starts before the wording stream, '
  + 'though both finders read inside it';

/**
 Message the refusal of a stretch ending past the stream carries.
 */
const PAST_END_REFUSAL = 'Error: unreachable: a repeat or clause stretch ends past the wording stream, '
  + 'though both finders read inside it';

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: quotedCountsOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'Counts nothing for a paragraph with no quotation mark',
          fn: async () => {
            expect(countsOf({ text: '猫咪窗台\n', },),).toEqual([0, 0, 0, 0, 0, 0,],);
          },
        },),

        it({
          name: 'Counts the opening mark, the speech and the closing mark of a quotation, and nothing around them',
          fn: async () => {
            // Elements: 猫 「 好 」 咪 and the break the paragraph ends with.
            expect(countsOf({ text: '猫「好」咪\n', },),).toEqual([0, 0, 1, 2, 3, 3, 3,],);
          },
        },),

        it({
          name: 'Counts a closing mark that no opening mark matches, though the depth stays at zero',
          fn: async () => {
            // Elements: 猫 」 咪 and the break.
            expect(countsOf({ text: '猫」咪\n', },),).toEqual([0, 0, 1, 1, 1,],);
          },
        },),

        it({
          name: 'Counts both straight double quotation marks and the speech between them',
          fn: async () => {
            // Elements: 猫 " 好 " 咪 and the break.
            expect(countsOf({ text: '猫"好"咪\n', },),).toEqual([0, 0, 1, 2, 3, 3, 3,],);
          },
        },),

        it({
          name: 'Counts the rest of a paragraph after an opening mark that never closes, and not its break',
          fn: async () => {
            // Elements: 猫 「 好 咪 and the break.
            expect(countsOf({ text: '猫「好咪\n', },),).toEqual([0, 0, 1, 2, 3, 3,],);
          },
        },),

        it({
          name: 'Counts the speech between an inner quotation and the end of the outer one',
          fn: async () => {
            // Elements: 「 外 『 内 』 面 」 猫 and the break.
            expect(countsOf({ text: '「外『内』面」猫\n', },),).toEqual([0, 1, 2, 3, 4, 5, 6, 7, 7, 7,],);
          },
        },),

        it({
          name: 'Counts nothing for an empty stream beyond the first place',
          fn: async () => {
            expect(countsOf({ text: '', },),).toEqual([0,],);
          },
        },),
      ],
    },),

    describe({
      name: touchesQuotation.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'Reports a stretch of narration beside a quotation as untouched, whichever side the quotation stands on',
          fn: async () => {
            // Elements: 猫 咪 「 好 」 窗 台 and the break.
            /** Counts for a paragraph with its quotation in the middle. */
            const counts = countsOf({ text: '猫咪「好」窗台\n', },);

            expect(touchesQuotation({ counts, start: 0, end: 2, },),).toBe(false,);
            expect(touchesQuotation({ counts, start: 5, end: 7, },),).toBe(false,);
          },
        },),

        it({
          name: 'Reports a stretch that ends with an opening mark or begins with a closing mark as touching',
          fn: async () => {
            // Elements: 猫 咪 「 好 」 窗 台 and the break.
            /** Counts for a paragraph with its quotation in the middle. */
            const counts = countsOf({ text: '猫咪「好」窗台\n', },);

            expect(touchesQuotation({ counts, start: 0, end: 3, },),).toBe(true,);
            expect(touchesQuotation({ counts, start: 4, end: 7, },),).toBe(true,);
          },
        },),

        it({
          name: 'Reports a stretch that starts in narration and ends inside the speech as touching',
          fn: async () => {
            // Elements: 猫 咪 「 好 暖 」 and the break.
            /** Counts for a paragraph whose quotation ends the paragraph. */
            const counts = countsOf({ text: '猫咪「好暖」\n', },);

            expect(touchesQuotation({ counts, start: 0, end: 4, },),).toBe(true,);
          },
        },),

        it({
          name: 'Reports a stretch inside the speech alone as touching',
          fn: async () => {
            // Elements: 「 好 暖 和 」 and the break.
            /** Counts for a paragraph that is one quotation. */
            const counts = countsOf({ text: '「好暖和」\n', },);

            expect(touchesQuotation({ counts, start: 1, end: 3, },),).toBe(true,);
          },
        },),

        it({
          name: 'Reports a stretch with a closing mark that no opening mark matches as touching',
          fn: async () => {
            // Elements: 猫 」 咪 and the break.
            /** Counts for a paragraph with a stray closing mark. */
            const counts = countsOf({ text: '猫」咪\n', },);

            expect(touchesQuotation({ counts, start: 0, end: 3, },),).toBe(true,);
            expect(touchesQuotation({ counts, start: 2, end: 3, },),).toBe(false,);
          },
        },),

        it({
          name: 'Reports a stretch of one quoted character as touching',
          fn: async () => {
            // Elements: 猫 「 好 咪 and the break.
            /** Counts for a paragraph with an opening mark that never closes. */
            const counts = countsOf({ text: '猫「好咪\n', },);

            expect(touchesQuotation({ counts, start: 2, end: 3, },),).toBe(true,);
          },
        },),

        it({
          name: 'Refuses an empty stretch, a reversed one and one that reaches past the stream',
          fn: async () => {
            /** Counts for a paragraph of two elements and its break. */
            const counts = countsOf({ text: '猫咪\n', },);
            /** What an empty stretch throws. */
            const empty = caught(function touchEmpty(): unknown {
              return touchesQuotation({ counts, start: 1, end: 1, },);
            },);
            /** What a reversed stretch throws. */
            const reversed = caught(function touchReversed(): unknown {
              return touchesQuotation({ counts, start: 2, end: 1, },);
            },);
            /** What a stretch ending past the stream throws. */
            const pastEnd = caught(function touchPastEnd(): unknown {
              return touchesQuotation({ counts, start: 0, end: 4, },);
            },);
            /** What a stretch starting before the stream throws. */
            const beforeStart = caught(function touchBeforeStart(): unknown {
              return touchesQuotation({ counts, start: -1, end: 1, },);
            },);

            expect(empty,).toBeInstanceOf(Error,);
            expect(String(empty,),).toBe(EMPTY_OR_BEFORE_REFUSAL,);
            expect(String(reversed,),).toBe(EMPTY_OR_BEFORE_REFUSAL,);
            expect(String(pastEnd,),).toBe(PAST_END_REFUSAL,);
            expect(String(beforeStart,),).toBe(EMPTY_OR_BEFORE_REFUSAL,);
          },
        },),
      ],
    },),
  ],
},);
