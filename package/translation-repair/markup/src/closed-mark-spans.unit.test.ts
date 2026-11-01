/**
 Tests the one pairing rule every reader of paired marks shares (ledger
 B38): a closing mark answers the last opening mark before it, so a mark
 that never closed encloses nothing.

 Fixtures are cat-themed invention; no corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { closedMarkSpans, } from '../dist/final/node/index.mjs';

await describe({
  name: closedMarkSpans.name,
  children: [
    it({
      name: 'PAIRS EACH OPENING MARK WITH THE NEXT CLOSING ONE, in order, an empty span and adjacent spans included',
      fn: async () => {
        expect(closedMarkSpans({
          text: '《猫》与《狗》',
          open: '《',
          close: '》',
        },),).toEqual([
          {
            open: 0,
            close: 2,
          },
          {
            open: 4,
            close: 6,
          },
        ],);
        expect(closedMarkSpans({
          text: '《》《》',
          open: '《',
          close: '》',
        },),).toEqual([
          {
            open: 0,
            close: 1,
          },
          {
            open: 2,
            close: 3,
          },
        ],);
        expect(closedMarkSpans({
          text: '没有书名号。',
          open: '《',
          close: '》',
        },),).toEqual([],);
      },
    },),
    it({
      name: 'ENCLOSES NOTHING WITH A MARK THAT NEVER CLOSED: stray opening marks yield to the last one before the '
        + 'closing mark, and one with no closing mark after it ends the reading',
      fn: async () => {
        expect(closedMarkSpans({
          text: '《猫，《猫经》',
          open: '《',
          close: '》',
        },),).toEqual([
          {
            open: 3,
            close: 6,
          },
        ],);
        expect(closedMarkSpans({
          text: '《《《猫》',
          open: '《',
          close: '》',
        },),).toEqual([
          {
            open: 2,
            close: 4,
          },
        ],);
        expect(closedMarkSpans({
          text: '《猫》《未完',
          open: '《',
          close: '》',
        },),).toEqual([
          {
            open: 0,
            close: 2,
          },
        ],);
      },
    },),
    it({
      name: 'READS A CLOSING MARK WITH NO OPENING MARK BEFORE IT AS TEXT, and keeps a newline inside a span for the '
        + 'reader to judge',
      fn: async () => {
        expect(closedMarkSpans({
          text: '猫》《狗》',
          open: '《',
          close: '》',
        },),).toEqual([
          {
            open: 2,
            close: 4,
          },
        ],);
        expect(closedMarkSpans({
          text: '《猫\n狗》',
          open: '《',
          close: '》',
        },),).toEqual([
          {
            open: 0,
            close: 4,
          },
        ],);
      },
    },),
    it({
      name: 'PAIRS IDENTICAL MARKS IN TURN, so the text between two quotations is not a quotation',
      fn: async () => {
        expect(closedMarkSpans({
          text: 'a "x" and "Cat"',
          open: '"',
          close: '"',
        },),).toEqual([
          {
            open: 2,
            close: 4,
          },
          {
            open: 10,
            close: 14,
          },
        ],);
        expect(closedMarkSpans({
          text: '"one" "two',
          open: '"',
          close: '"',
        },),).toEqual([
          {
            open: 0,
            close: 4,
          },
        ],);
      },
    },),
    it({
      name: 'READS MARKS LONGER THAN ONE UNIT and text beyond the basic plane by UTF-16 offset',
      fn: async () => {
        expect(closedMarkSpans({
          text: '<<a<<b>>',
          open: '<<',
          close: '>>',
        },),).toEqual([
          {
            open: 3,
            close: 6,
          },
        ],);
        expect(closedMarkSpans({
          text: '《\u{20BB7}》',
          open: '《',
          close: '》',
        },),).toEqual([
          {
            open: 0,
            close: 3,
          },
        ],);
      },
    },),
  ],
},);
