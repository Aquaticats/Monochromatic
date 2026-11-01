/**
 Tests the text a reader sees around a mark (n159, X7): the view keeps the units that are shown and every line break, and the
 units a rule wrote in the view go back into the text as written.

 Cat-themed invention throughout; no corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  shownViewOf,
  writtenBack,
} from '../dist/final/node/index.mjs';

await describe({
  name: 'typography-shown-view (n159)',
  children: [
    it({
      name: 'KEEPS THE SHOWN UNITS with the offset each has in the text, and leaves out the units that show nothing',
      fn: async () => {
        expect(shownViewOf({
          text: '*a*',
          visible: [false, true, false,],
        },),).toEqual({
          text: 'a',
          offsets: [1,],
        },);
      },
    },),
    it({
      name: 'KEEPS EVERY LINE BREAK, whether or not the parse shows it, since no quotation pair crosses one',
      fn: async () => {
        expect(shownViewOf({
          text: 'a\n*b',
          visible: [true, false, false, true,],
        },),).toEqual({
          text: 'a\nb',
          offsets: [0, 1, 3,],
        },);
      },
    },),
    it({
      name: 'PUTS THE UNITS A RULE WROTE BACK INTO THE TEXT AS WRITTEN, leaving the units outside the view alone',
      fn: async () => {
        expect(writtenBack({
          text: '*a*',
          offsets: [1,],
          units: ['b',],
        },),).toBe('*b*',);
      },
    },),
    it({
      name: 'KEEPS THE UNIT AS WRITTEN WHEN THE RULE WROTE NO UNIT FOR IT',
      fn: async () => {
        expect(writtenBack({
          text: '*ab*',
          offsets: [1, 2,],
          units: ['c',],
        },),).toBe('*cb*',);
      },
    },),
  ],
},);
