/**
 Tests the reading of where the parse of a fragment finds no footnote
 reference (ledger B212): the span of each place a marker shape is URL, code,
 image, definition or comment rather than a reference, written as offsets of
 the text, and the search that places an offset in them. Fixtures are
 cat-themed invention.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  fragmentReadingOf,
  insideParsedSpan,
} from '../dist/final/node/index.mjs';

/**
 Spans of a fragment in which the parse finds no footnote reference.

 @param text - fragment to read

 @returns Spans, disjoint and in source order

 @example
 ```ts
 const spans = spansOf({ text: 'A `nap [^6]`.', },);
 ```
 */
function spansOf({ text, }: { readonly text: string; },): readonly { readonly start: number; readonly end: number; }[] {
  return fragmentReadingOf({ text, },)
    .spans;
}

/**
 The span of one stretch of a text, found by its spelling.

 @param text - text holding the stretch

 @param stretch - spelling of the stretch

 @returns Offsets of the stretch's first occurrence
 */
function spanOf(
  {
    text,
    stretch,
  }: {
    readonly text: string;
    readonly stretch: string;
  },
): { readonly start: number; readonly end: number; } {
  return {
    start: text.indexOf(stretch,),
    end: text.indexOf(stretch,) + stretch.length,
  };
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: fragmentReadingOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'SPANS THE MARKUP AROUND AN INLINE LINK\'S LABEL and its destination, and none of the label\'s own text',
          fn: async () => {
            const text = 'A [paws [^1] nap](https://cat.example/[^9]x "t [^8]") then [^2].';
            expect(spansOf({ text, },),).toEqual([
              { start: 2, end: 3, },
              spanOf({ text, stretch: '](https://cat.example/[^9]x "t [^8]")', },),
            ],);
          },
        },),
        it({
          name: 'SPANS THE MARKUP AROUND A REFERENCE LINK\'S LABEL and its definition, so a marker shape read as the '
            + 'label\'s brackets is no reference',
          fn: async () => {
            const text = '[^9][cat] naps.\n\n[cat]: https://cat.example/[^4] "t"';
            expect(spansOf({ text, },),).toEqual([
              { start: 0, end: 1, },
              { start: 3, end: 9, },
              spanOf({ text, stretch: '[cat]: https://cat.example/[^4] "t"', },),
            ],);
          },
        },),
        it({
          name: 'SPANS AN ANGLE AUTOLINK, A LITERAL AFTER WHITESPACE AND A LITERAL AFTER A HAN CHARACTER whole',
          fn: async () => {
            const angle = '<https://cat.example/[^8]x>';
            const literal = 'www.cat.example/[^7]x';
            const afterHan = 'https://cat.example/[^6]x';
            const text = `A ${angle} nap ${literal} 猫见${afterHan} 睡 [^1].`;
            expect(spansOf({ text, },),).toEqual([
              spanOf({ text, stretch: angle, },),
              spanOf({ text, stretch: literal, },),
              spanOf({ text, stretch: afterHan, },),
            ],);
          },
        },),
        it({
          name: 'SPANS ONLY THE MARKER SHAPE of a literal the autolink transform built, which stands in text no '
            + 'position was kept for',
          fn: async () => {
            expect(spansOf({ text: 'A cat：www.c.example/[^9]x [^3]', },),).toEqual([{ start: 20, end: 24, },],);
            expect(spansOf({ text: '猫.www.c.example/[^7]y 睡 [^3]', },),).toEqual([{ start: 16, end: 20, },],);
          },
        },),
        it({
          name: 'SPANS A CODE SPAN, AN IMAGE AND A MASKED COMMENT, and joins a comment inside a code span with it',
          fn: async () => {
            const code = '`nap [^6]`';
            const image = '![cat [^5]](u)';
            const comment = '<!-- note [^3] -->';
            const text = `A ${code} ${image} ${comment} nap [^1]. B \`nap <!-- [^2] -->\` [^4].`;
            expect(spansOf({ text, },),).toEqual([
              spanOf({ text, stretch: code, },),
              spanOf({ text, stretch: image, },),
              spanOf({ text, stretch: comment, },),
              spanOf({ text, stretch: '`nap <!-- [^2] -->`', },),
            ],);
          },
        },),
        it({
          name: 'SPANS NOTHING where only references stand, and in a code block, which a fragment cannot settle for '
            + 'its page',
          fn: async () => {
            expect(spansOf({ text: 'A cat naps [^1].', },),).toEqual([],);
            expect(spansOf({ text: '```\n[^2]\n```\n[^1]', },),).toEqual([],);
          },
        },),
        it({
          name: 'SPANS AN UNCLOSED COMMENT to the text\'s end, as the page masks it',
          fn: async () => {
            expect(spansOf({ text: 'A [^1] <!-- [^2]', },),).toEqual([{ start: 7, end: 16, },],);
          },
        },),
        it({
          name: 'SPANS A REGION ONLY WHERE STRICT MDX AND PLAIN MARKDOWN BOTH READ IT: a fence indented four spaces '
            + 'closes the paragraph for one and pairs the backticks into a code span for the other',
          fn: async () => {
            expect(spansOf({ text: 'A ```[^8] cat\n    ``` [^3]', },),).toEqual([],);
            expect(spansOf({ text: 'A `nap [^6]` cat\n    ``` [^3]', },),).toEqual([{ start: 2, end: 12, },],);
          },
        },),
        it({
          name: 'SPANS NOTHING for a text with no marker shape',
          fn: async () => {
            expect(spansOf({ text: 'A cat naps at https://cat.example/ `here`.', },),).toEqual([],);
          },
        },),
        it({
          name: 'SPANS NOTHING for a text the parser refuses for its nesting, so every marker there stays counted',
          fn: async () => {
            expect(spansOf({ text: `${'>'.repeat(16_000,)} https://cat.example/[^9]x [^1]`, },),).toEqual([],);
          },
        },),
        it({
          name: 'READS THE DEFINITIONS OF THE PAGE\'S OWN GRAMMAR: the strict grammar\'s where it accepts the text, '
            + 'plain markdown\'s where it refuses it, none where the text holds no marker shape, and none settled '
            + 'where plain markdown cannot read the text',
          fn: async () => {
            expect(fragmentReadingOf({ text: '    [^1]: note\n', },).definitions,).toEqual({
              settled: true,
              starts: new Set([4,],),
            },);
            expect(fragmentReadingOf({ text: '    [^1]: note\n\nA <br> here.\n', },).definitions,).toEqual({
              settled: true,
              starts: new Set(),
            },);
            expect(fragmentReadingOf({ text: 'A cat naps.', },).definitions,).toEqual({
              settled: true,
              starts: new Set(),
            },);
            expect(fragmentReadingOf({ text: `${'>'.repeat(16_000,)} cat\n\n[^1]: note\n`, },).definitions,).toEqual({
              settled: false,
            },);
          },
        },),
      ],
    },),

    describe({
      name: insideParsedSpan.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'PLACES AN OFFSET by the span holding it, the start in and the end out, and finds none among no spans',
          fn: async () => {
            const spans = [{ start: 2, end: 5, }, { start: 9, end: 12, }, { start: 20, end: 21, },];
            expect(
              [0, 1, 2, 4, 5, 8, 9, 11, 12, 20, 21, 30,].map(function placed(offset,): boolean {
                return insideParsedSpan({ spans, offset, },);
              },),
            ).toEqual([false, false, true, true, false, false, true, true, false, true, false, false,],);
            expect(insideParsedSpan({ spans: [], offset: 3, },),).toBe(false,);
          },
        },),
      ],
    },),
  ],
},);
