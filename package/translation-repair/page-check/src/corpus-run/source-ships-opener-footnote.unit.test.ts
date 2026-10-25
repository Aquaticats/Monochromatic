/**
 Tests for taking the lines of a fence or display math that a footnote definition holds out of the block when they hold
 an opener of a comment that never closes.

 The strict grammar disables indented code, so the lines indented four columns under a definition are a fence or a display
 math of the footnote there, and the plain reading takes them for indented code. Each line that holds an opener is
 replaced by a closing mark, the line as a paragraph of the footnote, and the opening mark again, all carrying the four
 columns of indentation.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';
import { parseMarkdownMathBody, } from '@monochromatic-dev/translation-repair-document-structure';

import {
  openerStarts,
  openersInFootnoteFlowsOf,
} from '../../dist/final/node/index.mjs';

/**
 What the function answers for a text, with the footnote the text starts with held to its end.

 @param source - text with a footnote definition at its start and every opener it holds

 @returns Edits and the openers they write

 @example
 ```ts
 const split = splitOf({ source: '[^1]: x\n\n    ```\n    <!-- never\n    ```\n', },);
 ```
 */
function splitOf({ source, }: { readonly source: string; },): ReturnType<typeof openersInFootnoteFlowsOf> {
  return openersInFootnoteFlowsOf({
    source,
    root: parseMarkdownMathBody({ body: source, },),
    starts: openerStarts({
      text: source,
      from: 0,
    },),
    closed: [],
    footnotes: [
      {
        start: 0,
        end: source.length,
      },
    ],
  },);
}

await describe({
  name: openersInFootnoteFlowsOf.name,
  children: [
    it({
      name: 'REPLACES THE LINE WITH A CLOSING MARK, THE LINE AS A PARAGRAPH AND AN OPENING MARK, each with the indentation',
      fn: async () => {
        /**
         A fence indented four spaces under a definition, with an opener on its second line.
         */
        const source = '[^1]: x\n\n    ```\n    <!-- never\n    ```\n';
        expect(splitOf({ source, },),).toEqual({
          edits: [
            {
              start: 17,
              end: 31,
              replacement: '    ```\n\n    &lt;\\!\\-\\- never\n\n    ```',
            },
          ],
          handled: [21,],
        },);
      },
    },),
    it({
      name: 'READS A DISPLAY MATH, A FENCE OF TILDES AND A FENCE INDENTED BY A TAB, and one edit for each line that holds an opener',
      fn: async () => {
        expect(splitOf({ source: '[^1]: x\n\n    $$\n    a\n    <!-- never\n    $$\n', },).handled,).toEqual([26,],);
        expect(splitOf({ source: '[^1]: x\n\n    ~~~\n    a <!-- one\n    b <!-- two\n    ~~~\n', },).edits.length,).toBe(2,);
        expect(splitOf({ source: '[^1]: x\n\n\t```\n\t<!-- never\n\t```\n', },).edits,).toEqual([
          {
            start: 14,
            end: 25,
            replacement: '\t```\n\n\t&lt;\\!\\-\\- never\n\n\t```',
          },
        ],);
      },
    },),
    it({
      name: 'TAKES THE LINES OUT OF A FENCE THE FOOTNOTE LEAVES OPEN, to the end of the block',
      fn: async () => {
        expect(splitOf({ source: '[^1]: x\n\n    ```js\n    <!-- never\n', },).edits,).toEqual([
          {
            start: 19,
            end: 33,
            replacement: '    ```\n\n    &lt;\\!\\-\\- never\n\n    ```',
          },
        ],);
      },
    },),
    it({
      name: 'LEAVES AN OPENER OUTSIDE A FENCE ALONE, in a paragraph of the footnote after a closed fence',
      fn: async () => {
        expect(splitOf({
          source: '[^1]: x\n\n    ```\n    a\n    ```\n\n    p <!-- never\n',
        },),).toEqual({ edits: [], handled: [], },);
      },
    },),
    it({
      name: 'LEAVES A TEXT ALONE THAT HOLDS NO OPENER, no footnote, or a fence that stands at the top',
      fn: async () => {
        /**
         A fence at the top with an opener, read with a footnote definition elsewhere.
         */
        const source = '```\n<!-- never\n```\n\n[^1]: x\n';
        expect(openersInFootnoteFlowsOf({
          source,
          root: parseMarkdownMathBody({ body: source, },),
          starts: [source.indexOf('<!--',),],
          closed: [],
          footnotes: [{ start: source.indexOf('[^1]',), end: source.length, },],
        },),).toEqual({ edits: [], handled: [], },);
        expect(openersInFootnoteFlowsOf({
          source: '[^1]: x\n\n    ```\n    a\n    ```\n',
          root: parseMarkdownMathBody({ body: '[^1]: x\n\n    ```\n    a\n    ```\n', },),
          starts: [],
          closed: [],
          footnotes: [{ start: 0, end: 30, },],
        },),).toEqual({ edits: [], handled: [], },);
        expect(openersInFootnoteFlowsOf({
          source: '    ```\n    <!-- never\n    ```\n',
          root: parseMarkdownMathBody({ body: '    ```\n    <!-- never\n    ```\n', },),
          starts: [14,],
          closed: [],
          footnotes: [],
        },),).toEqual({ edits: [], handled: [], },);
      },
    },),
  ],
},);
