/**
 Tests for where a text holds syntax the strict MDX grammar reads, taken from the CommonMark parse of the text.

 Each case hands the function the parse of a text and reads back the stretches it calls text and the rewrites it makes.
 What the encoder does with them is in `source-ships-encoding.unit.test.ts`.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  caught,
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';
import { parseMarkdownMathBody, } from '@monochromatic-dev/translation-repair-document-structure';

import {
  spansOf,
  type SourceSpans,
} from '../../dist/final/node/index.mjs';

/**
 Spans of a text.

 @param source - text to read

 @returns What the parse of the text says of it

 @example
 ```ts
 const spans = spansIn({ source: '她说 {x}', },);
 ```
 */
function spansIn({ source, }: { readonly source: string; },): SourceSpans {
  return spansOf({
    source,
    root: parseMarkdownMathBody({ body: source, },),
  },);
}

/**
 The text of a stretch of a source.

 @param source - the text

 @param range - stretch to cut out

 @returns The characters of the stretch

 @example
 ```ts
 const stretch = cut({ source, range: { start: 0, end: 2, }, },);
 ```
 */
function cut(
  {
    source,
    range,
  }: {
    readonly source: string;
    readonly range: { readonly start: number; readonly end: number; };
  },
): string {
  return source.slice(
    range.start,
    range.end,
  );
}

await describe({
  name: spansOf.name,
  children: [
    it({
      name: 'CALLS TEXT AND RAW HTML WHAT A READER SEES, and nothing of a fence, a code span or math',
      fn: async () => {
        /**
         A text with a sentence, a tag, a code span, math and a fence.
         */
        const source = '她说 <b>是</b> `{a}` $c < d$\n\n```\n{e}\n```\n';
        expect(spansIn({ source, },).text.map(function textOf(range,): string {
          return cut({
            source,
            range,
          },);
        },),).toEqual(['她说 ', '<b>', '是', '</b>', ' ', ' ',],);
      },
    },),

    it({
      name: 'CALLS INDENTED CODE TEXT, which MDX reads as a paragraph, and a fence inside a list item code',
      fn: async () => {
        expect(spansIn({ source: '猫\n\n    {x}\n', },).text.length,).toBe(2,);
        expect(spansIn({ source: '- ```\n  {\n  ```\n', },).text,).toEqual([],);
      },
    },),

    it({
      name: 'CALLS THE ALTERNATIVE TEXT OF AN IMAGE TEXT and not its destination, whether written inline or by reference',
      fn: async () => {
        /**
         An inline image and a reference image, each with a brace in its alternative text.
         */
        const source = '![图 {a}](u.png) ![图 {b}][r]\n\n[r]: u.png\n';
        expect(spansIn({ source, },).text.map(function textOf(range,): string {
          return cut({
            source,
            range,
          },);
        },),).toEqual(['图 {a}', ' ', '图 {b}',],);
      },
    },),

    it({
      name: 'REWRITES AN ANGLE AUTOLINK WHOLE and reads the text of any other link as text',
      fn: async () => {
        /**
         An autolink and an inline link.
         */
        const source = '<https://example.net/a> [链接](https://example.net/b)';
        const { text, edits, } = spansIn({ source, },);
        expect(edits.map(function replacementOf({ replacement, },): string {
          return replacement;
        },),).toEqual(['[https://example.net/a](https://example.net/a)',],);
        expect(text.map(function textOf(range,): string {
          return cut({
            source,
            range,
          },);
        },),).toEqual([' ', '链接',],);
      },
    },),

    it({
      name: 'WRITES THE FIRST LETTER OF A PARAGRAPH THAT MDX READS AS AN ES MODULE as a character reference, at the '
        + 'first column of the root only',
      fn: async () => {
        expect(spansIn({ source: 'import a from "b"\n\nexport c\n', },).edits,).toEqual([
          {
            start: 0,
            end: 1,
            replacement: '&#105;',
          },
          {
            start: 19,
            end: 20,
            replacement: '&#101;',
          },
        ],);
        expect(spansIn({ source: '> import a\n\n  import b\n\n- import c\n\nimporting\n', },).edits,).toEqual([],);
      },
    },),

    it({
      name: 'REFUSES A NODE WITH NO POSITION, which the reading it is given never builds',
      fn: async () => {
        expect(caught(function readUnpositioned(): void {
          spansOf({
            source: 'x',
            root: {
              type: 'root',
              children: [{
                type: 'paragraph',
                children: [{
                  type: 'text',
                  value: 'x',
                },],
              },],
            },
          },);
        },),).toBeInstanceOf(Error,);
      },
    },),
  ],
},);
