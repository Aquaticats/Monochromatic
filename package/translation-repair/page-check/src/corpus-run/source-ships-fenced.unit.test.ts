/**
 Tests for the fences that are lengthened where their code holds a mark the strict grammar would close them at.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import { fenceLengthEdits, } from '../../dist/final/node/index.mjs';

/**
 Edits for a text that is one fenced block.

 @param source - text, whose whole is the block, less the container marks before the fence

 @param start - index of the opening fence

 @returns Edits

 @example
 ```ts
 const edits = editsOf({ source: '```\n    ```\n```', },);
 ```
 */
function editsOf(
  {
    source,
    start = 0,
  }: {
    readonly source: string;
    readonly start?: number;
  },
): ReturnType<typeof fenceLengthEdits> {
  return fenceLengthEdits({
    source,
    range: {
      start,
      end: source.trimEnd().length,
    },
  },);
}

await describe({
  name: fenceLengthEdits.name,
  children: [
    it({
      name: 'LENGTHENS THE OPENING AND THE CLOSING OF A FENCE HOLDING A MARK AS LONG AS IT, indented four spaces',
      fn: async () => {
        expect(editsOf({ source: '```\n    ```\n{\n```\n', },),).toEqual([
          {
            start: 0,
            end: 3,
            replacement: '````',
          },
          {
            start: 14,
            end: 17,
            replacement: '````',
          },
        ],);
      },
    },),

    it({
      name: 'LENGTHENS ONE MARK PAST THE LONGEST A FENCE HOLDS, of its own kind, and not the closing of a fence that '
        + 'never closes',
      fn: async () => {
        expect(editsOf({ source: '~~~\n     ~~~~\n{\n', },),).toEqual([
          {
            start: 0,
            end: 3,
            replacement: '~~~~~',
          },
        ],);
      },
    },),

    it({
      name: 'COUNTS THE MARKS OF A FENCE IN A BLOCK QUOTE AND LEAVES THE CONTAINER MARKS ALONE',
      fn: async () => {
        expect(editsOf({
          source: '> ```\n>     ```\n> {\n> ```\n',
          start: 2,
        },),).toEqual([
          {
            start: 2,
            end: 5,
            replacement: '````',
          },
          {
            start: 22,
            end: 25,
            replacement: '````',
          },
        ],);
      },
    },),

    it({
      name: 'TAKES A MARK INDENTED FOUR SPACES AT THE END OF THE BLOCK FOR CODE, not for its closing',
      fn: async () => {
        expect(editsOf({ source: '```\n    ```\n', },),).toEqual([
          {
            start: 0,
            end: 3,
            replacement: '````',
          },
        ],);
      },
    },),

    it({
      name: 'LEAVES ALONE A FENCE IN A NESTED LIST ITEM WHOSE CLOSING STANDS AT THE COLUMN OF ITS OPENING, since four '
        + 'columns or more there are the item\'s own indentation and the closing is no mark of the code',
      fn: async () => {
        /**
         A fence in a list item inside a list item, opened and closed at column four.
         */
        const source = '- a\n  - b\n\n    ```js\n    code {\n    ```\n';
        expect(editsOf({
          source,
          start: source.indexOf('```',),
        },),).toEqual([],);
      },
    },),

    it({
      name: 'LENGTHENS THE CLOSING OF A FENCE IN A NESTED LIST ITEM AS WELL AS ITS OPENING when the code holds a mark '
        + 'four columns past the opening, the closing standing at the column of the opening',
      fn: async () => {
        /**
         A fence in a nested list item holding a mark indented four spaces more than its opening.
         */
        const source = '- a\n  - b\n\n    ```js\n        ```\n    code {\n    ```\n';
        /**
         Index of the opening mark.
         */
        const opening = source.indexOf('```',);
        /**
         Index of the closing mark.
         */
        const closing = source.lastIndexOf('```',);
        expect(editsOf({
          source,
          start: opening,
        },),).toEqual([
          {
            start: opening,
            end: opening + 3,
            replacement: '````',
          },
          {
            start: closing,
            end: closing + 3,
            replacement: '````',
          },
        ],);
      },
    },),

    it({
      name: 'LEAVES A FENCE ALONE THAT HOLDS NO SUCH MARK, one shorter than its opening, or one of the other kind',
      fn: async () => {
        expect(editsOf({ source: '```\na\n```\n', },),).toEqual([],);
        expect(editsOf({ source: '````\n    ```\n{\n````\n', },),).toEqual([],);
        expect(editsOf({ source: '```\n    ~~~\n{\n```\n', },),).toEqual([],);
      },
    },),

    it({
      name: 'ANSWERS NOTHING FOR A RANGE THAT DOES NOT START WITH A FENCE',
      fn: async () => {
        expect(editsOf({ source: 'a\n    ```\n', },),).toEqual([],);
      },
    },),
  ],
},);
