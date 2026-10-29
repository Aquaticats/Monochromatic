/**
 Tests for the italic spans read off the parse (ledger B72): offsets into the
 text as given, nested spans, words a span shows, comments left unread, and a
 text the strict grammar refuses. Fixtures are cat-themed invention only.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  type EmphasisSpan,
  emphasisSpans,
  oneLine,
} from '../../dist/final/node/index.mjs';

/**
 Italic spans of a text, first opened first, since the reader promises no
 order.

 @param text - text to read

 @returns Its italic spans by start

 @example
 ```ts
 spansOf({ text: 'She loved *Long Nap*.', },);
 ```
 */
function spansOf({ text, }: { readonly text: string; },): readonly EmphasisSpan[] {
  return emphasisSpans({ text, },).toSorted(function byStart(
    left,
    right,
  ): number {
    return left.start - right.start;
  },);
}

/**
 Words of every italic span of a text, first opened first.

 @param text - text to read

 @returns Each span's words

 @example
 ```ts
 wordsIn({ text: 'She loved *Long Nap*.', },); // ['Long Nap']
 ```
 */
function wordsIn({ text, }: { readonly text: string; },): readonly string[] {
  return spansOf({ text, },).map(function wordsOf(span,): string {
    return span.words;
  },);
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: emphasisSpans.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'READS a span with its delimiters\' offsets and its words',
          fn: async () => {
            expect(spansOf({ text: 'She loved *Long Nap*.', },),).toEqual([
              { start: 10, end: 20, words: 'Long Nap', },
            ],);
          },
        },),
        it({
          name: 'READS a span set with underscores, and none in an empty text',
          fn: async () => {
            expect([
              spansOf({ text: '_Long Nap_', },),
              spansOf({ text: '', },),
            ],).toEqual([
              [{ start: 0, end: 10, words: 'Long Nap', },],
              [],
            ],);
          },
        },),
        it({
          name: 'READS a span inside another as a span of its own, the outer one\'s words taking in the inner\'s',
          fn: async () => {
            expect(spansOf({ text: '*The cat loved _Long Nap_ most.*', },),).toEqual([
              { start: 0, end: 32, words: 'The cat loved Long Nap most.', },
              { start: 15, end: 25, words: 'Long Nap', },
            ],);
          },
        },),
        it({
          name: 'READS NO SPAN out of bold, and one out of bold italics',
          fn: async () => {
            expect([
              wordsIn({ text: 'The cat loved **Nap Time** most.', },),
              wordsIn({ text: 'The cat loved ***Nap Time*** most.', },),
            ],).toEqual([
              [],
              ['Nap Time',],
            ],);
          },
        },),
        it({
          name: 'READS the words a span shows: code as its text, a link as its words, a line break and a hard break as '
            + 'one space',
          fn: async () => {
            expect([
              wordsIn({ text: '*the `nap` cat*', },),
              wordsIn({ text: '*[Long Nap](https://example.com/nap)*', },),
              wordsIn({ text: '*Long\nNap*', },),
              wordsIn({ text: '*Long\\\nNap*', },),
            ],).toEqual([
              ['the nap cat',],
              ['Long Nap',],
              ['Long Nap',],
              ['Long Nap',],
            ],);
          },
        },),
        it({
          name: 'READS NO SPAN inside an HTML comment, and gives a later span its offsets in the text as given',
          fn: async () => {
            expect(spansOf({ text: 'Naps. <!-- *Hidden* --> *Shown*', },),).toEqual([
              { start: 24, end: 31, words: 'Shown', },
            ],);
          },
        },),
        it({
          name: 'READS NO SPAN inside a JSX comment beside an HTML comment, which the strict grammar reads once the '
            + 'HTML comment is blanked',
          fn: async () => {
            expect(spansOf({ text: 'Naps. <!-- a note --> {/*Hidden*/} *Shown*', },),).toEqual([
              { start: 35, end: 42, words: 'Shown', },
            ],);
          },
        },),
        it({
          name: 'READS a span in a text the strict grammar refuses, through plain Markdown',
          fn: async () => {
            expect(spansOf({ text: 'The cat <3s *naps*.', },),).toEqual([
              { start: 12, end: 18, words: 'naps', },
            ],);
          },
        },),
      ],
    },),

    describe({
      name: oneLine.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'READS every run of spaces and line feeds as one space, and drops those at either end',
          fn: async () => {
            expect([
              oneLine({ text: 'Long:\nNap', },),
              oneLine({ text: '  Long   Nap  ', },),
              oneLine({ text: '\nLong\n\nNap\n', },),
              oneLine({ text: '', },),
            ],).toEqual([
              'Long: Nap',
              'Long Nap',
              'Long Nap',
              '',
            ],);
          },
        },),
        it({
          name: 'KEEPS a no-break space and a tab, which are the page\'s spacing and no line wrapping (ledger B72)',
          fn: async () => {
            expect([
              oneLine({ text: 'Long\u00A0Nap', },),
              oneLine({ text: 'Long\u202FNap', },),
              oneLine({ text: 'Long\tNap', },),
            ],).toEqual([
              'Long\u00A0Nap',
              'Long\u202FNap',
              'Long\tNap',
            ],);
          },
        },),
      ],
    },),
  ],
},);
