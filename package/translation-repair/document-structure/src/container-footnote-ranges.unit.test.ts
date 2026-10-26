/**
 Tests for the ranges of a footnote the two readings of it disagree on inside a block quote or a list.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import { containerFootnoteDisagreementsOf, } from '../dist/final/node/index.mjs';

await describe({
  name: containerFootnoteDisagreementsOf.name,
  children: [
    it({
      name: 'GIVES THE RANGE OF EACH READING WHEN THEY START A FOOTNOTE IN A BLOCK QUOTE AT THE SAME PLACE AND END IT AT DIFFERENT ONES',
      fn: async () => {
        expect(containerFootnoteDisagreementsOf({ body: '> [^1]: x\n> <q>\n> text\n> ===\n', },),).toEqual([
          {
            start: 2,
            end: 15,
          },
          {
            start: 2,
            end: 28,
          },
        ],);
        expect(containerFootnoteDisagreementsOf({ body: '> [^1]: x\n> <q>\n> text\n> ===\n> ', },),).toHaveLength(2,);
      },
    },),
    it({
      name: 'GIVES THE RANGE OF EACH READING WHEN THEY START A FOOTNOTE IN A LIST ITEM AT THE SAME PLACE AND END IT AT DIFFERENT ONES',
      fn: async () => {
        expect(containerFootnoteDisagreementsOf({ body: '- a\n  [^1]: x\n  <q>\n  text\n  ===\n  ', },),).toEqual([
          {
            start: 6,
            end: 19,
          },
          {
            start: 6,
            end: 32,
          },
        ],);
      },
    },),
    it({
      name: 'GIVES THE RANGE OF A FOOTNOTE ONLY THE PAGE STARTS, a line indented four spaces after a paragraph',
      fn: async () => {
        expect(containerFootnoteDisagreementsOf({ body: '> a `x\n>     [^1]: <b>` {d}\n', },),).toEqual([
          {
            start: 13,
            end: 27,
          },
        ],);
        expect(containerFootnoteDisagreementsOf({ body: '> a `x\n    [^1]: {c}` {d}\n', },),).toEqual([
          {
            start: 11,
            end: 25,
          },
        ],);
        expect(containerFootnoteDisagreementsOf({ body: '- a `x\n      [^1]: <b>` {d}\n', },),).toEqual([
          {
            start: 13,
            end: 27,
          },
        ],);
      },
    },),
    it({
      name: 'GIVES THE RANGES OF EVERY DISAGREEMENT IN TEXT ORDER, each of two blocks apart having its own',
      fn: async () => {
        expect(
          containerFootnoteDisagreementsOf({
            body: 'a\n\n> [^1]: x\n> <q>\n> text\n> ===\n\n- b `y\n      [^2]: <b>` {d}\n',
          },),
        ).toEqual([
          {
            start: 5,
            end: 18,
          },
          {
            start: 5,
            end: 31,
          },
          {
            start: 46,
            end: 60,
          },
        ],);
      },
    },),
    it({
      name: 'GIVES NONE FOR A FOOTNOTE AT THE TOP WHOSE READINGS DIFFER, with a block quote beside it',
      fn: async () => {
        expect(containerFootnoteDisagreementsOf({ body: '[^1]: x\n<q>\ntext\n===\n\n> after\n', },),).toEqual([],);
      },
    },),
    it({
      name: 'GIVES NONE WHERE THE TWO READINGS AGREE, in a block quote or not',
      fn: async () => {
        expect(containerFootnoteDisagreementsOf({ body: '> [^1]: x\n', },),).toEqual([],);
        expect(containerFootnoteDisagreementsOf({ body: '她说 {\n', },),).toEqual([],);
      },
    },),
  ],
},);
