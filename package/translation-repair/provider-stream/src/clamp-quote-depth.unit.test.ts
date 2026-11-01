/**
 Tests the clamp that brings an edit's over-deep quotation lines back to the
 depth the region already had. Its behaviour inside an application is pinned in
 `apply-patch.unit.test.ts`; these cases read the clamp itself, on lines that
 carry markers and no words. Fixtures are cat-themed invention.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { clampQuoteDepth, } from '../dist/final/node/index.mjs';

await describe({
  name: clampQuoteDepth.name,
  children: [
    it({
      name: 'CLAMPS A LINE OF MARKERS ALONE TO THE BOUND WITH NO TRAILING SPACE, and a line with words to the '
        + 'bound followed by its words',
      fn: async () => {
        expect([
          clampQuoteDepth({
            replacement: 'The cat naps.\n> > >\n> > She purrs.',
            bound: 1,
            startsLine: true,
          },),
          clampQuoteDepth({
            replacement: '>>>',
            bound: 2,
            startsLine: true,
          },),
        ],).toEqual([
          'The cat naps.\n>\n> She purrs.',
          '> >',
        ],);
      },
    },),
  ],
},);
