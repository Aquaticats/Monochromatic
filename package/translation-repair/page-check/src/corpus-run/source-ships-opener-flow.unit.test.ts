/**
 Tests for taking the lines of a fence or display math that hold an opener of a comment that never closes out of the block.

 The block is closed before the line, the line is written as a paragraph, and the block is opened again with the same mark;
 every line written carries the marks of the containers the block stands in.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { splitFlow, } from '../../dist/final/node/index.mjs';

await describe({
  name: splitFlow.name,
  children: [
    it({
      name: 'REPLACES THE LINE WITH A CLOSING MARK, THE LINE AS A PARAGRAPH AND AN OPENING MARK, and names the opener it wrote',
      fn: async () => {
        /**
         A fence that never closes, with an opener on its second line.
         */
        const source = '```\nx {a}\n<!-- never\n[a]: u\n';
        expect(splitFlow({
          source,
          range: {
            start: 0,
            end: source.length,
          },
          mark: '```',
          carried: '',
          starts: [source.indexOf('<!--',),],
          closed: [],
        },),).toEqual({
          edits: [
            {
              start: 10,
              end: 20,
              replacement: '```\n\n&lt;\\!\\-\\- never\n\n```',
            },
          ],
          handled: [10,],
        },);
      },
    },),
    it({
      name: 'WRITES ONE EDIT FOR TWO OPENERS ON A LINE, with the line endings of the text',
      fn: async () => {
        /**
         A fence of Windows line endings with two openers on one line.
         */
        const source = '```\r\nx\r\n<!-- a <!-- b\r\n';
        expect(splitFlow({
          source,
          range: {
            start: 0,
            end: source.length,
          },
          mark: '```',
          carried: '',
          starts: [source.indexOf('<!--',), source.lastIndexOf('<!--',),],
          closed: [],
        },),).toEqual({
          edits: [
            {
              start: 8,
              end: 21,
              replacement: '```\r\n\r\n&lt;\\!\\-\\- a &lt;\\!\\-\\- b\r\n\r\n```',
            },
          ],
          handled: [8, 15,],
        },);
      },
    },),
    it({
      name: 'CARRIES THE MARKS OF THE CONTAINER ON EACH LINE IT WRITES, and keeps the blank lines to the mark alone',
      fn: async () => {
        /**
         A fence in a block quote.
         */
        const source = '> ```\n> x\n> <!-- a\n> y\n';
        expect(splitFlow({
          source,
          range: {
            start: 2,
            end: source.length,
          },
          mark: '```',
          carried: '> ',
          starts: [source.indexOf('<!--',),],
          closed: [],
        },).edits,).toEqual([
          {
            start: 10,
            end: 18,
            replacement: '> ```\n>\n> &lt;\\!\\-\\- a\n>\n> ```',
          },
        ],);
      },
    },),
    it({
      name: 'LEAVES A LINE ALONE THAT DOES NOT CARRY THE MARKS OF ITS CONTAINER, and a block with no opener',
      fn: async () => {
        /**
         A fence at the top, read as if it stood in a block quote.
         */
        const source = '```\nx {a}\n<!-- never\n[a]: u\n';
        expect(splitFlow({
          source,
          range: {
            start: 0,
            end: source.length,
          },
          mark: '```',
          carried: '> ',
          starts: [source.indexOf('<!--',),],
          closed: [],
        },),).toEqual({ edits: [], handled: [], },);
        expect(splitFlow({
          source,
          range: {
            start: 0,
            end: source.length,
          },
          mark: '```',
          carried: '',
          starts: [],
          closed: [],
        },),).toEqual({ edits: [], handled: [], },);
      },
    },),
  ],
},);
