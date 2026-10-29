/**
 Tests the starts of a footnote definition that follow the row of a table, which are cut as any other and never joined to
 the row (`startsOffTables`).

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { startsOffTables, } from '../../dist/final/node/index.mjs';

await describe({
  name: startsOffTables.name,
  children: [
    it({
      name: 'TAKES OUT A START WHOSE LINE FOLLOWS THE LAST ROW OF A TABLE, the end of the table being the line ending',
      fn: async () => {
        /**
         A table of three rows, ending at the line ending after its last row, and a line after it.
         */
        const source = '| a | b |\n| - | - |\n| c | `d |\n    [^1]: y` |\n';
        expect(startsOffTables({
          source,
          starts: [35,],
          tables: [{ start: 0, end: 30, },],
        },),).toEqual([],);
        expect(source.charAt(30,),).toBe('\n',);
        expect(source.indexOf('[^1]',),).toBe(35,);
      },
    },),
    it({
      name: 'KEEPS A START WHOSE LINE FOLLOWS A LINE OUTSIDE EVERY TABLE, and every start where the text holds no table',
      fn: async () => {
        /**
         A paragraph with a line indented four spaces, then a table.
         */
        const source = 'a `x\n    [^1]: y` z\n\n| a |\n| - |\n| b |\n';
        /**
         Where the footnote starts, and where the table stands.
         */
        const start = source.indexOf('[^1]',);
        const table = {
          start: source.indexOf('| a |',),
          end: source.length - 1,
        };
        expect(startsOffTables({ source, starts: [start,], tables: [table,], },),).toEqual([start,],);
        expect(startsOffTables({ source, starts: [start,], tables: [], },),).toEqual([start,],);
      },
    },),
    it({
      name: 'KEEPS THE ORDER OF THE STARTS IT KEEPS, a start on the first line among them',
      fn: async () => {
        /**
         A footnote on the first line, a table, a line indented four spaces right after its row, and one after a blank line.
         */
        const source = '[^1]: x\n| a |\n    [^2]: y\n\n    [^3]: z\n';
        /**
         Where the three footnotes start.
         */
        const starts = [source.indexOf('[^1]',), source.indexOf('[^2]',), source.indexOf('[^3]',),];
        expect(startsOffTables({
          source,
          starts,
          tables: [{ start: source.indexOf('| a |',), end: source.indexOf('\n    [^2]',), },],
        },),).toEqual([starts[0], starts[2],],);
      },
    },),
  ],
},);
