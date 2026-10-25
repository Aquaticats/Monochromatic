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
      name: 'CARRIES THE MARKS AND THE INDENTATION OF A CONTAINER ON EVERY LINE IT WRITES, in a block quote and a list item',
      fn: async () => {
        /**
         A block quote and a list item whose footnote holds a fence with an opener on its second line.
         */
        const quote = '> [^1]: x\n>\n>     ```\n>     <!-- never\n>     ```\n';
        const item = '- [^1]: x\n\n      ```\n      <!-- never\n      ```\n';
        expect(openersInFootnoteFlowsOf({
          source: quote,
          starts: [quote.indexOf('<!--',),],
          closed: [],
          footnotes: [{ start: 2, end: quote.length - 1, },],
        },).edits,).toEqual([
          {
            start: quote.indexOf('>     <!--',),
            end: quote.indexOf('>     <!--',) + 16,
            replacement: '>     ```\n>\n>     &lt;\\!\\-\\- never\n>\n>     ```',
          },
        ],);
        expect(openersInFootnoteFlowsOf({
          source: item,
          starts: [item.indexOf('<!--',),],
          closed: [],
          footnotes: [{ start: 2, end: item.length - 1, },],
        },).handled,).toEqual([item.indexOf('<!--',),],);
      },
    },),
    it({
      name: 'READS A FENCE RIGHT AFTER THE FOOTNOTE LINE, with no blank line, and takes the opener out of it',
      fn: async () => {
        expect(splitOf({ source: '[^1]: x\n    ```\n    x <!-- never\n    ```\n', },).handled,).toEqual([22,],);
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
          starts: [source.indexOf('<!--',),],
          closed: [],
          footnotes: [{ start: source.indexOf('[^1]',), end: source.length, },],
        },),).toEqual({ edits: [], handled: [], },);
        expect(openersInFootnoteFlowsOf({
          source: '[^1]: x\n\n    ```\n    a\n    ```\n',
          starts: [],
          closed: [],
          footnotes: [{ start: 0, end: 30, },],
        },),).toEqual({ edits: [], handled: [], },);
        /**
         An indented fence at the top that no footnote holds, with an opener on its second line and a footnote after it.
         */
        const indented = '    ```\n    <!-- never\n    ```\n\n[^2]: w\n';
        expect(openersInFootnoteFlowsOf({
          source: indented,
          starts: [indented.indexOf('<!--',),],
          closed: [],
          footnotes: [{ start: indented.indexOf('[^2]',), end: indented.length, },],
        },),).toEqual({ edits: [], handled: [], },);
        expect(openersInFootnoteFlowsOf({
          source: indented,
          starts: [indented.indexOf('<!--',),],
          closed: [],
          footnotes: [],
        },),).toEqual({ edits: [], handled: [], },);
      },
    },),
  ],
},);
