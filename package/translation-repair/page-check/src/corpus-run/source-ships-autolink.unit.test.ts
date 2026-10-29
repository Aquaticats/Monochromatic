/**
 Tests for the reading of an angle autolink: written as a link, as its address inside the text of a link, and left as text
 where a table cuts it at a pipe.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import { autolinkReading, } from '../../dist/final/node/index.mjs';

await describe({
  name: autolinkReading.name,
  children: [
    it({
      name: 'WRITES AN ANGLE AUTOLINK AS A LINK WITH ITS ADDRESS FOR TEXT',
      fn: async () => {
        const { text, edits, } = autolinkReading({
          source: 'a <https://example.net/cat> b',
          url: 'https://example.net/cat',
          range: { start: 2, end: 27, },
          tables: [],
          insideLink: false,
        },);
        expect(text,).toEqual([],);
        expect(edits,).toEqual([{ start: 2, end: 27, replacement: '[https://example.net/cat](https://example.net/cat)', },],);
      },
    },),
    it({
      name: 'WRITES AN ANGLE AUTOLINK INSIDE THE TEXT OF A LINK AS ITS ADDRESS ALONE, since a link holds none',
      fn: async () => {
        const { edits, } = autolinkReading({
          source: '[<https://example.net/cat>](u)',
          url: 'https://example.net/cat',
          range: { start: 1, end: 26, },
          tables: [],
          insideLink: true,
        },);
        expect(edits,).toEqual([{ start: 1, end: 26, replacement: 'https://example.net/cat', },],);
      },
    },),
    it({
      name: 'LEAVES AN ANGLE AUTOLINK A TABLE CUTS AT A PIPE AS TEXT',
      fn: async () => {
        const { text, edits, } = autolinkReading({
          source: '| <https://example.net/a|b> | d |\n| - | - |\n',
          url: 'https://example.net/a|b',
          range: { start: 2, end: 27, },
          tables: [{ start: 0, end: 44, },],
          insideLink: false,
        },);
        expect(text,).toEqual([{ start: 2, end: 27, },],);
        expect(edits,).toEqual([],);
      },
    },),
  ],
},);
