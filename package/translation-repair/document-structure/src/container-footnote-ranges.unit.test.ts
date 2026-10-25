/**
 Tests for the check of whether the two readings of a footnote disagree inside a block quote or a list.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import { footnoteRangesDifferInContainer, } from '../dist/final/node/index.mjs';

await describe({
  name: footnoteRangesDifferInContainer.name,
  children: [
    it({
      name: 'IS TRUE FOR A FOOTNOTE IN A BLOCK QUOTE THAT A TAG ENDS FOR ONE READING AND NOT FOR THE OTHER',
      fn: async () => {
        expect(footnoteRangesDifferInContainer({ body: '> [^1]: x\n> <q>\n> text\n> ===\n> ', },),).toBe(true,);
      },
    },),
    it({
      name: 'IS TRUE FOR A FOOTNOTE IN A LIST ITEM THAT A TAG ENDS FOR ONE READING AND NOT FOR THE OTHER',
      fn: async () => {
        expect(footnoteRangesDifferInContainer({ body: '- a\n  [^1]: x\n  <q>\n  text\n  ===\n  ', },),).toBe(true,);
      },
    },),
    it({
      name: 'IS TRUE FOR A FOOTNOTE ONLY THE PAGE STARTS IN A BLOCK QUOTE, a line indented four spaces after a paragraph',
      fn: async () => {
        expect(footnoteRangesDifferInContainer({ body: '> a `x\n>     [^1]: <b>` {d}\n', },),).toBe(true,);
        expect(footnoteRangesDifferInContainer({ body: '> a `x\n    [^1]: {c}` {d}\n', },),).toBe(true,);
        expect(footnoteRangesDifferInContainer({ body: '- a `x\n      [^1]: <b>` {d}\n', },),).toBe(true,);
      },
    },),
    it({
      name: 'IS FALSE FOR A FOOTNOTE AT THE TOP WHOSE READINGS DIFFER, with a block quote beside it',
      fn: async () => {
        expect(footnoteRangesDifferInContainer({ body: '[^1]: x\n<q>\ntext\n===\n\n> after\n', },),).toBe(false,);
      },
    },),
    it({
      name: 'IS FALSE WHERE THE TWO READINGS AGREE, in a block quote or not',
      fn: async () => {
        expect(footnoteRangesDifferInContainer({ body: '> [^1]: x\n', },),).toBe(false,);
        expect(footnoteRangesDifferInContainer({ body: '她说 {\n', },),).toBe(false,);
      },
    },),
  ],
},);
