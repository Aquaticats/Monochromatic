/**
 Tests for putting spans in source order and joining those that overlap or touch.

 Fixtures are cat-themed invention; no corpus content appears here.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { disjointSpans, } from '../dist/final/node/index.mjs';

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: disjointSpans.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'PUTS SPANS IN SOURCE ORDER and keeps those that stand apart',
          fn: async () => {
            expect(disjointSpans({
              spans: [
                { start: 10, end: 12, },
                { start: 0, end: 2, },
                { start: 5, end: 7, },
              ],
            },),).toEqual([
              { start: 0, end: 2, },
              { start: 5, end: 7, },
              { start: 10, end: 12, },
            ],);
          },
        },),
        it({
          name: 'JOINS SPANS THAT OVERLAP OR TOUCH, whichever order they come in, and keeps the farther end',
          fn: async () => {
            expect([
              disjointSpans({ spans: [{ start: 4, end: 9, }, { start: 0, end: 5, },], },),
              disjointSpans({ spans: [{ start: 0, end: 5, }, { start: 5, end: 8, },], },),
              disjointSpans({ spans: [{ start: 0, end: 10, }, { start: 2, end: 4, },], },),
            ],).toEqual([
              [{ start: 0, end: 9, },],
              [{ start: 0, end: 8, },],
              [{ start: 0, end: 10, },],
            ],);
          },
        },),
        it({
          name: 'GIVES ONE SPAN FOR THE SAME SPAN TWICE AND NONE FOR NONE',
          fn: async () => {
            expect([
              disjointSpans({ spans: [{ start: 3, end: 6, }, { start: 3, end: 6, },], },),
              disjointSpans({ spans: [], },),
            ],).toEqual([
              [{ start: 3, end: 6, },],
              [],
            ],);
          },
        },),
        it({
          name: 'LEAVES THE SPANS IT WAS GIVEN AS THEY WERE, since the list it returns is its own',
          fn: async () => {
            /**
             Spans out of order.
             */
            const spans = [
              { start: 4, end: 9, },
              { start: 0, end: 5, },
            ];
            disjointSpans({ spans, },);
            expect(spans,).toEqual([
              { start: 4, end: 9, },
              { start: 0, end: 5, },
            ],);
          },
        },),
      ],
    },),
  ],
},);
