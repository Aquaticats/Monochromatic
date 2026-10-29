/**
 Differential property against postcss as the tolerant-parser oracle:
 every document css-edit's generator produces and css-edit accepts must
 also be accepted by postcss, and postcss must see the same rule-level
 content once css-edit stringifies it back.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  asCssSource,
  parseCss,
  stringifyCss,
} from '@monochromatic-dev/css-core-edit/ts';
import {
  assert,
  property,
} from 'fast-check';
import { parse as postcssParse, } from 'postcss';

import { cssDocumentArb, } from './css-arbitrary.ts';
import { fuzzRuns, } from './fuzz-budget.ts';

await describe({
  name: 'postcss differential property',
  children: [
    it({
      name: 'postcss accepts every generated document css-edit accepts',
      fn: async () => {
        assert(
          property(cssDocumentArb, (css,) => {
            /**
             css-edit output for the generated document; byte-equal to input
             per the round-trip property, re-checked here as the oracle input.
             */
            const echoed = stringifyCss({
              state: parseCss({ source: asCssSource(css,), },),
            },);
            /**
             Oracle parse; a throw fails the property.
             */
            const oracleRoot = postcssParse(echoed,);
            expect(oracleRoot.toString(),).toBe(css,);
          },),
          { numRuns: fuzzRuns, },
        );
      },
    },),
  ],
},);
