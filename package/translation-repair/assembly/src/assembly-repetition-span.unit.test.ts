/**
 Tests for growing repeated windows into the passages they belong to.

 WHY THESE ARE HERE rather than only through `findIntroducedRepetitions`. The
 finder's tests say what a document reports, which is the contract that
 matters, but they cannot distinguish the two ways a merge can be wrong: too
 eager, joining passages that merely abut, and too shy, leaving one
 duplication as many findings. These reach the rule itself and pin both
 directions with word lists short enough to read.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  countSpan,
  grownSpans,
  indexWindows,
} from '../dist/final/node/index.mjs';

/**
 Window length these tests grow from, short enough that a fixture fits a line.
 */
const LENGTH = 2;

/**
 Splits a sentence into the word list the real caller passes.

 @param text - words separated by single spaces

 @returns Those words

 @example
 ```ts
 const words = wordList({ text: 'tabby naps here', },);
 ```
 */
function wordList({ text, }: { readonly text: string; },): readonly string[] {
  return text.split(' ',);
}

/**
 Grows every repeated window of a word list, admitting all of them.

 ADMITS EVERY REPEATED WINDOW rather than applying the finder's content rules,
 because what is under test is the growing rather than the filtering. The
 finder decides admission; this decides what admitted windows become.

 @param text - document as a spaced sentence

 @returns Spans grown from it

 @example
 ```ts
 const spans = spansOf({ text: 'tabby naps tabby naps', },);
 ```
 */
function spansOf({ text, }: { readonly text: string; },) {
  /**
   Word list the windows index.
   */
  const words = wordList({ text, },);

  /**
   Windows of the fixture length.
   */
  const index = indexWindows({
    words,
    length: LENGTH,
  },);

  /**
   Offsets whose window occurs more than once.
   */
  const admitted = new Set<number>();
  for (const [, positions,] of index.byPhrase) {
    if (positions.length < 2)
      continue;
    for (const at of positions)
      admitted.add(at,);
  }

  return grownSpans({
    words,
    length: LENGTH,
    index,
    admitted,
  },);
}

/**
 Passages that would earn a finding, as against those kept only to suppress.

 THE TWO ARE DIFFERENT LISTS and a test that ignores the difference measures
 the wrong thing. Every span is returned, because every span suppresses the
 shorter phrases inside it; only the ones no earlier span accounts for are
 findings. A passage said twice is reached again at its second occurrence and
 appears in the full list a second time, marked.

 @param text - document as a spaced sentence

 @returns Phrases that would be reported, in order

 @example
 ```ts
 const phrases = reportedPhrases({ text: 'tabby naps tabby naps', },);
 ```
 */
function reportedPhrases({ text, }: { readonly text: string; },): readonly string[] {
  return spansOf({ text, },)
    .filter(function stands(span,): boolean {
      return !span.accountedFor;
    },)
    .map(function toPhrase(span,): string {
      return span.phrase;
    },);
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: indexWindows.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'RECORDS EVERY OFFSET A PHRASE OCCUPIES, not just that it recurred, '
            + 'because the merge test compares WHERE two windows occur and a count '
            + 'cannot answer that',
          fn: async () => {
            const index = indexWindows({
              words: wordList({ text: 'tabby naps tabby naps', },),
              length: LENGTH,
            },);

            expect(index.byOffset,).toStrictEqual([
              'tabby naps',
              'naps tabby',
              'tabby naps',
            ],);
            expect(index.byPhrase
              .get('tabby naps',),).toStrictEqual([
                0,
                2,
              ],);
          },
        },),

        it({
          name: 'IS EMPTY when the document is shorter than one window, rather '
            + 'than returning a short phrase that no caller asked about',
          fn: async () => {
            const index = indexWindows({
              words: ['tabby',],
              length: LENGTH,
            },);

            expect(index.byOffset.length,).toBe(0,);
            expect(index.byPhrase.size,).toBe(0,);
          },
        },),
      ],
    },),

    describe({
      name: grownSpans.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'JOINS WINDOWS OF ONE PASSAGE into a single span naming the whole '
            + 'of it, which is the merge this module exists for',
          fn: async () => {
            const spans = spansOf({ text: 'tabby naps beside lanterns tabby naps beside lanterns', },);

            expect(reportedPhrases({ text: 'tabby naps beside lanterns tabby naps beside lanterns', },),)
              .toStrictEqual(['tabby naps beside lanterns',],);
            expect(spans[0]?.count,).toBe(2,);
            expect(spans[0]?.accountedFor,).toBe(false,);

            // The same passage is reached again at its second occurrence and comes
            // back marked rather than missing, because it still has to suppress
            // the shorter phrases inside that copy.
            expect(spans.length,).toBe(2,);
            expect(spans[1]?.accountedFor,).toBe(true,);
          },
        },),

        it({
          name: 'REFUSES TO JOIN TWO PASSAGES THAT MERELY ABUT. Their windows sit '
            + 'next to each other but occur in unrelated places, and a rule reading '
            + 'adjacency would report a span the document never said',
          fn: async () => {
            // `tabby naps` and `lanterns glimmer` each recur, and they touch once,
            // but the window straddling them occurs only there.
            expect(reportedPhrases({
              text: 'tabby naps lanterns glimmer somewhere quiet tabby naps elsewhere entirely lanterns glimmer',
            },),).toStrictEqual([
              'tabby naps',
              'lanterns glimmer',
            ],);
          },
        },),

        it({
          name: 'MARKS A SPAN ALREADY ACCOUNTED FOR rather than dropping it, so a '
            + 'passage said three times is one finding while the join between '
            + 'copies still suppresses the shorter phrases inside it',
          fn: async () => {
            const spans = spansOf({ text: 'tabby naps beside tabby naps beside tabby naps beside', },);

            /**
             Spans that earn a finding, as against those kept only to suppress.
             */
            const reported = spans.filter(function stands(span,): boolean {
              return !span.accountedFor;
            },);

            expect(reported.length,).toBe(1,);
            expect(reported[0]?.count,).toBe(3,);
            expect(spans.length,).toBeGreaterThan(reported.length,);
          },
        },),

        it({
          name: 'GROWS NOTHING when no window is admitted, which is the ordinary '
            + 'case for a document that repeats nothing',
          fn: async () => {
            expect(spansOf({ text: 'tabby naps beside quiet harbour lanterns', },).length,).toBe(0,);
          },
        },),
      ],
    },),

    describe({
      name: countSpan.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'COUNTS EVERY OCCURRENCE of a passage, which is what tells an '
            + 'introduced repetition from one the archive already carried',
          fn: async () => {
            expect(countSpan({
              words: wordList({ text: 'tabby naps beside tabby naps beside tabby', },),
              phrase: 'tabby naps',
            },),).toBe(2,);
          },
        },),

        it({
          name: 'ANSWERS ZERO for a passage the document never says, including one '
            + 'longer than the document itself, rather than reading past the end',
          fn: async () => {
            expect(countSpan({
              words: wordList({ text: 'tabby naps', },),
              phrase: 'lanterns glimmer quietly',
            },),).toBe(0,);
          },
        },),
      ],
    },),
  ],
},);
